# Implement CacheCheckCommand

## Files

* src/types/violation_stats.rs
* src/types/property_name.rs
* src/traits/holds.rs
* src/properties.rs

## Tasks

* Implement [CacheCheckCommand](#cachecheckcommand)

## CacheCheckCommand

A command that checks the properties of the cache database and outputs the violations if found.

Requirements:

* Must output an FxHashMap<PropertyName, ViolationStats> (serialized via serde_json)
  * Use type_name() as PropertyName

Preferences:

* Should use a `Vec<Box<dyn Holds>>`
