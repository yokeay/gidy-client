package com.gidy.client.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ArrowDownward
import androidx.compose.material.icons.outlined.ArrowUpward
import androidx.compose.material.icons.outlined.CloudDownload
import androidx.compose.material.icons.outlined.CloudUpload
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gidy.client.R
import com.gidy.client.data.ConnectionLogEntry
import com.gidy.client.data.Format
import com.gidy.client.data.MockStatsEngine
import com.gidy.client.ui.components.AppleCard
import com.gidy.client.ui.components.KpiCard
import com.gidy.client.ui.components.SectionHeader
import com.gidy.client.ui.components.SpeedChart
import com.gidy.client.ui.components.Speedometer
import com.gidy.client.ui.components.ChartPoint
import com.gidy.client.ui.theme.GidyMono
import kotlinx.coroutines.delay

@Composable
fun TrafficMonitorScreen() {
    val engine = remember { MockStatsEngine() }
    var snapshot by remember { mutableStateOf(engine.next(true)) }
    val chart = remember { mutableStateListOf<ChartPoint>() }
    var logs by remember { mutableStateOf<List<ConnectionLogEntry>>(emptyList()) }

    LaunchedEffect(Unit) {
        repeat(30) {
            chart.add(ChartPoint(0.0, 0.0))
        }
        while (true) {
            snapshot = engine.next(true)
            chart.add(ChartPoint(snapshot.speedUpKbps, snapshot.speedDownKbps))
            if (chart.size > 60) chart.removeAt(0)
            logs = engine.log.toList()
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
        // KPIs 2x2
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            KpiCard(
                label = stringResource(R.string.dash_upload),
                value = Format.speed(snapshot.speedUpKbps),
                icon = Icons.Outlined.ArrowUpward,
                modifier = Modifier.weight(1f),
            )
            KpiCard(
                label = stringResource(R.string.dash_download),
                value = Format.speed(snapshot.speedDownKbps),
                icon = Icons.Outlined.ArrowDownward,
                modifier = Modifier.weight(1f),
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            KpiCard(
                label = stringResource(R.string.dash_total_up),
                value = Format.bytes(snapshot.bytesUp),
                icon = Icons.Outlined.CloudUpload,
                modifier = Modifier.weight(1f),
            )
            KpiCard(
                label = stringResource(R.string.dash_total_down),
                value = Format.bytes(snapshot.bytesDown),
                icon = Icons.Outlined.CloudDownload,
                modifier = Modifier.weight(1f),
            )
        }

        // Speedometer
        AppleCard {
            Speedometer(
                speedKbps = snapshot.speedUpKbps + snapshot.speedDownKbps,
                maxKbps = 2400.0,
                label = stringResource(R.string.traffic_realtime),
                valueText = Format.speed(snapshot.speedUpKbps + snapshot.speedDownKbps),
                modifier = Modifier.fillMaxWidth(),
            )
        }

        // Chart
        AppleCard {
            Box(modifier = Modifier.fillMaxWidth().height(180.dp)) {
                SpeedChart(
                    points = chart,
                    uploadLabel = stringResource(R.string.dash_upload),
                    downloadLabel = stringResource(R.string.dash_download),
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }

        // Log
        SectionHeader(text = stringResource(R.string.traffic_log))
        AppleCard(padding = 0.dp) {
            if (logs.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 36.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = stringResource(R.string.traffic_no_data),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                Column {
                    logs.take(30).forEachIndexed { i, entry ->
                        LogRow(entry)
                        if (i != logs.lastIndex.coerceAtMost(29)) {
                            HorizontalDivider()
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun LogRow(entry: ConnectionLogEntry) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 18.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = entry.target,
                style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = "${entry.timestamp} · ${entry.type}",
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = GidyMono.fontFamily),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Text(
            text = Format.bytes(entry.sizeBytes),
            style = MaterialTheme.typography.bodySmall.copy(
                fontFamily = GidyMono.fontFamily,
                fontFeatureSettings = "tnum",
            ),
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun HorizontalDivider() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 18.dp)
            .height(0.5.dp)
            .androidx.compose.foundation.background(MaterialTheme.colorScheme.outline),
    )
}

