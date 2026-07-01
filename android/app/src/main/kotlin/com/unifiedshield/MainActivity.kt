package com.unifiedshield

import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import com.unifiedshield.ui.MainScreen
import com.unifiedshield.ui.theme.UnifiedShieldTheme

class MainActivity : ComponentActivity() {

    private val vpnPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK) {
            startVpnService()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            UnifiedShieldTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen(
                        onConnectClick = { requestVpnPermission() },
                        onDisconnectClick = { stopVpnService() }
                    )
                }
            }
        }
    }

    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)
        if (intent != null) {
            vpnPermissionLauncher.launch(intent)
        } else {
            // Permission already granted
            startVpnService()
        }
    }

    private fun startVpnService() {
        val intent = Intent(this, VpnService::class.java).apply {
            action = VpnService.ACTION_START
        }
        startService(intent)
    }

    private fun stopVpnService() {
        val intent = Intent(this, VpnService::class.java).apply {
            action = VpnService.ACTION_STOP
        }
        startService(intent)
    }

    companion object {
        const val ACTION_START = "com.unifiedshield.START"
        const val ACTION_STOP = "com.unifiedshield.STOP"
    }
}
