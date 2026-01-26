pub fn progress_report_line(action: &str, count: usize, total: Option<usize>, limit: Option<usize>) -> String {
    let counter = match total {
        None => format!("{count} so far"),
        Some(total) => format!("{count} / {total}"),
    };
    match limit {
        None => format!("{action}: {counter}"),
        Some(limit) => format!("{action}: {counter} (limit: {limit})"),
    }
}
