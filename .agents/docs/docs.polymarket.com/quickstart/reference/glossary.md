# Glossary

> Key terms and concepts for Polymarket developers

## Markets & Events

| Term             | Definition                                                                                                                                                                             |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Event**        | A collection of related markets grouped under a common topic. Example: "2024 US Presidential Election" contains markets for each candidate.                                            |
| **Market**       | A single tradeable outcome within an event. Each market has a Yes and No side. Corresponds to a condition ID, question ID, and pair of token IDs.                                      |
| **Token**        | Represents a position in a specific outcome (Yes or No). Prices range from 0.00 to 1.00. Winning tokens redeem for \$1 USDCe. Also called *outcome token* or referenced by *token ID*. |
| **Token ID**     | The unique identifier for a specific outcome token. Required when placing orders or querying prices.                                                                                   |
| **Condition ID** | Onchain identifier for a market's resolution condition. Used in CTF operations.                                                                                                        |
| **Question ID**  | Identifier linking a market to its resolution oracle (UMA).                                                                                                                            |
| **Slug**         | Human-readable URL identifier for a market or event. Found in Polymarket URLs: `polymarket.com/event/[slug]`                                                                           |

***

## Trading

| Term          | Definition                                                                                                                 |
| ------------- | -------------------------------------------------------------------------------------------------------------------------- |
| **CLOB**      | Central Limit Order Book. Polymarket's off-chain order matching system. Orders are matched here before onchain settlement. |
| **Tick Size** | The minimum price increment for a market. Usually `0.01` (1 cent) or `0.001` (0.1 cent).                                   |
| **Fill**      | When an order is matched and executed. Orders can be partially or fully filled.                                            |

***

## Order Types

| Term    | Definition                                                                                                       |
| ------- | ---------------------------------------------------------------------------------------------------------------- |
| **GTC** | Good-Til-Cancelled. An order that remains open until filled or manually cancelled.                               |
| **GTD** | Good-Til-Date. An order that expires at a specified time if not filled.                                          |
| **FOK** | Fill-Or-Kill. An order that must be filled entirely and immediately, or it's cancelled. No partial fills.        |
| **FAK** | Fill-And-Kill. An order that fills as much as possible immediately, then cancels any remaining unfilled portion. |

***

## Market Types

| Term                        | Definition                                                                                                                                           |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Binary Market**           | A market with exactly two outcomes: Yes and No. The prices always sum to approximately \$1.                                                          |
| **Negative Risk (NegRisk)** | A multi-outcome event where only one outcome can resolve Yes. Requires `negRisk: true` in order parameters. [Details](/developers/neg-risk/overview) |

***

## Wallets

| Term               | Definition                                                                                       |
| ------------------ | ------------------------------------------------------------------------------------------------ |
| **EOA**            | Externally Owned Account. A standard Ethereum wallet controlled by a private key.                |
| **Funder Address** | The wallet address that holds funds and tokens for trading.                                      |
| **Signature Type** | Identifies wallet type when trading. `0` = EOA, `1` = Magic Link proxy, `2` = Gnosis Safe proxy. |

***

## Token Operations (CTF)

| Term       | Definition                                                                           |
| ---------- | ------------------------------------------------------------------------------------ |
| **CTF**    | Conditional Token Framework. The onchain smart contracts that manage outcome tokens. |
| **Split**  | Convert USDCe into a complete set of outcome tokens (one Yes + one No).              |
| **Merge**  | Convert a complete set of outcome tokens back into USDCe.                            |
| **Redeem** | After resolution, exchange winning tokens for \$1 USDCe each.                        |

***

## Infrastructure

| Term        | Definition                                                                |
| ----------- | ------------------------------------------------------------------------- |
| **Polygon** | The blockchain network where Polymarket operates. Chain ID: `137`.        |
| **USDCe**   | The stablecoin used as collateral on Polymarket. Bridged USDC on Polygon. |

***
