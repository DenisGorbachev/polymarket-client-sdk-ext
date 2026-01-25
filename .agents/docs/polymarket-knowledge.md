# Polymarket knowledge

## Documentation

- Polymarket has an [LLM-friendly documentation index](https://docs.polymarket.com/llms.txt).
  - All files from this index have already been downloaded to `.agents/docs/polymarket` on 2026-01-22
  - To download again, execute [Download Polymarket Docs for Developers](../../specs/download-polymarket-docs.md)

## Database

- Market's primary key is `condition_id`
- Orderbook's primary key is `asset_id` (aka `token_id`)

## Market fields

- `neg_risk_market_id` does not identify one specific binary market. It identifies the multi‑outcome "negative‑risk market" group (the whole event’s mutually exclusive set) in the NegRiskAdapter contract. Every market/outcome in that negative‑risk event shares the same `neg_risk_market_id`; the per‑market identifier is instead the market’s own `condition_id` (CLOB) and, on the neg‑risk adapter side, its `neg_risk_request_id`/`questionId` (which is derived from the group ID + an index).
  - Negative‑risk links all markets within an event, allowing NO in one market to convert into YES across the others—so the “market” that matters for neg‑risk is the event‑level grouping, not a single binary market.
  - In the NegRiskAdapter contract, the marketId is defined as a hash of oracle+fee+metadata, and each questionId shares the first 31 bytes with its marketId and differs only by the final byte (the question index). That design only makes sense if one marketId represents the group, and each individual binary market is one question within that group.

## Disputed markets

- The Polymarket event pages don’t expose a public “disputed list” endpoint
- The “Disputed” label appears in the event page data.
- Each event page has a Next.js data endpoint at: `https://polymarket.com/_next/data/{buildId}/event/{slug}.json`
- The `{buildId}` is embedded in Polymarket’s homepage HTML as `"buildId":"..."`.
- The JSON payload for an event contains a boolean flag indicating dispute status; in observed payloads this can appear as either `wasDisputed: true` or `isDisputed: true`.
- Therefore, to identify disputed markets you must:
  1) Fetch `https://polymarket.com/` and extract `buildId`.
  2) For each market slug, fetch the `_next/data` JSON.
  3) Mark the market as disputed if any JSON node contains `wasDisputed` or `isDisputed` set to `true`.
- Slugs can be enumerated from Polymarket’s Gamma API via the `markets` endpoint (used by `polymarket-client-sdk`), then deduplicated and checked one‑by‑one against the `_next/data` endpoints.
