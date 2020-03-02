use super::SqliteDB;
///! Implemenation of sqlite3 data handler
use crate::asset::Asset;
use crate::data_handler::{AssetHandler, DataError};
use rusqlite::{params, Row, NO_PARAMS};

impl AssetHandler for SqliteDB {
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO assets (name, wkn, isin, note) VALUES (?1, ?2, ?3, ?4)",
                params![asset.name, asset.wkn, asset.isin, asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM assets WHERE name=?;",
                params![asset.name],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }

    fn get_asset_id(&mut self, asset: &Asset) -> Option<usize> {
        let get_id = |row: &Row| -> rusqlite::Result<i64> { row.get(0) };
        let id = if let Some(isin) = &asset.isin {
            self.conn
                .query_row("SELECT id FROM assets WHERE isin=?", &[&isin], get_id)
        } else if let Some(wkn) = &asset.wkn {
            self.conn
                .query_row("SELECT id FROM assets WHERE wkn=?", &[&wkn], get_id)
        } else {
            self.conn
                .query_row("SELECT id FROM assets WHERE name=?", &[&asset.name], get_id)
        };
        match id {
            Ok(id) => Some(id as usize),
            _ => None,
        }
    }

    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        let asset = self
            .conn
            .query_row(
                "SELECT name, wkn, isin, note FROM assets
        WHERE id=?;",
                &[id as i64],
                |row| {
                    Ok(Asset {
                        id: Some(id),
                        name: row.get(0)?,
                        wkn: row.get(1)?,
                        isin: row.get(2)?,
                        note: row.get(3)?,
                    })
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(asset)
    }

    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, wkn, isin, note FROM assets;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let asset_map = stmt
            .query_map(NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                let id = Some(id as usize);
                Ok(Asset {
                    id,
                    name: row.get(1)?,
                    wkn: row.get(2)?,
                    isin: row.get(3)?,
                    note: row.get(4)?,
                })
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut assets = Vec::new();
        for asset in asset_map {
            assets.push(asset.unwrap());
        }
        Ok(assets)
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i64;
        self.conn
            .execute(
                "UPDATE assets SET name=?2, wkn=?3, isin=?4, note=?5 
                WHERE id=?1;",
                params![id, asset.name, asset.wkn, asset.isin, asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM assets WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}