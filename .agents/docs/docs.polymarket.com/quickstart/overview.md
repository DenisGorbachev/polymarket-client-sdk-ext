# Developer Quickstart

> Get started building with Polymarket APIs

Polymarket provides a suite of APIs and SDKs for building prediction market applications. This guide will help you understand what's available and where to find it.

***

## What Can You Build?

| If you want to...             | Start here                                                          |
| ----------------------------- | ------------------------------------------------------------------- |
| Fetch markets & prices        | [Fetching Market Data](/quickstart/fetching-data)                   |
| Place orders for yourself     | [Placing Your First Order](/quickstart/first-order)                 |
| Build a trading app for users | [Builders Program Introduction](/developers/builders/builder-intro) |
| Provide liquidity             | [Market Makers](/developers/market-makers/introduction)             |

***

## APIs at a Glance

### Markets & Data

<CardGroup cols={2}>
  <Card title="Gamma API" icon="database" href="/developers/gamma-markets-api/overview">
    **Market discovery & metadata**

    Fetch events, markets, categories, and resolution data. This is where you discover what's tradeable.

    `https://gamma-api.polymarket.com`
  </Card>

  <Card title="CLOB API" icon="book" href="/developers/CLOB/introduction">
    **Prices, orderbooks & trading**

    Get real-time prices, orderbook depth, and place orders. The core trading API.

    `https://clob.polymarket.com`
  </Card>

  <Card title="Data API" icon="chart-bar" href="/developers/misc-endpoints/data-api-get-positions">
    **Positions, activity & history**

    Query user positions, trade history, and portfolio data.

    `https://data-api.polymarket.com`
  </Card>

  <Card title="WebSocket" icon="bolt" href="/developers/CLOB/websocket/wss-overview">
    **Real-time updates**

    Subscribe to orderbook changes, price updates, and order status.

    `wss://ws-subscriptions-clob.polymarket.com`
  </Card>
</CardGroup>

### Additional Data Sources

<CardGroup cols={2}>
  <Card title="RTDS" icon="signal-stream" href="/developers/RTDS/RTDS-overview">
    **Low-latency data stream**

    Real-time crypto prices and comments. Optimized for market makers.
  </Card>

  <Card title="Subgraph" icon="diagram-project" href="/developers/subgraph/overview">
    **Onchain queries**

    Query blockchain state directly via GraphQL.
  </Card>
</CardGroup>

### Trading Infrastructure

<CardGroup cols={2}>
  <Card title="CTF Operations" icon="arrows-split-up-and-left" href="/developers/CTF/overview">
    **Token split/merge/redeem**

    Convert between USDC and outcome tokens. Essential for inventory management.
  </Card>

  <Card title="Relayer Client" icon="gas-pump" href="/developers/builders/relayer-client">
    **Gasless transactions**

    Builders can offer gasfree transactions via Polymarket's relayer.
  </Card>
</CardGroup>

***

## SDKs & Libraries

<CardGroup cols={2}>
  <Card title="CLOB Client (TypeScript)" icon="npm" href="https://github.com/Polymarket/clob-client">
    `npm install @polymarket/clob-client`
  </Card>

  <Card title="CLOB Client (Python)" icon="python" href="https://github.com/Polymarket/py-clob-client">
    `pip install py-clob-client`
  </Card>
</CardGroup>

For builders routing orders for users:

<CardGroup cols={2}>
  <Card title="Relayer Client" icon="bolt" href="https://github.com/Polymarket/builder-relayer-client">
    Gasless wallet operations
  </Card>

  <Card title="Signing SDK" icon="key" href="https://github.com/Polymarket/builder-signing-sdk">
    Builder authentication headers
  </Card>
</CardGroup>

***
