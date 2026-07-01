package com.unifiedshield.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class CoreInfo(
    val id: String,
    val name: String,
    val description: String,
    val protocol: String,
    val obfuscation: Boolean,
    val recommended: Boolean
)

@Composable
fun CoreSwitcher(
    currentCore: String,
    onCoreSelected: (String) -> Unit
) {
    var selectedCore by remember { mutableStateOf(currentCore) }
    var isSwitching by remember { mutableStateOf(false) }

    val cores = listOf(
        CoreInfo(
            id = "xray",
            name = "Xray",
            description = "VLESS/VMess with XTLS. Best for general use.",
            protocol = "VLESS/VMess",
            obfuscation = true,
            recommended = true
        ),
        CoreInfo(
            id = "naive",
            name = "NaïveProxy",
            description = "Chrome network stack. Anti-DPI with domain fronting.",
            protocol = "HTTP/2",
            obfuscation = true,
            recommended = false
        ),
        CoreInfo(
            id = "hysteria2",
            name = "Hysteria2",
            description = "QUIC-based. Fast on unstable connections.",
            protocol = "QUIC",
            obfuscation = false,
            recommended = false
        ),
        CoreInfo(
            id = "tuic",
            name = "TUIC",
            description = "QUIC proxy with multiplexing. Low overhead.",
            protocol = "QUIC",
            obfuscation = false,
            recommended = false
        )
    )

    Column(
        modifier = Modifier.fillMaxWidth()
    ) {
        Text(
            text = "Protocol Core",
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 8.dp)
        )

        Text(
            text = "Switch cores for DPI evasion. Each core uses different protocols and obfuscation.",
            fontSize = 13.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        // DPI warning banner
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 16.dp),
            colors = CardDefaults.cardColors(
                containerColor = Color(0xFFFFF3E0)
            )
        ) {
            Row(
                modifier = Modifier.padding(12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    painter = androidx.compose.material.icons.Icons.Default.Warning,
                    contentDescription = "Warning",
                    tint = Color(0xFFFF9800),
                    modifier = Modifier.size(20.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = "Auto-switch triggers when DPI score > 0.72",
                    fontSize = 13.sp,
                    color = Color(0xFFE65100)
                )
            }
        }

        LazyColumn(
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(cores) { core ->
                CoreCard(
                    core = core,
                    isSelected = selectedCore == core.id,
                    isSwitching = isSwitching,
                    onSelect = {
                        if (selectedCore != core.id) {
                            selectedCore = core.id
                            isSwitching = true
                            onCoreSelected(core.id)
                            // Simulate switch delay
                            isSwitching = false
                        }
                    }
                )
            }
        }
    }
}

@Composable
private fun CoreCard(
    core: CoreInfo,
    isSelected: Boolean,
    isSwitching: Boolean,
    onSelect: () -> Unit
) {
    val borderColor = if (isSelected) {
        MaterialTheme.colorScheme.primary
    } else {
        MaterialTheme.colorScheme.outlineVariant
    }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .border(
                width = if (isSelected) 2.dp else 1.dp,
                color = borderColor,
                shape = RoundedCornerShape(12.dp)
            )
            .clickable(onClick = onSelect),
        shape = RoundedCornerShape(12.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Radio button
            RadioButton(
                selected = isSelected,
                onClick = onSelect
            )

            Spacer(modifier = Modifier.width(12.dp))

            // Core info
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = core.name,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                    if (core.recommended) {
                        Spacer(modifier = Modifier.width(8.dp))
                        AssistChip(
                            onClick = { },
                            label = { Text("Recommended", fontSize = 10.sp) },
                            modifier = Modifier.height(24.dp)
                        )
                    }
                }
                Text(
                    text = core.description,
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 2.dp)
                )
                Row(
                    modifier = Modifier.padding(top = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    AssistChip(
                        onClick = { },
                        label = { Text(core.protocol, fontSize = 10.sp) },
                        modifier = Modifier.height(22.dp)
                    )
                    if (core.obfuscation) {
                        AssistChip(
                            onClick = { },
                            label = { Text("Obfuscation", fontSize = 10.sp) },
                            modifier = Modifier.height(22.dp)
                        )
                    }
                }
            }

            // Switching indicator
            if (isSelected && isSwitching) {
                CircularProgressIndicator(
                    modifier = Modifier.size(24.dp),
                    strokeWidth = 2.dp
                )
            }
        }
    }
}
