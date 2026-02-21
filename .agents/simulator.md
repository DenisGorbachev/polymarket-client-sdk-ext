# Simulator concepts

TODO:

* Move the code from `instahouse-private` to `simsim`
* Add `simsim` crate as a dependency

## World

A struct that represents the world.

* Must have the following fields:
  * `observables: Vec<u64>` /// variables whose values serve as resolution criteria for prediction markets created by agents
* Must have the following methods:
  * `pub fn new(now: OffsetDateTime, rng: &mut impl Rng) -> Self`
  * `pub fn step(rng: &mut impl Rng)`
    * Must mutate `observables`
      * Keep `observables[0]` unchanged
      * Set `observables[1]` to a random value
      * Set `observables[1]` to `observables[1].wrapping_add(1)`
      * Set `observables[2]` to `observables[1].wrapping_mul(2)`
      * Set `observables[3]` to `natural_log(observables[2])`
* Must not have a field for `impl Rng`
  * Rationale:
    * We want to serialize the world
    * We can pass an `impl Rng` to methods directly

Open questions:

* How to ensure fairness? (ensure that some actors will have a turn)

Notes:

* Online LLM actors have inherently different speed and stochastic outputs (LLMs)
* The LLM outputs should be cached
  * But then we won't be able to replay them because the replayed calls will be made at an earlier time compared to actual calls (so the exchange state will be different)

## Exchange

A struct that implements an exchange interface.

* Must have the following methods:
  * `place_order(&mut self, market_id: MarketId, price: Price, amount: Amount, algo: ExecutionAlgo) -> Result<Info<OrderId, Order>, ExchangePlaceOrderError>`
  * ... (proxy methods from market) ...
* Must pass the following tests:
  * `must_reject_order_if_not_enough_balance`

## Market

A struct that implements a market interface.

* Must use `NonZeroU64` for `Price` and `Amount`
* Must have the following methods:
  * `place_order(&mut self, price: Price, amount: Amount, algo: ExecutionAlgo) -> Result<Info<OrderId, Order>, MarketPlaceOrderError>`
    * Must return information about filled amount
  * `cancel_order(&mut self, id: OrderId) -> Result<(), MarketCancelOrderError>`
  * `book(&self) -> &Book`
* Must pass the following tests:
  * `must_always_have_valid_book`
    * Generate a random sequence of actions
    * Run the full sequence
    * Test that book is valid

Notes:

* Market orders are implemented as limit orders with min or max price

## Book

```rust
pub struct Book(IndexMap<FxHasher, Price, Amount>);
```

* Must have the following methods:
  * `validate(&self) -> Result<(), BookValidateError>`
    * Must validate that the book is not crossed

## Price

`Price(u64)`

## Amount

`Amount(i64)`

Notes:

* Negative amounts are sells, positive amounts are buys

## ExecutionStyle

```rust
pub enum ExecutionAlgo {
    #[default]
    GTC,
    FOK
}
```
