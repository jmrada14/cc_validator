//! CSV BIN database loader.
//!
//! Loads BIN data from CSV files into a MemoryBinDb.
//!
//! # Feature
//!
//! Requires the `bin-csv` feature.
//!
//! # Supported Format
//!
//! ```csv
//! bin,issuer,card_type,card_level,country,country_name,brand,bank_phone,bank_url
//! 411111,Test Bank,credit,standard,US,United States,Visa,1-800-555-0100,https://example.com
//! ```
//!
//! Column order doesn't matter as long as headers are present.
//! Only the `bin` column is required.

use super::{BinDbError, BinInfo, CardLevel, CardType, MemoryBinDb};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// CSV BIN database loader.
///
/// Loads BIN data from CSV files with header row.
pub struct CsvBinLoader;

impl CsvBinLoader {
    /// Loads a BIN database from a CSV file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CSV file.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cc_validator::bin::CsvBinLoader;
    ///
    /// let db = CsvBinLoader::from_file("bins.csv")?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<MemoryBinDb, BinDbError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    /// Loads a BIN database from a reader.
    pub fn from_reader<R: Read>(reader: R) -> Result<MemoryBinDb, BinDbError> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|e| BinDbError::ParseError(format!("Failed to read CSV headers: {}", e)))?
            .clone();

        // Map header names to column indices
        let col_map = ColumnMap::from_headers(&headers)?;

        let mut db = MemoryBinDb::new();

        for result in csv_reader.records() {
            let record = result
                .map_err(|e| BinDbError::ParseError(format!("CSV parse error: {}", e)))?;

            if let Some(info) = col_map.parse_record(&record) {
                let bin = info.bin.clone();
                db.insert(&bin, info);
            }
        }

        Ok(db)
    }

    /// Loads a BIN database from a CSV string.
    pub fn parse(csv: &str) -> Result<MemoryBinDb, BinDbError> {
        Self::from_reader(csv.as_bytes())
    }

    /// Loads a BIN database with custom delimiter.
    pub fn from_file_with_delimiter<P: AsRef<Path>>(
        path: P,
        delimiter: u8,
    ) -> Result<MemoryBinDb, BinDbError> {
        let file = File::open(path)?;
        Self::from_reader_with_delimiter(file, delimiter)
    }

    /// Loads from reader with custom delimiter.
    pub fn from_reader_with_delimiter<R: Read>(
        reader: R,
        delimiter: u8,
    ) -> Result<MemoryBinDb, BinDbError> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|e| BinDbError::ParseError(format!("Failed to read CSV headers: {}", e)))?
            .clone();

        let col_map = ColumnMap::from_headers(&headers)?;
        let mut db = MemoryBinDb::new();

        for result in csv_reader.records() {
            let record = result
                .map_err(|e| BinDbError::ParseError(format!("CSV parse error: {}", e)))?;

            if let Some(info) = col_map.parse_record(&record) {
                let bin = info.bin.clone();
                db.insert(&bin, info);
            }
        }

        Ok(db)
    }
}

/// Maps CSV column names to indices.
struct ColumnMap {
    bin: usize,
    issuer: Option<usize>,
    card_type: Option<usize>,
    card_level: Option<usize>,
    country: Option<usize>,
    country_name: Option<usize>,
    brand: Option<usize>,
    bank_phone: Option<usize>,
    bank_url: Option<usize>,
}

impl ColumnMap {
    fn from_headers(headers: &csv::StringRecord) -> Result<Self, BinDbError> {
        let mut bin_col = None;
        let mut issuer_col = None;
        let mut card_type_col = None;
        let mut card_level_col = None;
        let mut country_col = None;
        let mut country_name_col = None;
        let mut brand_col = None;
        let mut bank_phone_col = None;
        let mut bank_url_col = None;

        for (i, header) in headers.iter().enumerate() {
            match header.to_lowercase().trim() {
                "bin" | "iin" => bin_col = Some(i),
                "issuer" | "bank" | "bank_name" | "issuer_name" => issuer_col = Some(i),
                "card_type" | "type" | "cardtype" => card_type_col = Some(i),
                "card_level" | "level" | "tier" | "cardlevel" => card_level_col = Some(i),
                "country" | "country_code" => country_col = Some(i),
                "country_name" => country_name_col = Some(i),
                "brand" | "scheme" | "network" => brand_col = Some(i),
                "bank_phone" | "phone" => bank_phone_col = Some(i),
                "bank_url" | "url" | "website" => bank_url_col = Some(i),
                _ => {}
            }
        }

        let bin = bin_col
            .ok_or_else(|| BinDbError::ParseError("Missing required 'bin' column".to_string()))?;

        Ok(Self {
            bin,
            issuer: issuer_col,
            card_type: card_type_col,
            card_level: card_level_col,
            country: country_col,
            country_name: country_name_col,
            brand: brand_col,
            bank_phone: bank_phone_col,
            bank_url: bank_url_col,
        })
    }

    fn parse_record(&self, record: &csv::StringRecord) -> Option<BinInfo> {
        let bin = record.get(self.bin)?.trim();
        if bin.is_empty() {
            return None;
        }

        let get_field = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| record.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        Some(BinInfo {
            bin: bin.to_string(),
            issuer: get_field(self.issuer),
            card_type: self
                .card_type
                .and_then(|i| record.get(i))
                .map(parse_card_type),
            card_level: self
                .card_level
                .and_then(|i| record.get(i))
                .map(parse_card_level),
            country: get_field(self.country),
            country_name: get_field(self.country_name),
            brand: get_field(self.brand),
            bank_phone: get_field(self.bank_phone),
            bank_url: get_field(self.bank_url),
        })
    }
}

/// Parses a card type string into CardType enum.
fn parse_card_type(s: &str) -> CardType {
    match s.to_lowercase().trim() {
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
    match s.to_lowercase().trim() {
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

/// Simple CSV loader without the csv crate dependency.
///
/// Use this for basic CSV files when you want to avoid the csv dependency.
#[allow(dead_code)]
pub struct SimpleCsvLoader;

#[allow(dead_code)]
impl SimpleCsvLoader {
    /// Loads a simple CSV file (comma-delimited, no quotes).
    ///
    /// First line must be headers with at least a "bin" column.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<MemoryBinDb, BinDbError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        // Use map_while to stop on first error instead of filter_map which could run forever
        Self::from_lines(reader.lines().map_while(|l| l.ok()))
    }

    /// Loads from an iterator of lines.
    pub fn from_lines<I: Iterator<Item = String>>(mut lines: I) -> Result<MemoryBinDb, BinDbError> {
        // Parse header line
        let header_line = lines
            .next()
            .ok_or_else(|| BinDbError::ParseError("Empty CSV file".to_string()))?;

        let headers: Vec<&str> = header_line.split(',').map(|s| s.trim()).collect();
        let bin_idx = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("bin"))
            .ok_or_else(|| BinDbError::ParseError("Missing 'bin' column".to_string()))?;

        let issuer_idx = headers.iter().position(|h| {
            h.eq_ignore_ascii_case("issuer") || h.eq_ignore_ascii_case("bank")
        });

        let country_idx = headers.iter().position(|h| {
            h.eq_ignore_ascii_case("country") || h.eq_ignore_ascii_case("country_code")
        });

        let mut db = MemoryBinDb::new();

        for line in lines {
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            if let Some(&bin) = fields.get(bin_idx) {
                if !bin.is_empty() {
                    let mut info = BinInfo::with_bin(bin);

                    if let Some(idx) = issuer_idx {
                        if let Some(&issuer) = fields.get(idx) {
                            if !issuer.is_empty() {
                                info.issuer = Some(issuer.to_string());
                            }
                        }
                    }

                    if let Some(idx) = country_idx {
                        if let Some(&country) = fields.get(idx) {
                            if !country.is_empty() {
                                info.country = Some(country.to_string());
                            }
                        }
                    }

                    db.insert(bin, info);
                }
            }
        }

        Ok(db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin::BinDatabase;

    #[test]
    fn test_load_csv() {
        let csv = r#"bin,issuer,card_type,country
411111,Test Bank,credit,US
550000,Another Bank,debit,GB
378282,Amex Bank,charge,US"#;

        let db = CsvBinLoader::parse(csv).unwrap();
        assert_eq!(db.len(), 3);

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_header_aliases() {
        let csv = r#"iin,bank,type,country_code
411111,Test Bank,credit,US"#;

        let db = CsvBinLoader::parse(csv).unwrap();
        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_missing_columns() {
        let csv = r#"bin,issuer
411111,Test Bank
550000,Another Bank"#;

        let db = CsvBinLoader::parse(csv).unwrap();
        assert_eq!(db.len(), 2);

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
        assert!(info.card_type.is_none());
        assert!(info.country.is_none());
    }

    #[test]
    fn test_missing_bin_column() {
        let csv = r#"issuer,country
Test Bank,US"#;

        let result = CsvBinLoader::parse(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_values() {
        let csv = r#"bin,issuer,country
411111,,US
550000,Bank,"#;

        let db = CsvBinLoader::parse(csv).unwrap();

        let info1 = db.lookup_str("411111").unwrap();
        assert!(info1.issuer.is_none());
        assert_eq!(info1.country, Some("US".to_string()));

        let info2 = db.lookup_str("550000").unwrap();
        assert_eq!(info2.issuer, Some("Bank".to_string()));
        assert!(info2.country.is_none());
    }

    #[test]
    fn test_simple_csv_loader() {
        let lines = vec![
            "bin,issuer,country".to_string(),
            "411111,Test Bank,US".to_string(),
            "550000,Another Bank,GB".to_string(),
        ];

        let db = SimpleCsvLoader::from_lines(lines.into_iter()).unwrap();
        assert_eq!(db.len(), 2);

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Test Bank".to_string()));
    }
}
