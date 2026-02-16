* Refactor GammaEvent::is_date_cascade to be a standalone is_date_cascade(markets: impl IntoIterator<Item = &GammaMarket>)
* Refactor GammaEvent::markets field to be an `Vec<GammaMarket>` (use unwrap_or_default in the TryFrom)
* Add GammaEvent::is_date_cascade field
  * Set it in `impl TryFrom<GammaEventRaw> for GammaEvent`
* Add a check to get_time_spread_arbitrage_opportunity
  * if !self.is_date_cascade return None
* Remove: "This function assumes that `self` passes [`Self::is_date_cascade`]."
* Add struct TimeSpreadArbitrageOpportunity { event: &GammaEvent, prev: &GammaMarket, next: &GammaMarket }
* Rename pub fn get_time_spread_arbitrage_opportunity to pub fn get_time_spread_arbitrage_opportunities
* Make get_time_spread_arbitrage_opportunities return multiple TimeSpreadArbitrageOpportunity
* Refactor CacheGammaEventsMonitorDateCascadesCommand
  * Make refresh_date_cascades return the events
    * Use refs for serialization
  * On each iteration of the loop:
    * let time_spread_arbitrage_opportunities = events.iter().filter(|e| e.get_time_spread_arbitrage_opportunity().is_some())
    * time_spread_arbitrage_opportunities.map(|e| println!("{}", e.slug))
