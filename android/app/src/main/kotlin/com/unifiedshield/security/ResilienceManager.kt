// MICAFP UnifiedShield VIP-ULTRA — Android Resilience Manager
// Manages circuit breakers and fallback chain for transport resilience.
package com.unifiedshield.security

import android.util.Log
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicLong

private const val TAG = "ShieldResilience"

/** Circuit breaker states. */
enum class BreakerState { CLOSED, OPEN, HALF_OPEN }

/**
 * Per-transport circuit breaker.
 * Prevents cascade failures when a transport repeatedly fails.
 */
class CircuitBreaker(
    val transportName: String,
    private val failureThreshold: Int = 3,
    private val recoveryTimeoutMs: Long = 30_000L
) {
    private val failureCount = AtomicInteger(0)
    private val lastFailureMs = AtomicLong(0L)
    @Volatile private var state = BreakerState.CLOSED

    fun allowRequest(): Boolean {
        return when (state) {
            BreakerState.CLOSED -> true
            BreakerState.OPEN -> {
                if (System.currentTimeMillis() - lastFailureMs.get() >= recoveryTimeoutMs) {
                    state = BreakerState.HALF_OPEN
                    Log.i(TAG, "[$transportName] Open → HalfOpen")
                    true
                } else false
            }
            BreakerState.HALF_OPEN -> true
        }
    }

    fun recordSuccess() {
        if (state == BreakerState.HALF_OPEN)
            Log.i(TAG, "[$transportName] HalfOpen → Closed (recovered)")
        state = BreakerState.CLOSED
        failureCount.set(0)
    }

    fun recordFailure() {
        val count = failureCount.incrementAndGet()
        lastFailureMs.set(System.currentTimeMillis())
        if (count >= failureThreshold && state == BreakerState.CLOSED) {
            state = BreakerState.OPEN
            Log.w(TAG, "[$transportName] Closed → Open (failures=$count)")
        } else if (state == BreakerState.HALF_OPEN) {
            state = BreakerState.OPEN
            Log.w(TAG, "[$transportName] HalfOpen → Open (probe failed)")
        }
    }

    fun getState(): BreakerState = state
}

/**
 * Ordered fallback chain for extreme censorship resilience.
 * Tries each strategy in order until one succeeds.
 */
class FallbackChain {
    enum class Strategy {
        PRIMARY_TRANSPORT,
        CHINESE_CDN_WORKER,
        P2P_LIBP2P_RELAY,
        DOH_TUNNEL,
        ICMP_TUNNEL,
        MESH_NETWORK,
        TOR_BRIDGE_SNOWFLAKE,
        TOR_BRIDGE_MEEK
    }

    private val strategies = Strategy.values().toList()
    private var currentIndex = 0

    val current: Strategy get() = strategies[currentIndex]

    fun advance(): Strategy? {
        Log.w(TAG, "Fallback: ${current.name} failed, advancing")
        return if (currentIndex + 1 < strategies.size) {
            currentIndex++
            Log.i(TAG, "Fallback: activating ${current.name}")
            current
        } else {
            Log.e(TAG, "Fallback: all strategies exhausted")
            null
        }
    }

    fun reset() {
        currentIndex = 0
        Log.i(TAG, "Fallback: reset to PRIMARY_TRANSPORT")
    }

    fun position(): Int = currentIndex
    fun total(): Int = strategies.size
}
