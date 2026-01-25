# Placing Your First Order

> Set up authentication and submit your first trade

This guide walks you through placing an order on Polymarket using your own wallet.

***

## Installation

<CodeGroup>
  ```bash TypeScript theme={null}
  npm install @polymarket/clob-client ethers@5
  ```

  ```bash Python theme={null}
  pip install py-clob-client
  ```
</CodeGroup>

***

## Step 1: Initialize Client with Private Key

<CodeGroup>
  ```typescript TypeScript theme={null}
  import { ClobClient } from "@polymarket/clob-client";
  import { Wallet } from "ethers"; // v5.8.0

  const HOST = "https://clob.polymarket.com";
  const CHAIN_ID = 137; // Polygon mainnet
  const signer = new Wallet(process.env.PRIVATE_KEY);

  const client = new ClobClient(HOST, CHAIN_ID, signer);
  ```

  ```python Python theme={null}
  from py_clob_client.client import ClobClient
  import os

  host = "https://clob.polymarket.com"
  chain_id = 137  # Polygon mainnet
  private_key = os.getenv("PRIVATE_KEY")

  client = ClobClient(host, key=private_key, chain_id=chain_id)
  ```
</CodeGroup>

***

## Step 2: Derive User API Credentials

Your private key is used once to derive API credentials. These credentials authenticate all subsequent requests.

<CodeGroup>
  ```typescript TypeScript theme={null}
  // Get existing API key, or create one if none exists
  const userApiCreds = await client.createOrDeriveApiKey();

  console.log("API Key:", userApiCreds.apiKey);
  console.log("Secret:", userApiCreds.secret);
  console.log("Passphrase:", userApiCreds.passphrase);
  ```

  ```python Python theme={null}
  # Get existing API key, or create one if none exists
  user_api_creds = client.create_or_derive_api_creds()

  print("API Key:", user_api_creds["apiKey"])
  print("Secret:", user_api_creds["secret"])
  print("Passphrase:", user_api_creds["passphrase"])
  ```
</CodeGroup>

***

## Step 3: Configure Signature Type and Funder

Before reinitializing the client, determine your **signature type** and **funder address**:

| How do you want to trade?                                                                 | Type         | Value | Funder Address            |
| ----------------------------------------------------------------------------------------- | ------------ | ----- | ------------------------- |
| I want to use an EOA wallet. It holds USDCe and position tokens, and I'll pay my own gas. | EOA          | `0`   | Your EOA wallet address   |
| I want to trade through my Polymarket.com account (Magic Link email/Google login).        | POLY\_PROXY  | `1`   | Your proxy wallet address |
| I want to trade through my Polymarket.com account (browser wallet connection).            | GNOSIS\_SAFE | `2`   | Your proxy wallet address |

<Note>
  If you have a Polymarket.com account, your funds are in a proxy wallet (visible in the profile dropdown). Use type 1 or 2. Type 0 is for standalone EOA wallets only.
</Note>

***

## Step 4: Reinitialize with Full Authentication

<CodeGroup>
  ```typescript TypeScript theme={null}
  // Choose based on your wallet type (see table above)
  const SIGNATURE_TYPE = 0; // EOA example
  const FUNDER_ADDRESS = signer.address; // For EOA, funder is your wallet

  const client = new ClobClient(
    HOST,
    CHAIN_ID,
    signer,
    userApiCreds,
    SIGNATURE_TYPE,
    FUNDER_ADDRESS
  );
  ```

  ```python Python theme={null}
  # Choose based on your wallet type (see table above)
  signature_type = 0  # EOA example
  funder_address = "YOUR_WALLET_ADDRESS"  # For EOA, funder is your wallet

  client = ClobClient(
      host,
      key=private_key,
      chain_id=chain_id,
      creds=user_api_creds,
      signature_type=signature_type,
      funder=funder_address
  )
  ```
</CodeGroup>

<Warning>
  **Do not use Builder API credentials in place of User API credentials!** Builder credentials are for order attribution, not user authentication. See [Builder Order Attribution](/developers/builders/order-attribution).
</Warning>

***

## Step 5: Place an Order

Now you're ready to trade! First, get a token ID from the [Gamma API](/developers/gamma-markets-api/get-markets).

<CodeGroup>
  ```typescript TypeScript theme={null}
  import { Side, OrderType } from "@polymarket/clob-client";

  // Get market info first
  const market = await client.getMarket("TOKEN_ID");

  const response = await client.createAndPostOrder(
    {
      tokenID: "TOKEN_ID",
      price: 0.50,        // Price per share ($0.50)
      size: 10,           // Number of shares
      side: Side.BUY,     // BUY or SELL
    },
    {
      tickSize: market.tickSize,
      negRisk: market.negRisk,    // true for multi-outcome events
    },
    OrderType.GTC  // Good-Til-Cancelled
  );

  console.log("Order ID:", response.orderID);
  console.log("Status:", response.status);
  ```

  ```python Python theme={null}
  from py_clob_client.clob_types import OrderArgs, OrderType
  from py_clob_client.order_builder.constants import BUY

  # Get market info first
  market = client.get_market("TOKEN_ID")

  response = client.create_and_post_order(
      OrderArgs(
          token_id="TOKEN_ID",
          price=0.50,       # Price per share ($0.50)
          size=10,          # Number of shares
          side=BUY,         # BUY or SELL
      ),
      options={
          "tick_size": market["tickSize"],
          "neg_risk": market["negRisk"],    # True for multi-outcome events
      },
      order_type=OrderType.GTC  # Good-Til-Cancelled
  )

  print("Order ID:", response["orderID"])
  print("Status:", response["status"])
  ```
</CodeGroup>

***

## Step 6: Check Your Orders

<CodeGroup>
  ```typescript TypeScript theme={null}
  // View all open orders
  const openOrders = await client.getOpenOrders();
  console.log(`You have ${openOrders.length} open orders`);

  // View your trade history
  const trades = await client.getTrades();
  console.log(`You've made ${trades.length} trades`);

  // Cancel an order
  await client.cancelOrder(response.orderID);
  ```

  ```python Python theme={null}
  # View all open orders
  open_orders = trading_client.get_open_orders()
  print(f"You have {len(open_orders)} open orders")

  # View your trade history
  trades = trading_client.get_trades()
  print(f"You've made {len(trades)} trades")

  # Cancel an order
  trading_client.cancel_order(response["orderID"])
  ```
</CodeGroup>

***

## Troubleshooting

<AccordionGroup>
  <Accordion title="Invalid Signature / L2 Auth Not Available">
    Wrong private key, signature type, or funder address for the derived User API credentials.

```text
Double check the following values when creating User API credentials via `createOrDeriveApiKey()`:
```

* Do not use Builder API credentials in place of User API credentials
* Check `signatureType` matches your account type (0, 1, or 2)
* Ensure `funder` is correct for your wallet type
  </Accordion>

  <Accordion title="Unauthorized / Invalid API Key">
    Wrong API key, secret, or passphrase.

    Re-derive credentials with `createOrDeriveApiKey()` and update your config.
  </Accordion>

  <Accordion title="Not Enough Balance / Allowance">
    Either not enough USDCe / position tokens in your funder address, or you lack approvals to spend your tokens.

* Deposit USDCe to your funder address.
* Ensure you have more USDCe than what's committed in open orders.
* Check that you've set all necessary token approvals.
  </Accordion>

  <Accordion title="Blocked by Cloudflare / Geoblock">
    You're trying to place a trade from a restricted region.

    See [Geographic Restrictions](/developers/CLOB/geoblock) for details.
  </Accordion>
</AccordionGroup>

***

## Adding Builder API Credentials

If you're building an app that routes orders for your users, you can add builder credentials to get attribution on the [Builder Leaderboard](https://builders.polymarket.com/):

```typescript TypeScript theme={null}
import { BuilderConfig, BuilderApiKeyCreds } from "@polymarket/builder-signing-sdk";

const builderCreds: BuilderApiKeyCreds = {
  key: process.env.POLY_BUILDER_API_KEY!,
  secret: process.env.POLY_BUILDER_SECRET!,
  passphrase: process.env.POLY_BUILDER_PASSPHRASE!,
};

const builderConfig = new BuilderConfig({ localBuilderCreds: builderCreds });

// Add builderConfig as the last parameter
const client = new ClobClient(
  HOST, 
  CHAIN_ID, 
  signer, 
  userApiCreds, 
  signatureType, 
  funderAddress,
  undefined, 
  false, 
  builderConfig
);
```

<Info>
  Builder credentials are **separate** from user credentials. You use your builder
  credentials to tag orders, but each user still needs their own L2 credentials to trade.
</Info>

<Card title="Full Builder Guide" icon="hammer" href="/developers/builders/order-attribution">
  Complete documentation for order attribution and gasless transactions
</Card>

***
