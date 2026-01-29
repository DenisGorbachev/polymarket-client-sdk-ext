pub fn progress_report_line(action: &str, offset: usize, limit: Option<usize>, total: Option<usize>) -> String {
    let mut parts = vec![format!("{action} (offset: {offset})")];
    if let Some(limit) = limit {
        parts.push(format!("(limit: {limit})"))
    }
    if let Some(total) = total {
        parts.push(format!("(total: {total})"))
    }
    parts.join(" ")
}
