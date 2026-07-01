package com.unifiedshield.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun StatusCard(
    isConnected: Boolean,
    onConnectClick: () -> Unit,
    onDisconnectClick: () -> Unit
) {
    var uploadSpeed by remember { mutableStateOf("0 KB/s") }
    var downloadSpeed by remember { mutableStateOf("0 KB/s") }
    var currentCore by remember { mutableStateOf("Xray") }
    var connectionTime by remember { mutableStateOf("00:00:00") }

    // Pulse animation for connected state
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseAlpha by infiniteTransition.animateFloat(
        initialValue = 0.3f,
        targetValue = 0.8f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = EaseInOutSine),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulseAlpha"
    )

    val connectedColor = Color(0xFF4CAF50)
    val disconnectedColor = Color(0xFF9E9E9E)
    val statusColor = if (isConnected) connectedColor else disconnectedColor

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Connection indicator circle
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.size(160.dp)
            ) {
                // Animated pulse ring
                if (isConnected) {
                    Canvas(modifier = Modifier.size(160.dp)) {
                        drawCircle(
                            color = connectedColor.copy(alpha = pulseAlpha),
                            radius = 75f,
                            center = Offset(size.width / 2, size.height / 2)
                        )
                    }
                }

                // Main circle
                Surface(
                    modifier = Modifier.size(120.dp),
                    shape = CircleShape,
                    color = statusColor.copy(alpha = 0.15f),
                    border = androidx.compose.foundation.BorderStroke(3.dp, statusColor)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            painter = androidx.compose.material.icons.Icons.Default.Shield,
                            contentDescription = "Status",
                            modifier = Modifier.size(48.dp),
                            tint = statusColor
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Status text
            Text(
                text = if (isConnected) "Connected" else "Disconnected",
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold,
                color = statusColor
            )

            if (isConnected) {
                Text(
                    text = "Core: $currentCore",
                    fontSize = 14.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = "Time: $connectionTime",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            // Speed indicators
            if (isConnected) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    SpeedIndicator(
                        label = "Download",
                        speed = downloadSpeed,
                        color = Color(0xFF2196F3)
                    )
                    SpeedIndicator(
                        label = "Upload",
                        speed = uploadSpeed,
                        color = Color(0xFFFF9800)
                    )
                }
                Spacer(modifier = Modifier.height(16.dp))
            }

            // Connect/Disconnect button
            Button(
                onClick = if (isConnected) onDisconnectClick else onConnectClick,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(52.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = if (isConnected) {
                        MaterialTheme.colorScheme.error
                    } else {
                        connectedColor
                    }
                ),
                shape = MaterialTheme.shapes.large
            ) {
                Text(
                    text = if (isConnected) "DISCONNECT" else "CONNECT",
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}

@Composable
private fun SpeedIndicator(
    label: String,
    speed: String,
    color: Color
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = label,
            fontSize = 12.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            text = speed,
            fontSize = 18.sp,
            fontWeight = FontWeight.SemiBold,
            color = color
        )
    }
}
