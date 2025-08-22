pub fn generate_timestamp_index(
    timestamp: chrono::DateTime<chrono::Utc>,
) -> String {
    let max_timestamp = i64::MAX;
    let inverted_timestamp = max_timestamp - timestamp.timestamp();
    format!("{}", inverted_timestamp)
}
