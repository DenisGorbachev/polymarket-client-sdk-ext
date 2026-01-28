# Fjall knowledge

## General

* `UserValue` (returned from `guard.value()`) implements `AsRef<[u8]>` and `Borrow<[u8]>`, so you can pass `value.as_ref()` to functions that expect a `[u8]`
