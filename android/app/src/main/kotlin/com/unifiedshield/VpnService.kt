package com.unifiedshield

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.system.OsConstants
import androidx.core.app.NotificationCompat
import androidx.lifecycle.LifecycleService
import com.unifiedshield.splittunnel.SplitTunnel
import kotlinx.coroutines.*

class VpnService : LifecycleService() {

    private var vpnInterface: ParcelFileDescriptor? = null
    private var tunnelJob: Job? = null
    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val coreBridge = CoreBridge()
    private val splitTunnel = SplitTunnel(this)
    private val killSwitch = KillSwitch(this)
    private val dpiDetector = DpiDetector()
    private val ispDetector = IspDetector(this)

    private var currentCore = "xray"
    private var isRunning = false

    companion object {
        const val ACTION_START = "com.unifiedshield.START"
        const val ACTION_STOP = "com.unifiedshield.STOP"
        const val NOTIFICATION_CHANNEL_ID = "unifiedshield_vpn"
        const val NOTIFICATION_ID = 1
        const val TUN_MTU = 1380
        const val TUN_ADDRESS = "172.19.0.1"
        const val TUN_PREFIX = 24
        const val TUN_ROUTE_V4 = "0.0.0.0/0"
        const val TUN_ROUTE_V6 = "::/0"
        const val UNIX_SOCKET = "unifiedshield-tun"
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)

        when (intent?.action) {
            ACTION_START -> startVpn()
            ACTION_STOP -> stopVpn()
        }

        return START_STICKY
    }

    private fun startVpn() {
        if (isRunning) return

        val notification = buildNotification("Connecting...")
        startForeground(NOTIFICATION_ID, notification)

        serviceScope.launch {
            try {
                // Detect ISP for split tunnel decisions
                val isp = ispDetector.detect()
                splitTunnel.loadIranianIpRanges()

                // Build VPN interface with split tunnel
                val builder = Builder()
                    .setMtu(TUN_MTU)
                    .addAddress(TUN_ADDRESS, TUN_PREFIX)
                    .addRoute(TUN_ROUTE_V4, 0)
                    .addRoute(TUN_ROUTE_V6, 0)
                    .setSession("UnifiedShield")
                    .setBlocking(true)

                // Exclude own app from VPN to prevent feedback loop
                builder.addDisallowedApplication(packageName)

                // Split tunnel: only route non-Iranian IPs through VPN
                splitTunnel.applySplitTunnelRoutes(builder)

                // Set DNS servers (Chinese CDN primary - Cloudflare blocked in Iran)
                builder.addDnsServer("223.5.5.5")      // Alibaba DNS
                builder.addDnsServer("119.29.29.29")    // Tencent DNS
                builder.addDnsServer("1.12.12.12")      // Tencent DNS backup
                builder.addSearchDomain("local")

                vpnInterface = builder.establish()

                if (vpnInterface == null) {
                    stopSelf()
                    return@launch
                }

                // Start native daemon via JNI
                coreBridge.startDaemon(
                    tunFd = vpnInterface!!.fd,
                    core = currentCore,
                    unixSocket = UNIX_SOCKET,
                    isp = isp
                )

                // Enable kill switch
                killSwitch.enable()

                // Start DPI monitoring
                dpiDetector.startMonitoring { score ->
                    if (score > 0.72) {
                        // DPI detected - trigger obfuscation mode
                        coreBridge.triggerObfuscationMode()
                        // Consider switching core
                        currentCore = if (currentCore == "xray") "naive" else "xray"
                        coreBridge.switchCore(currentCore)
                    }
                }

                isRunning = true
                updateNotification("Connected - $currentCore")

            } catch (e: Exception) {
                updateNotification("Error: ${e.message}")
                stopVpn()
            }
        }
    }

    private fun stopVpn() {
        if (!isRunning) return

        runCatching { dpiDetector.stopMonitoring() }
        runCatching { coreBridge.stopDaemon() }
        runCatching { killSwitch.disable() }
        runCatching { vpnInterface?.close() }

        vpnInterface = null
        isRunning = false
        tunnelJob?.cancel()
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    override fun onDestroy() {
        stopVpn()
        serviceScope.cancel()
        super.onDestroy()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "UnifiedShield VPN",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "VPN connection status"
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(text: String): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this, 0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle("UnifiedShield")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    private fun updateNotification(text: String) {
        val notification = buildNotification(text)
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(NOTIFICATION_ID, notification)
    }
}
