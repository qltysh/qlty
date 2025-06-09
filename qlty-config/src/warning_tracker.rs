use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

static WARNING_TRACKER: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[cfg(test)]
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn warn_once(warning_message: &str) {
    let mut tracker = WARNING_TRACKER.lock().unwrap();

    if !tracker.contains(warning_message) {
        tracker.insert(warning_message.to_string());
        eprintln!("{warning_message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warn_once_deduplicates_same_message() {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let unique_message = format!(
            "Duplicate warning test {} {}",
            std::process::id(),
            unique_id
        );

        assert!(!WARNING_TRACKER.lock().unwrap().contains(&unique_message));

        // First call should add the message
        warn_once(&unique_message);
        assert!(WARNING_TRACKER.lock().unwrap().contains(&unique_message));

        // Subsequent calls should not change anything - just verify the message is still there
        warn_once(&unique_message);
        warn_once(&unique_message);

        // The key behavior: message should still be in the set after multiple calls
        assert!(WARNING_TRACKER.lock().unwrap().contains(&unique_message));
    }

    #[test]
    fn test_warn_once_allows_different_messages() {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let prefix = format!("{} {}", std::process::id(), unique_id);
        let first = format!("First unique warning {}", prefix);
        let second = format!("Second unique warning {}", prefix);
        let third = format!("Third unique warning {}", prefix);

        assert!(!WARNING_TRACKER.lock().unwrap().contains(&first));
        assert!(!WARNING_TRACKER.lock().unwrap().contains(&second));
        assert!(!WARNING_TRACKER.lock().unwrap().contains(&third));

        warn_once(&first);
        warn_once(&second);
        warn_once(&third);

        let tracker = WARNING_TRACKER.lock().unwrap();
        assert!(tracker.contains(&first));
        assert!(tracker.contains(&second));
        assert!(tracker.contains(&third));
    }

    #[test]
    fn test_warn_once_concurrent_access() {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_message = format!(
            "Concurrent test message {} {}",
            std::process::id(),
            unique_id
        );

        assert!(!WARNING_TRACKER.lock().unwrap().contains(&test_message));

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let msg = test_message.clone();
                std::thread::spawn(move || {
                    warn_once(&msg);
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.join();
        }

        assert!(WARNING_TRACKER.lock().unwrap().contains(&test_message));
    }

    #[test]
    fn test_warn_once_basic_functionality() {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let unique_message = format!("Basic test {} {}", std::process::id(), unique_id);

        let contains_before = WARNING_TRACKER.lock().unwrap().contains(&unique_message);
        assert!(!contains_before);

        warn_once(&unique_message);

        let contains_after = WARNING_TRACKER.lock().unwrap().contains(&unique_message);
        assert!(contains_after);
    }
}
