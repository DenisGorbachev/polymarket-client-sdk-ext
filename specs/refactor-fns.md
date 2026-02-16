* Refactor get_middle_diffs into get_middle_diff and get_middle_diffs (get_middle_diffs should call get_middle_diff on each element of the iterator)
* Refactor GammaMarket::question field to be a String instead of Option<String> (handle the error in TryFrom, use QuestionMissing variant)
* Refactor outcome_prices in TryFrom for GammaMarket
  * Add GammaMarket::no_price
  * let outcome_prices_iter = outcome_prices.unwrap_or_default().into_iter()
  * let yes_price = outcome_prices_iter.next()
  * let no_price = outcome_prices_iter.next()
  * let outcome_prices_rest = outcome_prices_iter.collect()
  * handle_bool!(!outcome_prices_rest.is_empty(), UnexpectedOutcomePrices, outcome_prices_rest)
* Refactor GammaEvent::is_date_cascade
  * Call get_middle_diff on market.question
  * Call .all(is_date_like) on the resulting iterator
  * Remove are_questions_date_cascade if it's unused
