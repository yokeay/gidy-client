use crate::client::Connection;
use crate::stats::TrafficStats;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

pub struct Socks5Server {
    listen_addr: String,
    conn: Arc<Connection>,
    stats: Arc<TrafficStats>,
}

impl Socks5Server {
    pub fn new(
        listen_addr: String,
        conn: Connection,
        stats: Arc<TrafficStats>,
    ) -> Self {
        Self {
            listen_addr,
            conn: Arc::new(conn),
            stats,
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

                    tokio::spawn(async move {
                        if let Err(e) = handle_socks5(stream, conn, stats).await {
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
) -> Result<(), String> {
    let mut buf = vec![0u8; 4096];
    let mut buf_len = 0usize;

    // Read initial data (may contain greeting + request in one packet)
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("read greeting: {}", e))?;
    buf_len = n;

    if n < 3 || buf[0] != 0x05 {
        return Err(format!("invalid socks5 greeting: {:?}", &buf[..n.min(8)]));
    }

    let nmethods = buf[1] as usize;
    let greeting_len = 2 + nmethods;

    // 2. Select no-auth method
    stream
        .write_all(&[0x05, 0x00])
        .await
        .map_err(|e| format!("write method select: {}", e))?;

    // Ensure we have enough data for the request header
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
                    .map_err(|e| format!("read request domain: {}", e))?;
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
    let total_request_len = request_start + header_len;
    let remaining_data = if buf_len > total_request_len {
        Some(buf[total_request_len..buf_len].to_vec())
    } else {
        None
    };

    info!("socks5 CONNECT to {}", target_str);

    // 4. Open gidy tunnel
    debug!("opening tunnel to {}", target_str);
    let (tunnel, init_data) = match conn.open_tunnel(&target_str).await {
        Ok(t) => {
            debug!("tunnel opened to {}", target_str);
            t
        }
        Err(e) => {
            let _ = send_reply(&mut stream, 0x01).await;
            return Err(format!("open tunnel to {}: {}", target_str, e));
        }
    };

    stats.add_up(target_str.len() as u64);

    // 5. Send success reply
    debug!("sending SOCKS5 success reply");
    send_reply(&mut stream, 0x00).await?;

    // 6. Write initial data from server to client
    if !init_data.is_empty() {
        debug!("writing {} bytes init data", init_data.len());
        stream
            .write_all(&init_data)
            .await
            .map_err(|e| format!("write init data: {}", e))?;
    }

    // 7. Send any leftover data from the SOCKS5 request (e.g., TLS ClientHello)
    if let Some(data) = remaining_data {
        if !data.is_empty() {
            debug!("relaying {} bytes of leftover data", data.len());
            let encoded = encode_target_data(&target_str, &data);
            match tunnel.send(&encoded).await {
                Ok(resp) => {
                    debug!("leftover relay response {} bytes", resp.len());
                    if !resp.is_empty() {
                        stream
                            .write_all(&resp)
                            .await
                            .map_err(|e| format!("write resp: {}", e))?;
                    }
                }
                Err(e) => {
                    warn!("tunnel send leftover: {}", e);
                }
            }
        }
    }

    // 8. Main relay loop
    debug!("entering relay loop for {}", target_str);
    let mut rbuf = vec![0u8; 32768];
    loop {
        let n = match stream.read(&mut rbuf).await {
            Ok(0) => {
                debug!("client closed connection");
                break;
            }
            Ok(n) => {
                debug!("read {} bytes from client", n);
                n
            }
            Err(e) => {
                debug!("read error: {}", e);
                break;
            }
        };

        let encoded = encode_target_data(&target_str, &rbuf[..n]);

        match tunnel.send(&encoded).await {
            Ok(resp) => {
                debug!("tunnel.send returned {} bytes", resp.len());
                if !resp.is_empty() {
                    if let Err(e) = stream.write_all(&resp).await {
                        debug!("write response error: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                warn!("tunnel send error: {}", e);
                break;
            }
        }
    }

    let _ = tunnel.close().await;
    debug!("socks5 session for {} finished", target_str);
    Ok(())
}

/// Encode target + data for the server: `target\0data`
fn encode_target_data(target: &str, data: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(target.len() + 1 + data.len());
    encoded.extend_from_slice(target.as_bytes());
    encoded.push(0);
    encoded.extend_from_slice(data);
    encoded
}

async fn send_reply(stream: &mut tokio::net::TcpStream, rep: u8) -> Result<(), String> {
    // SOCKS5 reply: VER=0x05, REP, RSV=0x00, ATYP=0x01(IPv4), BND.ADDR=0.0.0.0, BND.PORT=0
    let reply = [0x05, rep, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
    stream
        .write_all(&reply)
        .await
        .map_err(|e| format!("write reply: {}", e))
}
