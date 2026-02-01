# Implement TranscodeCommand

## Concepts

### TranscodeCommand

A command that reads a byte stream from stdin, deserializes the items using the `--input` deserializer, serializes the items using `--output` serializer, writes them to stdout.

Requirements:

* Must read the length of each item (`u64`) in little-endian, then read `len` bytes as item
* Must support `--input/-i` (`TranscodeFormat`)
* Must support `--output/-o` (`TranscodeFormat`)
* Must support `--prefix/-p` (`Option<PrefixKind>`)
* Must support `--suffix/-s` (`Option<String>`)
* Must support `--type` (as `typ` var, `TranscodeTyp`)
* Must write the prefix befor each item if prefix is `Some`
* Must write the suffix after each item if suffix is `Some`
* Must skip zero-length items fully (no prefix, no suffix)
* Must return an error on unexpected EOF (not at item boundary)
* Must be wired into existing `Command` according to the current CLI guidelines

Examples:

* `fjall list --kind value --value-prefix len-u64-le clob_markets --limit 10 | cargo run -- transcode --input rkyv --output serde_json --suffix "\n" --type Market`

Notes:

* Implement error handling according to current guidelines (already provided in your context)
* The caller is responsible for limiting the `len` of a single item so that it fits in memory
* `--suffix` is intentionally restricted to UTF-8
* `--prefix` and `--suffix` apply only to output

### TranscodeFormat

Variants:

* Rkyv
* SerdeJson

Requirements:

* Clap values must match the exact crate names (e.g. `rkyv`, `serde_json`), so specify them explicitly

Notes:

* The command caller must specify the correct prefix/suffix for the output format

### TranscodeTyp

Variants:

* Market (use `crate::Market` from src/types/market.rs)

Requirements:

* Clap values must match the exact type name (e.g. `Market`), so rename them to PascalCase

### PrefixKind

A kind of prefix for the output item.

Constructors:

* LenU64Le
* LenU64Be

Methods:

* `write<I: AsRef<[u8]>, W: Write>(self, item: &I, writer: &mut W) -> io::Result<()>`
  * Must call `item.len()` (this is the output item)
  * Must write the len as prefix according to variant in `self`

Notes:

* Use `#[clap(rename_all = "kebab")]`
* `Le` and `Be` refers to little-endian and big-endian
