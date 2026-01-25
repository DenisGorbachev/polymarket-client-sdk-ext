# Maker Rebates Program

We're rolling out **Maker Rebates** for **15-minute crypto markets**; a program designed to make these fast-moving markets deeper, tighter, and easier to trade.

Market makers who provide **active liquidity** (orders that get filled) earn **daily USDC rebates**, proportional to the liquidity they provide.

## Why Maker Rebates?

15-minute markets move quickly. When liquidity is deeper:

* Spreads tend to be tighter
* Price impact is lower
* Fills are more reliable
* Markets are more resilient during volatility

Maker Rebates incentivize **consistent, competitive quoting** so everyone gets a better trading experience.

## How Maker Rebates Work

* **Paid daily in USDC:** Rebates are calculated and distributed every day.
* **Performance-based:** You earn based on the share of liquidity you provided that actually got taken.

### Funding

Maker Rebates are funded by **taker fees collected in 15-minute crypto markets**. A percentage of these fees are redistributed to makers who keep the markets liquid.

| Period                                           | Maker Rebate | Distribution Method |
| ------------------------------------------------ | ------------ | ------------------- |
| Jan 9 – Jan 11, 2026 (Until Sunday Midnight UTC) | 100%         | Volume-weighted     |
| Jan 12 – Jan 18, 2026                            | 20%          | Volume-weighted     |
| Jan 19 – Jan 25, 2026                            | 20%          | Fee-curve weighted  |

<Note>
  Polymarket collects taker fees **only** in 15-minute crypto markets. The rebate percentage is at the sole discretion of Polymarket and may change over time.
</Note>

## Fee-Curve Weighted Rebates

Rebates are distributed using the **same formula as taker fees**. This ensures makers are rewarded proportionally to the fee value their liquidity generates.

For each filled maker order:

```text  theme={null}
fee_equivalent = shares * price * 0.25 * (price * (1 - price))^2
```

Your daily rebate:

```text  theme={null}
rebate = (your_fee_equivalent / total_fee_equivalent) * rebate_pool
```

## Taker Fee Structure

Taker fees are calculated in USDC and vary based on the share price. Fees are **highest at 50%** probability and **lowest at the extremes** (near 0% or 100%).

<img src="https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=5a4bdaf810ad1dafafd7c6f2be20719e" alt="Fee Curve" data-og-width="846" width="846" data-og-height="547" height="547" data-path="polymarket-learn/media/fee_image.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=280&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=962381a3c05f01e25468bf9031ac038d 280w, https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=560&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=d04194406ff27f13c1caf1389777c6bf 560w, https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=840&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=be276a6b8aaddef53f5cb08165e76c96 840w, https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=1100&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=5c412d08f1f66bfd68ebd11c18a21198 1100w, https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=1650&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=535668bea02d7916273076b94c22c964 1650w, https://mintcdn.com/polymarket-292d1b1b/YUHnSq4JdekVofRY/polymarket-learn/media/fee_image.png?w=2500&fit=max&auto=format&n=YUHnSq4JdekVofRY&q=85&s=18afd7369a806ee0a487be26a1c2ba2a 2500w" />

### Fee Table (100 shares)

| Price  | Trade Value | Fee (USDC) | Effective Rate |
| ------ | ----------- | ---------- | -------------- |
| \$0.01 | \$1         | \$0.00     | 0.00%          |
| \$0.05 | \$5         | \$0.003    | 0.06%          |
| \$0.10 | \$10        | \$0.02     | 0.20%          |
| \$0.15 | \$15        | \$0.06     | 0.41%          |
| \$0.20 | \$20        | \$0.13     | 0.64%          |
| \$0.25 | \$25        | \$0.22     | 0.88%          |
| \$0.30 | \$30        | \$0.33     | 1.10%          |
| \$0.35 | \$35        | \$0.45     | 1.29%          |
| \$0.40 | \$40        | \$0.58     | 1.44%          |
| \$0.45 | \$45        | \$0.69     | 1.53%          |
| \$0.50 | \$50        | \$0.78     | **1.56%**      |
| \$0.55 | \$55        | \$0.84     | 1.53%          |
| \$0.60 | \$60        | \$0.86     | 1.44%          |
| \$0.65 | \$65        | \$0.84     | 1.29%          |
| \$0.70 | \$70        | \$0.77     | 1.10%          |
| \$0.75 | \$75        | \$0.66     | 0.88%          |
| \$0.80 | \$80        | \$0.51     | 0.64%          |
| \$0.85 | \$85        | \$0.35     | 0.41%          |
| \$0.90 | \$90        | \$0.18     | 0.20%          |
| \$0.95 | \$95        | \$0.05     | 0.06%          |
| \$0.99 | \$99        | \$0.00     | 0.00%          |

The maximum effective fee rate is **1.56%** at 50% probability. Fees decrease symmetrically toward both extremes.

### Fee Precision

Fees are rounded to 4 decimal places. The smallest fee charged is **0.0001 USDC**. Anything smaller rounds to zero, so very small trades near the extremes may incur no fee at all.

## FAQ

<AccordionGroup>
  <Accordion title="How do I qualify for maker rebates?">
    Place orders that add liquidity to the book and get filled (i.e., your liquidity is taken by another trader).
  </Accordion>

  <Accordion title="When are rebates paid?">
    Daily, in USDC.
  </Accordion>

  <Accordion title="How are rebates calculated?">
    Rebates are proportional to your share of executed maker liquidity in each eligible market. During fee-curve weighted periods, this is based on fee-equivalent using the formula above.
  </Accordion>

  <Accordion title="Where does the rebate pool come from?">
    Taker fees collected in eligible markets are allocated to the maker rebate pool and distributed daily.
  </Accordion>

  <Accordion title="Which markets have fees enabled?">
    Currently, only 15-minute crypto markets have taker fees enabled.
  </Accordion>

  <Accordion title="Is Polymarket charging fees on all markets?">
    No. Polymarket is collecting taker fees **only** on 15-minute crypto markets. All other markets remain fee-free.
  </Accordion>
</AccordionGroup>

## For API Users

If you trade programmatically, you'll need to update your client to handle fees correctly.

<Card title="Developer Guide: Maker Rebates" icon="code" href="/developers/market-makers/maker-rebates-program">
  Technical documentation for handling fees in your trading code
</Card>
