package com.gidy.client.data

import kotlinx.serialization.Serializable

@Serializable
data class GidyConfig(
    val pskHex: String = "",
    val serverAddr: String = "gidy.eu.cc",
    val serverPort: Int = 443,
    val serverName: String = "gidy.eu.cc",
    val wsUrl: String = "wss://gidy.eu.cc/ws",
    val echConfigBase64: String = "",
    val echToken: String = "",
    val socks5Addr: String = "127.0.0.1",
    val socks5Port: Int = 5555,
    val httpAddr: String = "127.0.0.1",
    val httpPort: Int = 5556,
    val protocol: String = "ws",              // "ws" | "h2" | "h3" | "quic"
    val routingMode: String = "global",        // "global" | "pac"
    val autoStart: Boolean = false,
    val autoConnect: Boolean = false,
    val keepScreenOn: Boolean = false,
    val logRetentionDays: Int = 7,
    val theme: String = "system",              // "system" | "light" | "dark"
    val themeColor: String = "blue",           // "blue" | "green" | "purple" | "orange"
    val language: String = "system",           // "system" | "zh" | "en"
    val logLevel: String = "info",             // "trace" | "debug" | "info" | "warn" | "error"
)
