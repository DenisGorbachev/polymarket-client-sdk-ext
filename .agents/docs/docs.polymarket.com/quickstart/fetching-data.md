# Fetching Market Data

> Fetch Polymarket data in minutes with no authentication required

Get market data with zero setup. No API key, no authentication, no wallet required.

***

## Understanding the Data Model

Before fetching data, understand how Polymarket structures its markets:

<Steps>
  <Step title="Event">
    The top-level object representing a question like "Will X happen?"
  </Step>

  <Step title="Market">
    Each event contains one or more markets. Each market is a specific tradable binary outcome.
  </Step>

  <Step title="Outcomes & Prices">
    Markets have `outcomes` and `outcomePrices` arrays that map 1:1. These prices represent implied probabilities.
  </Step>
</Steps>

```json  theme={null}
{
  "outcomes": "[\"Yes\", \"No\"]",
  "outcomePrices": "[\"0.20\", \"0.80\"]"
}
// Index 0: "Yes" → 0.20 (20% probability)
// Index 1: "No" → 0.80 (80% probability)
```

***

## Fetch Active Events

List all currently active events on Polymarket:

```bash  theme={null}
curl "https://gamma-api.polymarket.com/events?active=true&closed=false&limit=5"
```

<Accordion title="Example Response">
  ```json  theme={null}
  [
    {
      "id": "123456",
      "slug": "will-bitcoin-reach-100k-by-2025",
      "title": "Will Bitcoin reach $100k by 2025?",
      "active": true,
      "closed": false,
      "tags": [
        { "id": "21", "label": "Crypto", "slug": "crypto" }
      ],
      "markets": [
        {
          "id": "789",
          "question": "Will Bitcoin reach $100k by 2025?",
          "clobTokenIds": ["TOKEN_YES_ID", "TOKEN_NO_ID"],
          "outcomes": "[\"Yes\", \"No\"]",
          "outcomePrices": "[\"0.65\", \"0.35\"]"
        }
      ]
    }
  ]
  ```
</Accordion>

<Tip>
  Always use `active=true&closed=false` to filter for live, tradable events.
</Tip>

***

## Market Discovery Best Practices

### For Sports Events

Use the `/sports` endpoint to discover leagues, then query by `series_id`:

```bash  theme={null}
# Get all supported sports leagues
curl "https://gamma-api.polymarket.com/sports"

# Get events for a specific league (e.g., NBA series_id=10345)
curl "https://gamma-api.polymarket.com/events?series_id=10345&active=true&closed=false"

# Filter to just game bets (not futures) using tag_id=100639
curl "https://gamma-api.polymarket.com/events?series_id=10345&tag_id=100639&active=true&closed=false&order=startTime&ascending=true"
```

<Note>
  `/sports` only returns automated leagues. For others (UFC, Boxing, F1, Golf, Chess), use tag IDs via `/events?tag_id=<tag_id>`.
</Note>

### For Non-Sports Topics

Use `/tags` to discover all available categories, then filter events:

```bash  theme={null}
# Get all available tags
curl "https://gamma-api.polymarket.com/tags?limit=100"

# Query events by topic
curl "https://gamma-api.polymarket.com/events?tag_id=2&active=true&closed=false"
```

<Tip>
  Each event response includes a `tags` array, useful for discovering categories from live data and building your own tag mapping.
</Tip>

***

## Get Market Details

Once you have an event, get details for a specific market using its ID or slug:

```bash  theme={null}
curl "https://gamma-api.polymarket.com/markets?slug=will-bitcoin-reach-100k-by-2025"
```

The response includes `clobTokenIds`, you'll need these to fetch prices and place orders.

***

## Get Current Price

Query the CLOB for the current price of any token:

```bash  theme={null}
curl "https://clob.polymarket.com/price?token_id=YOUR_TOKEN_ID&side=buy"
```

<Accordion title="Example Response">
  ```json  theme={null}
  {
    "price": "0.65"
  }
  ```
</Accordion>

***

## Get Orderbook Depth

See all bids and asks for a market:

```bash  theme={null}
curl "https://clob.polymarket.com/book?token_id=YOUR_TOKEN_ID"
```

<Accordion title="Example Response">
  ```json  theme={null}
  {
    "market": "0x...",
    "asset_id": "YOUR_TOKEN_ID",
    "bids": [
      { "price": "0.64", "size": "500" },
      { "price": "0.63", "size": "1200" }
    ],
    "asks": [
      { "price": "0.66", "size": "300" },
      { "price": "0.67", "size": "800" }
    ]
  }
  ```
</Accordion>

***

## More Data APIs

<CardGroup cols={2}>
  <Card title="Gamma API" icon="database" href="/developers/gamma-markets-api/overview">
    Deep dive into market discovery
  </Card>

  <Card title="Data API" icon="table" href="/developers/misc-endpoints/data-api-get-positions">
    Positions, activity, and holders data
  </Card>

  <Card title="WebSocket" icon="bolt" href="/developers/CLOB/websocket/wss-overview">
    Real-time orderbook updates
  </Card>

  <Card title="RTDS" icon="signal-stream" href="/developers/RTDS/RTDS-overview">
    Real-time data streaming service
  </Card>
</CardGroup>

***
