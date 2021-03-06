///! Data handler trait for market quotes

use chrono::{DateTime, Utc};

use super::AssetHandler;
use super::DataError;
use crate::currency::Currency;
use crate::quote::{Quote, Ticker};

/// Handler for globally available market quotes data
pub trait QuoteHandler: AssetHandler {
    // insert, get, update and delete for market data sources
    fn insert_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError>;
    fn get_ticker_id(&mut self, ticker: &str) -> Option<usize>;
    fn insert_if_new_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError> {
        match self.get_ticker_id(&ticker.name) {
            Some(id) => Ok(id),
            None => self.insert_ticker(ticker),
        }
    }
    fn get_ticker_by_id(&mut self, id: usize) -> Result<Ticker, DataError>;
    fn get_all_ticker(&mut self) -> Result<Vec<Ticker>, DataError>;
    fn get_all_ticker_for_source(
        &mut self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError>;

    /// Get all ticker that belong to a given asset specified by its asset ID
    fn get_all_ticker_for_asset(
        &mut self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError>;

    fn update_ticker(&mut self, ticker: &Ticker) -> Result<(), DataError>;
    fn delete_ticker(&mut self, id: usize) -> Result<(), DataError>;

    /// Insert, get, update and delete for market data sources
    fn insert_quote(&mut self, quote: &Quote) -> Result<usize, DataError>;

    /// Get the last quote in database for a specific asset name on or before the given time
    fn get_last_quote_before(
        &mut self,
        asset_name: &str,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    /// Get the last quote in database for a specific asset id on or before the given time
    fn get_last_quote_before_by_id(
        &mut self,
        asset_id: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    fn get_all_quotes_for_ticker(&mut self, ticker_id: usize) -> Result<Vec<Quote>, DataError>;
    fn update_quote(&mut self, quote: &Quote) -> Result<(), DataError>;
    fn delete_quote(&mut self, id: usize) -> Result<(), DataError>;

    // Get and set cash rounding conventions by currency
    // This method never throws, if currency could not be found in table, return 2 by default instead
    fn get_rounding_digits(&mut self, currency: Currency) -> i32;
    fn set_rounding_digits(&mut self, currency: Currency, digits: i32) -> Result<(), DataError>;
}
