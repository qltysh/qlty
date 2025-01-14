enum CompletionResult {
    Unspecified = 0,
    Success = 1,
    Error = 2,
}

struct Completion {
    // Metadata
    id: String,
    timestamp: Timestamp,
    qlty_cli_version: String,

    // Issue data
    plugin_name: String,
    driver_name: String,
    plugin_version: String,
    rule_key: String,
    level: Level,
    category: Category,
    language: Language,
    message: String,
    path: String,
    range: Range,
    documentation_url: String,
    fingerprint: String,

    // Results
    duration_secs: f32,
    result: CompletionResult,
    patch: Option<String>,
    error_message: Option<String>,
}
