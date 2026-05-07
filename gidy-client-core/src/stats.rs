use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TrafficSnapshot {
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub speed_up_kbps: f64,
    pub speed_down_kbps: f64,
    pub connected: bool,
    pub uptime_secs: u64,
}

#[derive(Debug)]
pub struct TrafficStats {
    bytes_up: Mutex<u64>,
    bytes_down: Mutex<u64>,
    started_at: Instant,
    last_snapshot: Mutex<Instant>,
    last_bytes_up: Mutex<u64>,
    last_bytes_down: Mutex<u64>,
}

impl TrafficStats {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            bytes_up: Mutex::new(0),
            bytes_down: Mutex::new(0),
            started_at: Instant::now(),
            last_snapshot: Mutex::new(Instant::now()),
            last_bytes_up: Mutex::new(0),
            last_bytes_down: Mutex::new(0),
        })
    }

    pub fn add_up(&self, n: u64) {
        *self.bytes_up.lock() += n;
    }

    pub fn add_down(&self, n: u64) {
        *self.bytes_down.lock() += n;
    }

    pub fn snapshot(&self) -> TrafficSnapshot {
        let now = Instant::now();
        let bytes_up = *self.bytes_up.lock();
        let bytes_down = *self.bytes_down.lock();

        let mut last_time = self.last_snapshot.lock();
        let mut last_up = self.last_bytes_up.lock();
        let mut last_down = self.last_bytes_down.lock();

        let elapsed = now.duration_since(*last_time).as_secs_f64();
        let speed_up = if elapsed > 0.0 {
            (bytes_up - *last_up) as f64 * 8.0 / 1000.0 / elapsed
        } else {
            0.0
        };
        let speed_down = if elapsed > 0.0 {
            (bytes_down - *last_down) as f64 * 8.0 / 1000.0 / elapsed
        } else {
            0.0
        };

        *last_time = now;
        *last_up = bytes_up;
        *last_down = bytes_down;

        TrafficSnapshot {
            bytes_up,
            bytes_down,
            speed_up_kbps: speed_up,
            speed_down_kbps: speed_down,
            connected: true,
            uptime_secs: now.duration_since(self.started_at).as_secs(),
        }
    }
}
