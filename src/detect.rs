//! Card brand detection using BIN/IIN prefix matching.
//!
//! The Bank Identification Number (BIN), also known as Issuer Identification
//! Number (IIN), is the first 6-8 digits of a card number. This module uses
//! pattern matching on these prefixes to detect the card brand.
//!
//! # Performance
//!
//! Detection is O(1) using pattern matching - no loops or hash lookups.

use crate::CardBrand;

/// Detects the card brand from a sequence of digits.
///
/// Uses the BIN/IIN prefix to identify the card network. This function
/// examines up to the first 8 digits to make the determination.
///
/// # Arguments
///
/// * `digits` - A slice of digits (0-9) representing the card number.
///
/// # Returns
///
/// `Some(CardBrand)` if a known brand is detected, `None` otherwise.
///
/// # Example
///
/// ```
/// use cc_validator::detect::detect_brand;
/// use cc_validator::CardBrand;
///
/// // Visa starts with 4
/// let visa = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
/// assert_eq!(detect_brand(&visa), Some(CardBrand::Visa));
///
/// // Amex starts with 34 or 37
/// let amex = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5];
/// assert_eq!(detect_brand(&amex), Some(CardBrand::Amex));
/// ```
#[inline]
pub fn detect_brand(digits: &[u8]) -> Option<CardBrand> {
    if digits.is_empty() {
        return None;
    }

    // Match on prefixes - order matters for overlapping ranges
    // More specific patterns must come before general ones
    match digits {
        // Mir: 2200-2204 (must be before Mastercard 2221-2720)
        [2, 2, 0, 0..=4, ..] => Some(CardBrand::Mir),

        // Mastercard: 51-55 or 2221-2720
        [5, 1..=5, ..] => Some(CardBrand::Mastercard),
        [2, 2, 2, 1..=9, ..] => Some(CardBrand::Mastercard), // 2221-2229
        [2, 2, 3..=9, _, ..] => Some(CardBrand::Mastercard), // 2230-2299
        [2, 3..=6, _, _, ..] => Some(CardBrand::Mastercard), // 2300-2699
        [2, 7, 0..=1, _, ..] => Some(CardBrand::Mastercard), // 2700-2719
        [2, 7, 2, 0, ..] => Some(CardBrand::Mastercard),     // 2720

        // American Express: 34 or 37
        [3, 4, ..] | [3, 7, ..] => Some(CardBrand::Amex),

        // Diners Club: 36, 38, 300-305, 309
        [3, 6, ..] | [3, 8, ..] => Some(CardBrand::DinersClub),
        [3, 0, 0..=5, ..] => Some(CardBrand::DinersClub),
        [3, 0, 9, ..] => Some(CardBrand::DinersClub),

        // JCB: 3528-3589
        [3, 5, 2, 8..=9, ..] => Some(CardBrand::Jcb),
        [3, 5, 3..=8, _, ..] => Some(CardBrand::Jcb),

        // Visa: starts with 4
        [4, ..] => Some(CardBrand::Visa),

        // Verve (Nigeria): 506, 507 (must be before Maestro 50x)
        [5, 0, 6..=7, ..] => Some(CardBrand::Verve),

        // Elo (Brazil): 509, 6362, 6363 (must be before Maestro 50x)
        [5, 0, 9, ..] => Some(CardBrand::Elo),       // 509xxx

        // Maestro: 50 (except 506, 507, 509), 56-58
        [5, 0, ..] => Some(CardBrand::Maestro),
        [5, 6..=8, ..] => Some(CardBrand::Maestro),

        // Discover: 6011, 644-649, 65
        [6, 0, 1, 1, ..] => Some(CardBrand::Discover),
        [6, 4, 4..=9, ..] => Some(CardBrand::Discover),
        [6, 5, ..] => Some(CardBrand::Discover),

        // Elo (Brazil): 6362, 6363 (must be after Discover 65, before Maestro 6x)
        [6, 3, 6, 2..=3, ..] => Some(CardBrand::Elo), // 6362, 6363

        // UnionPay: 62
        [6, 2, ..] => Some(CardBrand::UnionPay),

        // Maestro: remaining 6x ranges (60 except 6011, 61, 63, 66-69)
        [6, 0, ..] => Some(CardBrand::Maestro),
        [6, 1, ..] => Some(CardBrand::Maestro),
        [6, 3, ..] => Some(CardBrand::Maestro),
        [6, 6..=9, ..] => Some(CardBrand::Maestro),

        // RuPay: Indian cards - 81, 82
        [8, 1, ..] | [8, 2, ..] => Some(CardBrand::RuPay),

        // BC Card (South Korea): 94
        [9, 4, ..] => Some(CardBrand::BcCard),

        // Troy (Turkey): 9792
        [9, 7, 9, 2, ..] => Some(CardBrand::Troy),

        // Unknown
        _ => None,
    }
}

/// Validates that the card length is appropriate for the detected brand.
///
/// # Arguments
///
/// * `brand` - The detected card brand.
/// * `length` - The number of digits in the card.
///
/// # Returns
///
/// `true` if the length is valid for the brand, `false` otherwise.
#[inline]
pub fn is_valid_length_for_brand(brand: CardBrand, length: usize) -> bool {
    brand.is_valid_length(length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visa_detection() {
        // 16-digit Visa
        assert_eq!(
            detect_brand(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            Some(CardBrand::Visa)
        );
        // 13-digit Visa
        assert_eq!(
            detect_brand(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            Some(CardBrand::Visa)
        );
        // 19-digit Visa
        assert_eq!(
            detect_brand(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            Some(CardBrand::Visa)
        );
    }

    #[test]
    fn test_mastercard_detection() {
        // 51-55 range
        assert_eq!(
            detect_brand(&[5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mastercard)
        );
        assert_eq!(
            detect_brand(&[5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mastercard)
        );
        // 2221-2720 range
        assert_eq!(
            detect_brand(&[2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mastercard)
        );
        assert_eq!(
            detect_brand(&[2, 7, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mastercard)
        );
    }

    #[test]
    fn test_amex_detection() {
        // 34 prefix
        assert_eq!(
            detect_brand(&[3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Amex)
        );
        // 37 prefix
        assert_eq!(
            detect_brand(&[3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5]),
            Some(CardBrand::Amex)
        );
    }

    #[test]
    fn test_discover_detection() {
        // 6011 prefix
        assert_eq!(
            detect_brand(&[6, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Discover)
        );
        // 65 prefix
        assert_eq!(
            detect_brand(&[6, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Discover)
        );
        // 644-649 range
        assert_eq!(
            detect_brand(&[6, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Discover)
        );
    }

    #[test]
    fn test_diners_club_detection() {
        // 36 prefix
        assert_eq!(
            detect_brand(&[3, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::DinersClub)
        );
        // 300-305 range
        assert_eq!(
            detect_brand(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::DinersClub)
        );
        assert_eq!(
            detect_brand(&[3, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::DinersClub)
        );
    }

    #[test]
    fn test_jcb_detection() {
        // 3528-3589 range
        assert_eq!(
            detect_brand(&[3, 5, 2, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Jcb)
        );
        assert_eq!(
            detect_brand(&[3, 5, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Jcb)
        );
    }

    #[test]
    fn test_unionpay_detection() {
        // 62 prefix
        assert_eq!(
            detect_brand(&[6, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::UnionPay)
        );
    }

    #[test]
    fn test_maestro_detection() {
        // 50 prefix
        assert_eq!(
            detect_brand(&[5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Maestro)
        );
        // 56-69 range (excluding specific brands)
        assert_eq!(
            detect_brand(&[5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Maestro)
        );
    }

    #[test]
    fn test_mir_detection() {
        // 2200-2204 range
        assert_eq!(
            detect_brand(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mir)
        );
        assert_eq!(
            detect_brand(&[2, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(CardBrand::Mir)
        );
    }

    #[test]
    fn test_unknown_brand() {
        // Invalid prefix
        assert_eq!(
            detect_brand(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            None
        );
        assert_eq!(
            detect_brand(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            None
        );
        assert_eq!(
            detect_brand(&[9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            None
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(detect_brand(&[]), None);
    }

    #[test]
    fn test_length_validation() {
        // Visa valid lengths
        assert!(is_valid_length_for_brand(CardBrand::Visa, 13));
        assert!(is_valid_length_for_brand(CardBrand::Visa, 16));
        assert!(is_valid_length_for_brand(CardBrand::Visa, 19));
        assert!(!is_valid_length_for_brand(CardBrand::Visa, 15));

        // Amex only 15
        assert!(is_valid_length_for_brand(CardBrand::Amex, 15));
        assert!(!is_valid_length_for_brand(CardBrand::Amex, 16));

        // Mastercard only 16
        assert!(is_valid_length_for_brand(CardBrand::Mastercard, 16));
        assert!(!is_valid_length_for_brand(CardBrand::Mastercard, 15));
    }
}
