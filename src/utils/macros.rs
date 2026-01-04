//! Common utility macros used across the application.
//!
//! This module provides shared macros to avoid code duplication
//! and ensure consistent error handling patterns.

/// Helper macro to lock a mutex with consistent error handling.
///
/// This macro provides a standardized way to acquire a mutex lock,
/// panicking with a consistent error message if the mutex is poisoned.
///
/// # Example
///
/// ```rust,ignore
/// let data = lock_mutex!(my_mutex);
/// ```
#[macro_export]
macro_rules! lock_mutex {
    ($mutex:expr) => {
        $mutex.lock().expect("Mutex poisoned")
    };
}
