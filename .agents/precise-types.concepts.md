# Precise types concepts

## Bijection between Type A and Type B

"Bijection between Type A and Type B" is a relation that holds if:

* Conversion from A to B to A is identity
* Conversion from B to A to B is identity

## Bijection on Dataset D between Type A and Type B

"Bijection on Dataset D between Type A and Type B" is a relation that holds if for every element in Dataset D (that contains elements of Type A), the conversion from A to B to A is identity.

## Type A is clarified by Type B

"Type A is clarified by Type B" is a relation that holds if:

* Conversion from B to A to B is identity
* Type B imposes constraints on the data or restrictions on the API compared to Type A

Examples:

* `NonEmptyString` is a clarification of `String` because it imposes the "non-empty" constraint
* `Timestamp` is a clarification of `u64` because it restricts the API (for example, doesn't provide `impl Add<Timestamp> for Timestamp`, only `impl Add<Duration> for Timestamp`)

Notes:

* In other words, Type B is "safer" than Type A
* Conversion from A to B to A may fail
  * For example, it may fail if the data doesn't satisfy the constraints
* Conversion from A to B to A may always succeed
  * For example, if Type B only restricts the API without imposing any constraints (e.g. `Timestamp` clarifies `u64` without imposing constraints)

Requirements:

* Must have an `impl From<B> for A`
* If Type B imposes constraints:
  * Then: Must have an `impl TryFrom<A> for B` ([fallible clarification impl](#fallible-clarification-impl))
  * Else: Must have an `impl From<A> for B`

## Fallible clarification impl

An impl of `TryFrom` from Type A to Type B where Type A is clarified by Type B.

Requirements:

* Must have an `Error` associated type that is an error enum with a single variant that contains all fields of the input type:
  * Some of those fields must have `Result` type
  * The fields with `Result` type must have a `_result` suffix
  * May contain additional fields for variables that provide more information about why the conversion failed (e.g. `is_adult` in the example below)
  * Must return all fields, even those that are `Copy` (because the caller loses ownership of the whole input when it's passed into the `try_from` call)
* Must have a `try_from` function:
  * Must finish with a `match` with two arms:
    * The "success" arm that matches only on positive values or variants (`Ok`, `Some`, `true`)
    * The "failure" arm that matches on any values
      * Must return every value

Example:

```rust
use derive_getters::Getters;
use derive_more::Deref;
use errgonomic::handle_bool;
use thiserror::Error;

#[derive(Deref, Clone, Debug)]
pub struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = TryFromStringForNonEmptyStringError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        use TryFromStringForNonEmptyStringError::*;
        handle_bool!(input.is_empty(), EmptyInput, input);
        Ok(Self(input))
    }
}

#[derive(Error, Debug)]
pub enum TryFromStringForNonEmptyStringError {
    #[error("expected input to be non-empty")]
    EmptyInput { input: String },
}

#[derive(Getters, Clone, Debug)]
pub struct Human {
    name: String,
    #[getter(copy)]
    age: u32,
}

#[derive(Getters, Clone, Debug)]
pub struct Adult {
    name: NonEmptyString,
    #[getter(copy)]
    age: u32,
}

impl TryFrom<Human> for Adult {
    type Error = ConvertHumanToAdultError;

    fn try_from(input: Human) -> Result<Self, Self::Error> {
        use ConvertHumanToAdultError::*;
        let Human {
            name,
            age,
        } = input;
        let name_result = NonEmptyString::try_from(name);
        let is_adult = age > 18;
        match (name_result, is_adult) {
            (Ok(name), true) => Ok(Self {
                name,
                age,
            }),
            (name_result, is_adult) => Err(ConversionFailed {
                name_result,
                age,
                is_adult,
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConvertHumanToAdultError {
    #[error("failed to convert human to adult")]
    ConversionFailed { name_result: Result<NonEmptyString, TryFromStringForNonEmptyStringError>, age: u32, is_adult: bool },
}
```
