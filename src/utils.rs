pub fn generate_timestamp_index(
    config_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> String {
    let max_timestamp = i64::MAX;
    let inverted_timestamp = max_timestamp - timestamp.timestamp();
    format!("{:020}_{}", inverted_timestamp, config_id)
}
