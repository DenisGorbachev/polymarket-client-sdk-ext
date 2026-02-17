* Refactor is_date_cascade in src/types/gamma_event.rs
  * Accept a slice of markets
  * Return Option<bool>
  * if markets.len() < 2, return None
* The field is_date_cascade should be Option<bool>, too
* if statements should use event.is_date_cascade.unwrap_or_default()
