//! JSON BIN database loader.
//!
//! Loads BIN data from JSON files into a MemoryBinDb.
//!
//! # Feature
//!
//! Requires the `bin-json` feature.
//!
//! # Supported Formats
//!
//! ## Array format
//!
//! ```json
//! [
//!   {
//!     "bin": "411111",
//!     "issuer": "Bank Name",
//!     "card_type": "Credit",
//!     "country": "US"
//!   }
//! ]
//! ```
//!
//! ## Object format (keyed by BIN)
//!
//! ```json
//! {
//!   "411111": {
//!     "issuer": "Bank Name",
//!     "card_type": "Credit"
//!   }
//! }
//! ```

use super::{BinDbError, BinInfo, CardLevel, CardType, MemoryBinDb};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

/// JSON BIN database loader.
///
/// Supports multiple JSON formats commonly used for BIN data.
pub struct JsonBinLoader;

impl JsonBinLoader {
    /// Loads a BIN database from a JSON file.
    ///
    /// Automatically detects the JSON format (array or object).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cc_validator::bin::JsonBinLoader;
    ///
    /// let db = JsonBinLoader::from_file("bins.json")?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<MemoryBinDb, BinDbError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Loads a BIN database from a reader.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<MemoryBinDb, BinDbError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Self::parse(&content)
    }

    /// Loads a BIN database from a JSON string.
    pub fn parse(json: &str) -> Result<MemoryBinDb, BinDbError> {
        // Try to detect format
        let trimmed = json.trim();

        if trimmed.starts_with('[') {
            // Array format
            Self::parse_array(json)
        } else if trimmed.starts_with('{') {
            // Could be object keyed by BIN or single entry
            // Try object format first
            if let Ok(db) = Self::parse_object(json) {
                return Ok(db);
            }
            // Fall back to array of one
            Self::parse_array(json)
        } else {
            Err(BinDbError::ParseError(
                "Invalid JSON format: expected array or object".to_string(),
            ))
        }
    }

    /// Parses JSON array format.
    fn parse_array(json: &str) -> Result<MemoryBinDb, BinDbError> {
        let entries: Vec<JsonBinEntry> = serde_json::from_str(json)
            .map_err(|e| BinDbError::ParseError(format!("JSON parse error: {}", e)))?;

        let mut db = MemoryBinDb::with_capacity(entries.len());

        for entry in entries {
            let bin = entry.bin.clone();
            let info = entry.into_bin_info();
            db.insert(&bin, info);
        }

        Ok(db)
    }

    /// Parses JSON object format (keyed by BIN).
    fn parse_object(json: &str) -> Result<MemoryBinDb, BinDbError> {
        let map: HashMap<String, JsonBinEntry> = serde_json::from_str(json)
            .map_err(|e| BinDbError::ParseError(format!("JSON parse error: {}", e)))?;

        let mut db = MemoryBinDb::with_capacity(map.len());

        for (bin, mut entry) in map {
            // Use the key as the BIN if entry doesn't have one
            if entry.bin.is_empty() {
                entry.bin = bin.clone();
            }
            let info = entry.into_bin_info();
            db.insert(&bin, info);
        }

        Ok(db)
    }
}

/// Internal structure for deserializing JSON BIN entries.
#[derive(Debug, Deserialize, Default)]
struct JsonBinEntry {
    #[serde(default)]
    bin: String,

    #[serde(default, alias = "bank", alias = "bank_name")]
    issuer: Option<String>,

    #[serde(default, alias = "type")]
    card_type: Option<String>,

    #[serde(default, alias = "level", alias = "tier")]
    card_level: Option<String>,

    #[serde(default, alias = "country_code")]
    country: Option<String>,

    #[serde(default)]
    country_name: Option<String>,

    #[serde(default, alias = "scheme", alias = "network")]
    brand: Option<String>,

    #[serde(default, alias = "phone")]
    bank_phone: Option<String>,

    #[serde(default, alias = "url", alias = "website")]
    bank_url: Option<String>,
}

impl JsonBinEntry {
    fn into_bin_info(self) -> BinInfo {
        BinInfo {
            bin: self.bin,
            issuer: self.issuer,
            card_type: self.card_type.as_ref().map(|s| parse_card_type(s)),
            card_level: self.card_level.as_ref().map(|s| parse_card_level(s)),
            country: self.country,
            country_name: self.country_name,
            brand: self.brand,
            bank_phone: self.bank_phone,
            bank_url: self.bank_url,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin::BinDatabase;

    #[test]
    fn test_parse_array_format() {
        let json = r#"[
            {
                "bin": "411111",
                "issuer": "Test Bank",
                "card_type": "Credit",
                "country": "US"
            },
            {
                "bin": "550000",
                "issuer": "Another Bank",
                "card_type": "Debit",
                "country": "GB"
            }
        ]"#;

        let db = JsonBinLoader::parse(json).unwrap();
        assert_eq!(db.len(), 2);

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_parse_object_format() {
        let json = r#"{
            "411111": {
                "issuer": "Test Bank",
                "card_type": "Credit"
            },
            "550000": {
                "issuer": "Another Bank",
                "card_type": "Debit"
            }
        }"#;

        let db = JsonBinLoader::parse(json).unwrap();
        assert_eq!(db.len(), 2);

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
    }

    #[test]
    fn test_field_aliases() {
        let json = r#"[
            {
                "bin": "411111",
                "bank": "Aliased Bank",
                "type": "credit",
                "level": "gold",
                "country_code": "US",
                "scheme": "Visa"
            }
        ]"#;

        let db = JsonBinLoader::parse(json).unwrap();
        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Aliased Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.card_level, Some(CardLevel::Gold));
        assert_eq!(info.country, Some("US".to_string()));
        assert_eq!(info.brand, Some("Visa".to_string()));
    }

    #[test]
    fn test_card_type_parsing() {
        assert_eq!(parse_card_type("Credit"), CardType::Credit);
        assert_eq!(parse_card_type("CREDIT"), CardType::Credit);
        assert_eq!(parse_card_type("debit"), CardType::Debit);
        assert_eq!(parse_card_type("prepaid"), CardType::Prepaid);
        assert_eq!(parse_card_type("unknown_type"), CardType::Unknown);
    }

    #[test]
    fn test_card_level_parsing() {
        assert_eq!(parse_card_level("Gold"), CardLevel::Gold);
        assert_eq!(parse_card_level("PLATINUM"), CardLevel::Platinum);
        assert_eq!(parse_card_level("signature"), CardLevel::Signature);
        assert_eq!(parse_card_level("black"), CardLevel::Infinite);
        assert_eq!(parse_card_level("world elite"), CardLevel::World);
    }

    #[test]
    fn test_empty_json() {
        let db = JsonBinLoader::parse("[]").unwrap();
        assert!(db.is_empty());
    }

    #[test]
    fn test_invalid_json() {
        let result = JsonBinLoader::parse("not valid json");
        assert!(result.is_err());
    }
}
