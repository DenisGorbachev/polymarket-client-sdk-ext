# Precise types concepts

## Bijection between Type A and Type B

"Bijection between Type A and Type B" is a relation that holds if:

* Conversion from A to B to A is identity
* Conversion from B to A to B is identity

## Injection on specific `List<A>` between Type A and Type B

"Injection on specific `List<A>` between Type A and Type B" is a relation that holds if for every element in specific `List<A>` the conversion from A to B to A is identity.

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
  * Then:
    * Must have an `impl TryFrom<A> for B` ([fallible clarification impl](#fallible-clarification-impl))
    * Must pass the identity test: `assert_eq!(Ok(b), B::try_from(A::from(b.clone())))`
  * Else:
    * Must have an `impl From<A> for B`
    * Must pass the identity test: `assert_eq!(b, B::from(A::from(b.clone())))`

Notes:

* The requirements assume that `impl From<B> for A` exists

## Fallible clarification impl

An impl of `TryFrom` from Type A to Type B where Type A is clarified by Type B.

Requirements:

* If there is only one boolean constraint:
  * Then:
    * Must have a `try_from` function:
      * Must finish with an `if`
    * Must have an `Error` associated type that is an error enum with a single variant with fields for all variables that are accessible in the "failure" arm of the `if`
  * Else:
    * Must have a `try_from` function:
      * Must finish with a `match` with two arms:
        * The "success" arm that matches only on positive values or variants (`Ok`, `Some`, `true`)
        * The "failure" arm that matches on any values
          * Must return every value in the error enum variant
    * Must have an `Error` associated type that is an error enum with a single variant with fields for all variables that are accessible in the "failure" arm of the `match`:
      * Some variables may come from the initial destructuring assignment (such variables are not a part of the match arm because they do not determine the success or failure of the conversion)
      * Some variables may come from the fallible expressions after the initial destructuring assignment
        * Those variables may have `Result` or `Option` type
      * Some variables may come from calculations that were necessary to check if the conversion should succeed or fail (e.g. `is_adult` in the example below) (such variables provide more information about why the conversion failed)
      * The fields with `Result` type must have `_result` suffix
      * The fields with `Option` type must have `_option` suffix
      * Must return all variables, even those that are `Copy` (because the caller loses ownership of the whole input when it's passed into the `try_from` call)
        * Note: the variables are part of the input, not the whole input, so the rule "If an argument of callee implements `Copy`, the callee must not include it in the list of error enum variant fields" does not apply here (it only applies to arguments, not parts of arguments).

Example:

```rust
use derive_getters::Getters;
use derive_more::Deref;
use errgonomic::handle_bool;
use thiserror::Error;

#[derive(Deref, Clone, Debug)]
pub struct NonEmptyString(String);

/// This is an example of a "simple" fallible conversion
/// `handle_bool!` is used because there's only one boolean constraint
impl TryFrom<String> for NonEmptyString {
    type Error = ConvertStringToNonEmptyStringError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        use ConvertStringToNonEmptyStringError::*;
        handle_bool!(input.is_empty(), EmptyInput, input);
        Ok(Self(input))
    }
}

#[derive(Error, Debug)]
pub enum ConvertStringToNonEmptyStringError {
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

/// This is an example of "normal" fallible conversion
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
    ConversionFailed { name_result: Result<NonEmptyString, ConvertStringToNonEmptyStringError>, age: u32, is_adult: bool },
}
```
