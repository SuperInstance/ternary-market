# ternary-market: Economic exchange and resource allocation in ternary systems

A market engine for balanced ternary {-1, 0, +1} systems with order books, price discovery, auctions, market making, and portfolio management. Buy/sell/hold orders are matched by ternary price signals, and resource allocation is tracked across multiple assets.

## Why This Exists

Ternary agents need to exchange resources. A scout that discovers territory should be able to sell that information. A builder with excess energy should trade it for materials. Without a market, you rely on centralized allocation — slow, rigid, and single-point-of-failure. This crate provides decentralized price discovery and matching, using ternary signals (buy/hold/sell) as the basic economic language.

## Core Concepts

- **Balanced ternary**: Three values: -1 (Neg), 0 (Zero), +1 (Pos). Used here as price signals — Pos means bullish, Neg means bearish, Zero means neutral/hold.
- **Order**: A buy, sell, or hold instruction for a resource with a quantity and ternary price signal.
- **OrderBook**: A matching engine per resource. Buy orders match sell orders when their price signals agree (or either is Zero, meaning "any price").
- **PriceDiscovery**: Aggregates all submitted signals to determine the current market sentiment. Sum of signals → market direction.
- **MarketMaker**: Maintains standing buy/sell orders with a spread. Provides liquidity so traders can always execute.
- **Auction**: Three types — English (ascending), Dutch (descending), Vickrey (sealed second-price). For one-off resource allocation.
- **Portfolio**: Tracks holdings across multiple resources with their ternary signals. Net signal shows overall position bias.

## Quick Start

```toml
[dependencies]
ternary-market = "0.1"
```

```rust
use ternary_market::{Market, OrderSide, Ternary, Auction, AuctionType};

let mut market = Market::new();

// Alice sells energy, Bob buys
market.submit_order("energy", OrderSide::Sell, 10, Ternary::Pos, "alice");
let trade_count = market.total_trades(); // 0, no buyer yet

market.submit_order("energy", OrderSide::Buy, 10, Ternary::Pos, "bob");
assert_eq!(market.total_trades(), 1); // matched!

// Run an auction for a rare resource
let mut auction = Auction::new(AuctionType::Vickrey, "artifact", 5);
auction.bid("alice", 30, Ternary::Pos);
auction.bid("bob", 25, Ternary::Pos);
auction.close();
let winner = auction.winner().unwrap();
let price = auction.vickrey_price().unwrap(); // 25 (second price)
```

## API Overview

| Type | Description |
|------|-------------|
| `Ternary` | A ternary value: Neg (-1), Zero (0), Pos (+1) |
| `Order` | A buy/sell/hold instruction with quantity and price signal |
| `OrderSide` | Buy, Sell, or Hold |
| `Trade` | A matched trade between buy and sell orders |
| `OrderBook` | Per-resource matching engine |
| `PriceDiscovery` | Aggregates signals to determine market direction |
| `MarketMaker` | Provides liquidity with a standing spread |
| `Auction` | English, Dutch, or Vickrey auction for one-off allocation |
| `Bid` | A single bid in an auction |
| `Portfolio` | Tracks holdings and signals across resources |
| `Market` | Top-level structure combining books, price discovery, and portfolio |

## How It Works

The `OrderBook` uses a simple matching algorithm: incoming orders are compared against the opposite side. A buy matches a sell if their price signals are equal, or if either signal is Zero (acting as a market order). Partial fills are supported — the remaining quantity stays on the book.

`PriceDiscovery` is a running sum of all submitted signals. No weighting, no decay — every signal counts equally. This keeps it simple and predictable.

`MarketMaker` quotes a buy and sell price separated by a spread (the ternary signal distance). The spread determines profit margin: Pos spread means buy at Zero, sell at Pos (margin = 1).

`Auction` supports three formats. English and Dutch both award to the highest bidder above reserve. Vickrey awards to the highest but charges the second-highest price, encouraging truthful bidding.

`Portfolio` is a simple inventory with ternary signals. Net signal is the sum of all position signals — positive means overall bullish, negative means bearish.

## Known Limitations

- **No time-based priority**: Orders don't have timestamps for priority ordering. First-in-first-out is not guaranteed beyond insertion order.
- **No order cancellation**: Once submitted, orders can't be cancelled. They either match or sit on the book forever.
- **Simplistic matching**: Only exact signal matches or Zero-wildcards. No partial signal matching, no multi-level order books.
- **No fees or commissions**: Trades execute at face value. Real markets need transaction costs.
- **Auction is synchronous**: Bidding and closing happen in the same thread. Real auctions need async timeout-based closing.
- **No short selling**: Portfolios can't go negative. You can only sell what you have.

## Use Cases

- **Agent resource trading**: Agents in a ternary fleet exchange energy, information, and materials using ternary price signals.
- **Scheduling market**: Time slots are resources; agents bid for preferred slots. Combines with `ternary-scheduling`.
- **Information market**: Scouts sell territory maps. Builders buy them. Price signals indicate confidence.
- **Game economy**: NPCs trade items in a ternary-valued economy with auctions and market makers.

## Ecosystem Context

Part of the SuperInstance ternary crate family. Combines ideas from:
- `ternary-econ` (economic primitives this crate builds on)
- `ternary-scheduling` (time-slot allocation as a market)
- `ternary-frontier` (territory as tradeable resource)
- `ternary-archive` (historical trade data stored as scrolls)

## License

MIT
