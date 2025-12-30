//! BIN (Bank Identification Number) database integration.
//!
//! This module provides a pluggable interface for looking up card issuer
//! information based on the BIN (first 6-8 digits of a card number).
//!
//! # Features
//!
//! BIN database support is optional and requires feature flags:
//!
//! - `bin-json` - JSON file loader
//! - `bin-csv` - CSV file loader
//! - `bin-sqlite` - SQLite database loader
//!
//! # Example
//!
//! ```rust,ignore
//! use cc_validator::bin::{BinDatabase, MemoryBinDb, BinInfo};
//!
//! // Load BIN database from JSON
//! let db = MemoryBinDb::from_json("bins.json")?;
//!
//! // Look up card info
//! if let Some(info) = db.lookup(b"411111") {
//!     println!("Issuer: {:?}", info.issuer);
//!     println!("Country: {:?}", info.country);
//! }
//! ```

mod memory;

#[cfg(feature = "bin-json")]
mod json;

#[cfg(feature = "bin-csv")]
mod csv;

#[cfg(feature = "bin-sqlite")]
mod sqlite;

pub use memory::{MemoryBinDb, MemoryBinDbBuilder};

#[cfg(feature = "bin-json")]
pub use json::JsonBinLoader;

#[cfg(feature = "bin-csv")]
pub use csv::CsvBinLoader;

#[cfg(feature = "bin-sqlite")]
pub use sqlite::SqliteBinDb;

use std::fmt;

/// Trait for BIN database implementations.
///
/// Implement this trait to provide custom BIN lookup functionality.
/// The library provides several implementations:
///
/// - `MemoryBinDb` - In-memory database using sorted vector + binary search
/// - `SqliteBinDb` - SQLite-backed database (requires `bin-sqlite` feature)
pub trait BinDatabase: Send + Sync {
    /// Looks up BIN information for the given digits.
    ///
    /// # Arguments
    ///
    /// * `bin` - The BIN digits (typically first 6-8 digits of card)
    ///
    /// # Returns
    ///
    /// `Some(BinInfo)` if found, `None` otherwise.
    fn lookup(&self, bin: &[u8]) -> Option<BinInfo>;

    /// Looks up BIN information from a string.
    ///
    /// Convenience method that converts the string to digits first.
    fn lookup_str(&self, bin: &str) -> Option<BinInfo> {
        let digits: Vec<u8> = bin
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| (c as u8) - b'0')
            .collect();
        self.lookup(&digits)
    }

    /// Returns the number of BIN entries in the database.
    fn len(&self) -> usize;

    /// Returns true if the database is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Information about a card BIN (Bank Identification Number).
///
/// All fields are optional since not all BIN databases contain
/// complete information for every entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "bin-json", derive(serde::Serialize, serde::Deserialize))]
pub struct BinInfo {
    /// The BIN/IIN number this info applies to.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub bin: String,

    /// Name of the issuing bank/institution.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub issuer: Option<String>,

    /// Type of card (Credit, Debit, Prepaid, etc.)
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub card_type: Option<CardType>,

    /// Card level/tier (Standard, Gold, Platinum, etc.)
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub card_level: Option<CardLevel>,

    /// ISO 3166-1 alpha-2 country code of issuer.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub country: Option<String>,

    /// Full country name.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub country_name: Option<String>,

    /// Card brand/network (Visa, Mastercard, etc.)
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub brand: Option<String>,

    /// Bank's customer service phone number.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub bank_phone: Option<String>,

    /// Bank's website URL.
    #[cfg_attr(feature = "bin-json", serde(default))]
    pub bank_url: Option<String>,
}

impl BinInfo {
    /// Creates a new empty BinInfo.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a BinInfo with the given BIN.
    pub fn with_bin(bin: impl Into<String>) -> Self {
        Self {
            bin: bin.into(),
            ..Default::default()
        }
    }

    /// Builder method to set the issuer.
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Builder method to set the card type.
    pub fn card_type(mut self, card_type: CardType) -> Self {
        self.card_type = Some(card_type);
        self
    }

    /// Builder method to set the card level.
    pub fn card_level(mut self, card_level: CardLevel) -> Self {
        self.card_level = Some(card_level);
        self
    }

    /// Builder method to set the country.
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }
}

/// Type of payment card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bin-json", derive(serde::Serialize, serde::Deserialize))]
pub enum CardType {
    /// Credit card - line of credit from issuer.
    Credit,
    /// Debit card - direct access to bank account.
    Debit,
    /// Prepaid card - preloaded with funds.
    Prepaid,
    /// Charge card - must pay balance in full each month.
    Charge,
    /// Corporate/Business card.
    Corporate,
    /// Unknown card type.
    Unknown,
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Credit => write!(f, "Credit"),
            Self::Debit => write!(f, "Debit"),
            Self::Prepaid => write!(f, "Prepaid"),
            Self::Charge => write!(f, "Charge"),
            Self::Corporate => write!(f, "Corporate"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Default for CardType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Card level/tier indicating benefits and status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bin-json", derive(serde::Serialize, serde::Deserialize))]
pub enum CardLevel {
    /// Standard/Classic tier.
    Standard,
    /// Gold tier.
    Gold,
    /// Platinum tier.
    Platinum,
    /// Signature/Premium tier.
    Signature,
    /// Infinite/Black tier (highest).
    Infinite,
    /// Business card.
    Business,
    /// Corporate card.
    Corporate,
    /// World/World Elite tier.
    World,
    /// Unknown level.
    Unknown,
}

impl fmt::Display for CardLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "Standard"),
            Self::Gold => write!(f, "Gold"),
            Self::Platinum => write!(f, "Platinum"),
            Self::Signature => write!(f, "Signature"),
            Self::Infinite => write!(f, "Infinite"),
            Self::Business => write!(f, "Business"),
            Self::Corporate => write!(f, "Corporate"),
            Self::World => write!(f, "World"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Default for CardLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A range of BIN numbers.
///
/// Used for efficient lookup when BINs are assigned in ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinRange {
    /// Start of the range (inclusive).
    pub start: u64,
    /// End of the range (inclusive).
    pub end: u64,
}

impl BinRange {
    /// Creates a new BIN range.
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Creates a range for a single BIN.
    pub fn single(bin: u64) -> Self {
        Self {
            start: bin,
            end: bin,
        }
    }

    /// Checks if a BIN falls within this range.
    #[inline]
    pub fn contains(&self, bin: u64) -> bool {
        bin >= self.start && bin <= self.end
    }

    /// Parses a BIN string into a u64.
    pub fn parse_bin(bin: &str) -> Option<u64> {
        bin.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .ok()
    }
}

impl PartialOrd for BinRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BinRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

/// Error type for BIN database operations.
#[derive(Debug)]
pub enum BinDbError {
    /// Failed to read the database file.
    IoError(std::io::Error),
    /// Failed to parse the database format.
    ParseError(String),
    /// Database is corrupted or invalid.
    InvalidDatabase(String),
    /// Feature not available.
    FeatureNotEnabled(String),
}

impl fmt::Display for BinDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
            Self::InvalidDatabase(s) => write!(f, "Invalid database: {}", s),
            Self::FeatureNotEnabled(s) => write!(f, "Feature not enabled: {}", s),
        }
    }
}

impl std::error::Error for BinDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BinDbError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_info_builder() {
        let info = BinInfo::with_bin("411111")
            .issuer("Test Bank")
            .card_type(CardType::Credit)
            .card_level(CardLevel::Gold)
            .country("US");

        assert_eq!(info.bin, "411111");
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.card_level, Some(CardLevel::Gold));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_bin_range() {
        let range = BinRange::new(411111, 411199);
        assert!(range.contains(411111));
        assert!(range.contains(411150));
        assert!(range.contains(411199));
        assert!(!range.contains(411100));
        assert!(!range.contains(411200));
    }

    #[test]
    fn test_bin_range_single() {
        let range = BinRange::single(411111);
        assert!(range.contains(411111));
        assert!(!range.contains(411112));
    }

    #[test]
    fn test_parse_bin() {
        assert_eq!(BinRange::parse_bin("411111"), Some(411111));
        assert_eq!(BinRange::parse_bin("4111-11"), Some(411111));
        assert_eq!(BinRange::parse_bin(""), None);
    }

    #[test]
    fn test_card_type_display() {
        assert_eq!(CardType::Credit.to_string(), "Credit");
        assert_eq!(CardType::Debit.to_string(), "Debit");
        assert_eq!(CardType::Prepaid.to_string(), "Prepaid");
    }

    #[test]
    fn test_card_level_display() {
        assert_eq!(CardLevel::Standard.to_string(), "Standard");
        assert_eq!(CardLevel::Platinum.to_string(), "Platinum");
        assert_eq!(CardLevel::Infinite.to_string(), "Infinite");
    }
}
