//! PCI-DSS compliant masking and truncation utilities.
//!
//! This module provides functions to safely mask credit card numbers for
//! display and logging purposes. All functions comply with PCI-DSS requirements
//! for handling cardholder data.
//!
//! # PCI-DSS Compliance
//!
//! PCI-DSS allows displaying:
//! - First 6 digits (BIN) and last 4 digits
//! - Only the last 4 digits (preferred for customer-facing display)
//!
//! Never display or log the full card number.

use crate::ValidatedCard;

/// Masks a card number showing only the last 4 digits.
///
/// Format: `****-****-****-1234`
///
/// This is the safest format for customer-facing display.
///
/// # Example
///
/// ```
/// use cc_validator::{validate, mask};
///
/// let card = validate("4111-1111-1111-1111").unwrap();
/// assert_eq!(card.masked(), "****-****-****-1111");
/// ```
#[inline]
pub fn mask_card(card: &ValidatedCard) -> String {
    let last_four = card.last_four();
    let len = card.length();

    // Calculate how many mask characters we need
    let masked_count = len.saturating_sub(4);

    // Build the masked string with grouping
    let mut result = String::with_capacity(len + (len / 4));
    let mut i = 0;

    while i < masked_count {
        if i > 0 && i % 4 == 0 {
            result.push('-');
        }
        result.push('*');
        i += 1;
    }

    // Add separator before last 4 if needed
    if masked_count > 0 && masked_count % 4 == 0 {
        result.push('-');
    }

    result.push_str(&last_four);
    result
}

/// Masks a card number showing the BIN (first 6) and last 4 digits.
///
/// Format: `411111******1234`
///
/// This format is sometimes acceptable for logging in secure environments.
///
/// # Example
///
/// ```
/// use cc_validator::validate;
///
/// let card = validate("4111-1111-1111-1111").unwrap();
/// assert_eq!(card.masked_with_bin(), "411111******1111");
/// ```
#[inline]
pub fn mask_with_bin(card: &ValidatedCard) -> String {
    let digits = card.digits();
    let len = digits.len();

    if len <= 10 {
        // If card is too short, just mask everything except last 4
        return mask_card(card);
    }

    let mut result = String::with_capacity(len);

    // First 6 digits (BIN)
    for &d in &digits[..6] {
        result.push((b'0' + d) as char);
    }

    // Masked middle section
    let middle_count = len - 10; // 6 (bin) + 4 (last four)
    for _ in 0..middle_count {
        result.push('*');
    }

    // Last 4 digits
    for &d in &digits[len - 4..] {
        result.push((b'0' + d) as char);
    }

    result
}

/// Masks a raw card number string.
///
/// This function is useful when you have a string but haven't validated it yet.
/// It will strip non-digit characters and apply masking.
///
/// # Security Note
///
/// Prefer using `mask_card()` with a `ValidatedCard` when possible,
/// as this function will process any input string.
#[inline]
pub fn mask_string(input: &str) -> String {
    // Extract only digits
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();
    let len = digits.len();

    if len <= 4 {
        return "*".repeat(len);
    }

    let masked_count = len - 4;
    let mut result = String::with_capacity(len + (len / 4));

    // Add masked portion with grouping
    for i in 0..masked_count {
        if i > 0 && i % 4 == 0 {
            result.push('-');
        }
        result.push('*');
    }

    // Add separator before last 4 if needed
    if masked_count % 4 == 0 {
        result.push('-');
    }

    // Add last 4 digits
    for &c in &digits[len - 4..] {
        result.push(c);
    }

    result
}

/// Extracts just the last 4 digits from a card number string.
///
/// Returns an empty string if there are fewer than 4 digits.
#[inline]
pub fn last_four_from_string(input: &str) -> String {
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        digits[digits.len() - 4..].iter().collect()
    } else {
        String::new()
    }
}

/// Constant-time comparison of two byte slices.
///
/// This function takes the same amount of time regardless of where
/// (or if) the slices differ, preventing timing attacks.
///
/// # Security
///
/// Use this function when comparing sensitive data like card numbers
/// or tokens to prevent timing side-channel attacks.
///
/// # Example
///
/// ```
/// use cc_validator::mask::constant_time_eq;
///
/// let a = b"4111111111111111";
/// let b = b"4111111111111111";
/// let c = b"4111111111111112";
///
/// assert!(constant_time_eq(a, b));
/// assert!(!constant_time_eq(a, c));
/// ```
#[inline]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    // XOR all bytes together, accumulating differences
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }

    diff == 0
}

/// Constant-time comparison of two strings.
///
/// Wrapper around `constant_time_eq` for string convenience.
#[inline]
pub fn constant_time_eq_str(a: &str, b: &str) -> bool {
    constant_time_eq(a.as_bytes(), b.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardBrand, ValidatedCard, MAX_CARD_DIGITS};

    fn make_card(digits_slice: &[u8]) -> ValidatedCard {
        let mut digits = [0u8; MAX_CARD_DIGITS];
        digits[..digits_slice.len()].copy_from_slice(digits_slice);
        ValidatedCard::new(CardBrand::Visa, digits, digits_slice.len() as u8)
    }

    #[test]
    fn test_mask_card_16_digits() {
        let card = make_card(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(mask_card(&card), "****-****-****-1111");
    }

    #[test]
    fn test_mask_card_15_digits() {
        let card = make_card(&[3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5]);
        let masked = mask_card(&card);
        assert!(masked.ends_with("0005"));
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_mask_with_bin() {
        let card = make_card(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(mask_with_bin(&card), "411111******1111");
    }

    #[test]
    fn test_mask_string() {
        assert_eq!(mask_string("4111111111111111"), "****-****-****-1111");
        assert_eq!(mask_string("4111-1111-1111-1111"), "****-****-****-1111");
        assert_eq!(mask_string("4111 1111 1111 1111"), "****-****-****-1111");
    }

    #[test]
    fn test_last_four_from_string() {
        assert_eq!(last_four_from_string("4111111111111111"), "1111");
        assert_eq!(last_four_from_string("4111-1111-1111-1234"), "1234");
        assert_eq!(last_four_from_string("123"), "");
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn test_constant_time_eq_str() {
        assert!(constant_time_eq_str("4111111111111111", "4111111111111111"));
        assert!(!constant_time_eq_str("4111111111111111", "4111111111111112"));
    }

    #[test]
    fn test_mask_short_card() {
        // 12-digit card (minimum)
        let card = make_card(&[5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4]);
        let masked = mask_card(&card);
        assert!(masked.ends_with("1234"));
    }
}
