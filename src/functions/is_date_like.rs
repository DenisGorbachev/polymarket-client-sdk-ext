// TODO: check for many different date formats, including partial dates
// TODO: don't check for time
pub fn is_date_like(_input: &str) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn must_check_is_date_like() {
        assert!(is_date_like("January 17"));
        assert!(!is_date_like("Banana"));
    }
}
