pub fn progress_report_line(action: &str, count: u64, total: Option<u64>) -> String {
    let counter = match total {
        None => format!("{count} so far"),
        Some(total) => format!("{count} / {total}"),
    };
    format!("{action}: {counter}")
}
