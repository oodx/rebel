// --- Benchmarking & Time Macros ---
// Namespaced re-exports for selective imports
pub use crate::{benchmark, date, math};
#[macro_export]
macro_rules! benchmark {
    ($body:block) => {
        {
            let start = std::time::Instant::now();
            $body
            let duration = start.elapsed();
            $crate::info!("Benchmark completed in: {:?}", duration);
            duration
        }
    };
}

#[macro_export]
macro_rules! date {
    () => {
        chrono::Local::now().to_string()
    };
    (iso) => {
        chrono::Local::now().to_rfc3339()
    };
    (epoch) => {
        chrono::Local::now().timestamp().to_string()
    };
    (human) => {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    };
    ($format:expr) => {
        chrono::Local::now().format($format).to_string()
    };
}

// --- Math Macros ---
#[macro_export]
macro_rules! math {
    ($expr:expr) => {
        match $crate::math::evaluate_expression($expr) {
            Ok(_) => {},
            Err(e) => {
                $crate::error!("Math expression failed: {}", e);
            }
        }
    };
}
