use similar_asserts::SimpleDiff;

pub fn format_debug_diff<T: core::fmt::Debug>(left: &T, right: &T, left_label: &str, right_label: &str) -> String {
    let left_string = format!("{left:#?}");
    let right_string = format!("{right:#?}");
    SimpleDiff::from_str(&left_string, &right_string, left_label, right_label).to_string()
}
