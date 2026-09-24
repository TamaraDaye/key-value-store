use std::time::Instant;
fn main() {
    // 1. Capture the instant
    let now = Instant::now();

    // Debug print ({:?}) works, but shows internal OS tick structure:
    // Output looks similar to: Instant { tv_sec: 14205, tv_nsec: 312049581 }
    println!("Debug format: {:?}", now);

    // 2. Measure elapsed time -> returns a Duration
    let elapsed = now.elapsed();

    // 3. Extract standard numerical formats from Duration
    println!("Seconds:      {}s", elapsed.as_secs());
    println!("Milliseconds: {}ms", elapsed.as_millis());
    println!("Microseconds: {}µs", elapsed.as_micros());
    println!("Nanoseconds:  {}ns", elapsed.as_nanos());
    println!("Float secs:   {}s", elapsed.as_secs_f64());
}
