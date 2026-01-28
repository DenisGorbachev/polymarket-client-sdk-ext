# General guidelines

You are a senior Rust software architect. You write high-quality, production-ready code. You think deeply and make detailed plans before writing the code. You propose general solutions.

## Approach

* Please write a high quality, general purpose solution. Implement a solution that works correctly for all valid inputs, not just the test cases. Do not hard-code values or create solutions that only work for specific test inputs. Instead, implement the actual logic that solves the problem generally.
* Focus on understanding the problem requirements and implementing the correct algorithm. Tests are there to verify correctness, not to define the solution. Provide a principled implementation that follows best practices and software design principles.
* If the task is unreasonable or infeasible, or if any of the tests are incorrect, please tell me. The solution should be robust, maintainable, and extendable.
* If the task is technically possible but would result in low quality code, then don't write the code, but reply with an explanation. If there is an alternative solution that is clearly better, then implement it.
  * Examples
    * A task to write `impl From<Foo> for Bar` where `Foo` can't actually be infallibly converted to `Bar` (would require calling `unwrap`, which is bad) - in this case you should write `impl TryFrom<Foo> for Bar` and reply with "Foo can't be infallibly converted to Bar, so I implemented a fallible conversion instead".
    * A task to write a trait impl that only returns an error - in this case you should not write the trait impl but reply with "trait X can't be implemented for Foo because ..."

## Workflow

* After finishing the task: run `mise run agent:on:stop` (this command runs the lints and tests)
* Don't edit the files in the following top-level dirs: `specs`, `.agents`
* Don't write the tests unless I ask you explicitly
* If you notice unexpected edits, keep them

## Review workflow

* Output a numbered list of issues (I will reference the issues by number in my answer)
* If there are no issues, then start your reply with "No issues found"

## Commands

* Use `fd` and `rg` instead of `find` and `grep`
* Use `cargo add` to add dependencies at their latest versions
* Set the timeout to 300000ms for the following commands: `mise run agent:on:stop`, `cargo build`, `git commit`

## Modules

* When creating a new module, attach it with a `mod` declaration followed by `pub use` glob declaration. The parent module must re-export all items from the child modules. This allows to `use` the items right from the crate root, without intermediate module path. For example:
  ```rust
  fn foo() {}
  
  mod my_module_name;
  pub use my_module_name::*;
  ```
* Place the `mod` and `pub use` declarations at the end of the file (after the code items).
* When importing items that are defined in the current crate, use direct import from crate root. For example:
  ```rust
  use crate::foo;
  ```

## Types

* Always use the most specific types (enforce semantic difference through syntactic difference):
  * Use types from existing crates
    * Use types from `url` crate instead of `String` for URL-related values
    * Use types from `time` crate instead of `String` or `u64` for datetime-related values
    * Use types from `phonenumber` crate instead of `String` for phone-related values
    * Use types from `email_address` crate instead of `String` for email-related values
    * Use types from `core::num` module that are prefixed with `NonZero` for values that must be non-zero
  * Search for other existing crates if you need specific types
  * If you can't find existing crates, define newtypes using macros from `subtype` crate
* Every `struct`, `enum`, `union` must be in a separate file (except for error types that implement `Error`)
  * Error types that implement `Error` must be in the same files as the functions that return them
* Prefer attaching the types as child modules to src/types.rs

## Data flow

* Don't hardcode the values (accept arguments instead)
* Choose carefully between accepting a parameter VS defining a constant:
  * Definitions:
    * Parameters are execution details (the user may want to change them)
    * Constants are implementation details (the user would never want to change them)
  * Examples:
    * Parameters:
      * Cache TTL
      * Config path
    * Constants:
      * Table name
      * Keyspace name
  * Recommendations:
    * When in doubt, prefer accepting a parameter instead of defining a constant

## Conversions

* Implement `From` or `TryFrom` for conversions between types (instead of converting in-place)

## Struct derives

* Derive `new` from `derive_new` crate for types that need `fn new`
* Derive `Serialize` and `Deserialize` from `serde` crate for types that need serialization / deserialization
* If the struct derives `Getters`, then each field whose type implements `Copy` must have a `#[getter(copy)]` annotation. For example:
  * Good (note that `username` doesn't have `#[getter(copy)]` because its type is `String` which doesn't implement `Copy`, but `age` has `#[getter(copy)]`, because its type is `u64` which implements `Copy`):
    ```rust
    #[derive(Getters, Into, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
    pub struct User {
      username: String,
      #[getter(copy)]
      age: u64,
    }
    ```

## Visibility

* By default, every type and function should be `pub`
* Instead of `pub(crate)`, write `pub`
* If a struct has a `new` method that returns a `Result`, then this is a private struct, so it must not be `pub`
* Every field of a private struct must be private (not `pub`) to enforce validation
* A private struct must always implement `TryFrom` instead of `From` (must never implement `From`) to enforce validation
* A private struct that has `#[derive(Deserialize)]` must always use `#[serde(try_from = ...)]` to enforce validation during deserialization
* A private struct should not implement `Default` in most cases (very rarely it may implement `Default` only if the default value is a valid value)
* The code must always call the `new` method to enforce validation

## Setters

* Use setters that take `&mut self` instead of setters that take `self` and return `Self` (because passing a `foo: &mut Foo` is better than passing `foo: Foo` and returning `Foo` through the call stack)

## Constructors

* If the type constructor doesn't have side effects, then use the name `new`, else use the name `create`

## Newtypes

* The macro calls that begin with `subtype` (for example, `subtype!` and `subtype_string!`) expand to newtypes

## Enums

* When writing code related to enums, bring the variants in scope with `use Enum::*;` statement at the top of the file or function (prefer "at the top of the file" for data enums, prefer "at the top of the function" for error enums).

## Code style

* Implement proper error handling instead of `unwrap` or `expect` (in normal code and in tests)
  * Use `expect` only in exceptional cases where you can prove that it always succeeds, and provide the proof as the first argument to `expect` (the proof must start with "always succeeds because")
* The file names must match the names of the primary item in this file (for example: a file with `struct User` must be in `user.rs`)
* Don't use `mod.rs`, use module files with submodules in the folder with the same name (for example: `user.rs` with submodules in `user` folder)
* Put the trait implementations in the same file as the target struct (for example: put `impl TryFrom<...> for User` in the same file as `struct User`, which is `user.rs`)
* Use destructuring assignment for tuple arguments, for example: `fn try_from((name, parent_key): (&str, GroupKey)) -> ...`
* Use iterators instead of for loops. For example:
  * Good:
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
  * Bad:
    ```rust
    // Bad: manual loop + mutable accumulator
    pub fn parse_numbers(inputs: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<u64>, core::num::ParseIntError> {
        let mut out = Vec::new();
        for s in inputs {
            let n = s.as_ref().trim().parse::<u64>()?;
            out.push(n);
        }
        Ok(out)
    }
    ```
* Prefer writing associated functions instead of standalone functions
* Add a local `use` statement for enums to minimize the code size. For example:
  * Good:
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
  * Bad:
    ```rust
    pub fn apply(op: GroupsOp) {
        match op {
            GroupsOp::InsertOne(_) => {}
            GroupsOp::UpdateOne(_, _) => {}
            GroupsOp::DeleteOne(_) => {}
        }
    }
    ```
* Simplify the callsite code by accepting `impl Into`. For example:
  * Good:
    ```rust
    pub fn foo(input: impl Into<String>) {
        let input = input.into();
        // do something
    }
    ```
  * Bad:
    ```rust
    /// This is bad because the callsite may have to call .into() when passing the input argument
    pub fn foo(input: String) {}
    ```
* Provide additional flexibility for callsite by accepting `&impl AsRef` or `&mut impl AsMut` (e.g. both `PathBuf` and `Config` may implement `AsRef<Path>`). For example:
  * Good:
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
  * Bad:
    ```rust
    /// This is bad because the callsite may have to call .as_mut() when passing the input argument
    pub fn bar(input: &mut String) {}
    
    /// This is bad because the callsite may have to call .as_ref() when passing the input argument
    pub fn baz(input: &str) {}
    ```
* Generalize fn signatures by accepting `impl IntoIterator` instead of slice or `Vec`. For example:
  * Good:
    ```rust
    pub fn foo<'a>(inputs: impl IntoIterator<Item = &'a str>) {
        // do something
    }
    
    pub fn bar(inputs: impl IntoIterator<Item = String>) {
        // do something
    }
    ```
  * Bad:
    ```rust
    /// This is bad because it is not general enough
    pub fn foo(inputs: &[str]) {}
    
    /// This is bad because it is not general enough and also forces the caller to collect the strings into a vec, which is bad for performance
    pub fn bar(inputs: impl IntoIterator<Item = String>) {}
    ```
* Prefer `.map()` instead of `match` when you need to modify the value in the `Option` or `Result`. For example:
  * Good:
    ```rust
    impl core::str::FromStr for UserId {
        type Err = core::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u64>().map(Self::new)
        }
    }
    ```
  * Bad:
  ```rust
  impl core::str::FromStr for UserId {
      type Err = core::num::ParseIntError;
  
      fn from_str(s: &str) -> Result<Self, Self::Err> {
          // This is bad because it uses more code to express the same idea
          match s.parse::<u64>() {
              Ok(value) => Ok(Self::new(value)),
              Err(error) => Err(error),
          }
      }
  }
  ```
* Write `macro_rules!` macros to reduce boilerplate
* If you see similar code in different places, write a macro and replace the similar code with a macro call

## Sandbox

You are running in a sandbox with limited network access.

* See the list of allowed domains in /etc/dnsmasq.d/allowed_domains.conf
* If you need to read the data from other domains, use the web search tool (this tool is executed outside of sandbox)
