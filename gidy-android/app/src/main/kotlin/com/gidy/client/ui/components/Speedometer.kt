package com.gidy.client.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gidy.client.ui.theme.GidyMono
import kotlin.math.cos
import kotlin.math.sin

/**
 * 270° sweep arc: starts at 135° (bottom-left), sweeps clockwise to 45° (bottom-right).
 */
@Composable
fun Speedometer(
    speedKbps: Double,
    maxKbps: Double = 2000.0,
    label: String,
    valueText: String,
    modifier: Modifier = Modifier,
) {
    val ratio = (speedKbps / maxKbps).coerceIn(0.0, 1.0)
    val sweep = (ratio * 270f).toFloat()
    val track = MaterialTheme.colorScheme.surfaceVariant
    val active = MaterialTheme.colorScheme.onSurface
    val tickMajor = MaterialTheme.colorScheme.onSurfaceVariant
    val tickMinor = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.35f)

    Box(modifier = modifier.aspectRatio(1.25f), contentAlignment = Alignment.Center) {
        val density = LocalDensity.current
        val strokePx = with(density) { 10.dp.toPx() }

        Canvas(modifier = Modifier.fillMaxSize()) {
            val w = size.width
            val h = size.height
            val pad = strokePx * 1.6f
            val side = minOf(w, h) - pad * 2
            val topLeft = Offset((w - side) / 2 + pad, (h - side) / 2 + pad)
            val arcSize = Size(side, side)

            drawArc(
                color = track,
                startAngle = 135f,
                sweepAngle = 270f,
                useCenter = false,
                topLeft = topLeft,
                size = arcSize,
                style = Stroke(width = strokePx, cap = StrokeCap.Round),
            )
            if (sweep > 0f) {
                drawArc(
                    color = active,
                    startAngle = 135f,
                    sweepAngle = sweep,
                    useCenter = false,
                    topLeft = topLeft,
                    size = arcSize,
                    style = Stroke(width = strokePx, cap = StrokeCap.Round),
                )
            }

            // Ticks
            val cx = w / 2
            val cy = h / 2
            val rOuter = side / 2 - strokePx * 0.8f
            for (i in 0..18) {
                val isMajor = i % 3 == 0
                val angleDeg = 135f + i * (270f / 18f)
                val rad = Math.toRadians(angleDeg.toDouble())
                val len = if (isMajor) strokePx * 1.5f else strokePx * 0.8f
                val rIn = rOuter - len
                val x1 = cx + rIn * cos(rad).toFloat()
                val y1 = cy + rIn * sin(rad).toFloat()
                val x2 = cx + rOuter * cos(rad).toFloat()
                val y2 = cy + rOuter * sin(rad).toFloat()
                drawLine(
                    color = if (isMajor) tickMajor else tickMinor,
                    start = Offset(x1, y1),
                    end = Offset(x2, y2),
                    strokeWidth = if (isMajor) 2.2f else 1.4f,
                    cap = StrokeCap.Round,
                )
            }
        }

        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                text = valueText,
                style = MaterialTheme.typography.displayMedium.copy(
                    fontFamily = GidyMono.fontFamily,
                    fontFeatureSettings = "tnum",
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 28.sp,
                ),
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(6.dp))
            Text(
                text = label.uppercase(),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

    }
}
