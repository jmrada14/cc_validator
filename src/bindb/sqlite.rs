//! SQLite BIN database implementation.
//!
//! Provides a SQLite-backed BIN database that queries the database
//! directly, suitable for large BIN datasets.
//!
//! # Feature
//!
//! Requires the `bin-sqlite` feature.
//!
//! # Database Schema
//!
//! The loader expects a table with at least a `bin` column:
//!
//! ```sql
//! CREATE TABLE bins (
//!     bin TEXT PRIMARY KEY,
//!     issuer TEXT,
//!     card_type TEXT,
//!     card_level TEXT,
//!     country TEXT,
//!     country_name TEXT,
//!     brand TEXT,
//!     bank_phone TEXT,
//!     bank_url TEXT
//! );
//! ```

#![cfg(feature = "bin-sqlite")]

use super::{BinDatabase, BinDbError, BinInfo, CardLevel, CardType, MemoryBinDb};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::Mutex;

/// SQLite-backed BIN database.
///
/// Queries the SQLite database directly for each lookup.
/// Suitable for large BIN datasets where loading everything
/// into memory is not practical.
///
/// # Thread Safety
///
/// The connection is wrapped in a Mutex to allow sharing across threads.
/// For high-concurrency scenarios, consider using a connection pool.
pub struct SqliteBinDb {
    conn: Mutex<Connection>,
    table_name: String,
}

impl SqliteBinDb {
    /// Opens a SQLite BIN database from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cc_validator::bin::SqliteBinDb;
    ///
    /// let db = SqliteBinDb::open("bins.db")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BinDbError> {
        Self::open_with_table(path, "bins")
    }

    /// Opens a SQLite BIN database with a custom table name.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file.
    /// * `table` - Name of the table containing BIN data.
    pub fn open_with_table<P: AsRef<Path>>(path: P, table: &str) -> Result<Self, BinDbError> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| BinDbError::IoError(std::io::Error::other(e.to_string())))?;

        // Optimize for read-only queries
        conn.execute_batch(
            "PRAGMA journal_mode = OFF;
             PRAGMA synchronous = OFF;
             PRAGMA cache_size = 10000;",
        )
        .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
            table_name: table.to_string(),
        })
    }

    /// Opens an in-memory SQLite database.
    ///
    /// Useful for testing or temporary databases.
    pub fn open_in_memory() -> Result<Self, BinDbError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| BinDbError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(Self {
            conn: Mutex::new(conn),
            table_name: "bins".to_string(),
        })
    }

    /// Creates the BIN table schema.
    ///
    /// Call this when creating a new database.
    pub fn create_schema(&self) -> Result<(), BinDbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS {} (
                bin TEXT PRIMARY KEY,
                issuer TEXT,
                card_type TEXT,
                card_level TEXT,
                country TEXT,
                country_name TEXT,
                brand TEXT,
                bank_phone TEXT,
                bank_url TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_{}_bin ON {} (bin);",
            self.table_name, self.table_name, self.table_name
        ))
        .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        Ok(())
    }

    /// Inserts a BIN info entry.
    pub fn insert(&self, info: &BinInfo) -> Result<(), BinDbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (bin, issuer, card_type, card_level, country, country_name, brand, bank_phone, bank_url)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                self.table_name
            ),
            rusqlite::params![
                info.bin,
                info.issuer,
                info.card_type.map(|t| format!("{:?}", t)),
                info.card_level.map(|l| format!("{:?}", l)),
                info.country,
                info.country_name,
                info.brand,
                info.bank_phone,
                info.bank_url,
            ],
        )
        .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        Ok(())
    }

    /// Bulk inserts multiple BIN entries.
    pub fn insert_many(&self, entries: &[BinInfo]) -> Result<(), BinDbError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        {
            let mut stmt = tx
                .prepare(&format!(
                    "INSERT OR REPLACE INTO {} (bin, issuer, card_type, card_level, country, country_name, brand, bank_phone, bank_url)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    self.table_name
                ))
                .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

            for info in entries {
                stmt.execute(rusqlite::params![
                    info.bin,
                    info.issuer,
                    info.card_type.map(|t| format!("{:?}", t)),
                    info.card_level.map(|l| format!("{:?}", l)),
                    info.country,
                    info.country_name,
                    info.brand,
                    info.bank_phone,
                    info.bank_url,
                ])
                .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        Ok(())
    }

    /// Loads all entries into a MemoryBinDb.
    ///
    /// Useful if you want to switch to in-memory lookups after loading.
    pub fn load_all(&self) -> Result<MemoryBinDb, BinDbError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!("SELECT bin, issuer, card_type, card_level, country, country_name, brand, bank_phone, bank_url FROM {}", self.table_name))
            .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(BinInfo {
                    bin: row.get(0)?,
                    issuer: row.get(1)?,
                    card_type: row
                        .get::<_, Option<String>>(2)?
                        .as_ref()
                        .map(|s| parse_card_type(s)),
                    card_level: row
                        .get::<_, Option<String>>(3)?
                        .as_ref()
                        .map(|s| parse_card_level(s)),
                    country: row.get(4)?,
                    country_name: row.get(5)?,
                    brand: row.get(6)?,
                    bank_phone: row.get(7)?,
                    bank_url: row.get(8)?,
                })
            })
            .map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;

        let mut db = MemoryBinDb::new();
        for entry in entries {
            let info = entry.map_err(|e| BinDbError::InvalidDatabase(e.to_string()))?;
            let bin = info.bin.clone();
            db.insert(&bin, info);
        }

        Ok(db)
    }
}

impl BinDatabase for SqliteBinDb {
    fn lookup(&self, bin: &[u8]) -> Option<BinInfo> {
        if bin.is_empty() {
            return None;
        }

        // Convert digits to string
        let bin_str: String = bin.iter().take(8).map(|&d| (b'0' + d) as char).collect();

        // Try progressively shorter BIN lengths
        for len in (6..=8).rev() {
            if bin_str.len() >= len {
                let search_bin = &bin_str[..len];
                if let Some(info) = self.lookup_str(search_bin) {
                    return Some(info);
                }
            }
        }

        None
    }

    fn lookup_str(&self, bin: &str) -> Option<BinInfo> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare_cached(&format!(
                "SELECT bin, issuer, card_type, card_level, country, country_name, brand, bank_phone, bank_url
                 FROM {} WHERE bin = ?1",
                self.table_name
            ))
            .ok()?;

        stmt.query_row([bin], |row| {
            Ok(BinInfo {
                bin: row.get(0)?,
                issuer: row.get(1)?,
                card_type: row
                    .get::<_, Option<String>>(2)?
                    .as_ref()
                    .map(|s| parse_card_type(s)),
                card_level: row
                    .get::<_, Option<String>>(3)?
                    .as_ref()
                    .map(|s| parse_card_level(s)),
                country: row.get(4)?,
                country_name: row.get(5)?,
                brand: row.get(6)?,
                bank_phone: row.get(7)?,
                bank_url: row.get(8)?,
            })
        })
        .ok()
    }

    fn len(&self) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            &format!("SELECT COUNT(*) FROM {}", self.table_name),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
    }
}

/// Parses a card type string into CardType enum.
fn parse_card_type(s: &str) -> CardType {
    match s.to_lowercase().as_str() {
        "credit" => CardType::Credit,
        "debit" => CardType::Debit,
        "prepaid" => CardType::Prepaid,
        "charge" => CardType::Charge,
        "corporate" | "business" => CardType::Corporate,
        _ => CardType::Unknown,
    }
}

/// Parses a card level string into CardLevel enum.
fn parse_card_level(s: &str) -> CardLevel {
    match s.to_lowercase().as_str() {
        "standard" | "classic" => CardLevel::Standard,
        "gold" => CardLevel::Gold,
        "platinum" => CardLevel::Platinum,
        "signature" | "premium" => CardLevel::Signature,
        "infinite" | "black" => CardLevel::Infinite,
        "business" => CardLevel::Business,
        "corporate" => CardLevel::Corporate,
        "world" | "world elite" => CardLevel::World,
        _ => CardLevel::Unknown,
    }
}

// Ensure thread safety
unsafe impl Send for SqliteBinDb {}
unsafe impl Sync for SqliteBinDb {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> SqliteBinDb {
        let db = SqliteBinDb::open_in_memory().unwrap();
        db.create_schema().unwrap();

        db.insert(&BinInfo::with_bin("411111")
            .issuer("Test Bank")
            .card_type(CardType::Credit)
            .country("US"))
            .unwrap();

        db.insert(&BinInfo::with_bin("550000")
            .issuer("Another Bank")
            .card_type(CardType::Debit)
            .country("GB"))
            .unwrap();

        db
    }

    #[test]
    fn test_lookup() {
        let db = create_test_db();

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_lookup_not_found() {
        let db = create_test_db();
        assert!(db.lookup_str("999999").is_none());
    }

    #[test]
    fn test_lookup_by_digits() {
        let db = create_test_db();
        let digits = [4, 1, 1, 1, 1, 1];
        let info = db.lookup(&digits).unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
    }

    #[test]
    fn test_len() {
        let db = create_test_db();
        assert_eq!(db.len(), 2);
    }

    #[test]
    fn test_bulk_insert() {
        let db = SqliteBinDb::open_in_memory().unwrap();
        db.create_schema().unwrap();

        let entries = vec![
            BinInfo::with_bin("411111").issuer("Bank 1"),
            BinInfo::with_bin("550000").issuer("Bank 2"),
            BinInfo::with_bin("378282").issuer("Bank 3"),
        ];

        db.insert_many(&entries).unwrap();
        assert_eq!(db.len(), 3);
    }

    #[test]
    fn test_load_all() {
        let db = create_test_db();
        let memory_db = db.load_all().unwrap();

        assert_eq!(memory_db.len(), 2);
        let info = memory_db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
    }
}
