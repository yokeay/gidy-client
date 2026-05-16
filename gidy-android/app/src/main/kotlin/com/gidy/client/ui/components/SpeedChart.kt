package com.gidy.client.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

data class ChartPoint(val up: Double, val down: Double)

@Composable
fun SpeedChart(
    points: List<ChartPoint>,
    uploadLabel: String,
    downloadLabel: String,
    modifier: Modifier = Modifier,
) {
    val active = MaterialTheme.colorScheme.onSurface
    val muted = MaterialTheme.colorScheme.onSurfaceVariant
    val grid = MaterialTheme.colorScheme.outline.copy(alpha = 0.5f)

    Column(modifier = modifier) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            LegendDot(color = active, label = downloadLabel)
            LegendDot(color = muted, label = uploadLabel)
        }
        Spacer(Modifier.height(10.dp))
        Canvas(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f, fill = true),
        ) {
            val w = size.width
            val h = size.height
            if (points.size < 2) return@Canvas

            for (i in 1..3) {
                val y = h * i / 4
                drawLine(
                    color = grid,
                    start = Offset(0f, y),
                    end = Offset(w, y),
                    strokeWidth = 0.8f,
                    pathEffect = PathEffect.dashPathEffect(floatArrayOf(4f, 6f)),
                )
            }

            val maxVal = (points.maxOf { maxOf(it.up, it.down) }).coerceAtLeast(50.0)
            val step = w / (points.size - 1).coerceAtLeast(1)

            fun pointAt(i: Int, v: Double): Offset {
                val x = i * step
                val y = h - (v / maxVal * h * 0.92).toFloat()
                return Offset(x, y)
            }

            // Download (solid + fill)
            val downPath = Path()
            val downFill = Path()
            points.forEachIndexed { i, p ->
                val o = pointAt(i, p.down)
                if (i == 0) {
                    downPath.moveTo(o.x, o.y)
                    downFill.moveTo(o.x, h)
                    downFill.lineTo(o.x, o.y)
                } else {
                    downPath.lineTo(o.x, o.y)
                    downFill.lineTo(o.x, o.y)
                }
            }
            downFill.lineTo(w, h)
            downFill.close()
            drawPath(
                downFill,
                brush = Brush.verticalGradient(
                    colors = listOf(active.copy(alpha = 0.25f), active.copy(alpha = 0f)),
                ),
            )
            drawPath(
                downPath,
                color = active,
                style = Stroke(width = 2.4f, cap = StrokeCap.Round),
            )

            // Upload (dashed)
            val upPath = Path()
            points.forEachIndexed { i, p ->
                val o = pointAt(i, p.up)
                if (i == 0) upPath.moveTo(o.x, o.y) else upPath.lineTo(o.x, o.y)
            }
            drawPath(
                upPath,
                color = muted,
                style = Stroke(
                    width = 1.8f,
                    cap = StrokeCap.Round,
                    pathEffect = PathEffect.dashPathEffect(floatArrayOf(6f, 4f)),
                ),
            )
        }
    }
}

@Composable
private fun LegendDot(color: Color, label: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(
            modifier = Modifier
                .size(8.dp)
                .clip(CircleShape)
                .background(color),
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
