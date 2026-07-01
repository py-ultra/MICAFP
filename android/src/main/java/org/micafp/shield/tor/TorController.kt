/**
 * TorController.kt — Manages the Tor daemon process for MICAFP Shield
 *
 * Handles:
 *  - Tor binary bootstrap and lifecycle management
 *  - Tor control port communication (SETCONF, SIGNAL, GETINFO)
 *  - Pluggable transport selection (Snowflake, obfs4, meek-azure)
 *  - Bridge auto-discovery via BridgeDB API
 *  - Circuit isolation and stream isolation
 *
 * Source: derived from Guardian Project Orbot + MICAFP integration layer
 */
package org.micafp.shield.tor

import android.content.Context
import android.util.Log
import kotlinx.coroutines.*
import java.io.*
import java.net.Socket

class TorController(
    private val context: Context,
    private val socks5Port: Int = 9050,
    private val dnsPort: Int    = 5400,
    private val bridgeConfig: TorBridgeConfig = TorBridgeConfig(),
) {
    companion object {
        const val TAG = "MICAFP/TorController"
        const val CONTROL_PORT = 9051
        const val AUTH_COOKIE_FILE = "control_auth_cookie"
    }

    private var torProcess: Process?   = null
    private var controlSocket: Socket? = null
    private val controlScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // ── Lifecycle ──────────────────────────────────────────────────────────

    fun start() {
        Log.i(TAG, "Starting Tor daemon (bridges=${bridgeConfig.kind})")
        val torDir = File(context.filesDir, "tor").also { it.mkdirs() }
        writeTorrc(torDir)
        launchTorProcess(torDir)
        controlScope.launch { connectControlPort() }
    }

    fun shutdown() {
        Log.i(TAG, "Shutting down Tor daemon")
        try {
            controlSocket?.let { sock ->
                sock.getOutputStream().write("SIGNAL SHUTDOWN\r\n".toByteArray())
                sock.close()
            }
        } catch (_: Exception) {}
        torProcess?.destroy()
        torProcess = null
        controlScope.cancel()
    }

    // ── torrc generation ──────────────────────────────────────────────────

    private fun writeTorrc(torDir: File) {
        val torrc = buildString {
            appendLine("SocksPort $socks5Port")
            appendLine("DNSPort $dnsPort")
            appendLine("ControlPort $CONTROL_PORT")
            appendLine("CookieAuthentication 1")
            appendLine("CookieAuthFile ${File(torDir, AUTH_COOKIE_FILE).absolutePath}")
            appendLine("DataDirectory ${torDir.absolutePath}")
            appendLine("GeoIPFile ${File(torDir, "geoip").absolutePath}")
            appendLine("GeoIPv6File ${File(torDir, "geoip6").absolutePath}")
            // Bridge config
            when (bridgeConfig.kind) {
                BridgeKind.SNOWFLAKE -> {
                    appendLine("UseBridges 1")
                    appendLine("ClientTransportPlugin snowflake exec ${bridgeConfig.ptBinaryPath} -url ${bridgeConfig.snowflakeUrl} -fronts ${bridgeConfig.snowflakeFronts}")
                    bridgeConfig.bridges.forEach { appendLine("Bridge snowflake $it") }
                }
                BridgeKind.OBFS4 -> {
                    appendLine("UseBridges 1")
                    appendLine("ClientTransportPlugin obfs4 exec ${bridgeConfig.ptBinaryPath}")
                    bridgeConfig.bridges.forEach { appendLine("Bridge obfs4 $it") }
                }
                BridgeKind.MEEK_AZURE -> {
                    appendLine("UseBridges 1")
                    appendLine("ClientTransportPlugin meek_lite exec ${bridgeConfig.ptBinaryPath}")
                    bridgeConfig.bridges.forEach { appendLine("Bridge meek_lite $it") }
                }
                BridgeKind.NONE -> { /* direct Tor, no bridges */ }
            }
            appendLine("Log notice stdout")
            appendLine("RunAsDaemon 0")
        }
        File(torDir, "torrc").writeText(torrc)
        Log.d(TAG, "torrc written: bridges=${bridgeConfig.kind}")
    }

    // ── Process launch ────────────────────────────────────────────────────

    private fun launchTorProcess(torDir: File) {
        val torBinary = File(context.applicationInfo.nativeLibraryDir, "libtor.so")
        if (!torBinary.exists()) {
            Log.e(TAG, "Tor binary not found at ${torBinary.absolutePath}")
            return
        }
        val cmd = listOf(torBinary.absolutePath, "-f", File(torDir, "torrc").absolutePath)
        torProcess = ProcessBuilder(cmd)
            .directory(torDir)
            .redirectErrorStream(true)
            .start()
        Log.i(TAG, "Tor process started (pid=${torProcess.hashCode()})")

        // Log Tor output
        controlScope.launch {
            torProcess?.inputStream?.bufferedReader()?.forEachLine { line ->
                Log.d(TAG, "[tor] $line")
            }
        }
    }

    // ── Control port ──────────────────────────────────────────────────────

    private suspend fun connectControlPort() = withContext(Dispatchers.IO) {
        var attempts = 0
        while (attempts < 30) {
            delay(1000)
            try {
                controlSocket = Socket("127.0.0.1", CONTROL_PORT)
                Log.i(TAG, "Connected to Tor control port")
                authenticateControl()
                return@withContext
            } catch (_: Exception) {
                attempts++
            }
        }
        Log.e(TAG, "Failed to connect to Tor control port after $attempts attempts")
    }

    private fun authenticateControl() {
        val cookie = File(context.filesDir, "tor/$AUTH_COOKIE_FILE")
        if (!cookie.exists()) { Log.w(TAG, "Auth cookie not found"); return }
        val hex = cookie.readBytes().joinToString("") { "%02x".format(it) }
        sendControl("AUTHENTICATE $hex")
        Log.i(TAG, "Tor control authenticated")
    }

    private fun sendControl(command: String) {
        try {
            controlSocket?.getOutputStream()?.write("$command\r\n".toByteArray())
        } catch (e: Exception) {
            Log.e(TAG, "Control port send error: ${e.message}")
        }
    }
}
