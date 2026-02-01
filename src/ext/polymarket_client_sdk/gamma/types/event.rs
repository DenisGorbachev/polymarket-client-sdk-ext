use crate::{BOOLEAN_OUTCOMES, is_date_like};
use polymarket_client_sdk::gamma::types::response::{Event, Market};
use std::ops::Deref;

/// This function may return multiple plans (because multiple pairs of markets may exhibit inverted pricing)
///
/// This function assumes that the `event` passes [`is_date_cascade`] check
///
/// Returns a vec of prices
pub fn get_date_cascade_opportunity(event: &Event) -> Option<Vec<rust_decimal::Decimal>> {
    let _prices = event
        .markets
        .as_ref()?
        .iter()
        .map(|x| x.outcome_prices.as_ref());
    todo!()
}

/// This function assumes that prev `Market` ends before next `Market`
pub fn is_inverted_pricing(prev: &Market, next: &Market) -> Option<bool> {
    debug_assert!(prev.end_date < next.end_date);
    let prev_outcomes = prev.outcomes.as_ref()?;
    let next_outcomes = next.outcomes.as_ref()?;
    debug_assert_eq!(prev_outcomes, BOOLEAN_OUTCOMES.deref());
    debug_assert_eq!(next_outcomes, BOOLEAN_OUTCOMES.deref());
    let prev_yes_price = prev.outcome_prices.as_ref()?.first()?;
    let next_yes_price = next.outcome_prices.as_ref()?.first()?;
    Some(prev_yes_price > next_yes_price)
}

pub fn is_date_cascade(event: &Event) -> Option<bool> {
    let questions = event
        .markets
        .as_ref()?
        .iter()
        .map(|x| x.question.as_deref())
        .collect::<Option<Vec<_>>>()?;
    let mut diffs = get_middle_diffs(questions);
    let all_diffs_are_date_like = diffs.all(is_date_like);
    Some(all_diffs_are_date_like)
}

fn get_middle_diffs<'a>(inputs: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
    let inputs = inputs.into_iter().collect::<Vec<_>>();
    let (prefix_len, suffix_len) = match inputs.split_first() {
        Some((base, rest)) => {
            let prefix_len = rest
                .iter()
                .map(|input| common_prefix_len(base, input))
                .min()
                .unwrap_or(0);
            let suffix_len = rest
                .iter()
                .map(|input| common_suffix_len(base, input))
                .min()
                .unwrap_or(0);
            (prefix_len, suffix_len)
        }
        None => (0, 0),
    };
    inputs.into_iter().map(move |input| {
        let start = prefix_len.min(input.len());
        let max_suffix = input.len().saturating_sub(start);
        let end = input.len().saturating_sub(suffix_len.min(max_suffix));
        &input[start..end]
    })
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .map(|(left, _)| left.len_utf8())
        .sum()
}

fn common_suffix_len(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left, right)| left == right)
        .map(|(left, _)| left.len_utf8())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn must_get_middle_diffs() {
        let strike_dates_actual = get_middle_diffs([
            "Another US strike on Venezuela by January 31?",
            "Another US strike on Venezuela by January 17?",
            "Another US strike on Venezuela by March 31?",
        ])
        .collect_vec();
        let strike_dates_expected = vec!["January 31", "January 17", "March 31"];
        assert_eq!(strike_dates_actual, strike_dates_expected);
    }

    #[test]
    fn must_get_middle_diffs_empty_iter() {
        let diffs = get_middle_diffs(Vec::<&str>::new()).collect_vec();
        let expected: Vec<&str> = Vec::new();
        assert_eq!(diffs, expected);
    }

    #[test]
    fn must_get_middle_diffs_single_input() {
        let diffs = get_middle_diffs(["  lone value  "]).collect_vec();
        let expected = vec!["  lone value  "];
        assert_eq!(diffs, expected);
    }

    #[test]
    fn must_get_middle_diffs_unicode() {
        let diffs = get_middle_diffs(["Привет, январь 2024!", "Привет, февраль 2025!"]).collect_vec();
        let expected = vec!["январь 2024", "февраль 2025"];
        assert_eq!(diffs, expected);
    }

    #[test]
    fn must_get_middle_diffs_only_suffix_shared() {
        let diffs = get_middle_diffs(["Foo 2024?", "Bar 2025?"]).collect_vec();
        let expected = vec!["Foo 2024", "Bar 2025"];
        assert_eq!(diffs, expected);
    }

    #[test]
    fn must_get_middle_diffs_identical_inputs() {
        let diffs = get_middle_diffs(["Same", "Same"]).collect_vec();
        let expected = vec!["", ""];
        assert_eq!(diffs, expected);
    }

    #[test]
    fn must_get_middle_diffs_varied_length() {
        let diffs = get_middle_diffs(["ID: a?", "ID: bb?", "ID: ccc?"]).collect_vec();
        let expected = vec!["a", "bb", "ccc"];
        assert_eq!(diffs, expected);
    }
}
mod slug_is_none;
pub use slug_is_none::*;
