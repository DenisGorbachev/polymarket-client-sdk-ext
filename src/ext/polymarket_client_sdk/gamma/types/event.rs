use crate::is_date_like;
use polymarket_client_sdk::gamma::types::response::Event;
use std::iter::empty;

pub fn is_date_cascade(_event: &Event) -> Option<bool> {
    let questions = _event
        .markets
        .as_ref()?
        .iter()
        .map(|x| x.question.as_deref())
        .collect::<Option<Vec<_>>>()?;
    let mut diffs = get_middle_diffs(questions);
    let all_diffs_are_date_like = diffs.all(is_date_like);
    Some(all_diffs_are_date_like)
}

fn get_middle_diffs<'a>(_inputs: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
    // TODO
    empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[ignore]
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
}
