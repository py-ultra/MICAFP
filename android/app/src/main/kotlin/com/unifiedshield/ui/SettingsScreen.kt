package com.unifiedshield.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun SettingsScreen() {
    var killSwitchEnabled by remember { mutableStateOf(true) }
    var splitTunnelEnabled by remember { mutableStateOf(true) }
    var autoUpdateEnabled by remember { mutableStateOf(true) }
    var autoCoreSwitchEnabled by remember { mutableStateOf(true) }
    var startOnBootEnabled by remember { mutableStateOf(false) }
    var dnsProvider by remember { mutableStateOf("alibaba") }
    var obfuscationLevel by remember { mutableStateOf(1) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(
            text = "Settings",
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 8.dp)
        )

        // Security Section
        SectionHeader("Security")

        SettingsSwitch(
            title = "Kill Switch",
            description = "Block all traffic if VPN disconnects unexpectedly",
            checked = killSwitchEnabled,
            onCheckedChange = { killSwitchEnabled = it }
        )

        SettingsSwitch(
            title = "Split Tunnel",
            description = "Keep Iranian traffic on direct network (banking, gov sites)",
            checked = splitTunnelEnabled,
            onCheckedChange = { splitTunnelEnabled = it }
        )

        SettingsSwitch(
            title = "Auto Core Switch",
            description = "Switch protocol when DPI is detected (score > 0.72)",
            checked = autoCoreSwitchEnabled,
            onCheckedChange = { autoCoreSwitchEnabled = it }
        )

        // Obfuscation Level
        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "Obfuscation Level",
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    text = "Higher = more resistant but slower",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Low", fontSize = 12.sp)
                    Slider(
                        value = obfuscationLevel.toFloat(),
                        onValueChange = { obfuscationLevel = it.toInt() },
                        valueRange = 0f..3f,
                        steps = 2,
                        modifier = Modifier.weight(1f)
                    )
                    Text("Max", fontSize = 12.sp)
                }
            }
        }

        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        // DNS Section
        SectionHeader("DNS")

        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "DNS Provider",
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    text = "Chinese CDN primary (Cloudflare blocked in Iran)",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                Spacer(modifier = Modifier.height(8.dp))

                val dnsOptions = listOf(
                    "alibaba" to "Alibaba DNS (223.5.5.5)",
                    "tencent" to "Tencent DNS (119.29.29.29)",
                    "tencent-backup" to "Tencent Backup (1.12.12.12)"
                )

                dnsOptions.forEach { (value, label) ->
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 4.dp)
                    ) {
                        RadioButton(
                            selected = dnsProvider == value,
                            onClick = { dnsProvider = value }
                        )
                        Text(text = label, fontSize = 14.sp)
                    }
                }
            }
        }

        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        // General Section
        SectionHeader("General")

        SettingsSwitch(
            title = "Auto Update",
            description = "Check for updates every 6 hours (Alibaba/Tencent CDN)",
            checked = autoUpdateEnabled,
            onCheckedChange = { autoUpdateEnabled = it }
        )

        SettingsSwitch(
            title = "Start on Boot",
            description = "Automatically start VPN when device boots",
            checked = startOnBootEnabled,
            onCheckedChange = { startOnBootEnabled = it }
        )

        // About Section
        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        SectionHeader("About")

        Card(modifier = Modifier.fillMaxWidth()) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text("UnifiedShield v1.0.0", fontSize = 14.sp, fontWeight = FontWeight.Medium)
                Text(
                    "Next-gen anti-censorship VPN for Iran",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    "No root required • Split tunnel • DPI evasion",
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
        }

        Spacer(modifier = Modifier.height(32.dp))
    }
}

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title,
        fontSize = 14.sp,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(start = 4.dp, bottom = 4.dp)
    )
}

@Composable
private fun SettingsSwitch(
    title: String,
    description: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    text = description,
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Switch(
                checked = checked,
                onCheckedChange = onCheckedChange
            )
        }
    }
}
