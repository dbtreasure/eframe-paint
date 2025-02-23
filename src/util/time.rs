/// Get the current time in seconds since the UNIX epoch
#[cfg(not(target_arch = "wasm32"))]
pub fn current_time_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Get the current time in seconds since the UNIX epoch
#[cfg(target_arch = "wasm32")]
pub fn current_time_secs() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|perf| perf.now() / 1000.0)
        .unwrap_or(0.0)
}

/// Get the current time in seconds (floating point)
pub fn current_time() -> f32 {
    current_time_secs() as f32
}

/// Get a timestamp in seconds since the UNIX epoch
pub fn timestamp_secs() -> u64 {
    current_time_secs() as u64
} 