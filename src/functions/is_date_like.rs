pub fn is_date_like(input: &str) -> bool {
    let input = input.trim();
    if input.is_empty() {
        return false;
    }
    let input = trim_edges(input);
    if input.is_empty() {
        return false;
    }
    is_month_name_date_like(input) || is_numeric_date_like(input) || is_year_only(input)
}

fn trim_edges(input: &str) -> &str {
    input.trim_matches(|c: char| !c.is_ascii_alphanumeric())
}

fn is_month_name_date_like(input: &str) -> bool {
    let alphabetic_tokens = alphabetic_tokens(input);
    if alphabetic_tokens.is_empty() {
        return false;
    }
    let all_months = alphabetic_tokens.iter().all(|token| is_month_name(token));
    if !all_months {
        return false;
    }
    let numbers = numeric_tokens_with_len(input);
    let has_day = numbers
        .iter()
        .copied()
        .any(|(value, _)| is_day_value(value));
    let has_year = numbers.iter().copied().any(is_year_token);
    if has_day || has_year {
        return true;
    }
    true
}

fn is_numeric_date_like(input: &str) -> bool {
    if input.chars().any(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    let numbers = numeric_tokens_with_len(input);
    match numbers.as_slice() {
        [first, second] => is_two_part_numeric_date(*first, *second),
        [first, second, third] => is_three_part_numeric_date(*first, *second, *third),
        _ => false,
    }
}

fn is_two_part_numeric_date(first: (u32, usize), second: (u32, usize)) -> bool {
    let (first_value, _) = first;
    let (second_value, _) = second;
    let year_month = is_year_token(first) && is_month_value(second_value);
    let month_year = is_year_token(second) && is_month_value(first_value);
    let month_day = is_month_day_pair(first_value, second_value);
    year_month || month_year || month_day
}

fn is_three_part_numeric_date(first: (u32, usize), second: (u32, usize), third: (u32, usize)) -> bool {
    let (first_value, _) = first;
    let (second_value, _) = second;
    let (third_value, _) = third;
    let year_in_first = is_year_token(first) && is_month_day_ordered(second_value, third_value);
    let year_in_second = is_year_token(second) && is_month_day_pair(first_value, third_value);
    let year_in_third = is_year_token(third) && is_month_day_pair(first_value, second_value);
    if year_in_first || year_in_second || year_in_third {
        return true;
    }
    let year2_in_first = is_two_digit_year_token(first) && is_month_day_ordered(second_value, third_value);
    let year2_in_second = is_two_digit_year_token(second) && is_month_day_pair(first_value, third_value);
    let year2_in_third = is_two_digit_year_token(third) && is_month_day_pair(first_value, second_value);
    year2_in_first || year2_in_second || year2_in_third
}

fn is_year_only(input: &str) -> bool {
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if input.len() != 4 {
        return false;
    }
    input.parse::<u32>().is_ok_and(is_year_value)
}

fn is_month_day_pair(left: u32, right: u32) -> bool {
    (is_month_value(left) && is_day_value(right)) || (is_month_value(right) && is_day_value(left))
}

fn is_month_day_ordered(month: u32, day: u32) -> bool {
    is_month_value(month) && is_day_value(day)
}

fn is_year_token((value, len): (u32, usize)) -> bool {
    len == 4 && is_year_value(value)
}

fn is_two_digit_year_token((value, len): (u32, usize)) -> bool {
    len == 2 && value <= 99
}

fn is_year_value(value: u32) -> bool {
    (1000..=9999).contains(&value)
}

fn is_month_value(value: u32) -> bool {
    (1..=12).contains(&value)
}

fn is_day_value(value: u32) -> bool {
    (1..=31).contains(&value)
}

fn numeric_tokens_with_len(input: &str) -> Vec<(u32, usize)> {
    input
        .split(|c: char| !c.is_ascii_digit())
        .filter(|token| !token.is_empty())
        .filter_map(|token| token.parse::<u32>().ok().map(|value| (value, token.len())))
        .collect()
}

fn alphabetic_tokens(input: &str) -> Vec<&str> {
    input
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|token| !token.is_empty())
        .collect()
}

fn is_month_name(token: &str) -> bool {
    MONTH_TOKENS
        .iter()
        .any(|month| token.eq_ignore_ascii_case(month))
}

const MONTH_TOKENS: [&str; 24] = [
    "jan",
    "january",
    "feb",
    "february",
    "mar",
    "march",
    "apr",
    "april",
    "may",
    "jun",
    "june",
    "jul",
    "july",
    "aug",
    "august",
    "sep",
    "sept",
    "september",
    "oct",
    "october",
    "nov",
    "november",
    "dec",
    "december",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn must_check_is_date_like() {
        assert!(is_date_like("January 17"));
        assert!(!is_date_like("Banana"));
    }

    #[test]
    fn must_accept_common_date_formats() {
        let cases = [
            "January",
            "Jan 17",
            "January 2024",
            "2024-01-17",
            "17/01/2024",
            "01-17-24",
            "2024/01",
            "01/2024",
            "01-02",
            "2024",
        ];
        cases
            .into_iter()
            .for_each(|input| assert!(is_date_like(input), "{input}"));
    }

    #[test]
    fn must_reject_non_date_like_inputs() {
        let cases = [
            "Banana 2024",
            "2024-13-01",
            "2024-01-32",
            "20240117",
            "Jan 2024th",
            "Early January",
        ];
        cases
            .into_iter()
            .for_each(|input| assert!(!is_date_like(input), "{input}"));
    }
}
