# Maker Rebates Program

> Technical guide for handling taker fees and earning maker rebates on Polymarket

Polymarket has enabled **taker fees** on **15-minute crypto markets**. These fees fund a **Maker Rebates** program that pays daily USDC rebates to liquidity providers.

## Fee Handling by Implementation Type

### Option 1: Official CLOB Clients (Recommended)

The official CLOB clients **automatically handle fees** for you. Update to the latest version:

<CardGroup cols={2}>
  <Card title="TypeScript Client" icon="js" href="https://github.com/Polymarket/clob-client">
    npm install @polymarket/clob-client\@latest
  </Card>

  <Card title="Python Client" icon="python" href="https://github.com/Polymarket/py-clob-client">
    pip install --upgrade py-clob-client
  </Card>
</CardGroup>

**What the client does automatically:**

1. Fetches the fee rate for the market's token ID
2. Includes `feeRateBps` in the order structure
3. Signs the order with the fee rate included

**You don't need to do anything extra**. Just update your client and your orders will work on fee-enabled markets.

***

### Option 2: REST API / Custom Implementations

If you're calling the REST API directly or building your own order signing, you must manually include the fee rate in your **signed order payload**.

#### Step 1: Fetch the Fee Rate

Query the fee rate for the token ID before creating your order:

```bash  theme={null}
GET https://clob.polymarket.com/fee-rate?token_id={token_id}
```

**Response:**

```json  theme={null}
{
  "fee_rate_bps": 1000
}
```

* **Fee-enabled markets** return a value like `1000`
* **Fee-free markets** return `0`

#### Step 2: Include in Your Signed Order

Add the `feeRateBps` field to your order object. This value is **part of the signed payload**, the CLOB validates your signature against it.

```json  theme={null}
{
  "salt": "12345",
  "maker": "0x...",
  "signer": "0x...",
  "taker": "0x...",
  "tokenId": "71321045679252212594626385532706912750332728571942532289631379312455583992563",
  "makerAmount": "50000000",
  "takerAmount": "100000000",
  "expiration": "0",
  "nonce": "0",
  "feeRateBps": "1000",
  "side": "0",
  "signatureType": 2,
  "signature": "0x..."
}
```

#### Step 3: Sign and Submit

1. Include `feeRateBps` in the order object **before signing**
2. Sign the complete order
3. POST to `/order` endpoint

<Note>
  **Important:** Always fetch `fee_rate_bps` dynamically, do not hardcode. The fee rate may vary by market or change over time. You only need to pass `feeRateBps`
</Note>

See the [Create Order documentation](/developers/CLOB/orders/create-order) for full signing details.

***

## Fee Behavior

Fees are calculated in USDC and vary based on the share price. The effective rate **peaks at 50%** probability and decreases symmetrically toward the extremes.

### Fee Table (100 shares)

| Price  | Trade Value | Fee (USDC) | Effective Rate |
| ------ | ----------- | ---------- | -------------- |
| \$0.10 | \$10        | \$0.02     | 0.20%          |
| \$0.20 | \$20        | \$0.13     | 0.64%          |
| \$0.30 | \$30        | \$0.33     | 1.10%          |
| \$0.40 | \$40        | \$0.58     | 1.44%          |
| \$0.50 | \$50        | \$0.78     | **1.56%**      |
| \$0.60 | \$60        | \$0.86     | 1.44%          |
| \$0.70 | \$70        | \$0.77     | 1.10%          |
| \$0.80 | \$80        | \$0.51     | 0.64%          |
| \$0.90 | \$90        | \$0.18     | 0.20%          |

The maximum effective fee rate is **1.56%** at 50% probability. Fees are the same for both buying and selling.

***

## Maker Rebates

### How Rebates Work

* **Eligibility:** Your orders must add liquidity (maker orders) and get filled
* **Calculation:** Proportional to your share of executed maker volume in each eligible market
* **Payment:** Daily in USDC, paid directly to your wallet

### Rebate Pool

The rebate pool for each market is funded by taker fees collected in that market. The payout percentage is subject to change:

| Period                                           | Maker Rebate |
| ------------------------------------------------ | ------------ |
| Jan 9 – Jan 11, 2026 (Until Sunday Midnight UTC) | 100%         |
| Jan 12 – Jan 18, 2026                            | 20%          |

The rebate percentage is at the sole discretion of Polymarket.

***

## Which Markets Have Fees?

Currently, only **15-minute crypto markets** have fees enabled. Query the fee-rate endpoint to check:

```bash  theme={null}
GET https://clob.polymarket.com/fee-rate?token_id={token_id}

# Fee-enabled: { "fee_rate_bps": 1000 }
# Fee-free:    { "fee_rate_bps": 0 }
```

***

## Related Documentation

<CardGroup cols={2}>
  <Card title="Maker Rebates Program" icon="coins" href="/polymarket-learn/trading/maker-rebates-program">
    User-facing overview with full fee tables
  </Card>

  <Card title="Create CLOB Order via REST API" icon="code" href="/developers/CLOB/orders/create-order">
    Full order structure and signing documentation
  </Card>
</CardGroup>

***
