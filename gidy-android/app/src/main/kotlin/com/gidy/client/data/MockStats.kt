package com.gidy.client.data

import kotlin.math.sin
import kotlin.random.Random

data class StatsSnapshot(
    val bytesUp: Long,
    val bytesDown: Long,
    val speedUpKbps: Double,
    val speedDownKbps: Double,
    val uptimeSecs: Long,
    val activeConnections: Int,
)

data class ConnectionLogEntry(
    val timestamp: String,
    val target: String,
    val type: String,
    val sizeBytes: Long,
)

/**
 * Deterministic-ish mock generator that creates plausible-looking metrics
 * driven by `tick` so previews and demo mode are stable but lively.
 */
class MockStatsEngine {
    private var tick = 0L
    private var totalUp = 0L
    private var totalDown = 0L
    private val rng = Random(System.currentTimeMillis())
    val log = ArrayDeque<ConnectionLogEntry>()
    private val types = listOf("HTTPS", "HTTP", "SOCKS5", "TLS")

    fun next(running: Boolean): StatsSnapshot {
        tick++
        if (!running) {
            return StatsSnapshot(totalUp, totalDown, 0.0, 0.0, 0, 0)
        }
        val phase = tick * 0.18
        val baseDown = 700 + 600 * (0.5 + 0.5 * sin(phase))
        val baseUp = 180 + 140 * (0.5 + 0.5 * sin(phase + 1.7))
        val jitter = rng.nextDouble(-30.0, 30.0)
        val down = (baseDown + jitter).coerceAtLeast(0.0)
        val up = (baseUp + jitter * 0.6).coerceAtLeast(0.0)
        totalDown += (down * 1024 / 8).toLong()
        totalUp += (up * 1024 / 8).toLong()

        if (tick % 4 == 0L) {
            val host = listOf("api.example.com", "cdn.assets.net", "edge-7.svc", "static.fastly.io", "telemetry.local").random(rng)
            log.addFirst(
                ConnectionLogEntry(
                    timestamp = nowHms(),
                    target = host,
                    type = types.random(rng),
                    sizeBytes = rng.nextLong(800L, 1_200_000L),
                )
            )
            while (log.size > 80) log.removeLast()
        }

        return StatsSnapshot(
            bytesUp = totalUp,
            bytesDown = totalDown,
            speedUpKbps = up,
            speedDownKbps = down,
            uptimeSecs = tick,
            activeConnections = (12 + (sin(phase) * 8)).toInt().coerceAtLeast(1),
        )
    }

    private fun nowHms(): String {
        val sec = (System.currentTimeMillis() / 1000) % 86400
        val h = sec / 3600
        val m = (sec / 60) % 60
        val s = sec % 60
        return "%02d:%02d:%02d".format(h, m, s)
    }
}

object Format {
    fun bytes(b: Long): String {
        if (b < 1024) return "$b B"
        val units = listOf("KB", "MB", "GB", "TB")
        var v = b.toDouble() / 1024
        var i = 0
        while (v >= 1024 && i < units.lastIndex) {
            v /= 1024; i++
        }
        return "%.2f %s".format(v, units[i])
    }

    fun speed(kbps: Double): String {
        if (kbps < 1024) return "%.0f KB/s".format(kbps)
        val mb = kbps / 1024
        if (mb < 1024) return "%.2f MB/s".format(mb)
        return "%.2f GB/s".format(mb / 1024)
    }

    fun uptime(secs: Long): String {
        val h = secs / 3600
        val m = (secs / 60) % 60
        val s = secs % 60
        return "%02d:%02d:%02d".format(h, m, s)
    }
}
