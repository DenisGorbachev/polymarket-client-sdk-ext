pub fn progress_report_line(action: &str, offset: usize, limit: Option<usize>, total: Option<usize>, page_offset: usize, page_limit: Option<usize>) -> String {
    let mut parts = vec![format!("{action} (offset: {offset})")];
    if let Some(limit) = limit {
        parts.push(format!("(limit: {limit})"))
    }
    if let Some(total) = total {
        parts.push(format!("(total: {total})"))
    }
    parts.push(format!("(page offset: {page_offset})"));
    if let Some(page_limit) = page_limit {
        parts.push(format!("(total: {page_limit})"))
    }
    parts.join(" ")
}
