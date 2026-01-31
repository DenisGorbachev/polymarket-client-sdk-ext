# Project concepts

## polymarket-client-sdk-ext

A Rust package that implements a Polymarket client with more precise types than `polymarket-client-sdk` package.

Notes:

* Naming:
  * `polymarket-client-sdk-ext` is a package
  * `polymarket_client_sdk_ext` is a crate

Requirements:

* If the type derives both the `serde` and `rkyv` traits
  * Then: the derive macros must be prefixed with the crate name (for example: `serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`)
  * Else: the derive macros must not be prefixed with the crate name (for example: `Serialize, Deserialize`)
* Derive macros must be in the following order:
  * `derive_new`
  * `derive_more`
  * `serde`
  * `rkyv`
  * `core`

## Foundational crate

The `polymarket_client_sdk` crate (it is extended by `polymarket_client_sdk_ext` crate).

## External data

Data received from the API.

Examples:

* A list of Polymarket markets.

## External data test

A test fn that reads external data.

Requirements:

* Must read the external data from cache, so that the test still runs quickly.
* Must report an error if the cache is not present and REFRESH_TEST_CACHE is unset or falsy.
* Must refresh the cache if REFRESH_TEST_CACHE env var is set to a truthy value.
  * Must write [progress report lines](#progress-report-line) to stderr during cache refresh.
  * Must overwrite existing data only after the new data has been downloaded completely.

## External data test cache

A file with cached external data.

Requirements:

* Must be in a [streaming data format](#streaming-data-format).

Notes:

* The file modification date is the latest cache write date.

## Progress report line

A string with a verb in present continuous tense that contains a count of processed objects.

Examples:

* "Downloading objects: 10 / 158"
* "Downloading objects: 10 so far"

Implementation:

```rust
pub fn progress_report_line(action: &str, count: u64, total: Option<u64>) -> String {
    let counter = match total {
        None => count.to_string(),
        Some(total) => format!("{count} / {total}")
    };
    format!("{action}: {counter}")
}
```

## Streaming data format

A data format that allows streaming the individual data objects.

Examples:

* JSONL
* CSV

## Extension type

A type that carries the same data as the type from [foundational crate](#foundational-crate).

## Polymarket CLOB read method

A method that reads the data from a Polymarket CLOB API.
