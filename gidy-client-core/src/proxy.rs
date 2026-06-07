use crate::client::Connection;
use crate::stats::TrafficStats;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

/// A single connection log entry, shared between proxy servers and the GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Target address in host:port form
    pub target: String,
    /// "SOCKS5" or "HTTP"
    pub protocol: String,
    /// ISO-8601 timestamp of when the connection was established
    pub connected_at: String,
}

/// Shared connection log buffer used by both proxy servers.
pub type SharedLogBuffer = Arc<Mutex<Vec<LogEntry>>>;

/// Create a new shared log buffer (capacity-capped at 1000 entries).
pub fn new_log_buffer() -> SharedLogBuffer {
    Arc::new(Mutex::new(Vec::with_capacity(64)))
}

/// Push a log entry, capping the buffer at 1000 entries (drop oldest).
fn push_log(logs: &SharedLogBuffer, entry: LogEntry) {
    let mut buf = logs.lock();
    if buf.len() >= 1000 {
        buf.remove(0);
    }
    buf.push(entry);
}

/// Return the current local time as a "YYYY-MM-DD HH:MM:SS" string
/// without depending on chrono (uses std only).
fn now_iso() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Calculate date from epoch seconds (simplified Gregorian calendar math)
    let (year, month, day, hour, minute, second) = epoch_to_local(secs);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

/// Convert UNIX epoch seconds to (year, month, day, hour, minute, second)
/// using UTC (no timezone offset — good enough for log display).
fn epoch_to_local(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let second = secs % 60;
    let mins = secs / 60;
    let minute = mins % 60;
    let hrs = mins / 60;
    let hour = hrs % 24;
    let days = hrs / 24;

    // Days since 1970-01-01 → year/month/day
    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let dy = if is_leap(y) { 366 } else { 365 };
        if remaining < dy {
            break;
        }
        remaining -= dy;
        y += 1;
    }
    let leap = is_leap(y);
    let mdays: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0u64;
    for &d in &mdays {
        if remaining < d {
            break;
        }
        remaining -= d;
        m += 1;
    }
    (y, m + 1, remaining + 1, hour, minute, second)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

pub struct Socks5Server {
    listen_addr: String,
    conn: Arc<Connection>,
    stats: Arc<TrafficStats>,
    logs: SharedLogBuffer,
}

impl Socks5Server {
    pub fn new(
        listen_addr: String,
        conn: Connection,
        stats: Arc<TrafficStats>,
        logs: SharedLogBuffer,
    ) -> Self {
        Self {
            listen_addr,
            conn: Arc::new(conn),
            stats,
            logs,
        }
    }

    /// Create from an already-Arc-wrapped Connection (for sharing between SOCKS5 and HTTP proxy).
    pub fn from_arc(
        listen_addr: String,
        conn: Arc<Connection>,
        stats: Arc<TrafficStats>,
        logs: SharedLogBuffer,
    ) -> Self {
        Self {
            listen_addr,
            conn,
            stats,
            logs,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let listener = TcpListener::bind(&self.listen_addr)
            .await
            .map_err(|e| format!("bind {}: {}", self.listen_addr, e))?;

        info!("SOCKS5 proxy listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("new connection from {}", addr);
                    let conn = self.conn.clone();
                    let stats = self.stats.clone();
                    let logs = self.logs.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_socks5(stream, conn, stats, logs).await {
                            warn!("socks5 error for {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_socks5(
    mut stream: tokio::net::TcpStream,
    conn: Arc<Connection>,
    stats: Arc<TrafficStats>,
    logs: SharedLogBuffer,
) -> Result<(), String> {
    let mut buf = vec![0u8; 4096];

    let mut buf_len = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("read greeting: {}", e))?;

    if buf_len < 3 || buf[0] != 0x05 {
        return Err("invalid socks5 greeting".into());
    }

    let nmethods = buf[1] as usize;
    let greeting_len = 2 + nmethods;

    stream
        .write_all(&[0x05, 0x00])
        .await
        .map_err(|e| format!("write method select: {}", e))?;

    let request_start = greeting_len;
    while buf_len < request_start + 10 {
        let n = stream
            .read(&mut buf[buf_len..])
            .await
            .map_err(|e| format!("read request: {}", e))?;
        if n == 0 {
            return Err("client closed before request".into());
        }
        buf_len += n;
    }

    if buf[request_start] != 0x05 {
        return Err("invalid socks5 request".into());
    }

    let cmd = buf[request_start + 1];
    if cmd != 0x01 {
        send_reply(&mut stream, 0x07).await?;
        return Err(format!("unsupported cmd: {}", cmd));
    }

    let atyp = buf[request_start + 3];
    let (target, port, header_len) = match atyp {
        0x01 => {
            let a = &buf[request_start..];
            let addr = format!("{}.{}.{}.{}", a[4], a[5], a[6], a[7]);
            let port = u16::from_be_bytes([a[8], a[9]]);
            (addr, port, 10)
        }
        0x03 => {
            let a = &buf[request_start..];
            let domain_len = a[4] as usize;
            let total = 5 + domain_len + 2;
            while buf_len < request_start + total {
                let n = stream
                    .read(&mut buf[buf_len..])
                    .await
                    .map_err(|e| format!("read domain: {}", e))?;
                if n == 0 {
                    return Err("client closed during domain read".into());
                }
                buf_len += n;
            }
            let a = &buf[request_start..];
            let domain = std::str::from_utf8(&a[5..5 + domain_len])
                .map_err(|e| format!("invalid domain: {}", e))?;
            let port = u16::from_be_bytes([a[5 + domain_len], a[6 + domain_len]]);
            (domain.to_string(), port, total)
        }
        0x04 => {
            let a = &buf[request_start..];
            let addr = format!(
                "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11],
                a[12], a[13], a[14], a[15], a[16], a[17], a[18], a[19],
            );
            let port = u16::from_be_bytes([a[20], a[21]]);
            (addr, port, 22)
        }
        _ => {
            send_reply(&mut stream, 0x08).await?;
            return Err(format!("unsupported atyp: {}", atyp));
        }
    };

    let target_str = format!("{}:{}", target, port);

    info!("socks5 CONNECT to {}", target_str);

    let tunnel = match conn.connect_tcp(&target, port).await {
        Ok(t) => {
            // Connection succeeded — push a log entry
            push_log(&logs, LogEntry {
                target: target_str.clone(),
                protocol: "SOCKS5".to_string(),
                connected_at: now_iso(),
            });
            t
        }
        Err(e) => {
            let _ = send_reply(&mut stream, 0x01).await;
            return Err(format!("tunnel {}: {}", target_str, e));
        }
    };

    stats.add_up(target_str.len() as u64);

    send_reply(&mut stream, 0x00).await?;

    // Any data read beyond the SOCKS5 header is prefix data for the relay
    let prefix_end = request_start + header_len;
    let prefix = if buf_len > prefix_end {
        debug!("socks5 prefix: {} bytes extra after header", buf_len - prefix_end);
        Some(&buf[prefix_end..buf_len])
    } else {
        None
    };

    if let Err(e) = tunnel.relay(stream, prefix).await {
        warn!("relay error for {}: {}", target_str, e);
    }

    debug!("socks5 session for {} finished", target_str);
    Ok(())
}

async fn send_reply(stream: &mut tokio::net::TcpStream, rep: u8) -> Result<(), String> {
    let reply = [0x05, rep, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
    stream
        .write_all(&reply)
        .await
        .map_err(|e| format!("write reply: {}", e))
}

// ── HTTP CONNECT proxy ──────────────────────────────────────────────────────

pub struct HttpProxyServer {
    listen_addr: String,
    conn: Arc<Connection>,
    stats: Arc<TrafficStats>,
    logs: SharedLogBuffer,
}

impl HttpProxyServer {
    pub fn new(
        listen_addr: String,
        conn: Connection,
        stats: Arc<TrafficStats>,
        logs: SharedLogBuffer,
    ) -> Self {
        Self {
            listen_addr,
            conn: Arc::new(conn),
            stats,
            logs,
        }
    }

    /// Create from an already-Arc-wrapped Connection.
    pub fn from_arc(
        listen_addr: String,
        conn: Arc<Connection>,
        stats: Arc<TrafficStats>,
        logs: SharedLogBuffer,
    ) -> Self {
        Self {
            listen_addr,
            conn,
            stats,
            logs,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let listener = TcpListener::bind(&self.listen_addr)
            .await
            .map_err(|e| format!("bind {}: {}", self.listen_addr, e))?;

        info!("HTTP proxy listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("http proxy: new connection from {}", addr);
                    let conn = self.conn.clone();
                    let stats = self.stats.clone();
                    let logs = self.logs.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_http_connect(stream, conn, stats, logs).await {
                            warn!("http proxy error for {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("http proxy accept error: {}", e);
                }
            }
        }
    }
}

/// Handle an HTTP CONNECT request.
/// Example request: `CONNECT www.google.com:443 HTTP/1.1\r\nHost: www.google.com:443\r\n\r\n`
async fn handle_http_connect(
    mut stream: tokio::net::TcpStream,
    conn: Arc<Connection>,
    stats: Arc<TrafficStats>,
    logs: SharedLogBuffer,
) -> Result<(), String> {
    let mut buf = vec![0u8; 8192];
    let mut total = 0;

    // Read HTTP request header until \r\n\r\n
    loop {
        let n = stream
            .read(&mut buf[total..])
            .await
            .map_err(|e| format!("read http request: {}", e))?;
        if n == 0 {
            return Err("client closed before request".into());
        }
        total += n;
        if total >= 4 && &buf[total - 4..total] == b"\r\n\r\n" {
            break;
        }
        if total >= buf.len() {
            return Err("http request too large".into());
        }
    }

    let request_str = std::str::from_utf8(&buf[..total])
        .map_err(|e| format!("invalid utf8 in http request: {}", e))?;

    // Parse: CONNECT host:port HTTP/1.1
    let first_line = request_str.lines().next().unwrap_or("");
    if !first_line.starts_with("CONNECT ") {
        // Not a CONNECT request — respond with 400
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(resp.as_bytes()).await.map_err(|e| format!("write 400: {}", e))?;
        return Err(format!("not a CONNECT request: {}", first_line));
    }

    // Extract host:port from "CONNECT host:port HTTP/1.1"
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("malformed CONNECT request".into());
    }
    let target = parts[1]; // e.g. "www.google.com:443"

    // Parse host and port
    let (host, port) = if let Some(colon_pos) = target.rfind(':') {
        let h = &target[..colon_pos];
        let p: u16 = target[colon_pos + 1..]
            .parse()
            .unwrap_or(443);
        (h.to_string(), p)
    } else {
        (target.to_string(), 443)
    };

    info!("http CONNECT to {}:{}", host, port);

    // Connect through the tunnel
    let tunnel = match conn.connect_tcp(&host, port).await {
        Ok(t) => {
            // Connection succeeded — push a log entry
            push_log(&logs, LogEntry {
                target: format!("{}:{}", host, port),
                protocol: "HTTP".to_string(),
                connected_at: now_iso(),
            });
            t
        }
        Err(e) => {
            let resp = "HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            return Err(format!("tunnel {}:{}", host, e));
        }
    };

    stats.add_up(target.len() as u64);

    // Send 200 Connection Established
    let resp = "HTTP/1.1 200 Connection Established\r\n\r\n";
    stream
        .write_all(resp.as_bytes())
        .await
        .map_err(|e| format!("write 200: {}", e))?;

    // Relay data between client and tunnel
    if let Err(e) = tunnel.relay(stream, None).await {
        warn!("http relay error for {}: {}", target, e);
    }

    debug!("http proxy session for {} finished", target);
    Ok(())
}
