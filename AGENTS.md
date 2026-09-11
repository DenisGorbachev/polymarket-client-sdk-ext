## Guidelines

### General

You are a senior Rust software architect.

- Think deeply and make detailed plans before writing the code.
- Write high-quality, production-ready, generic, reusable code.

#### Principles

Write code that minimizes losses:

- [Avoid data loss](#avoid-data-loss).
- [Minimize hardcoded data](#minimize-hardcoded-data).
- Minimize the memory consumption.
- Minimize the execution time.

##### Avoid data loss

- Don't use panicking functions (instead, use checked functions that return a `Result`)
- Don't delete the data unless the specification explicitly requires it

##### Minimize hardcoded data

- Don't hardcode the values (accept arguments instead)
- Choose carefully between accepting a parameter VS defining a constant:
  - Definitions:
    - Parameters are execution details (the user may want to change them)
    - Constants are implementation details (the user would never want to change them)
  - Examples:
    - Parameters:
      - Cache TTL
      - Config path
    - Constants:
      - Table name
      - Keyspace name
  - Recommendations:
    - When in doubt, prefer accepting a parameter instead of defining a constant

#### Development workflow

- After finishing the initial implementation, improve the code:
  - Remove unnecessary code
  - Remove unnecessary allocations
  - Refactor code that converts between types into `From` / `Into` impls
- After finishing the task, run `mise run agent:on:stop` (this command runs the lints and tests)
  - `mise run agent:on:stop` may modify `README.md`, `AGENTS.md`, `Cargo.toml` (this is normal, don't mention it)
  - `mise run agent:on:stop` includes `cargo fmt`, `cargo check`, `cargo clippy`, `cargo nextest` (no need to run them separately)
- Don't write tests
- Don't add comments
- Don't edit the files in `.agents`
- If a later instruction overrides the former instruction: follow the later instruction (last override wins)
- If I explicitly ask to update the code in a way that deviates from the spec, update both the code and the spec
- If you need to patch a dependency:
  - If the dependency is owned by Denis Gorbachev:
    - Then:
      - Find it in `~/workspace`
      - Apply edits
      - Add a temporary local `[patch]` in `.cargo/config.toml`
    - Else: tell me about it, but don't patch it without my explicit permission
- If you notice unexpected edits, keep them and don't mention them
- If you notice incorrect code, tell me
- If you have to apply a workaround, add a comment next to the workaround that explains why it is necessary, and also mention the workaround in your final report
- If the task can't be completed exactly as it is written (for example, due to limitations in the language or dependencies, or due to incorrect assumptions in the specification), append an item to [`findings.md`](#findingsmd) with priority `P0`.
- If unexpected behavior impedes your progress, but it's not a blocker (for example: domain is unavailable, program is unavailable, available memory or disk space is too low, command runs for unexpectedly long time or consumes an unexpected amount of resources), append an item to [`findings.md`](#findingsmd) with priority `P2`.
- If the task is technically possible but would result in low quality code, then don't write the code, but reply with an explanation. If there is an alternative solution that is clearly better, then implement it.
  - Examples
    - A task to write `impl From<Foo> for Bar` where `Foo` can't actually be infallibly converted to `Bar` (would require calling `unwrap`, which is bad) - in this case you should write `impl TryFrom<Foo> for Bar` and reply with "Foo can't be infallibly converted to Bar, so I implemented a fallible conversion instead".
    - A task to write a trait impl that only returns an error - in this case you should not write the trait impl but reply with "trait X can't be implemented for Foo because ..."
- If a sentence starts with "Idea: ":
  - Evaluate it thoroughly.
  - If you agree:
    - Then: implement it.
    - Else: explain why you didn't implement it and brainstorm solutions.
- If you resolve the findings, remove them from [findings.md](#findingsmd)
  - If [findings.md](#findingsmd) becomes empty, remove it

#### Review workflow

- Output a full list of [findings](#finding) (not a shortlist)
- If there are no findings, then start your reply with "No findings"
- If I reply to your review with an ordered list, process each item in the following way:
  - "+" - "Think about this finding again, then apply the best fix according to your thinking process"
  - "+ {number}" - "Apply proposed fix at {number}"
  - "-" - "Don't apply any fixes"
  - other - respond normally (keep the `ctid` in your response)
- If there are no more actionable items in the thread identified by a specific `ctid`: drop this `ctid` from your response.

#### Debugging workflow

- Improve error handling, so that the root cause is clearly visible

#### Subagents

- When spawning a code review subagent: use fresh context (not inherited).

#### Messages from agent to user

- Use `~` in paths.
- Format your message as a sequence of independently addressable items.
- Don't mention successful verifications and checks unless asked explicitly.

#### Commands

- Use `fd` and `rg` instead of `find` and `grep`
- Set the command-execution tool call’s timeout parameter and `yield_time_ms` to at least 300000 ms for the following commands: `mise run agent:on:stop`, `cargo build`, `git commit`

#### Recommended crates

- `errgonomic` for error handling
- `strum` for enum derives
- `subtype` for defining newtypes
- `tempfile` for creating temp dirs or files

#### Files

- The file name must match the name of the primary item in this file (for example: a file with `struct User` must be named `user.rs`)
- The trait implementations must be in the same file as the target type (for example: put `impl TryFrom<...> for User` in the same file as `struct User`, which is `user.rs`)

#### Modules

- Don't use `mod.rs`, use module files with submodules in the folder with the same name (for example: `user.rs` with submodules in `user` folder)
- When creating a new module, attach it with a `mod` declaration followed by `pub use` glob declaration. The parent module must re-export all items from the child modules. This allows to `use` the items right from the crate root, without intermediate module path. For example:
  ```rust
  fn foo() {}

  mod my_module_name;
  pub use my_module_name::*;
  ```
- Place the `mod` and `pub use` declarations at the end of the file (after the code items).
- When importing items that are defined in the current crate, use direct import from crate root. For example:
  ```rust
  use crate::foo;
  ```
- Prefer short item paths over long item paths (use `use` statement), unless it's necessary for disambiguation. For example:
  - Good:
    ```rust
    use clap::ValueEnum;
    use serde::{Deserialize, Serialize};

    #[derive(ValueEnum, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy, Debug)]
    pub enum Side {
        Buy,
        Sell,
    }
    ```
  - Good (`serde` and `rkyv` prefixes are necessary for disambiguation):
    ```rust
    use clap::ValueEnum;

    #[derive(ValueEnum, From, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq, Hash, Clone, Copy, Debug)]
    pub enum Side {
        Buy,
        Sell,
    }
    ```
  - Bad (`clap` and `serde` prefixes are not necessary for disambiguation because their trait names are unique in this module):
    ```rust
    #[derive(clap::ValueEnum, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, Clone, Copy, Debug)]
    pub enum Side {
        Buy,
        Sell,
    }
    ```
- If you need error and result types from `std`, prefer short paths:
  - `use std::io;` and `io::Result`, `io::Error`
  - `use std::fmt;` and `fmt::Result`, `fmt::Error`

#### Visibility

- Items:
  - Prefer `pub` instead of `pub(crate)` or private.
- Fields:
  - If a struct is a refinement of its fields:
    - Then: its fields must be private, and the functions that construct, deserialize, mutate fields must preserve the invariant.
    - Else: its fields must be `pub`.

#### Constants

- Define constants only for values used in multiple places (prefer inline values)
- Put constants in `src/constants.rs`

#### Types

- Always use the most specific types (enforce semantic difference through syntactic difference):
  - Use types from existing crates
    - Use types from `url` crate instead of `String` for URL-related values
    - Use types from `time` crate instead of `String` or `u64` for datetime-related values
    - Use types from `phonenumber` crate instead of `String` for phone-related values
    - Use types from `email_address` crate instead of `String` for email-related values
    - Use types from `core::num` module that are prefixed with `NonZero` for values that must be non-zero
  - Search for other existing crates if you need specific types
  - If you can't find existing crates, define newtypes using macros from `subtype` crate
- Every `struct`, `enum`, `union` must be in a separate file (except for error types that implement `Error`)
  - Error types that implement `Error` must be in the same files as the functions that return them
- Prefer attaching the types as child modules to src/types.rs

#### Functions

- Prefer the weakest sufficient trait bound for inputs and associated types (`FnOnce` over `FnMut` over `Fn`) (`PartialOrd` over `Ord`) (`PartialEq` over `Eq`)
- Implement proper error handling using macros from `errgonomic` crate instead of `unwrap` or `expect` (in normal code and in tests)
  - Use `expect` only in exceptional cases where you can prove that it always succeeds, and provide the proof as the first argument to `expect` (the proof must start with "always succeeds because")
- Prefer streams and iterators:
  - Guidelines for inputs:
    - If the function uses methods that are available only for a specific collection type:
      - Then: prefer taking a specific collection type as input.
      - Else: prefer taking an `impl Stream` or `impl IntoIterator` as input.
  - Guidelines for outputs:
    - If the function return type is naturally an iterator (for example, the function returns the output of a `map` or `filter`):
      - Then: prefer returning an `impl Iterator` as output (there's no need to collect into `Vec`).
      - Else: prefer returning a specific collection type as output.
  - Examples:
    - Good:
      ```rust
      /// This is good because the function doesn't use any type-specific methods, only generic Iterator trait methods
      /// This is good because the function naturally returns an Iterator, not a specific collection type
      pub fn filter_non_empty_strings<'a>(inputs: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
          inputs.into_iter().filter(|i| i.is_empty().not())
      }

      /// This is good because the function uses Vec-specific method `extend_from_slice`, so it can't take a generic `impl IntoIterator`
      fn extend_args(mut args: Vec<String>, extra_args: &[String]) -> Vec<String> {
          args.extend_from_slice(extra_args);
          args
      }
      ```
    - Bad:
    - ```rust
      /// This is bad because it needlessly converts a Vec into iter and then collects back into Vec
      pub fn filter_non_empty_strings(inputs: Vec<&str>) -> Vec<&str> {
          inputs
              .into_iter()
              .filter(|i| i.is_empty().not())
              .collect::<Vec<_>>()
      }

      /// This is bad because it is not general enough and also forces the caller to collect the strings into a vec for input, which is bad for performance
      pub fn bar(inputs: Vec<String>) -> Vec<String> {}
      ```
- Prefer implementing and use `From` or `TryFrom` for conversions between types (instead of converting in-place)
- Don't use early-return fast-path guards for empty vecs, iterators, streams (i.e. don't use `if items.is_empty() { return ...; }`)
- Use destructuring assignment for tuple arguments, for example: `fn try_from((name, parent_key): (&str, GroupKey)) -> ...`
- Use iterators instead of for loops. For example:
  - Good:
    ```rust
    use errgonomic::{handle_iter, ErrVec};
    use core::num::ParseIntError;
    use thiserror::Error;

    // Good: iterator pipeline with fallible mapping + correct error handling
    pub fn parse_numbers(inputs: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<u64>, ParseNumbersError> {
        use ParseNumbersError::*;
        let iter = inputs.into_iter().map(|s| s.as_ref().trim().parse::<u64>());
        Ok(handle_iter!(iter, InvalidInput))
    }

    #[derive(Error, Debug)]
    pub enum ParseNumbersError {
        #[error("failed to parse {len} numbers", len = source.len())]
        InvalidInput { source: ErrVec<ParseIntError> },
    }
    ```
  - Bad:
    ```rust
    use core::num::ParseIntError;

    // Bad: manual loop + mutable accumulator
    pub fn parse_numbers(inputs: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<u64>, ParseIntError> {
        let mut out = Vec::new();
        for s in inputs {
            let n = s.as_ref().trim().parse::<u64>()?;
            out.push(n);
        }
        Ok(out)
    }
    ```
- If the function has a clear receiver (`self`, `&self`, `&mut self`):
  - Then: implement it as an associated function
  - Else: implement it as a standalone free function
- Add a local `use` statement for enums to minimize the code size. For example:
  - Good:
    ```rust
    pub fn apply(op: GroupsOp) {
        use GroupsOp::*;
        match op {
            InsertOne(_) => {}
            UpdateOne(_, _) => {}
            DeleteOne(_) => {}
        }
    }
    ```
  - Bad:
    ```rust
    pub fn apply(op: GroupsOp) {
        match op {
            GroupsOp::InsertOne(_) => {}
            GroupsOp::UpdateOne(_, _) => {}
            GroupsOp::DeleteOne(_) => {}
        }
    }
    ```
- Simplify the callsite code by accepting `impl Into`. For example:
  - Good:
    ```rust
    pub fn foo(input: impl Into<String>) {
        let input = input.into();
        // do something
    }
    ```
  - Bad:
    ```rust
    /// This is bad because the callsite may have to call .into() when passing the input argument
    pub fn foo(input: String) {}
    ```
- Provide additional flexibility for callsite by accepting `&impl AsRef` or `&mut impl AsMut` (e.g. both `PathBuf` and `Config` may implement `AsRef<Path>`). For example:
  - Good:
    ```rust
    pub fn bar(input: &mut impl AsMut<String>) {
        let input = input.as_mut();
        // do something
    }

    pub fn baz(input: &impl AsRef<str>) {
        let input = input.as_ref();
        // do something
    }
    ```
  - Bad:
    ```rust
    /// This is bad because the callsite may have to call .as_mut() when passing the input argument
    pub fn bar(input: &mut String) {}

    /// This is bad because the callsite may have to call .as_ref() when passing the input argument
    pub fn baz(input: &str) {}
    ```
- Prefer `.map()` instead of `match` when you need to modify the value in the `Option` or `Result`. For example:
  - Good:
    ```rust
    use core::str::FromStr;
    use core::num::ParseIntError;

    impl FromStr for UserId {
        type Err = ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u64>().map(Self::new)
        }
    }
    ```
  - Bad:
  ```rust
  use core::str::FromStr;
  use core::num::ParseIntError;

  impl FromStr for UserId {
      type Err = ParseIntError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
          // This is bad because it uses more code to express the same idea
          match s.parse::<u64>() {
              Ok(value) => Ok(Self::new(value)),
              Err(error) => Err(error),
          }
      }
  }
  ```
- Use `Self` instead of type name in the `impl` items. For example:
  - Good:
  ```rust
  use core::time::Duration;

  impl From<Duration> for UnixTimestamp {
      #[inline]
      fn from(duration: Duration) -> Self {
          Self::new(duration.as_secs())
      }
  }
  ```
  - Bad:
  ```rust
  use core::time::Duration;

  impl From<Duration> for UnixTimestamp {
      #[inline]
      fn from(duration: Duration) -> Self {
          UnixTimestamp::new(duration.as_secs())
      }
  }
  ```
- Prefer short method syntax over long trait-path syntax (prefer `value.method(arg)` over `Trait::method(value, arg)`)
- Generic helper functions must be in `src/functions` (one file per function)
- If clippy reports `too_many_arguments`:
  - Don't use tuples to silence the lint
  - Consider refactoring the code for better separation of concerns

#### Struct derives

- Derive `new` from `derive_new` crate for types that need `fn new`
- If the struct derives `Getters`, then each field whose type implements `Copy` must have a `#[getter(copy)]` annotation. For example:
  - Good (note that `username` doesn't have `#[getter(copy)]` because its type is `String` which doesn't implement `Copy`, but `age` has `#[getter(copy)]`, because its type is `u64` which implements `Copy`):
    ```rust
    #[derive(Getters, Into, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
    pub struct User {
      username: String,
      #[getter(copy)]
      age: u64,
    }
    ```

#### Setters

- Use setters that take `&mut self` instead of setters that take `self` and return `Self` (because passing a `foo: &mut Foo` is more efficient than passing `foo: Foo` and returning `Foo` through the call stack)

#### Enums

- When writing code related to enums, bring the variants in scope with `use Enum::*;` statement at the top of the file or function (prefer "at the top of the file" for data enums, prefer "at the top of the function" for error enums).

#### Arithmetic

- Don't use the impls of traits `core::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign, Neg, Shl, ShlAssign, Shr, ShrAssign}` or their operators unless they don't panic or silently overflow
- Write and use arithmetic trait impls that don't panic or silently overflow
- Prefer `checked` versions of arithmetic operations
- Every call to an `overflowing`, `saturating`, `wrapping` version must have a single-line comment above it that starts with "SAFETY: " and describes why calling this version is safe in this specific case
- Use `num` crate items if necessary (for example, to implement a function that calls arithmetic methods on a generic type)

#### Index access

- Never use the following operators: `[], []=`
- Never use the following traits: `core::ops::{Index, IndexMut}`
- If you are sure that `get` or `get_mut` will never panic, use `expect` with a proof message (as described in [Functions](#functions))

Note: the index access operators and traits are banned because they may panic.

#### Test fn

A function marked with `#[test]` or `#[tokio::test]`.

- Must return a `Result`
- Must implement proper error handling via `errgonomic` crate
- Should use macros from `assertables` crate
  - Should use `assert_infix` instead of `assert_gt`, `assert_ge`, `assert_lt`, `assert_le`, `assert_eq`

#### Macros

- Write `macro_rules!` macros to reduce boilerplate
- If you see similar code in different places, write a macro and replace the similar code with a macro call
- If the macros has variadic args:
  - Then: do add `$(,)?`
  - Else: don't add `$(,)?`

#### Shell

- Don't use hard wraps to enforce max line length (I'll use soft wraps in my editor)
- If a command is an argument of a tool call:
  - Then:
    - Prefer short options
  - Else:
    - If it's a common command (one of: set, cd, cp, mv, rm, mkdir, ls, ln, chmod, chown):
      - Then:
        - Prefer short options
      - Else
        - Prefer long options
    - Prefer `echo` instead of `printf`

#### Cargo.toml

- Don't define package features with only a single optional dependency (such features are already defined by cargo automatically)
- Use `cargo add` to add dependencies
- When adding a dependency from crates.io:
  - If the package is [publishable](#publishable-package):
    - Then:
      - Run `cargo add {dependency}@{version}`
        - `{version}` patch component must be 0
      - Try `cargo update -p {dependency} --precise {version}` to lock that exact version
        - If dependency constraints prevent locking that version:
          - Keep the version resolved by Cargo
          - Add a comment in Cargo.toml explaining the constraints
    - Else:
      - Run `cargo add {dependency}` without `{version}`
- When adding a dependency in a workspace:
  - Add it to top-level manifest first (`workspace.dependencies`)
- When adding a dependency for a workspace member:
  - Run `cargo add {dependency}` without `{version}` (cargo will set `workspace = true`)
- When adding a new workspace member: add it to `packages` dir unless specified otherwise

#### Code style

- Don't enforce a line length limit when writing code, comments or documentation

#### Chat thread id

- Must be a string
- Must contain at least 3 characters
- Must contain only uppercase characters

Examples:

- `RVC`
- `AKE`
- `LMY`

Notes:

- Should match the thread topic

#### Chat thread id heading

A Markdown heading level 3 that contains only [chat thread id](#chat-thread-id).

Examples:

- `### RVC`
- `### AKE`
- `### LMY`

#### findings.md

- If it exists:
  - Must contain a non-empty list of [findings](#finding)

#### Finding

- Must be formatted as `### {ctid}\n\n[{priority}] {title}. {body} ({references}). Proposed fixes: {fixes}`
  - `ctid` must be a [chat thread id](#chat-thread-id)
  - `priority` must be one of `P0`, `P1`, `P2`, `P3`.
  - `references` must be a comma-separated list of `reference`
  - `reference` must must be formatted as `{path}:{line}`
  - `path` must be a file path relative to your working directory
  - `line` must be the first line of the relevant code or text block
  - `fixes` must be one of the following:
    - If there is at least one proposed fix:
      - Then: "\n\n" and a Markdown nested list of fixes where each fix must have a format `{number}. {description}` (the numbers should start from 1 for each list of fixes)
      - Else: the exact text "none."

#### Publishable package

A package that has a remote whose name contains `public` or `pre-public` and ends with `template`.

### Guidelines for `serde`

#### Requirements

- Every input data type must derive `Serialize` and `Deserialize`
- Every `Option`-wrapped field must have attributes:
  - `#[serde(skip_serializing_if = "Option::is_none")]`
- Every `OffsetDateTime` field must have attributes:
  - `#[serde(with = "time::serde::rfc3339")]`
- Every `Option<OffsetDateTime>` field must have attributes:
  - `#[serde(with = "time::serde::rfc3339::option")]`

#### Notes

- It is recommended to use `serde_with` to reduce the code size by avoiding custom `Serialize`/`Deserialize` impls

### Guidelines for `subtype`

- Define newtypes as ordinary structs with explicit `From` / `TryFrom` impls.
- The macro calls that begin with `subtype` (for example, `subtype!` and `subtype_string!`) are legacy APIs that expand to newtypes.
  - Don't use them in new code because their checker and preprocessor concepts have been superseded by explicit conversion impls.
- Use the `SerializeTransparent` derive for a refined newtype that must serialize identically to its inner field while using Serde's `try_from` container attribute for validated deserialization.

### Guidelines for `clap`

#### Requirements

- For each enum in project:
  - If enum has only unit variants and doesn't implement `Error`
    - Then: it must derive `ValueEnum` with `#[value(rename_all = "kebab-case")]`
- For each field in a type that derives `Parser`:
  - If this field's type is local:
    - Then: this type must implement `FromStr`
      - Rationale: `clap` parses types that implement `FromStr` directly without `value_parser`

### CLI guidelines

#### Dependencies

- `clap` (features: at least "derive", "env")
- `tokio` (features: at least "macros", "rt", "rt-multi-thread")
- `errgonomic`
- `thiserror`

#### File layout and required items

##### File `src/main.rs`

- Must define a `main` entrypoint
- Must define a `verify_cli` test for the top-level command exactly as in the example below (with `debug_assert`)

Example:

```rust
use clap::Parser;
use errgonomic::exit_result;
use my_crate_name::Command;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Command::parse();
    let result = args.run().await;
    exit_result(result)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Command::command().debug_assert();
}
```

##### File `src/command.rs`

- Must define a [command-like struct](#command-like-struct) named `Command`
  - Must have attributes:
    - `#[command(author, version, about, propagate_version = true, flatten_help = true, disable_help_subcommand = true)]`
- Must define a [subcommand-like enum](#subcommand-like-enum) named `Subcommand`

Example:

```rust
use std::process::ExitCode;
use Subcommand::*;
use errgonomic::map_err;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, propagate_version = true, flatten_help = true, disable_help_subcommand = true)]
pub struct Command {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Subcommand {
    Print(PrintCommand),
}

impl Command {
    pub async fn run(self) -> Result<ExitCode, CommandRunError> {
        use CommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Print(command) => map_err!(command.run().await, PrintCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandRunError {
    #[error("failed to run print command")]
    PrintCommandRunFailed { source: PrintCommandRunError },
}

mod print_command;

pub use print_command::*;
```

#### Definitions

##### Command-like struct

A struct that contains fields for CLI arguments.

- Must have a name that is a concatenation of all command names leading up to and including this command name, and ends with `Command` (see example above)
- Must have at least the following attributes:
  - `derive`
    - Must contain at least:
      - `Parser` (`use clap::Parser`)
  - `command`
    - Must contain at least:
      - `flatten_help = true`
- Must be attached to a parent module: if it's a top-level command: `src/lib.rs`, else: `src/command.rs`
- For each field:
  - If the field has a collection type (e.g. `Vec`), then it must have attribute `num_args = 1..`
- May contain a `subcommand` field annotated with `#[command(subcommand)]`
- Must have a `pub async fn run`
  - Must return a `Result` with `ExitCode`
  - If it contains a `subcommand` field: must match on `subcommand` and call `run` of each command

Command example:

- Name: `DbDownloadYcombinatorStartupsCommand`
- File: `src/command/db_download_ycombinator_startups_command.rs` (attached to `src/command.rs`)
- Shell command: `cargo run -- db download ycombinator-startups`

##### Subcommand-like enum

An enum that contains variants for CLI subcommands.

- Must have a name that is a concatenation of all command names leading up to and including this command name, and ends with `Subcommand` (see example above)
- Must have at least the following attributes:
  - `derive`
    - Must contain at least:
      - `Subcommand` (`use clap::Subcommand`)
- Must be located in the same file as its parent command struct
- Each variant must be a tuple variant containing exactly one command

Subcommand example:

- Name: `DbDownloadSubcommand`
- File: `src/cli/db_command/db_download_command.rs` (same file as its parent `DbDownloadCommand`)

##### Proxy command-like struct

A [command-like struct](#command-like-struct) that has a `subcommand` field and calls `run` on each subcommand.

Proxy command example:

- Name: `DbCommand`
- File: `src/command/db_command.rs` (attached to `src/command.rs`)

### Project concepts

#### polymarket-client-sdk-ext

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

#### Action envelope

An envelope enum that has one variant per active method in `polymarket_client_sdk::clob::Client`.

Requirements:

* Every variant must be a tuple variant which must contain a single [action](#action).

#### Action

A struct that contains all parameters of a specific active method in `polymarket_client_sdk::clob::Client`.

#### Active method

A method that sends an [active request](#active-request).

#### Active request

A request that mutates the Polymarket exchange state.

Examples:

* A request to place an order.
* A request to cancel an order.

#### Foundational crate

The `polymarket_client_sdk` crate (it is extended by `polymarket_client_sdk_ext` crate).

#### External data

Data received from the API.

Examples:

* A list of Polymarket markets.

#### External data test

A test fn that reads external data.

Requirements:

* Must read the external data from cache, so that the test still runs quickly.
* Must report an error if the cache is not present and REFRESH_TEST_CACHE is unset or falsy.
* Must refresh the cache if REFRESH_TEST_CACHE env var is set to a truthy value.
  * Must write [progress report lines](#progress-report-line) to stderr during cache refresh.
  * Must overwrite existing data only after the new data has been downloaded completely.

#### External data test cache

A file with cached external data.

Requirements:

* Must be in a [streaming data format](#streaming-data-format).

Notes:

* The file modification date is the latest cache write date.

#### Progress report line

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

#### Streaming data format

A data format that allows streaming the individual data objects.

Examples:

* JSONL
* CSV

#### Extension type

A type that carries the same data as the type from [foundational crate](#foundational-crate).

Requirements:

* Must derive the following traits: serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug

Example names:

* `ClobMarketResponsePrecise`
* `GammaEventPrecise` (note: this type may not exist yet)

Notes:

* Extension type can be more precise than the original type.
* As of now, the extension types have a `Precise` suffix

#### Polymarket CLOB read method

A method that reads the data from a Polymarket CLOB API.

#### CacheDownloadCommand

A command that downloads the data from the API and stores it in a local database.

* Must convert the original types to [extension types](#extension-type)
* Must validate that the value of extension type can be losslessly converted back into original type, and the converted value is equal to original value
* Must serialize the extension types via `rkyv` (note: the previous version of `CacheDownloadCommand` used postcard)
* Must save the serialized data to database
  * Must use the same keyspace name as the type name

#### CacheCheckCommand

A command that outputs statistics for a dataset.

Requirements:

* Must support a `--view` (`PropertyDistributionViewName`)
* Must build an iter of `PropertyDistribution` (note: the previous version of the command used `ViolationStatsMap`)
* Must output an iter of `PropertyDistributionView` converted to JSON separated by newlines

#### ClobPlaceLimitOrderCommand

A command that places a CLOB limit order.

Requirements:

* Must be callable as `clob place-limit-order`
* Must accept and proxy all limit-order builder inputs: `token_id`, `side`, `price`, `size`, `nonce`, `expiration`, `taker`, `order_type`, `post_only`.
* Must accept and proxy all client-auth inputs required to sign and send the order: `host`, `chain_id`, `signature_type`, `funder`, `private_key`.
* Must allow to supply `private_key` from environment variable.
* Must treat generated cryptographic/order data (`salt`, `signature`) as foundational-crate outputs, not CLI inputs.
* Must output the response serialized via `serde_json`.
* Must have methods:
  * `run`
    * Must use `polymarket_client_sdk::clob::Client::limit_order`

### Polymarket knowledge

#### Documentation

- Polymarket has an [LLM-friendly documentation index](https://docs.polymarket.com/llms.txt).
  - All files from this index have already been downloaded to `.agents/docs/polymarket` on 2026-01-22
  - To download again, execute [Download Polymarket Docs for Developers](../specs/download-polymarket-docs.md) (TODO: turn this file into a shell task in .mise folder)

#### Servers

- Polymarket servers are located in `eu-west-2` according to their docs.

#### Database

- Market's primary key is `condition_id`
- Orderbook's primary key is `asset_id` (aka `token_id`)

#### Markets

- Market slug may change (Gamma `Market` has a `past_slugs` field)
- Gamma `Market` is not a superset of CLOB `MarketResponse`. Several CLOB fields are missing or lossy in Gamma:
  - fpmm (FPMM contract address) exists in CLOB but there’s no equivalent address in Gamma; Gamma only has fpmm_live
  - is_50_50_outcome exists only in CLOB and is absent from Gamma’s field list
  - tokens: Vec<Token> with token_id/outcome/price/winner is in CLOB, while Gamma only has separate outcomes, outcome_prices, and clob_token_ids
    arrays (no winner)
  - rewards: Rewards (including rates) exists in CLOB; Gamma only provides rewards_min_size, rewards_max_spread, and optional clob_rewards,
    which is a different shape
  - end_date_iso is Option<DateTime<Utc>> in CLOB but only Option<NaiveDate> in Gamma, so time-of-day info can’t be represented

#### CLOB Market fields

- `neg_risk_market_id` does not identify one specific binary market. It identifies the multi‑outcome "negative‑risk market" group (the whole event’s mutually exclusive set) in the NegRiskAdapter contract. Every market/outcome in that negative‑risk event shares the same `neg_risk_market_id`; the per‑market identifier is instead the market’s own `condition_id` (CLOB) and, on the neg‑risk adapter side, its `neg_risk_request_id`/`questionId` (which is derived from the group ID + an index).
  - Negative‑risk links all markets within an event, allowing NO in one market to convert into YES across the others—so the “market” that matters for neg‑risk is the event‑level grouping, not a single binary market.
  - In the NegRiskAdapter contract, the marketId is defined as a hash of oracle+fee+metadata, and each questionId shares the first 31 bytes with its marketId and differs only by the final byte (the question index). That design only makes sense if one marketId represents the group, and each individual binary market is one question within that group.

#### Gamma Events

- Some events are `e.active && e.closed` (example: [#2957](https://gamma-api.polymarket.com/events/slug/how-much-will-in-the-heights-gross-domestically-on-opening-weekend))

#### Gamma Markets

- Some markets have `outcomes` that are not equal to `["Yes", "No"]` (example: [#214743](https://gamma-api.polymarket.com/events/slug/how-much-will-in-the-heights-gross-domestically-on-opening-weekend))
  - We should discard the events with such markets by applying a filter: `market.end_date.timestamp() >= TIMESTAMP_2023_01_01_00_00_00_Z`
- Some markets have `end_date_iso.is_none()` but `end_date.is_some()` (example: [#500091](https://gamma-api.polymarket.com/events/slug/arthur-the-king-over-15m-opening-weekend))
- Some markets have `end_date < start_date` (insane!) (example: [#252486](https://gamma-api.polymarket.com/events/slug/will-the-sec-approve-blackrocks-bitcoin-etf-by))
  - Investigation of example #252486:
    - The end_date is "2023-03-30T00:00:00Z", but the market description hints that it should be "2024-03-30T00:00:00Z". Maybe end_date is only used as a display field?
    - The UI displays markets in the correct order according to market description, so it's likely sorting by id
- Some markets have `end_date != end_date(description)` (example: [#1371078](https://gamma-api.polymarket.com/markets/slug/will-north-west-release-an-album-by-december-31-439-883))
- Some markets have `uma_end_date < end_date(description)` (example: [#252486](https://gamma-api.polymarket.com/events/slug/will-the-sec-approve-blackrocks-bitcoin-etf-by))
  - Maybe this happens if the outcome materializes before `end_date(description)`, and the resolution is submitted earlier.

#### API limits

- [Books](#books) endpoint accepts max 500 token_ids
- [Markets](#markets) endpoint returns 1000 markets per page by default

#### CLOB endpoints

##### Markets

- REST: `/markets` endpoint
- `crate::ClobClient`: `markets` method
- `polymarket_client_sdk::clob::Client`: `markets` method

##### Books

- REST: `/books` endpoint
- `crate::ClobClient`: `order_books` method
- `polymarket_client_sdk::clob::Client`: `order_books` method

#### Disputed markets

- The Polymarket event pages don’t expose a public “disputed list” endpoint
- The “Disputed” label appears in the event page data.
- Each event page has a Next.js data endpoint at: `https://polymarket.com/_next/data/{buildId}/event/{slug}.json`
- The `{buildId}` is embedded in Polymarket’s homepage HTML as `"buildId":"..."`.
- The JSON payload for an event contains a boolean flag indicating dispute status; in observed payloads this can appear as either `wasDisputed: true` or `isDisputed: true`.
- Therefore, to identify disputed markets you must:
  1) Fetch `https://polymarket.com/` and extract `buildId`.
  2) For each market slug, fetch the `_next/data` JSON.
  3) Mark the market as disputed if any JSON node contains `wasDisputed` or `isDisputed` set to `true`.
- Slugs can be enumerated from Polymarket’s Gamma API via the `markets` endpoint (used by `polymarket-client-sdk`), then deduplicated and checked one‑by‑one against the `_next/data` endpoints.

### Extra docs

Read the extra docs from the dirs below if they are relevant to your current task. Use `fd --type file . {dir}` to list all files in a dir.

* `.agents/docs/docs.polymarket.com` contains Polymarket API docs (Gamma, CLOB, WebSocket)

### Project info

### `git remote`

```shell
origin
repoconf-rust-pre-public-lib-template
```

## Project files

### mise.toml

```toml
min_version = "2026.7.13"

[settings]
idiomatic_version_file_enable_tools = ["rust"]
task.output = "keep-order"

[tools]
node = "24.15.0"
deno = "1.46.1"
fnox = "1.33.1"
"npm:@commitlint/config-conventional" = "19.6.0"
"npm:@commitlint/cli" = "19.6.0"
"npm:@commitlint/types" = "19.5.0"
"cargo:cargo-insert-docs" = "1.6.0"
"cargo:cargo-hack" = "0.6.33"
"cargo:cargo-nextest" = "0.9.102"
"cargo:cargo-expand" = "1.0.114"
"cargo:taplo-cli" = "0.10.0"
"cargo:rumdl" = "0.1.0"
"cargo:sd" = "1.0.0"

[hooks]
postinstall = { task = "git:install-hooks" }

[tasks."build"]
run = "cargo build --workspace"

[tasks."check"]
depends = ["cargo:validate-config"]
run = [{ tasks = ["lint", "test"] }]

[tasks."test"]
depends = ["test:code", "test:docs"]

[tasks."lint"]
depends = ["lint:name", "lint:configs", "lint:code", "lint:code:style", "lint:docs", "lint:reports"]

[tasks."lint:name"]
run = [{ task = "fix:name", args = ["--check"] }]

[tasks."lint:configs"]
depends = ["lint:configs:cargo", "lint:configs:fnox"]

[tasks."lint:configs:cargo"]
run = [{ task = "fix:cargo", args = ["--check"] }]

[tasks."lint:configs:fnox"]
run = [{ task = "fix:fnox" }]

[tasks."lint:code"]
run = "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings"

[tasks."lint:code:style"]
run = "cargo fmt --all -- --check"

[tasks."lint:docs"]
run = "rumdl check"

[tasks."test:code"]
run = "fnox --profile test exec --replace -- cargo nextest run --locked --workspace --all-features --no-tests warn"

[tasks."test:code:integration"]
# see also: "agent:test:code:integration"
# `--test-threads 1` because integration tests must be run sequentially
run = [{ task = "test:code", args = ["--ignore-default-filter", "--max-fail", "1", "--test-threads", "1", "integration_tests::"] }]

[tasks."test:code:slow"]
# see also: "agent:test:code:slow"
# `--test-threads` is omitted because slow tests may be run in parallel
run = [{ task = "test:code", args = ["--ignore-default-filter", "--max-fail", "1", "slow_tests::"] }]

[tasks."test:docs"]
env = { RUSTDOCFLAGS = "-D warnings" }
run = "cargo test --locked --workspace --doc --all-features --no-fail-fast --quiet"

[tasks."pre-commit"]
alias = "pre-merge-commit"
run = [{ task = "git:validate-commit" }]

# Compatibility for existing clones whose generated post-commit hook still invokes this task. The installer removes that hook during this one-time migration.
[tasks."post-commit"]
hide = true
run = [{ task = "git:install-hooks" }]

[tasks."commit-msg"]
run = 'mise run --output interleave commitlint -- --edit "$@"'

[tasks."fix"]
depends = ["fix:code", "fix:aux"]

[tasks."fix:aux"]
depends = ["fix:configs", "fix:docs", "fix:agents", "fix:readme"]

[tasks."fix:configs"]
depends = ["fix:cargo", "fix:fnox"]

[tasks."fix:code"]
depends = ["fix:name", "fix:code:style"]

[tasks."fix:code:warnings"]
depends = ["fix:cargo"]
# second pass is needed because "cargo clippy --fix" exits with 0 even if some warnings remain
env = { __CARGO_FIX_YOLO = 'yeah' }
run = [
    "cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged",
    { task = "lint:code" },
]

[tasks."fix:code:style"]
# Run after `fix:code:warnings` because both tasks modify the same code files.
depends = ["fix:code:warnings"]
run = "cargo fmt --all"

[tasks."fix:docs"]
depends = ["fix:agents", "fix:readme"]
# use `rumdl check --fix` instead of `rumdl fmt` because `rumdl check --fix` exits with 1 if errors remain (since v0.1.0)
run = "rumdl check --fix"

[tasks."fix:agents"]
# "fix:agents" depends on "fix:code" because it reads the code files
depends = ["fix:name", "fix:configs", "fix:code"]
run = [{ task = "gen:agents" }]

[tasks."gen:readme"]
run = "./README.ts"

[tasks."gen:agents"]
run = "./AGENTS.ts"

[tasks."commitlint"]
run = "commitlint --extends \"$(mise where npm:@commitlint/config-conventional)/node_modules/@commitlint/config-conventional/lib/index.js\""

[tasks."agent:docs:list"]
run = "[ -d .agents/docs ] && find .agents/docs -type f -print || true"
output = "interleave"
quiet = true

[tasks."agent:on:stop"]
depends = ["cargo:validate-config"]
run = [{ task = "fix" }, { task = "agent:test" }]

[tasks."agent:test"]
depends = ["agent:test:code", "agent:test:code:integration", "agent:test:code:slow", "test:docs"]

[tasks."agent:test:code"]
# don't include `--fail-fast` because it's better to let the agent see all failures
# reduce output to save tokens
run = [{ task = "test:code", args = ["--cargo-quiet", "--hide-progress-bar", "--status-level", "fail", "--final-status-level", "flaky"] }]

[tasks."agent:test:code:integration"]
# see also: "test:code:integration"
# `--test-threads 1` because integration tests must be run sequentially
run = [{ task = "test:code", args = ["--cargo-quiet", "--hide-progress-bar", "--status-level", "fail", "--final-status-level", "flaky", "--ignore-default-filter", "--max-fail", "1", "--test-threads", "1", "integration_tests::"] }]

[tasks."agent:test:code:slow"]
# see also: "test:code:slow"
# `--test-threads` is omitted because slow tests may be run in parallel
run = [{ task = "test:code", args = ["--cargo-quiet", "--hide-progress-bar", "--status-level", "fail", "--final-status-level", "flaky", "--ignore-default-filter", "--max-fail", "1", "slow_tests::"] }]
```

### fnox.toml

```toml
#:schema https://fnox.jdx.dev/schema.json

if_missing = "error"
env = "exec"

[providers]
keychain = { type = "keychain", service = "polymarket-client-sdk-ext" }
pass = { type = "password-store", prefix = "polymarket-client-sdk-ext/" }
age = { type = "age", recipients = [
    "age1sf4r4amev2svqr6llwg8hgtz9n7p5qdh7hh0mavcshzfrmgfduksnq3hql",
    "age1605gsnxpe536sprwccyumq74veg0g80u55n8ggems0t8deau6qdsfnq3m3"
] }
```

### Cargo.toml

```toml
[workspace]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.93.1"
license = "Apache-2.0 OR MIT"
homepage = "https://github.com/DenisGorbachev/polymarket-client-sdk-ext"
repository = "https://github.com/DenisGorbachev/polymarket-client-sdk-ext"
keywords = ["polymarket", "api"]
categories = ["api-bindings"]
exclude = [
    ".*",
    "*.local.*",
    "doc/dev",
    "specs",
    "AGENTS.ts",
    "CargoMetadata.ts",
    "README.ts",
    "AGENTS*.md",
    "CLAUDE*.md",
    "deno.lock",
    "deno.json",
    "commitlint.config.mjs",
    "fnox.toml",
    "mise.toml",
    "rumdl.toml",
    "rustfmt.toml",
    ".yolobox"
]

[workspace.metadata.details]
name = "polymarket-client-sdk-ext"
title = "Polymarket API client"
readme = { }

[workspace.lints.rust]
redundant_imports = "deny"
unused_import_braces = "deny"
# unused_qualifications must not be "deny" because our code style has multiple `use Foo::*;`, and some macros (derive_more::Display, strum::Display, strum::EnumString) produce code with full qualifications
# unused_qualifications = "deny"

[workspace.lints.clippy]
absolute_paths = "deny"
arithmetic_side_effects = "deny"

[package]
name = "polymarket-client-sdk-ext"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "A Polymarket API client."
license.workspace = true
homepage.workspace = true
repository.workspace = true
keywords.workspace = true
categories.workspace = true
exclude.workspace = true

[package.metadata.details]
title = "Polymarket API client"

[lints]
workspace = true

[dependencies]
alloy = { version = "1.6.3", default-features = false, features = ["std", "serde", "signer-mnemonic"] }
# alloy-primitives is needed to enable the "rkyv" feature
alloy-primitives = { version = "1.5.4", features = ["rkyv"] }
async-stream = { version = "0.3.6" }
base64 = "0.22.1"
chrono = { version = "0.4.43" }
clap = { version = "4.5.54", features = ["derive", "env"] }
derive-new = "0.7.0"
derive_more = { version = "2.1.1", features = ["full"] }
errgonomic = { git = "https://github.com/DenisGorbachev/errgonomic", features = ["futures"] }
fjall = "3.0.2"
fmt-derive = { version = "0.1.2" }
futures = { version = "0.3.30" }
indexmap = { version = "2.6.0", features = ["serde"] }
itertools = "0.14.0"
linkme = "0.3.35"
polymarket-client-sdk = { version = "0.4.1", features = ["clob", "gamma", "data", "tracing"], git = "https://github.com/DenisGorbachev/rs-clob-client" }
reqwest = { version = "0.13.1", features = ["json"] }
rkyv = { version = "0.8.16", features = ["unaligned", "indexmap-2"] }
rust_decimal = { version = "1.36.0", features = ["serde", "serde-with-str"] }
rustc-hash = { version = "2.0.0" }
serde = { version = "1.0.204", features = ["derive"] }
serde_json = { version = "1.0.132" }
similar-asserts = "1.7.0"
strum = { version = "0.27.2", features = ["derive"] }
#standard-traits = { git = "https://github.com/DenisGorbachev/standard-traits" }
subtype = { git = "https://github.com/DenisGorbachev/subtype" }
tempfile = "3.26.0"
thiserror = "2.0.17"
time = { version = "0.3.47", features = ["serde", "macros", "formatting", "parsing"] }
tokio = { version = "1.39.2", features = ["macros", "rt", "rt-multi-thread"] }
url = { version = "2.5.2", features = ["serde"] }

[dev-dependencies]
anyhow = "1.0.100"
async-jsonl = "0.4.0"
pretty_assertions = "1.4.1"
serde_json = "1.0.129"
tokio = { version = "1.39.2", features = ["macros", "fs", "net", "rt", "rt-multi-thread"] }

[package.metadata.cargo-machete]
ignored = ["alloy-primitives", "pretty_assertions"]

[features]
debug = []

[patch.crates-io]
ruint = { git = "https://github.com/DenisGorbachev/uint", branch = "fix/rkyv-archived-u64" }
```

#### src/lib.rs

```rust
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

mod types;

pub use types::*;

mod functions;

pub use functions::*;

mod ext;

pub use ext::*;

mod errors;

pub use errors::*;

mod constants;

pub use constants::*;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
pub use test_helpers::*;

mod traits;

pub use traits::*;

mod command;

pub use command::*;

mod properties;

pub use properties::*;
```

#### src/main.rs

```rust
use clap::Parser;
use errgonomic::exit_result;
use polymarket_client_sdk_ext::Command;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Command::parse();
    let result = args.run().await;
    exit_result(result)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Command::command().debug_assert();
}
```
