package com.gidy.client.data

import kotlinx.serialization.Serializable

@Serializable
data class GidyConfig(
    val serverAddr: String = "127.0.0.1",
    val serverPort: Int = 443,
    val pskHex: String = "",
    val protocol: String = "gidy",
    val socks5Addr: String = "127.0.0.1",
    val socks5Port: Int = 1080,
    val httpAddr: String = "127.0.0.1",
    val httpPort: Int = 8080,
    val routingMode: String = "global",        // "global" | "pac"
    val autoStart: Boolean = false,
    val autoConnect: Boolean = false,
    val keepScreenOn: Boolean = false,
    val logRetentionDays: Int = 7,
    val theme: String = "system",              // "system" | "light" | "dark"
    val language: String = "system",           // "system" | "zh" | "en"
)
