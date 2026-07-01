package com.unifiedshield

import android.os.ParcelFileDescriptor
import kotlinx.coroutines.*
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.UnixDomainSocketAddress
import java.nio.channels.Channels

/**
 * Manages the TUN interface lifecycle: reading packets from the TUN fd,
 * forwarding them to the Rust core via Unix socket, and writing responses back.
 */
class TunnelManager {

    private var tunFd: ParcelFileDescriptor? = null
    private var readJob: Job? = null
    private var writeJob: Job? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var isRunning = false

    @Volatile
    private var packetCount: Long = 0

    @Volatile
    private var byteCount: Long = 0

    fun start(fd: ParcelFileDescriptor, unixSocket: String) {
        if (isRunning) return
        tunFd = fd
        isRunning = true

        val inputFile = FileInputStream(fd.fileDescriptor)
        val outputFile = FileOutputStream(fd.fileDescriptor)

        // Read from TUN -> forward to core via Unix socket
        readJob = scope.launch {
            val buffer = ByteArray(VpnService.TUN_MTU + 28) // MTU + header overhead
            while (isActive && isRunning) {
                try {
                    val bytesRead = inputFile.read(buffer)
                    if (bytesRead > 0) {
                        packetCount++
                        byteCount += bytesRead
                        // Forward packet to Rust core via JNI/Unix socket
                        CoreBridge().forwardPacket(buffer.copyOf(bytesRead))
                    }
                } catch (e: Exception) {
                    if (isActive) continue else break
                }
            }
        }

        // Write from core -> TUN interface
        writeJob = scope.launch {
            while (isActive && isRunning) {
                try {
                    val packet = CoreBridge().receivePacket()
                    if (packet != null && packet.isNotEmpty()) {
                        outputFile.write(packet)
                    }
                } catch (e: Exception) {
                    if (isActive) continue else break
                }
            }
        }
    }

    fun stop() {
        isRunning = false
        readJob?.cancel()
        writeJob?.cancel()
        runCatching { tunFd?.close() }
        tunFd = null
        scope.cancel()
    }

    fun getStats(): TunnelStats {
        return TunnelStats(
            packetCount = packetCount,
            byteCount = byteCount,
            isRunning = isRunning
        )
    }

    data class TunnelStats(
        val packetCount: Long,
        val byteCount: Long,
        val isRunning: Boolean
    )
}
