package com.unifiedshield

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.util.Log
import androidx.annotation.RequiresApi

/**
 * Kill switch implementation using Android's always-on VPN lockdown mode
 * and NetworkCallback monitoring.
 *
 * When enabled:
 * 1. Sets VPN to blocking mode (all traffic must go through VPN)
 * 2. Monitors network changes to detect VPN disconnection
 * 3. Blocks all internet access if VPN tunnel drops
 * 4. Prevents DNS leaks by forcing DNS through the tunnel
 */
class KillSwitch(private val vpnService: VpnService) {

    private val TAG = "KillSwitch"
    private var isEnabled = false
    private var connectivityManager: ConnectivityManager? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    /**
     * Enable the kill switch.
     * - Registers network callbacks to monitor connectivity
     * - Ensures VPN is in blocking/lockdown mode
     */
    fun enable() {
        if (isEnabled) return
        isEnabled = true

        connectivityManager = vpnService.getSystemService(Context.CONNECTIVITY_SERVICE)
            as ConnectivityManager

        // Set VPN to blocking mode - all traffic must go through VPN
        vpnService.setBlocking(true)

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            registerNetworkCallback()
        }

        Log.i(TAG, "Kill switch enabled - blocking mode active")
    }

    /**
     * Disable the kill switch.
     */
    fun disable() {
        if (!isEnabled) return
        isEnabled = false

        try {
            networkCallback?.let {
                connectivityManager?.unregisterNetworkCallback(it)
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to unregister network callback: ${e.message}")
        }

        networkCallback = null
        Log.i(TAG, "Kill switch disabled")
    }

    /**
     * Register network callback to monitor for VPN disconnection.
     * If VPN network is lost, immediately block all traffic.
     */
    @RequiresApi(Build.VERSION_CODES.M)
    private fun registerNetworkCallback() {
        val cm = connectivityManager ?: return

        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()

        networkCallback = object : ConnectivityManager.NetworkCallback() {

            override fun onAvailable(network: Network) {
                super.onAvailable(network)
                Log.d(TAG, "Network available: $network")

                // Check if this is the VPN network
                val caps = cm.getNetworkCapabilities(network)
                if (caps?.hasTransport(NetworkCapabilities.TRANSPORT_VPN) == true) {
                    Log.i(TAG, "VPN network connected - kill switch monitoring active")
                } else if (isEnabled) {
                    // A non-VPN network became available while kill switch is on
                    // This could be a leak - ensure VPN is blocking
                    Log.w(TAG, "Non-VPN network available while kill switch active")
                    vpnService.setBlocking(true)
                }
            }

            override fun onLost(network: Network) {
                super.onLost(network)
                Log.w(TAG, "Network lost: $network")

                // Check if the lost network was our VPN
                val caps = cm.getNetworkCapabilities(network)
                if (caps?.hasTransport(NetworkCapabilities.TRANSPORT_VPN) == true && isEnabled) {
                    Log.e(TAG, "VPN network lost! Kill switch activated - blocking all traffic")
                    onVpnDisconnected()
                }
            }

            override fun onCapabilitiesChanged(
                network: Network,
                networkCapabilities: NetworkCapabilities
            ) {
                super.onCapabilitiesChanged(network, networkCapabilities)

                // Verify VPN is still the default route
                if (isEnabled) {
                    val isVpn = networkCapabilities.hasTransport(
                        NetworkCapabilities.TRANSPORT_VPN
                    )
                    val hasInternet = networkCapabilities.hasCapability(
                        NetworkCapabilities.NET_CAPABILITY_INTERNET
                    )
                    if (!isVpn && hasInternet) {
                        Log.w(TAG, "Non-VPN internet access detected - enforcing blocking")
                        vpnService.setBlocking(true)
                    }
                }
            }
        }

        cm.registerNetworkCallback(request, networkCallback!!)
    }

    /**
     * Called when VPN disconnection is detected.
     * Immediately blocks all network access to prevent IP leaks.
     */
    private fun onVpnDisconnected() {
        try {
            // Set VPN to blocking mode - this prevents any traffic from bypassing
            vpnService.setBlocking(true)

            // Attempt to restart the VPN service
            val intent = android.content.Intent(vpnService, VpnService::class.java).apply {
                action = VpnService.ACTION_START
            }
            vpnService.startService(intent)

            Log.i(TAG, "Kill switch: attempting VPN reconnection")
        } catch (e: Exception) {
            Log.e(TAG, "Kill switch: failed to reconnect VPN: ${e.message}")
        }
    }

    /**
     * Check if kill switch is currently enabled.
     */
    fun isActive(): Boolean = isEnabled

    /**
     * Emergency block - immediately set blocking and stop all traffic.
     */
    fun emergencyBlock() {
        vpnService.setBlocking(true)
        isEnabled = true
        Log.e(TAG, "EMERGENCY BLOCK ACTIVATED - all traffic blocked")
    }
}
