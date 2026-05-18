package com.gidy.client.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ArrowDownward
import androidx.compose.material.icons.outlined.ArrowUpward
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material.icons.outlined.Schedule
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gidy.client.R
import com.gidy.client.data.Format
import com.gidy.client.data.GidyConfig
import com.gidy.client.data.MockStatsEngine
import androidx.compose.foundation.clickable
import androidx.compose.material3.ripple
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.ui.res.stringResource
import com.gidy.client.ui.components.AppleCard
import com.gidy.client.ui.components.KpiCard
import com.gidy.client.ui.theme.GidyMono
import kotlinx.coroutines.delay

@Composable
fun DashboardScreen(config: GidyConfig) {
    var running by remember { mutableStateOf(config.autoConnect) }
    val engine = remember { MockStatsEngine() }
    var snapshot by remember { mutableStateOf(engine.next(false)) }

    LaunchedEffect(running) {
        while (true) {
            snapshot = engine.next(running)
            delay(1000)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        ConnectionHero(
            running = running,
            onToggle = { running = !running },
        )

        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            KpiCard(
                label = stringResource(R.string.dash_upload),
                value = Format.speed(snapshot.speedUpKbps),
                icon = Icons.Outlined.ArrowUpward,
                subtitle = "${stringResource(R.string.dash_total_up)} · ${Format.bytes(snapshot.bytesUp)}",
                modifier = Modifier.weight(1f),
            )
            KpiCard(
                label = stringResource(R.string.dash_download),
                value = Format.speed(snapshot.speedDownKbps),
                icon = Icons.Outlined.ArrowDownward,
                subtitle = "${stringResource(R.string.dash_total_down)} · ${Format.bytes(snapshot.bytesDown)}",
                modifier = Modifier.weight(1f),
            )
        }

        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            KpiCard(
                label = stringResource(R.string.dash_active_conn),
                value = snapshot.activeConnections.toString(),
                icon = Icons.Outlined.Bolt,
                modifier = Modifier.weight(1f),
            )
            KpiCard(
                label = stringResource(R.string.dash_uptime),
                value = Format.uptime(snapshot.uptimeSecs),
                icon = Icons.Outlined.Schedule,
                modifier = Modifier.weight(1f),
            )
        }

        AppleCard {
            Text(
                text = stringResource(R.string.dash_dns),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(6.dp))
            Text(
                text = if (running) "12 ms" else "—",
                style = MaterialTheme.typography.headlineMedium.copy(
                    fontFamily = GidyMono.fontFamily,
                    fontFeatureSettings = "tnum",
                    fontWeight = FontWeight.SemiBold,
                ),
            )
        }
    }
}

@Composable
private fun ConnectionHero(
    running: Boolean,
    onToggle: () -> Unit,
) {
    val bg = MaterialTheme.colorScheme.onSurface
    val fg = MaterialTheme.colorScheme.surface
    val shape = RoundedCornerShape(22.dp)
    val interaction = remember { MutableInteractionSource() }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(bg)
            .clickable(
                interactionSource = interaction,
                indication = ripple(color = fg),
            ) { onToggle() }
            .padding(24.dp),
    ) {
        Column {
            Text(
                text = stringResource(R.string.app_name).uppercase(),
                style = MaterialTheme.typography.labelMedium.copy(letterSpacing = 3.sp),
                color = fg.copy(alpha = 0.6f),
            )
            Spacer(Modifier.height(8.dp))
            Text(
                text = if (running)
                    stringResource(R.string.dash_status_connected)
                else
                    stringResource(R.string.dash_status_disconnected),
                style = MaterialTheme.typography.displayLarge.copy(
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 30.sp,
                ),
                color = fg,
            )
            Spacer(Modifier.height(14.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    imageVector = if (running) Icons.Outlined.Bolt else Icons.Outlined.Bolt,
                    contentDescription = null,
                    tint = fg.copy(alpha = 0.7f),
                    modifier = Modifier.size(16.dp),
                )
                Spacer(Modifier.width(6.dp))
                Text(
                    text = if (running)
                        stringResource(R.string.dash_disconnect)
                    else
                        stringResource(R.string.dash_connect),
                    style = MaterialTheme.typography.labelLarge,
                    color = fg.copy(alpha = 0.85f),
                )
            }
        }
    }

}
