//! Core card types for validated credit cards.
//!
//! This module provides the `CardBrand` enum for identifying card networks
//! and the `ValidatedCard` struct for holding validated card data securely.

use std::fmt;
use zeroize::Zeroize;

/// Supported credit card brands/networks.
///
/// Each variant represents a major payment network with its own BIN ranges
/// and validation rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardBrand {
    /// Visa - Prefix 4, lengths 13, 16, 19
    Visa,
    /// Mastercard - Prefix 51-55, 2221-2720, length 16
    Mastercard,
    /// American Express - Prefix 34, 37, length 15
    Amex,
    /// Discover - Prefix 6011, 644-649, 65, length 16-19
    Discover,
    /// Diners Club - Prefix 36, 38, 300-305, length 14-19
    DinersClub,
    /// JCB - Prefix 3528-3589, length 16-19
    Jcb,
    /// UnionPay - Prefix 62, length 16-19
    UnionPay,
    /// Maestro - Prefix 50, 56-69, length 12-19
    Maestro,
    /// Mir - Russian payment system, Prefix 2200-2204, length 16-19
    Mir,
    /// RuPay - Indian payment system, Prefix 81, 82, length 16
    RuPay,
    /// Verve - Nigerian payment system, Prefix 506, 507, 650, length 16-19
    Verve,
    /// Elo - Brazilian payment system, various prefixes, length 16
    Elo,
    /// Troy - Turkish payment system, Prefix 9792, length 16
    Troy,
    /// BC Card - South Korean payment system, Prefix 94, length 16
    BcCard,
}

impl CardBrand {
    /// Returns the valid lengths for this card brand.
    #[inline]
    pub const fn valid_lengths(&self) -> &'static [u8] {
        match self {
            Self::Visa => &[13, 16, 19],
            Self::Mastercard => &[16],
            Self::Amex => &[15],
            Self::Discover => &[16, 17, 18, 19],
            Self::DinersClub => &[14, 15, 16, 17, 18, 19],
            Self::Jcb => &[16, 17, 18, 19],
            Self::UnionPay => &[16, 17, 18, 19],
            Self::Maestro => &[12, 13, 14, 15, 16, 17, 18, 19],
            Self::Mir => &[16, 17, 18, 19],
            Self::RuPay => &[16],
            Self::Verve => &[16, 17, 18, 19],
            Self::Elo => &[16],
            Self::Troy => &[16],
            Self::BcCard => &[16],
        }
    }

    /// Returns true if the given length is valid for this brand.
    #[inline]
    pub const fn is_valid_length(&self, length: usize) -> bool {
        let valid = self.valid_lengths();
        let mut i = 0;
        while i < valid.len() {
            if valid[i] as usize == length {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Returns a human-readable name for the card brand.
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Visa => "Visa",
            Self::Mastercard => "Mastercard",
            Self::Amex => "American Express",
            Self::Discover => "Discover",
            Self::DinersClub => "Diners Club",
            Self::Jcb => "JCB",
            Self::UnionPay => "UnionPay",
            Self::Maestro => "Maestro",
            Self::Mir => "Mir",
            Self::RuPay => "RuPay",
            Self::Verve => "Verve",
            Self::Elo => "Elo",
            Self::Troy => "Troy",
            Self::BcCard => "BC Card",
        }
    }
}

impl fmt::Display for CardBrand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Maximum number of digits in a credit card number.
pub const MAX_CARD_DIGITS: usize = 19;

/// Minimum number of digits in a credit card number.
pub const MIN_CARD_DIGITS: usize = 12;

/// A validated credit card with secure memory handling.
///
/// The card number is stored in a fixed-size array that is automatically
/// zeroed when the struct is dropped, preventing sensitive data from
/// lingering in memory.
///
/// # Security
///
/// - Full card number is private and only accessible via controlled methods
/// - Memory is zeroed on drop using the `zeroize` crate
/// - Debug output is masked to prevent accidental logging
/// - Implements Display with masking for safe printing
#[derive(Clone)]
pub struct ValidatedCard {
    /// The detected card brand.
    brand: CardBrand,
    /// The full card number as digits (0-9).
    digits: [u8; MAX_CARD_DIGITS],
    /// Number of actual digits in the card.
    digit_count: u8,
}

impl ValidatedCard {
    /// Creates a new ValidatedCard.
    ///
    /// # Safety
    ///
    /// This is an internal constructor. Use `validate()` to create instances.
    #[inline]
    pub(crate) fn new(brand: CardBrand, digits: [u8; MAX_CARD_DIGITS], digit_count: u8) -> Self {
        Self {
            brand,
            digits,
            digit_count,
        }
    }

    /// Returns the detected card brand.
    #[inline]
    pub const fn brand(&self) -> CardBrand {
        self.brand
    }

    /// Returns the number of digits in the card number.
    #[inline]
    pub const fn length(&self) -> usize {
        self.digit_count as usize
    }

    /// Returns the last four digits as a string.
    ///
    /// Safe for logging and display per PCI-DSS requirements.
    #[inline]
    pub fn last_four(&self) -> String {
        let len = self.digit_count as usize;
        if len >= 4 {
            self.digits[len - 4..len]
                .iter()
                .map(|&d| (b'0' + d) as char)
                .collect()
        } else {
            self.digits[..len]
                .iter()
                .map(|&d| (b'0' + d) as char)
                .collect()
        }
    }

    /// Returns the BIN (Bank Identification Number), first 6-8 digits.
    ///
    /// The BIN identifies the issuing bank. Modern cards use 8-digit BINs,
    /// but 6-digit BINs are still common.
    #[inline]
    pub fn bin(&self, length: usize) -> String {
        let bin_len = length.min(8).min(self.digit_count as usize);
        self.digits[..bin_len]
            .iter()
            .map(|&d| (b'0' + d) as char)
            .collect()
    }

    /// Returns the 6-digit BIN (traditional format).
    #[inline]
    pub fn bin6(&self) -> String {
        self.bin(6)
    }

    /// Returns the 8-digit BIN (modern format).
    #[inline]
    pub fn bin8(&self) -> String {
        self.bin(8)
    }

    /// Returns the full card number as a string.
    ///
    /// # Security Warning
    ///
    /// This method exposes the full card number. Use with extreme caution
    /// and never log the result. For display purposes, use `masked()` instead.
    #[inline]
    pub fn number(&self) -> String {
        self.digits[..self.digit_count as usize]
            .iter()
            .map(|&d| (b'0' + d) as char)
            .collect()
    }

    /// Returns the card number with masking for safe display.
    ///
    /// Format: `****-****-****-1234` (shows only last 4 digits).
    /// This format is PCI-DSS compliant for display purposes.
    #[inline]
    pub fn masked(&self) -> String {
        crate::mask::mask_card(self)
    }

    /// Returns the card number with BIN visible.
    ///
    /// Format: `411111******1234` (shows first 6 and last 4 digits).
    /// This format is PCI-DSS compliant for some logging scenarios.
    #[inline]
    pub fn masked_with_bin(&self) -> String {
        crate::mask::mask_with_bin(self)
    }

    /// Returns the raw digit array (for internal/advanced use).
    ///
    /// # Security Warning
    ///
    /// This exposes the full card number. Prefer `masked()` for display.
    #[inline]
    pub(crate) fn digits(&self) -> &[u8] {
        &self.digits[..self.digit_count as usize]
    }
}

impl fmt::Debug for ValidatedCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask the card number in debug output for security
        f.debug_struct("ValidatedCard")
            .field("brand", &self.brand)
            .field("number", &self.masked())
            .field("length", &self.digit_count)
            .finish()
    }
}

impl fmt::Display for ValidatedCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Always display masked for safety
        write!(f, "{} {}", self.brand, self.masked())
    }
}

// Ensure sensitive data is properly handled
impl Drop for ValidatedCard {
    fn drop(&mut self) {
        // Zeroize the digits array
        self.digits.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_brand_valid_lengths() {
        assert!(CardBrand::Visa.is_valid_length(16));
        assert!(CardBrand::Visa.is_valid_length(13));
        assert!(CardBrand::Visa.is_valid_length(19));
        assert!(!CardBrand::Visa.is_valid_length(15));

        assert!(CardBrand::Amex.is_valid_length(15));
        assert!(!CardBrand::Amex.is_valid_length(16));

        assert!(CardBrand::Mastercard.is_valid_length(16));
        assert!(!CardBrand::Mastercard.is_valid_length(15));
    }

    #[test]
    fn test_card_brand_names() {
        assert_eq!(CardBrand::Visa.name(), "Visa");
        assert_eq!(CardBrand::Amex.name(), "American Express");
        assert_eq!(CardBrand::Mastercard.to_string(), "Mastercard");
    }

    #[test]
    fn test_validated_card_last_four() {
        let mut digits = [0u8; MAX_CARD_DIGITS];
        digits[..16].copy_from_slice(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let card = ValidatedCard::new(CardBrand::Visa, digits, 16);
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn test_validated_card_bin() {
        let mut digits = [0u8; MAX_CARD_DIGITS];
        digits[..16].copy_from_slice(&[4, 5, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let card = ValidatedCard::new(CardBrand::Visa, digits, 16);
        assert_eq!(card.bin6(), "453211");
        assert_eq!(card.bin8(), "45321111");
    }

    #[test]
    fn test_debug_is_masked() {
        let mut digits = [0u8; MAX_CARD_DIGITS];
        digits[..16].copy_from_slice(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let card = ValidatedCard::new(CardBrand::Visa, digits, 16);
        let debug = format!("{:?}", card);
        // Should not contain the full card number
        assert!(!debug.contains("4111111111111111"));
        // Should contain masked version
        assert!(debug.contains("****"));
    }

    #[test]
    fn test_card_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ValidatedCard>();
    }
}
