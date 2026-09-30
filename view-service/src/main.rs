#![allow(unused)]
use std::time::{Duration, Instant};

const PING_INTERVAL: Duration = Duration::from_millis(100);
const DEAD_PINGS: u32 = 3;
const DEAD_TIMEOUT: Duration = PING_INTERVAL.saturating_mul(DEAD_PINGS);
const TICK_INTERVAL: Duration = Duration::from_millis(100);
fn main() {
    println!("Hello, world!");
}
