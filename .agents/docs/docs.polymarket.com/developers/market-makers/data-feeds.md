# Data Feeds

> Real-time and historical data sources for market makers

## Overview

Market makers need fast, reliable data to price markets and manage inventory. Polymarket provides several data feeds at different latency and detail levels.

| Feed      | Latency    | Use Case                  | Access |
| --------- | ---------- | ------------------------- | ------ |
| WebSocket | \~100ms    | Standard MM operations    | Public |
| Gamma API | \~1s       | Market metadata, indexing | Public |
| Onchain   | Block time | Settlement, resolution    | Public |

## WebSocket Feeds

The WebSocket API provides real-time market data with low latency. This is sufficient for most market making strategies.

### Connecting

```typescript  theme={null}
const ws = new WebSocket("wss://ws-subscriptions-clob.polymarket.com/ws/market");

ws.onopen = () => {
  // Subscribe to orderbook updates
  ws.send(JSON.stringify({
    type: "market",
    assets_ids: [tokenId]
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Handle orderbook update
};
```

### Available Channels

| Channel  | Message Types                              | Documentation                                               |
| -------- | ------------------------------------------ | ----------------------------------------------------------- |
| `market` | `book`, `price_change`, `last_trade_price` | [Market Channel](/developers/CLOB/websocket/market-channel) |
| `user`   | Order fills, cancellations                 | [User Channel](/developers/CLOB/websocket/user-channel)     |

### User Channel (Authenticated)

Monitor your order activity in real-time:

```typescript  theme={null}
// Requires authentication
const userWs = new WebSocket("wss://ws-subscriptions-clob.polymarket.com/ws/user");

userWs.onopen = () => {
  userWs.send(JSON.stringify({
    type: "user",
    auth: {
      apiKey: "your-api-key",
      secret: "your-secret",
      passphrase: "your-passphrase"
    },
    markets: [conditionId] // Optional: filter to specific markets
  }));
};

userWs.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Handle order fills, cancellations, etc.
};
```

See [WebSocket Authentication](/developers/CLOB/websocket/wss-auth) for auth details.

### Best Practices

1. **Reconnection logic** - Implement automatic reconnection with exponential backoff
2. **Heartbeats** - Respond to ping messages to maintain connection
3. **Local orderbook** - Maintain a local copy and apply incremental updates
4. **Sequence numbers** - Track sequence to detect missed messages

See [WebSocket Overview](/developers/CLOB/websocket/wss-overview) for complete documentation.

## Gamma API

The Gamma API provides market metadata and indexing. Use it for:

* Market titles, slugs, categories
* Event/condition mapping
* Volume and liquidity data
* Outcome token metadata

### Get Markets

```typescript  theme={null}
const response = await fetch(
  "https://gamma-api.polymarket.com/markets?active=true"
);
const markets = await response.json();
```

### Get Events

```typescript  theme={null}
const response = await fetch(
  "https://gamma-api.polymarket.com/events?slug=us-presidential-election"
);
const event = await response.json();
```

### Key Fields for MMs

| Field           | Description              |
| --------------- | ------------------------ |
| `conditionId`   | Unique market identifier |
| `clobTokenIds`  | Outcome token IDs        |
| `outcomes`      | Outcome names            |
| `outcomePrices` | Current outcome prices   |
| `volume`        | Trading volume           |
| `liquidity`     | Current liquidity        |

See [Gamma API Overview](/developers/gamma-markets-api/overview) for complete documentation.

## Onchain Data

For settlement, resolution, and position tracking, market makers may query onchain data directly.

### Data Sources

| Data                 | Source              | Use Case                     |
| -------------------- | ------------------- | ---------------------------- |
| Token balances       | ERC1155 `balanceOf` | Position tracking            |
| Resolution           | UMA Oracle events   | Pre-resolution risk modeling |
| Condition resolution | CTF contract        | Post-resolution redemption   |

### RPC Providers

Common providers for Polygon:

* Alchemy
* QuickNode
* Infura

### UMA Oracle

Markets are resolved via UMA's Optimistic Oracle. Monitor resolution events for risk management.

See [Resolution](/developers/resolution/UMA) for details on the resolution process.

## Related Documentation

<CardGroup cols={3}>
  <Card title="WebSocket Overview" icon="plug" href="/developers/CLOB/websocket/wss-overview">
    Complete WebSocket documentation
  </Card>

  <Card title="Gamma API" icon="database" href="/developers/gamma-markets-api/overview">
    Market metadata and indexing
  </Card>

  <Card title="Resolution" icon="gavel" href="/developers/resolution/UMA">
    UMA Oracle resolution process
  </Card>
</CardGroup>
