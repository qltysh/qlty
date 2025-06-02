use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

static WARNING_TRACKER: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

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
        let initial_count = WARNING_TRACKER.lock().unwrap().len();

        warn_once("Duplicate warning test");
        let after_first = WARNING_TRACKER.lock().unwrap().len();

        warn_once("Duplicate warning test");
        warn_once("Duplicate warning test");
        let after_duplicates = WARNING_TRACKER.lock().unwrap().len();

        assert_eq!(after_first, initial_count + 1);
        assert_eq!(after_duplicates, after_first);
        assert!(WARNING_TRACKER
            .lock()
            .unwrap()
            .contains("Duplicate warning test"));
    }

    #[test]
    fn test_warn_once_allows_different_messages() {
        let initial_count = WARNING_TRACKER.lock().unwrap().len();

        warn_once("First unique warning");
        warn_once("Second unique warning");
        warn_once("Third unique warning");

        let final_count = WARNING_TRACKER.lock().unwrap().len();
        assert_eq!(final_count, initial_count + 3);

        let tracker = WARNING_TRACKER.lock().unwrap();
        assert!(tracker.contains("First unique warning"));
        assert!(tracker.contains("Second unique warning"));
        assert!(tracker.contains("Third unique warning"));
    }

    #[test]
    fn test_warn_once_with_empty_string() {
        let initial_count = WARNING_TRACKER.lock().unwrap().len();

        warn_once("");
        let after_first = WARNING_TRACKER.lock().unwrap().len();

        warn_once("");
        let after_second = WARNING_TRACKER.lock().unwrap().len();

        assert_eq!(after_first, initial_count + 1);
        assert_eq!(after_second, after_first);
        assert!(WARNING_TRACKER.lock().unwrap().contains(""));
    }

    #[test]
    fn test_warn_once_concurrent_access() {
        let test_message = "Concurrent test message";

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let msg = test_message.to_string();
                std::thread::spawn(move || {
                    warn_once(&msg);
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.join();
        }

        assert!(WARNING_TRACKER.lock().unwrap().contains(test_message));
    }

    #[test]
    fn test_warn_once_basic_functionality() {
        let unique_message = format!("Basic test {}", std::process::id());

        let initial_count = WARNING_TRACKER.lock().unwrap().len();
        warn_once(&unique_message);
        let after_count = WARNING_TRACKER.lock().unwrap().len();

        assert_eq!(after_count, initial_count + 1);
        assert!(WARNING_TRACKER.lock().unwrap().contains(&unique_message));
    }
}
