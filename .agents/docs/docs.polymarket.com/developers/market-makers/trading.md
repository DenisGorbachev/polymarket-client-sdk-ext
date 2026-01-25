# Trading

> CLOB order entry and management for market makers

## Overview

Market makers primarily interact with Polymarket through the CLOB (Central Limit Order Book) API to post and manage limit orders.

## Order Entry

### Posting Limit Orders

Use the CLOB client to create and post limit orders:

```typescript  theme={null}
import { ClobClient, Side, OrderType } from "@polymarket/clob-client";

const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  wallet,
  credentials,
  signatureType,
  funder
);

// Post a bid (buy order)
const bidOrder = await client.createAndPostOrder({
  tokenID: "34097058504275310827233323421517291090691602969494795225921954353603704046623",
  side: Side.BUY,
  price: 0.48,
  size: 1000,
  orderType: OrderType.GTC
});

// Post an ask (sell order)
const askOrder = await client.createAndPostOrder({
  tokenID: "34097058504275310827233323421517291090691602969494795225921954353603704046623",
  side: Side.SELL,
  price: 0.52,
  size: 1000,
  orderType: OrderType.GTC
});
```

See [Create Order](/developers/CLOB/clients/methods-l1#createandpostorder) for full documentation.

### Batch Orders

For efficiency, post multiple orders in a single request:

```typescript  theme={null}
const orders = await Promise.all([
  client.createOrder({ tokenID, side: Side.BUY, price: 0.48, size: 500 }),
  client.createOrder({ tokenID, side: Side.BUY, price: 0.47, size: 500 }),
  client.createOrder({ tokenID, side: Side.SELL, price: 0.52, size: 500 }),
  client.createOrder({ tokenID, side: Side.SELL, price: 0.53, size: 500 })
]);

const response = await client.postOrders(
  orders.map(order => ({ order, orderType: OrderType.GTC }))
);
```

See [Post Orders Batch](/developers/CLOB/clients/methods-l2#postorders) for details.

## Order Types

| Type                          | Behavior                                | MM Use Case                             |
| ----------------------------- | --------------------------------------- | --------------------------------------- |
| **GTC** (Good Till Cancelled) | Rests on book until filled or cancelled | Default for passive quoting             |
| **GTD** (Good Till Date)      | Auto-expires at specified time          | Auto-expire before events               |
| **FOK** (Fill or Kill)        | Fill entirely immediately or cancel     | Aggressive rebalancing (all or nothing) |
| **FAK** (Fill and Kill)       | Fill available immediately, cancel rest | Partial rebalancing acceptable          |

### When to Use Each

**For passive market making (maker orders):**

* **GTC** - Standard quotes that sit on the book
* **GTD** - Time-limited quotes (e.g., expire before market close)

**For rebalancing (taker orders):**

* **FOK** - When you need exact size or nothing
* **FAK** - When partial fills are acceptable

```typescript  theme={null}
// GTD example: expire in 1 hour
const expiringOrder = await client.createOrder({
  tokenID,
  side: Side.BUY,
  price: 0.50,
  size: 1000,
  orderType: OrderType.GTD,
  expiration: Math.floor(Date.now() / 1000) + 3600 // 1 hour from now
});
```

## Order Management

### Cancel Orders

Cancel individual orders or all orders:

```typescript  theme={null}
// Cancel single order
await client.cancelOrder(orderId);

// Cancel multiple orders in a single calls
await client.cancelOrders(orderIds: string[]);

// Cancel all orders for a market
await client.cancelMarketOrders(conditionId);

// Cancel all orders
await client.cancelAll();
```

See [Cancel Orders](/developers/CLOB/clients/methods-l2#cancelorder) for full documentation.

### Get Active Orders

Monitor your open orders:

```typescript  theme={null}
// Get active order
const order = await client.getOrder(orderId);

// Get active orders optionally filtered
const orders = await client.getOpenOrders({
  id?: string; // Order ID (hash)
  market?: string; // Market condition ID
  asset_id?: string; // Token ID
});
```

See [Get Active Orders](/developers/CLOB/clients/methods-l2#getorder) for details.

## Best Practices

### Quote Management

1. **Two-sided quoting** - Post both bids and asks to earn maximum [liquidity rewards](/developers/market-makers/liquidity-rewards)
2. **Monitor inventory** - Skew quotes based on your position
3. **Cancel stale quotes** - Remove orders when market conditions change
4. **Use GTD for events** - Auto-expire quotes before known events

### Latency Optimization

1. **Batch orders** - Use `postOrders()` instead of multiple `createAndPostOrder()` calls
2. **WebSocket for data** - Use WebSocket feeds instead of polling REST endpoints

### Risk Management

1. **Size limits** - Check token balances before quoting; don't exceed inventory
2. **Price guards** - Validate against book midpoint; reject outlier prices
3. **Kill switch** - Use `cancelAll()` on error or position breach
4. **Monitor fills** - Subscribe to WebSocket user channel for real-time fill updates

## Tick Sizes

Markets have different minimum price increments:

```typescript  theme={null}
const tickSize = await client.getTickSize(tokenID);
// Returns: "0.1" | "0.01" | "0.001" | "0.0001"
```

Ensure your prices conform to the market's tick size.

## Fee Structure

| Role  | Fee   |
| ----- | ----- |
| Maker | 0 bps |
| Taker | 0 bps |

Current fees are 0% for both makers and takers. See [CLOB Introduction](/developers/CLOB/introduction) for fee calculation details.

## Related Documentation

<CardGroup cols={2}>
  <Card title="CLOB Client Overview" icon="code" href="/developers/CLOB/clients/methods-overview">
    Complete client method reference
  </Card>

  <Card title="L2 Methods" icon="lock" href="/developers/CLOB/clients/methods-l2">
    Authenticated order management methods
  </Card>

  <Card title="WebSocket Feeds" icon="plug" href="/developers/CLOB/websocket/wss-overview">
    Real-time order and market data
  </Card>

  <Card title="Liquidity Rewards" icon="gift" href="/developers/market-makers/liquidity-rewards">
    Earn rewards for providing liquidity
  </Card>
</CardGroup>
