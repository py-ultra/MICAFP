package com.unifiedshield.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.unifiedshield.ui.theme.UnifiedShieldTheme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    onConnectClick: () -> Unit,
    onDisconnectClick: () -> Unit
) {
    var isConnected by remember { mutableStateOf(false) }
    var selectedTab by remember { mutableIntStateOf(0) }

    val tabs = listOf("Status", "Cores", "Settings")

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("UnifiedShield") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                )
            )
        },
        bottomBar = {
            NavigationBar {
                tabs.forEachIndexed { index, title ->
                    NavigationBarItem(
                        icon = {
                            when (index) {
                                0 -> Icon(
                                    painter = androidx.compose.material.icons.Icons.Default.Shield,
                                    contentDescription = title
                                )
                                1 -> Icon(
                                    painter = androidx.compose.material.icons.Icons.Default.SwapHoriz,
                                    contentDescription = title
                                )
                                2 -> Icon(
                                    painter = androidx.compose.material.icons.Icons.Default.Settings,
                                    contentDescription = title
                                )
                            }
                        },
                        label = { Text(title) },
                        selected = selectedTab == index,
                        onClick = { selectedTab = index }
                    )
                }
            }
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            when (selectedTab) {
                0 -> {
                    StatusCard(
                        isConnected = isConnected,
                        onConnectClick = {
                            isConnected = true
                            onConnectClick()
                        },
                        onDisconnectClick = {
                            isConnected = false
                            onDisconnectClick()
                        }
                    )
                }
                1 -> {
                    CoreSwitcher(
                        currentCore = "xray",
                        onCoreSelected = { core ->
                            // Handle core switch
                        }
                    )
                }
                2 -> {
                    SettingsScreen()
                }
            }
        }
    }
}
