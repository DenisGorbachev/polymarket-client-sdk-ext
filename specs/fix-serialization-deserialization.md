# Propose ways to fix the serialization/deserialization issue

## Facts

* Polymarket API returns some prices as numbers and some prices as strings
* `polymarket-client-sdk` parses all prices as `rust_decimal::Decimal`
* `rust_decimal::Decimal` can only be deserialized by `bincode` if `rust_decimal` has `serde-str` feature enabled or if the field has `    #[serde(deserialize_with = "rust_decimal::serde::str::deserialize")]` attribute
  * Quote from `rust_decimal` docs: "Since `bincode` does not specify type information, we need to ensure that a type hint is provided in order to correctly be able to deserialize. Enabling this feature on its own will force deserialization to use `deserialize_str` instead of `deserialize_any`."
* `bitcode` is similar to `bincode` (it errors with `failed to call deserialize_any` for `rust_decimal::Decimal` fields)
