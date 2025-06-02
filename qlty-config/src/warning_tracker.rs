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
