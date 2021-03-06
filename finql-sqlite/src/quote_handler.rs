///! Implementation for quote handler with Sqlite3 database as backend

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc, Local, TimeZone};
use rusqlite::{params, Row, NO_PARAMS};

use finql_data::Currency;
use finql_data::{DataError, QuoteHandler};
use finql_data::{Quote, Ticker};

use super::SqliteDB;


/// Convert string to DateTime<Utc>
pub fn to_time(time: &str) -> Result<DateTime<Utc>, DataError> {
    let time =
        DateTime::parse_from_rfc3339(time).map_err(|e| DataError::NotFound(e.to_string()))?;
    let time: DateTime<Utc> = DateTime::from(time);
    Ok(time)
}

/// Given a date and time construct a UTC DateTime, assuming that
/// the date belongs to local time zone
pub fn make_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Utc>> {
    let time: NaiveDateTime = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second);
    let time = Local.from_local_datetime(&time).single();
    match time {
        Some(time) => Some(DateTime::from(time)),
        None => None,
    }
}


/// Sqlite implementation of quote handler
impl QuoteHandler for SqliteDB<'_> {
    // insert, get, update and delete for market data sources
    fn insert_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO ticker (name, asset_id, source, priority, currency, factor) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    ticker.name,
                    ticker.asset as i64,
                    ticker.source.to_string(),
                    ticker.priority,
                    ticker.currency.to_string(),
                    ticker.factor,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM ticker
        WHERE name=? AND source=?;",
                params![ticker.name, ticker.source.to_string()],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }

    fn get_ticker_id(&mut self, ticker: &str) -> Option<usize> {
        let get_id = |row: &Row| -> rusqlite::Result<i64> { row.get(0) };
        let id = self.conn.query_row(
            "SELECT id FROM ticker WHERE name=?",
            params![ticker],
            get_id,
        );
        match id {
            Ok(id) => Some(id as usize),
            _ => None,
        }
    }

    fn get_ticker_by_id(&mut self, id: usize) -> Result<Ticker, DataError> {
        let (name, asset, source, priority, currency, factor) = self
            .conn
            .query_row(
                "SELECT name, asset_id, source, priority, currency, factor FROM ticker WHERE id=?;",
                params![id as i64],
                |row| {
                    let name: String = row.get(0)?;
                    let asset: i64 = row.get(1)?;
                    let source: String = row.get(2)?;
                    let priority: i32 = row.get(3)?;
                    let currency: String = row.get(4)?;
                    let factor: f64 = row.get(5)?;
                    Ok((name, asset, source, priority, currency, factor))
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Ticker {
            id: Some(id),
            name,
            asset: asset as usize,
            source,
            priority,
            currency,
            factor,
        })
    }

    fn get_all_ticker(&mut self) -> Result<Vec<Ticker>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, asset_id, priority, source, currency, factor FROM ticker;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let ticker_map = stmt
            .query_map(NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let asset: i64 = row.get(2)?;
                let priority: i32 = row.get(3)?;
                let source: String = row.get(4)?;
                let currency: String = row.get(5)?;
                let factor: f64 = row.get(6)?;
                Ok((id, name, asset, priority, source, currency, factor))
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut all_ticker = Vec::new();
        for ticker in ticker_map {
            let (id, name, asset, priority, source, currency, factor) = ticker.unwrap();
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name,
                asset: asset as usize,
                source,
                priority,
                currency,
                factor,
            });
        }
        Ok(all_ticker)
    }

    fn get_all_ticker_for_source(
        &mut self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, asset_id, priority, currency, factor FROM ticker WHERE source=?;",
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let ticker_map = stmt
            .query_map(params![source.to_string()], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let asset: i64 = row.get(2)?;
                let priority: i32 = row.get(3)?;
                let currency: String = row.get(4)?;
                let factor: f64 = row.get(5)?;
                Ok((id, name, asset, priority, currency, factor))
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut all_ticker = Vec::new();
        for ticker in ticker_map {
            let (id, name, asset, priority, currency, factor) = ticker.unwrap();
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name,
                asset: asset as usize,
                source: source.to_string(),
                priority,
                currency,
                factor,
            });
        }
        Ok(all_ticker)
    }
    fn get_all_ticker_for_asset(
        &mut self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, priority, source, currency, factor FROM ticker WHERE asset_id=?;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let ticker_map = stmt
            .query_map(params![asset_id as i32], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let priority: i32 = row.get(2)?;
                let source: String = row.get(3)?;
                let currency: String = row.get(4)?;
                let factor: f64 = row.get(5)?;
                Ok((id, name, priority, source, currency, factor))
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut all_ticker = Vec::new();
        for ticker in ticker_map {
            let (id, name, priority, source, currency, factor) = ticker.unwrap();
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name,
                asset: asset_id,
                source,
                priority,
                currency,
                factor,
            });
        }
        Ok(all_ticker)
    }

    fn update_ticker(&mut self, ticker: &Ticker) -> Result<(), DataError> {
        if ticker.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = ticker.id.unwrap() as i64;
        self.conn
            .execute(
                "UPDATE ticker SET name=?2, asset_id=?3, source=?4, priority=?5, currency=?6, factor=?7
                WHERE id=?1",
                params![
                    id,
                    ticker.name,
                    ticker.asset as i64,
                    ticker.source.to_string(),
                    ticker.priority,
                    ticker.currency.to_string(),
                    ticker.factor,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_ticker(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM ticker WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for market data sources
    fn insert_quote(&mut self, quote: &Quote) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO quotes (ticker_id, price, time, volume) VALUES (?, ?, ?, ?)",
                params![
                    quote.ticker as i64,
                    quote.price,
                    quote.time.to_rfc3339(),
                    quote.volume
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row("SELECT last_insert_rowid();", NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                Ok(id as usize)
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }
    fn get_last_quote_before(
        &mut self,
        asset_name: &str,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError> {
        let time = time.to_rfc3339();
        let row = self
            .conn
            .query_row(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t, assets a 
                WHERE a.name=? AND t.asset_id=a.id AND t.id=q.ticker_id AND q.time<= ?
                ORDER BY q.time, t.priority DESC LIMIT 1",
                params![asset_name, time],
                |row| {
                    let id: i64 = row.get(0)?;
                    let ticker: i64 = row.get(1)?;
                    let price: f64 = row.get(2)?;
                    let time: String = row.get(3)?;
                    let volume: Option<f64> = row.get(4)?;
                    let currency: String = row.get(5)?;
                    Ok((id, ticker, price, time, volume, currency))
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let (id, ticker, price, time, volume, currency) = row;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        let time = to_time(&time).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price,
                time,
                volume,
            },
            currency,
        ))
    }
    fn get_last_quote_before_by_id(
        &mut self,
        asset_id: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError> {
        let time = time.to_rfc3339();
        let row = self
            .conn
            .query_row(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t 
                WHERE t.asset_id=? AND t.id=q.ticker_id AND q.time<= ?
                ORDER BY q.time, t.priority DESC LIMIT 1",
                params![asset_id as i64, time],
                |row| {
                    let id: i64 = row.get(0)?;
                    let ticker: i64 = row.get(1)?;
                    let price: f64 = row.get(2)?;
                    let time: String = row.get(3)?;
                    let volume: Option<f64> = row.get(4)?;
                    let currency: String = row.get(5)?;
                    Ok((id, ticker, price, time, volume, currency))
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let (id, ticker, price, time, volume, currency) = row;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        let time = to_time(&time).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price,
                time,
                volume,
            },
            currency,
        ))
    }
    fn get_all_quotes_for_ticker(&mut self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, price, time, volume FROM quotes 
            WHERE ticker_id=? ORDER BY time ASC;",
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let quotes_map = stmt
            .query_map(params![ticker_id as i64], |row| {
                let id: i64 = row.get(0)?;
                let price: f64 = row.get(1)?;
                let time: String = row.get(2)?;
                let volume: Option<f64> = row.get(3)?;
                Ok((id, price, time, volume))
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut quotes = Vec::new();
        for quote in quotes_map {
            let (id, price, time, volume) = quote.unwrap();
            let time = to_time(&time)?;
            quotes.push(Quote {
                id: Some(id as usize),
                ticker: ticker_id,
                price,
                time,
                volume,
            });
        }
        Ok(quotes)
    }

    fn update_quote(&mut self, quote: &Quote) -> Result<(), DataError> {
        if quote.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = quote.id.unwrap() as i64;
        self.conn
            .execute(
                "UPDATE quotes SET ticker_id=?2, price=?2, time=?4, volume=?5
                WHERE id=?1",
                params![
                    id,
                    quote.ticker as i64,
                    quote.price,
                    quote.time.to_rfc3339(),
                    quote.volume
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_quote(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM quotes WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn get_rounding_digits(&mut self, currency: Currency) -> i32 {
        let digits = self
            .conn
            .query_row(
                "SELECT digits FROM rounding_digits WHERE currency=?;",
                params![currency.to_string()],
                |row| {
                    let digits: i32 = row.get(0)?;
                    Ok(digits)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()));
        match digits {
            Ok(digits) => digits,
            Err(_) => 2,
        }
    }

    fn set_rounding_digits(&mut self, currency: Currency, digits: i32) -> Result<(), DataError> {
        self.conn
            .execute(
                "INSERT INTO rounding_digits (currency, digits) VALUES (?1, ?2)",
                params![currency.to_string(), digits],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
