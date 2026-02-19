# Simulator concepts

## Exchange

* Must have the following methods:
  * `place_order(&mut self, market_id: MarketId, price: Price, amount: Amount, algo: ExecutionAlgo) -> Result<Info<OrderId, Order>, ExchangePlaceOrderError>`
  * ... (proxy methods from market) ...
* Must pass the following tests:
  * `must_reject_order_if_not_enough_balance`

## Market

A struct that implements a trading interface.

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
