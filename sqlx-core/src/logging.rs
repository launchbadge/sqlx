/// Logs the query and execution time of a statement as it runs.
macro_rules! log_execution {
    ( $query:expr, $block:expr ) => {{
        // TODO: Log bound parameters
        let query_string = $query.query_string();
        let timer = std::time::Instant::now();
        let result = $block;
        let elapsed = timer.elapsed();
        if elapsed >= std::time::Duration::from_secs(1) {
            log::warn!(
                "{} ..., elapsed: {:.3?}\n\n    {}\n",
                crate::logging::parse_query_summary(query_string),
                elapsed,
                query_string
            );
        } else {
            log::debug!(
                "{} ..., elapsed: {:.3?}\n\n    {}\n",
                crate::logging::parse_query_summary(query_string),
                elapsed,
                query_string
            );
        }
        result
    }};
}

pub(crate) fn parse_query_summary(query: &str) -> String {
    // For now, just take the first 3 words
    query
        .split_whitespace()
        .take(3)
        .collect::<Vec<&str>>()
        .join(" ")
}
