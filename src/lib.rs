#![forbid(unsafe_code)]

//! Economic exchange and resource allocation in balanced ternary {-1, 0, +1} systems.
//!
//! Provides market structures, order books, price discovery, auctions, and portfolio
//! management, all operating on ternary-valued signals and quantities.

use std::collections::HashMap;

/// A ternary value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Order direction: buy (want), sell (offer), or hold (no action).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderSide {
    Buy,
    Sell,
    Hold,
}

/// A single order in the market.
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub resource: String,
    pub quantity: u32,
    pub price_signal: Ternary,
    pub trader: String,
}

impl Order {
    pub fn new(id: u64, side: OrderSide, resource: &str, quantity: u32, price_signal: Ternary, trader: &str) -> Self {
        Self {
            id,
            side,
            resource: resource.to_string(),
            quantity,
            price_signal,
            trader: trader.to_string(),
        }
    }

    pub fn is_buy(&self) -> bool {
        self.side == OrderSide::Buy
    }

    pub fn is_sell(&self) -> bool {
        self.side == OrderSide::Sell
    }
}

/// A matched trade between a buy and sell order.
#[derive(Debug, Clone)]
pub struct Trade {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub resource: String,
    pub quantity: u32,
    pub price_signal: Ternary,
}

/// Order book matching engine.
#[derive(Debug, Clone)]
pub struct OrderBook {
    resource: String,
    buys: Vec<Order>,
    sells: Vec<Order>,
    next_order_id: u64,
    trades: Vec<Trade>,
}

impl OrderBook {
    pub fn new(resource: &str) -> Self {
        Self {
            resource: resource.to_string(),
            buys: Vec::new(),
            sells: Vec::new(),
            next_order_id: 1,
            trades: Vec::new(),
        }
    }

    /// Submit an order. Returns the order ID. Matches immediately if possible.
    pub fn submit(&mut self, side: OrderSide, quantity: u32, price_signal: Ternary, trader: &str) -> u64 {
        let id = self.next_order_id;
        self.next_order_id += 1;
        let order = Order::new(id, side, &self.resource, quantity, price_signal, trader);

        if side == OrderSide::Buy {
            let mut remaining = quantity;
            // We need to track sell quantities that need updating
            let mut sell_updates: Vec<(usize, u32)> = Vec::new(); // (index, new_qty)
            let mut to_remove = Vec::new();
            for (i, sell) in self.sells.iter().enumerate() {
                if remaining == 0 { break; }
                if sell.price_signal == price_signal || sell.price_signal == Ternary::Zero || price_signal == Ternary::Zero {
                    let matched = remaining.min(sell.quantity);
                    self.trades.push(Trade {
                        buy_order_id: id,
                        sell_order_id: sell.id,
                        resource: self.resource.clone(),
                        quantity: matched,
                        price_signal,
                    });
                    remaining -= matched;
                    let new_qty = sell.quantity - matched;
                    if new_qty == 0 {
                        to_remove.push(i);
                    } else {
                        sell_updates.push((i, new_qty));
                    }
                }
            }
            // Apply removals in reverse order
            for &i in to_remove.iter().rev() {
                self.sells.remove(i);
            }
            // Apply quantity updates
            for (i, new_qty) in sell_updates {
                self.sells[i].quantity = new_qty;
            }
            if remaining > 0 {
                let mut leftover = order.clone();
                leftover.quantity = remaining;
                self.buys.push(leftover);
            }
        } else if side == OrderSide::Sell {
            let mut remaining = quantity;
            let mut buy_updates: Vec<(usize, u32)> = Vec::new();
            let mut to_remove = Vec::new();
            for (i, buy) in self.buys.iter().enumerate() {
                if remaining == 0 { break; }
                if buy.price_signal == price_signal || buy.price_signal == Ternary::Zero || price_signal == Ternary::Zero {
                    let matched = remaining.min(buy.quantity);
                    self.trades.push(Trade {
                        buy_order_id: buy.id,
                        sell_order_id: id,
                        resource: self.resource.clone(),
                        quantity: matched,
                        price_signal,
                    });
                    remaining -= matched;
                    let new_qty = buy.quantity - matched;
                    if new_qty == 0 {
                        to_remove.push(i);
                    } else {
                        buy_updates.push((i, new_qty));
                    }
                }
            }
            for &i in to_remove.iter().rev() {
                self.buys.remove(i);
            }
            for (i, new_qty) in buy_updates {
                self.buys[i].quantity = new_qty;
            }
            if remaining > 0 {
                let mut leftover = order.clone();
                leftover.quantity = remaining;
                self.sells.push(leftover);
            }
        }

        id
    }

    pub fn open_buys(&self) -> &[Order] {
        &self.buys
    }

    pub fn open_sells(&self) -> &[Order] {
        &self.sells
    }

    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }
}

/// Ternary price signal discovery.
#[derive(Debug, Clone)]
pub struct PriceDiscovery {
    signals: Vec<Ternary>,
}

impl PriceDiscovery {
    pub fn new() -> Self {
        Self { signals: Vec::new() }
    }

    pub fn push(&mut self, signal: Ternary) {
        self.signals.push(signal);
    }

    /// Current price signal: the sum tendency of all signals.
    /// Returns Pos if bullish, Neg if bearish, Zero if neutral.
    pub fn current_signal(&self) -> Ternary {
        let sum: i64 = self.signals.iter().map(|s| s.to_i8() as i64).sum();
        match sum.cmp(&0) {
            std::cmp::Ordering::Greater => Ternary::Pos,
            std::cmp::Ordering::Less => Ternary::Neg,
            std::cmp::Ordering::Equal => Ternary::Zero,
        }
    }

    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Clear all signals.
    pub fn reset(&mut self) {
        self.signals.clear();
    }
}

impl Default for PriceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Provides liquidity by maintaining standing buy/sell orders.
#[derive(Debug, Clone)]
pub struct MarketMaker {
    pub name: String,
    pub spread: Ternary, // difference between buy and sell price
    buy_signal: Ternary,
    sell_signal: Ternary,
}

impl MarketMaker {
    pub fn new(name: &str, spread: Ternary) -> Self {
        let (buy, sell) = match spread {
            Ternary::Pos => (Ternary::Zero, Ternary::Pos),
            Ternary::Zero => (Ternary::Zero, Ternary::Zero),
            Ternary::Neg => (Ternary::Neg, Ternary::Zero),
        };
        Self {
            name: name.to_string(),
            spread,
            buy_signal: buy,
            sell_signal: sell,
        }
    }

    pub fn quote_buy(&self) -> Ternary {
        self.buy_signal
    }

    pub fn quote_sell(&self) -> Ternary {
        self.sell_signal
    }

    pub fn profit_margin(&self) -> i8 {
        self.sell_signal.to_i8() - self.buy_signal.to_i8()
    }
}

/// Auction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionType {
    English,  // ascending price
    Dutch,    // descending price
    Vickrey,  // sealed second-price
}

/// A bid in an auction.
#[derive(Debug, Clone)]
pub struct Bid {
    pub bidder: String,
    pub amount: u32,
    pub signal: Ternary,
}

/// An auction for resource allocation.
#[derive(Debug, Clone)]
pub struct Auction {
    pub auction_type: AuctionType,
    pub resource: String,
    bids: Vec<Bid>,
    reserve_price: u32,
    closed: bool,
}

impl Auction {
    pub fn new(auction_type: AuctionType, resource: &str, reserve_price: u32) -> Self {
        Self {
            auction_type,
            resource: resource.to_string(),
            bids: Vec::new(),
            reserve_price,
            closed: false,
        }
    }

    pub fn bid(&mut self, bidder: &str, amount: u32, signal: Ternary) -> bool {
        if self.closed { return false; }
        self.bids.push(Bid {
            bidder: bidder.to_string(),
            amount,
            signal,
        });
        true
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Determine the winner based on auction type.
    pub fn winner(&self) -> Option<&Bid> {
        if !self.closed || self.bids.is_empty() {
            return None;
        }

        match self.auction_type {
            AuctionType::English | AuctionType::Dutch => {
                // Highest bid above reserve wins
                self.bids.iter()
                    .filter(|b| b.amount >= self.reserve_price)
                    .max_by_key(|b| b.amount)
            }
            AuctionType::Vickrey => {
                // Highest bid wins, pays second-highest price
                // We return the winner (highest), caller reads second price
                self.bids.iter()
                    .filter(|b| b.amount >= self.reserve_price)
                    .max_by_key(|b| b.amount)
            }
        }
    }

    /// For Vickrey: the price the winner pays (second-highest bid).
    pub fn vickrey_price(&self) -> Option<u32> {
        if self.auction_type != AuctionType::Vickrey || self.bids.len() < 2 {
            return None;
        }
        let mut amounts: Vec<u32> = self.bids.iter()
            .filter(|b| b.amount >= self.reserve_price)
            .map(|b| b.amount)
            .collect();
        amounts.sort_by(|a, b| b.cmp(a)); // descending
        amounts.into_iter().nth(1) // second highest
    }

    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

/// Resource allocation across multiple resources.
#[derive(Debug, Clone)]
pub struct Portfolio {
    holdings: HashMap<String, u32>,
    signals: HashMap<String, Ternary>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            holdings: HashMap::new(),
            signals: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, resource: &str, amount: u32, signal: Ternary) {
        *self.holdings.entry(resource.to_string()).or_insert(0) += amount;
        self.signals.insert(resource.to_string(), signal);
    }

    pub fn deallocate(&mut self, resource: &str, amount: u32) -> bool {
        if let Some(held) = self.holdings.get_mut(resource) {
            if *held >= amount {
                *held -= amount;
                if *held == 0 {
                    self.holdings.remove(resource);
                    self.signals.remove(resource);
                }
                return true;
            }
        }
        false
    }

    pub fn holding(&self, resource: &str) -> u32 {
        *self.holdings.get(resource).unwrap_or(&0)
    }

    pub fn signal(&self, resource: &str) -> Option<Ternary> {
        self.signals.get(resource).copied()
    }

    /// Net ternary signal across all holdings (sum of signals).
    pub fn net_signal(&self) -> Ternary {
        let sum: i64 = self.signals.values().map(|s| s.to_i8() as i64).sum();
        match sum.cmp(&0) {
            std::cmp::Ordering::Greater => Ternary::Pos,
            std::cmp::Ordering::Less => Ternary::Neg,
            std::cmp::Ordering::Equal => Ternary::Zero,
        }
    }

    pub fn resource_count(&self) -> usize {
        self.holdings.len()
    }

    pub fn total_holdings(&self) -> u32 {
        self.holdings.values().sum()
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

/// The main market structure.
#[derive(Debug, Clone)]
pub struct Market {
    books: HashMap<String, OrderBook>,
    price_discovery: PriceDiscovery,
    portfolio: Portfolio,
}

impl Market {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
            price_discovery: PriceDiscovery::new(),
            portfolio: Portfolio::new(),
        }
    }

    /// Get or create an order book for a resource.
    pub fn book(&mut self, resource: &str) -> &mut OrderBook {
        self.books.entry(resource.to_string())
            .or_insert_with(|| OrderBook::new(resource))
    }

    /// Submit an order to a resource's book.
    pub fn submit_order(&mut self, resource: &str, side: OrderSide, quantity: u32, price_signal: Ternary, trader: &str) -> u64 {
        let id = self.book(resource).submit(side, quantity, price_signal, trader);
        self.price_discovery.push(price_signal);
        id
    }

    pub fn current_signal(&self) -> Ternary {
        self.price_discovery.current_signal()
    }

    pub fn portfolio(&self) -> &Portfolio {
        &self.portfolio
    }

    pub fn portfolio_mut(&mut self) -> &mut Portfolio {
        &mut self.portfolio
    }

    pub fn resources(&self) -> Vec<&str> {
        self.books.keys().map(|s| s.as_str()).collect()
    }

    pub fn total_trades(&self) -> usize {
        self.books.values().map(|b| b.trade_count()).sum()
    }
}

impl Default for Market {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_roundtrip() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::Neg));
        assert_eq!(Ternary::from_i8(0), Some(Ternary::Zero));
        assert_eq!(Ternary::from_i8(1), Some(Ternary::Pos));
    }

    #[test]
    fn test_order_creation() {
        let o = Order::new(1, OrderSide::Buy, "energy", 10, Ternary::Pos, "alice");
        assert!(o.is_buy());
        assert!(!o.is_sell());
        assert_eq!(o.quantity, 10);
    }

    #[test]
    fn test_order_book_match() {
        let mut book = OrderBook::new("energy");
        let sell_id = book.submit(OrderSide::Sell, 5, Ternary::Pos, "bob");
        let buy_id = book.submit(OrderSide::Buy, 5, Ternary::Pos, "alice");
        assert_eq!(book.trade_count(), 1);
        assert_eq!(book.trades()[0].quantity, 5);
        assert_eq!(book.open_buys().len(), 0);
        assert_eq!(book.open_sells().len(), 0);
    }

    #[test]
    fn test_order_book_partial_fill() {
        let mut book = OrderBook::new("energy");
        book.submit(OrderSide::Sell, 10, Ternary::Pos, "bob");
        book.submit(OrderSide::Buy, 3, Ternary::Pos, "alice");
        assert_eq!(book.trade_count(), 1);
        assert_eq!(book.trades()[0].quantity, 3);
        assert_eq!(book.open_sells().len(), 1);
        assert_eq!(book.open_sells()[0].quantity, 7);
    }

    #[test]
    fn test_order_book_no_match_wrong_price() {
        let mut book = OrderBook::new("energy");
        book.submit(OrderSide::Sell, 5, Ternary::Pos, "bob");
        book.submit(OrderSide::Buy, 5, Ternary::Neg, "alice");
        assert_eq!(book.trade_count(), 0);
        assert_eq!(book.open_buys().len(), 1);
        assert_eq!(book.open_sells().len(), 1);
    }

    #[test]
    fn test_order_book_resource() {
        let book = OrderBook::new("gold");
        assert_eq!(book.resource(), "gold");
    }

    #[test]
    fn test_price_discovery_bullish() {
        let mut pd = PriceDiscovery::new();
        pd.push(Ternary::Pos);
        pd.push(Ternary::Pos);
        pd.push(Ternary::Zero);
        assert_eq!(pd.current_signal(), Ternary::Pos);
    }

    #[test]
    fn test_price_discovery_bearish() {
        let mut pd = PriceDiscovery::new();
        pd.push(Ternary::Neg);
        pd.push(Ternary::Neg);
        assert_eq!(pd.current_signal(), Ternary::Neg);
    }

    #[test]
    fn test_price_discovery_neutral() {
        let mut pd = PriceDiscovery::new();
        pd.push(Ternary::Pos);
        pd.push(Ternary::Neg);
        assert_eq!(pd.current_signal(), Ternary::Zero);
    }

    #[test]
    fn test_price_discovery_reset() {
        let mut pd = PriceDiscovery::new();
        pd.push(Ternary::Pos);
        pd.reset();
        assert_eq!(pd.signal_count(), 0);
        assert_eq!(pd.current_signal(), Ternary::Zero);
    }

    #[test]
    fn test_market_maker_spread() {
        let mm = MarketMaker::new("mm1", Ternary::Pos);
        assert_eq!(mm.quote_buy(), Ternary::Zero);
        assert_eq!(mm.quote_sell(), Ternary::Pos);
        assert_eq!(mm.profit_margin(), 1);
    }

    #[test]
    fn test_market_maker_zero_spread() {
        let mm = MarketMaker::new("mm1", Ternary::Zero);
        assert_eq!(mm.profit_margin(), 0);
    }

    #[test]
    fn test_auction_english() {
        let mut a = Auction::new(AuctionType::English, "artifact", 10);
        a.bid("alice", 15, Ternary::Pos);
        a.bid("bob", 20, Ternary::Pos);
        a.bid("carol", 18, Ternary::Zero);
        a.close();
        let winner = a.winner().unwrap();
        assert_eq!(winner.bidder, "bob");
        assert_eq!(winner.amount, 20);
    }

    #[test]
    fn test_auction_reserve_not_met() {
        let mut a = Auction::new(AuctionType::English, "artifact", 100);
        a.bid("alice", 50, Ternary::Pos);
        a.close();
        assert!(a.winner().is_none());
    }

    #[test]
    fn test_auction_vickrey() {
        let mut a = Auction::new(AuctionType::Vickrey, "slot", 5);
        a.bid("alice", 30, Ternary::Pos);
        a.bid("bob", 25, Ternary::Zero);
        a.bid("carol", 20, Ternary::Neg);
        a.close();
        assert_eq!(a.winner().unwrap().bidder, "alice");
        assert_eq!(a.vickrey_price(), Some(25)); // second price
    }

    #[test]
    fn test_auction_cannot_bid_after_close() {
        let mut a = Auction::new(AuctionType::Dutch, "x", 0);
        a.close();
        assert!(!a.bid("alice", 10, Ternary::Pos));
    }

    #[test]
    fn test_portfolio_allocate() {
        let mut p = Portfolio::new();
        p.allocate("energy", 100, Ternary::Pos);
        assert_eq!(p.holding("energy"), 100);
        assert_eq!(p.signal("energy"), Some(Ternary::Pos));
    }

    #[test]
    fn test_portfolio_deallocate() {
        let mut p = Portfolio::new();
        p.allocate("energy", 100, Ternary::Pos);
        assert!(p.deallocate("energy", 30));
        assert_eq!(p.holding("energy"), 70);
    }

    #[test]
    fn test_portfolio_deallocate_full() {
        let mut p = Portfolio::new();
        p.allocate("energy", 100, Ternary::Pos);
        assert!(p.deallocate("energy", 100));
        assert_eq!(p.holding("energy"), 0);
        assert_eq!(p.signal("energy"), None);
    }

    #[test]
    fn test_portfolio_deallocate_too_much() {
        let mut p = Portfolio::new();
        p.allocate("energy", 50, Ternary::Pos);
        assert!(!p.deallocate("energy", 100));
    }

    #[test]
    fn test_portfolio_net_signal() {
        let mut p = Portfolio::new();
        p.allocate("a", 10, Ternary::Pos);
        p.allocate("b", 10, Ternary::Neg);
        assert_eq!(p.net_signal(), Ternary::Zero);
        p.allocate("c", 10, Ternary::Pos);
        assert_eq!(p.net_signal(), Ternary::Pos);
    }

    #[test]
    fn test_market_submit_and_trade() {
        let mut m = Market::new();
        m.submit_order("energy", OrderSide::Sell, 10, Ternary::Pos, "bob");
        m.submit_order("energy", OrderSide::Buy, 10, Ternary::Pos, "alice");
        assert_eq!(m.total_trades(), 1);
        assert_eq!(m.current_signal(), Ternary::Pos);
    }

    #[test]
    fn test_market_multiple_resources() {
        let mut m = Market::new();
        m.submit_order("energy", OrderSide::Buy, 5, Ternary::Pos, "a");
        m.submit_order("gold", OrderSide::Buy, 3, Ternary::Neg, "b");
        assert_eq!(m.resources().len(), 2);
    }
}
