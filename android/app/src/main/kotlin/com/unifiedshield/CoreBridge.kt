package com.unifiedshield

/**
 * JNI bridge to the Rust/Go core library (libunifiedshield.so).
 *
 * The native library handles:
 * - VPN protocol cores (Xray, Naïve, Hysteria2, TUIC)
 * - Obfuscation and domain fronting
 * - Connection management
 * - Packet routing
 */
class CoreBridge {

    companion object {
        init {
            System.loadLibrary("unifiedshield")
        }

        // Core types
        const val CORE_XRAY = "xray"
        const val CORE_NAIVE = "naive"
        const val CORE_HYSTERIA2 = "hysteria2"
        const val CORE_TUIC = "tuic"

        @Volatile
        private var isInitialized = false
    }

    /**
     * Start the VPN daemon with the specified core.
     *
     * @param tunFd File descriptor of the TUN interface
     * @param core Protocol core to use (xray, naive, hysteria2, tuic)
     * @param unixSocket Unix domain socket path for IPC
     * @param isp Detected ISP name for routing decisions
     */
    external fun startDaemon(tunFd: Int, core: String, unixSocket: String, isp: String): Int

    /**
     * Stop the VPN daemon gracefully.
     */
    external fun stopDaemon(): Int

    /**
     * Get current daemon status.
     * Returns: 0 = stopped, 1 = running, 2 = connecting, -1 = error
     */
    external fun getStatus(): Int

    /**
     * Switch the active protocol core at runtime.
     * Used for DPI evasion when detection score exceeds threshold.
     *
     * @param core New core to switch to
     */
    external fun switchCore(core: String): Int

    /**
     * Update the reward/credit balance for bandwidth accounting.
     *
     * @param reward New reward value
     */
    external fun updateReward(reward: Long): Int

    /**
     * Enable or disable the kill switch.
     *
     * @param enabled true to enable, false to disable
     */
    external fun setKillSwitch(enabled: Boolean): Int

    /**
     * Trigger obfuscation mode when DPI is detected.
     * Activates domain fronting and traffic shaping.
     */
    external fun triggerObfuscationMode(): Int

    /**
     * Forward a packet from TUN to the core.
     */
    external fun forwardPacket(packet: ByteArray): Int

    /**
     * Receive a processed packet from the core.
     * Returns null if no packet is available.
     */
    external fun receivePacket(): ByteArray?

    /**
     * Get connection statistics from the core.
     */
    external fun getConnectionStats(): String

    /**
     * Validate a server configuration.
     */
    external fun validateConfig(configJson: String): Boolean

    /**
     * Set the CDN preference for domain fronting.
     * Chinese CDNs are primary (Cloudflare blocked in Iran).
     */
    external fun setCdnPreference(cdnProvider: String): Int
}
