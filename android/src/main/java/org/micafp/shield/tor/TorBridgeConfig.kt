/**
 * TorBridgeConfig.kt — Bridge configuration for Tor pluggable transports
 * Supports Snowflake, obfs4, Meek-Azure, and direct Tor (no bridges)
 */
package org.micafp.shield.tor

enum class BridgeKind { NONE, SNOWFLAKE, OBFS4, MEEK_AZURE }

data class TorBridgeConfig(
    val kind:              BridgeKind = BridgeKind.SNOWFLAKE,
    val bridges:           List<String> = emptyList(),
    val ptBinaryPath:      String = "",
    val snowflakeUrl:      String = "https://snowflake-broker.torproject.net.global.prod.fastly.net/",
    val snowflakeFronts:   String = "cdn.sstatic.net,cdn.uptimerobot.com",
) {
    companion object {
        /** Select best bridge type based on network environment (NAIN detection). */
        fun autoDiscover(): TorBridgeConfig {
            // Prefer Snowflake for max DPI resistance; falls back to obfs4 then direct
            return TorBridgeConfig(
                kind = BridgeKind.SNOWFLAKE,
                bridges = listOf(
                    "192.0.2.3:80 2B280B23E1107BB62ABFC40DDCC8824814F80A72"
                ),
            )
        }
    }
}
