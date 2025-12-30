//! In-memory BIN database implementation.
//!
//! Provides a fast, in-memory BIN lookup using a sorted vector
//! with binary search for O(log n) lookups.

use super::{BinDatabase, BinInfo, BinRange};
use std::collections::HashMap;

/// In-memory BIN database using sorted entries and binary search.
///
/// This implementation is optimized for:
/// - Fast lookups: O(log n) using binary search
/// - Memory efficiency: Compact representation
/// - Range support: Can match BIN ranges, not just exact values
///
/// # Example
///
/// ```
/// use cc_validator::bin::{MemoryBinDb, BinInfo, BinDatabase, CardType};
///
/// let mut db = MemoryBinDb::new();
///
/// // Add a single BIN
/// db.insert("411111", BinInfo::with_bin("411111")
///     .issuer("Test Bank")
///     .card_type(CardType::Credit));
///
/// // Look it up
/// if let Some(info) = db.lookup_str("411111") {
///     println!("Issuer: {:?}", info.issuer);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct MemoryBinDb {
    /// Entries sorted by BIN range start for binary search.
    entries: Vec<(BinRange, BinInfo)>,
    /// Optional exact-match index for faster single-BIN lookups.
    exact_index: Option<HashMap<u64, usize>>,
    /// Whether the entries are sorted (for lazy sorting).
    sorted: bool,
}

impl MemoryBinDb {
    /// Creates a new empty in-memory BIN database.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            exact_index: None,
            sorted: true,
        }
    }

    /// Creates a new database with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            exact_index: None,
            sorted: true,
        }
    }

    /// Inserts a single BIN entry.
    pub fn insert(&mut self, bin: &str, info: BinInfo) {
        if let Some(bin_num) = BinRange::parse_bin(bin) {
            self.entries.push((BinRange::single(bin_num), info));
            self.sorted = false;
            self.exact_index = None; // Invalidate index
        }
    }

    /// Inserts a BIN range entry.
    pub fn insert_range(&mut self, start: &str, end: &str, info: BinInfo) {
        if let (Some(start_num), Some(end_num)) =
            (BinRange::parse_bin(start), BinRange::parse_bin(end))
        {
            self.entries.push((BinRange::new(start_num, end_num), info));
            self.sorted = false;
            self.exact_index = None;
        }
    }

    /// Ensures entries are sorted for binary search.
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.entries.sort_by(|a, b| a.0.cmp(&b.0));
            self.sorted = true;
        }
    }

    /// Builds an exact-match index for faster lookups.
    ///
    /// Call this after inserting all entries if you expect many
    /// exact-match lookups.
    pub fn build_index(&mut self) {
        self.ensure_sorted();

        let mut index = HashMap::with_capacity(self.entries.len());
        for (i, (range, _)) in self.entries.iter().enumerate() {
            if range.start == range.end {
                index.insert(range.start, i);
            }
        }

        if !index.is_empty() {
            self.exact_index = Some(index);
        }
    }

    /// Looks up BIN info using binary search.
    fn lookup_bin(&self, bin: u64) -> Option<&BinInfo> {
        // Try exact index first
        if let Some(ref index) = self.exact_index {
            if let Some(&idx) = index.get(&bin) {
                return Some(&self.entries[idx].1);
            }
        }

        // Binary search for range containing this BIN
        let result = self.entries.binary_search_by(|(range, _)| {
            if bin < range.start {
                std::cmp::Ordering::Greater
            } else if bin > range.end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });

        match result {
            Ok(idx) => Some(&self.entries[idx].1),
            Err(_) => None,
        }
    }

    /// Converts digit slice to u64 for lookup.
    fn digits_to_u64(digits: &[u8]) -> u64 {
        let mut result: u64 = 0;
        for &d in digits.iter().take(8) {
            result = result * 10 + (d as u64);
        }
        result
    }

    /// Returns an iterator over all entries.
    pub fn iter(&self) -> impl Iterator<Item = &(BinRange, BinInfo)> {
        self.entries.iter()
    }

    /// Clears all entries from the database.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.exact_index = None;
        self.sorted = true;
    }

    /// Loads entries from a slice of (BIN, BinInfo) tuples.
    pub fn from_entries(entries: Vec<(String, BinInfo)>) -> Self {
        let mut db = Self::with_capacity(entries.len());
        for (bin, info) in entries {
            db.insert(&bin, info);
        }
        db.ensure_sorted();
        db
    }
}

impl BinDatabase for MemoryBinDb {
    fn lookup(&self, bin: &[u8]) -> Option<BinInfo> {
        if bin.is_empty() {
            return None;
        }

        // Try progressively shorter BIN lengths (8, 7, 6)
        // This handles both 8-digit and 6-digit BIN databases
        for len in (6..=8).rev() {
            if bin.len() >= len {
                let bin_num = Self::digits_to_u64(&bin[..len]);
                if let Some(info) = self.lookup_bin(bin_num) {
                    return Some(info.clone());
                }
            }
        }

        None
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Builder for creating MemoryBinDb instances.
#[derive(Debug, Default)]
pub struct MemoryBinDbBuilder {
    entries: Vec<(BinRange, BinInfo)>,
}

impl MemoryBinDbBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a single BIN entry.
    pub fn add(mut self, bin: &str, info: BinInfo) -> Self {
        if let Some(bin_num) = BinRange::parse_bin(bin) {
            self.entries.push((BinRange::single(bin_num), info));
        }
        self
    }

    /// Adds a BIN range entry.
    pub fn add_range(mut self, start: &str, end: &str, info: BinInfo) -> Self {
        if let (Some(start_num), Some(end_num)) =
            (BinRange::parse_bin(start), BinRange::parse_bin(end))
        {
            self.entries.push((BinRange::new(start_num, end_num), info));
        }
        self
    }

    /// Builds the MemoryBinDb.
    pub fn build(mut self) -> MemoryBinDb {
        self.entries.sort_by(|a, b| a.0.cmp(&b.0));
        MemoryBinDb {
            entries: self.entries,
            exact_index: None,
            sorted: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin::CardType;

    fn sample_db() -> MemoryBinDb {
        MemoryBinDbBuilder::new()
            .add(
                "411111",
                BinInfo::with_bin("411111")
                    .issuer("Visa Test Bank")
                    .card_type(CardType::Credit)
                    .country("US"),
            )
            .add(
                "550000",
                BinInfo::with_bin("550000")
                    .issuer("Mastercard Test Bank")
                    .card_type(CardType::Debit)
                    .country("US"),
            )
            .add_range(
                "400000",
                "400099",
                BinInfo::with_bin("400000-400099")
                    .issuer("Range Bank")
                    .card_type(CardType::Credit),
            )
            .build()
    }

    #[test]
    fn test_exact_lookup() {
        let db = sample_db();

        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Visa Test Bank".to_string()));
        assert_eq!(info.card_type, Some(CardType::Credit));
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_range_lookup() {
        let db = sample_db();

        // Should match the 400000-400099 range
        let info = db.lookup_str("400050").unwrap();
        assert_eq!(info.issuer, Some("Range Bank".to_string()));

        // Should also match
        let info = db.lookup_str("400000").unwrap();
        assert_eq!(info.issuer, Some("Range Bank".to_string()));

        let info = db.lookup_str("400099").unwrap();
        assert_eq!(info.issuer, Some("Range Bank".to_string()));
    }

    #[test]
    fn test_not_found() {
        let db = sample_db();
        assert!(db.lookup_str("999999").is_none());
        assert!(db.lookup_str("123456").is_none());
    }

    #[test]
    fn test_lookup_with_digits() {
        let db = sample_db();
        let digits = [4, 1, 1, 1, 1, 1];
        let info = db.lookup(&digits).unwrap();
        assert_eq!(info.issuer, Some("Visa Test Bank".to_string()));
    }

    #[test]
    fn test_longer_bin_lookup() {
        let db = sample_db();
        // Should still find 411111 even with more digits
        let digits = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let info = db.lookup(&digits).unwrap();
        assert_eq!(info.issuer, Some("Visa Test Bank".to_string()));
    }

    #[test]
    fn test_build_index() {
        let mut db = sample_db();
        db.build_index();

        // Should still work with index
        let info = db.lookup_str("411111").unwrap();
        assert_eq!(info.issuer, Some("Visa Test Bank".to_string()));
    }

    #[test]
    fn test_len() {
        let db = sample_db();
        assert_eq!(db.len(), 3);
        assert!(!db.is_empty());

        let empty = MemoryBinDb::new();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut db = sample_db();
        assert!(!db.is_empty());
        db.clear();
        assert!(db.is_empty());
    }

    #[test]
    fn test_empty_lookup() {
        let db = sample_db();
        assert!(db.lookup(&[]).is_none());
    }
}
