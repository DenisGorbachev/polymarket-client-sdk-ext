# Market Maker Introduction

> Overview of market making on Polymarket and available tools for liquidity providers

## What is a Market Maker?

A Market Maker (MM) on Polymarket is a sophisticated trader who provides liquidity to prediction markets by continuously posting bid and ask orders. By "laying the spread," market makers enable other users to trade efficiently while earning the spread as compensation for the risk they take.

Market makers are essential to Polymarket's ecosystem:

* **Provide liquidity** across all markets
* **Tighten spreads** for better user experience
* **Enable price discovery** through continuous quoting
* **Absorb trading flow** from retail and institutional users

**Not a Market Maker?** If you're building an application that routes orders for your
users, see the [Builders Program](/developers/builders/builder-intro) instead. Builders
get access to gasless transactions via the Relayer Client and can earn grants through order attribution.

## Getting Started

To become a market maker on Polymarket:

1. **Complete setup** - Deploy wallets, fund with USDCe, set token approvals
2. **Connect to data feeds** - WebSocket for orderbook, RTDS for low-latency data
3. **Start quoting** - Post orders via CLOB REST API

## Available Tools

### By Action Type

<CardGroup cols={2}>
  <Card title="Setup" icon="gear" href="/developers/market-makers/setup">
    Deposits, token approvals, wallet deployment, API keys
  </Card>

  <Card title="Trading" icon="chart-line" href="/developers/market-makers/trading">
    CLOB order entry, order types, quoting best practices
  </Card>

  <Card title="Data Feeds" icon="database" href="/developers/market-makers/data-feeds">
    WebSocket, RTDS, Gamma API, on-chain data
  </Card>

  <Card title="Inventory Management" icon="boxes-stacked" href="/developers/market-makers/inventory">
    Split, merge, and redeem outcome tokens
  </Card>

  <Card title="Liquidity Rewards" icon="gift" href="/developers/market-makers/liquidity-rewards">
    Earn rewards for providing liquidity
  </Card>

  <Card title="Maker Rebates Program" icon="gift" href="/developers/market-makers/maker-rebates-program">
    Earn rebates for providing liquidity
  </Card>
</CardGroup>

## Quick Reference

| Action                | Tool           | Documentation                                                 |
| --------------------- | -------------- | ------------------------------------------------------------- |
| Deposit USDCe         | Bridge API     | [Bridge Overview](/developers/misc-endpoints/bridge-overview) |
| Approve tokens        | Relayer Client | [Setup Guide](/developers/market-makers/setup)                |
| Post limit orders     | CLOB REST API  | [CLOB Client](/developers/CLOB/clients/methods-l2)            |
| Monitor orderbook     | WebSocket      | [WebSocket Overview](/developers/CLOB/websocket/wss-overview) |
| Low-latency data      | RTDS           | [Data Feeds](/developers/market-makers/data-feeds)            |
| Split USDCe to tokens | CTF / Relayer  | [Inventory](/developers/market-makers/inventory)              |
| Merge tokens to USDCe | CTF / Relayer  | [Inventory](/developers/market-makers/inventory)              |

## Support

For market maker onboarding and support, contact [support@polymarket.com](mailto:support@polymarket.com).
