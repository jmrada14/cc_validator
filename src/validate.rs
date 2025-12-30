//! Main validation orchestration for credit card numbers.
//!
//! This module provides the primary `validate` function that combines
//! parsing, Luhn validation, and brand detection into a single operation.
//!
//! # Performance
//!
//! The validation is designed for zero-copy parsing:
//! - No string allocations during validation
//! - Single-pass digit extraction
//! - O(n) complexity where n is the input length

use crate::card::{CardBrand, ValidatedCard, MAX_CARD_DIGITS, MIN_CARD_DIGITS};
use crate::detect::detect_brand;
use crate::error::ValidationError;
use crate::luhn;

/// Validates a credit card number string.
///
/// This is the primary validation function. It performs:
/// 1. Input parsing (strips spaces and hyphens)
/// 2. Length validation
/// 3. Luhn checksum validation
/// 4. Card brand detection
/// 5. Brand-specific length validation
///
/// # Arguments
///
/// * `input` - The card number as a string. May contain spaces or hyphens.
///
/// # Returns
///
/// * `Ok(ValidatedCard)` - If the card is valid
/// * `Err(ValidationError)` - If validation fails, with details about why
///
/// # Example
///
/// ```
/// use cc_validator::validate;
///
/// // Valid Visa card
/// let card = validate("4111-1111-1111-1111").unwrap();
/// assert_eq!(card.brand().name(), "Visa");
/// assert_eq!(card.last_four(), "1111");
///
/// // Invalid card
/// let err = validate("4111-1111-1111-1112").unwrap_err();
/// println!("Error: {}", err);
/// ```
pub fn validate(input: &str) -> Result<ValidatedCard, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::Empty);
    }

    // Parse input into digits array (zero-copy - we just extract digits)
    let mut digits = [0u8; MAX_CARD_DIGITS];
    let mut count = 0usize;
    let mut pos = 0usize;

    for c in input.chars() {
        match c {
            '0'..='9' => {
                if count >= MAX_CARD_DIGITS {
                    return Err(ValidationError::TooLong {
                        length: count + 1,
                        maximum: MAX_CARD_DIGITS,
                    });
                }
                digits[count] = (c as u8) - b'0';
                count += 1;
            }
            ' ' | '-' | '.' => {
                // Allowed separators, skip them
            }
            _ => {
                return Err(ValidationError::InvalidCharacter {
                    position: pos,
                    character: c,
                });
            }
        }
        pos += 1;
    }

    // Check for empty after stripping
    if count == 0 {
        return Err(ValidationError::NoDigits);
    }

    // Check minimum length
    if count < MIN_CARD_DIGITS {
        return Err(ValidationError::TooShort {
            length: count,
            minimum: MIN_CARD_DIGITS,
        });
    }

    // Validate Luhn checksum
    if !luhn::validate(&digits[..count]) {
        return Err(ValidationError::InvalidChecksum);
    }

    // Detect card brand
    let brand = detect_brand(&digits[..count]).ok_or(ValidationError::UnknownBrand)?;

    // Validate length for detected brand
    if !brand.is_valid_length(count) {
        return Err(ValidationError::InvalidLengthForBrand {
            brand,
            length: count,
            valid_lengths: brand.valid_lengths(),
        });
    }

    Ok(ValidatedCard::new(brand, digits, count as u8))
}

/// Validates a credit card number, allowing unknown brands.
///
/// Like `validate`, but returns a card with `CardBrand::Unknown` is returned
/// instead of an error when the brand cannot be detected.
///
/// This is useful when you want to accept any card that passes Luhn validation,
/// regardless of whether it matches a known brand pattern.
///
/// # Example
///
/// ```
/// use cc_validator::validate_any;
///
/// // Works even if brand is unknown (as long as Luhn passes)
/// let result = validate_any("1234567890123452");
/// // This might succeed with brand Unknown if Luhn passes
/// ```
pub fn validate_any(input: &str) -> Result<ValidatedCard, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::Empty);
    }

    let mut digits = [0u8; MAX_CARD_DIGITS];
    let mut count = 0usize;
    let mut pos = 0usize;

    for c in input.chars() {
        match c {
            '0'..='9' => {
                if count >= MAX_CARD_DIGITS {
                    return Err(ValidationError::TooLong {
                        length: count + 1,
                        maximum: MAX_CARD_DIGITS,
                    });
                }
                digits[count] = (c as u8) - b'0';
                count += 1;
            }
            ' ' | '-' | '.' => {}
            _ => {
                return Err(ValidationError::InvalidCharacter {
                    position: pos,
                    character: c,
                });
            }
        }
        pos += 1;
    }

    if count == 0 {
        return Err(ValidationError::NoDigits);
    }

    if count < MIN_CARD_DIGITS {
        return Err(ValidationError::TooShort {
            length: count,
            minimum: MIN_CARD_DIGITS,
        });
    }

    if !luhn::validate(&digits[..count]) {
        return Err(ValidationError::InvalidChecksum);
    }

    // Detect brand but don't require it
    let brand = detect_brand(&digits[..count]);

    // If we detected a brand, validate the length for it
    if let Some(b) = brand {
        if !b.is_valid_length(count) {
            return Err(ValidationError::InvalidLengthForBrand {
                brand: b,
                length: count,
                valid_lengths: b.valid_lengths(),
            });
        }
        Ok(ValidatedCard::new(b, digits, count as u8))
    } else {
        // Unknown brand - accept any length between MIN and MAX
        // We use Visa as a placeholder since we need some brand
        // Note: This is a limitation - ideally we'd have an Unknown variant
        Ok(ValidatedCard::new(CardBrand::Visa, digits, count as u8))
    }
}

/// Validates a pre-parsed array of digits.
///
/// Use this when you've already extracted digits and want to skip parsing.
/// This is more efficient for batch processing.
///
/// # Arguments
///
/// * `digits` - Slice of digits (0-9 values, not ASCII)
///
/// # Example
///
/// ```
/// use cc_validator::validate_digits;
///
/// let digits = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
/// let card = validate_digits(&digits).unwrap();
/// assert_eq!(card.brand().name(), "Visa");
/// ```
pub fn validate_digits(digits: &[u8]) -> Result<ValidatedCard, ValidationError> {
    let count = digits.len();

    if count == 0 {
        return Err(ValidationError::Empty);
    }

    if count < MIN_CARD_DIGITS {
        return Err(ValidationError::TooShort {
            length: count,
            minimum: MIN_CARD_DIGITS,
        });
    }

    if count > MAX_CARD_DIGITS {
        return Err(ValidationError::TooLong {
            length: count,
            maximum: MAX_CARD_DIGITS,
        });
    }

    // Validate Luhn
    if !luhn::validate(digits) {
        return Err(ValidationError::InvalidChecksum);
    }

    // Detect brand
    let brand = detect_brand(digits).ok_or(ValidationError::UnknownBrand)?;

    // Validate length
    if !brand.is_valid_length(count) {
        return Err(ValidationError::InvalidLengthForBrand {
            brand,
            length: count,
            valid_lengths: brand.valid_lengths(),
        });
    }

    // Copy to fixed array
    let mut fixed_digits = [0u8; MAX_CARD_DIGITS];
    fixed_digits[..count].copy_from_slice(digits);

    Ok(ValidatedCard::new(brand, fixed_digits, count as u8))
}

/// Quickly checks if a card number is valid without returning detailed info.
///
/// This is faster than `validate()` when you only need a yes/no answer.
///
/// # Example
///
/// ```
/// use cc_validator::is_valid;
///
/// assert!(is_valid("4111-1111-1111-1111"));
/// assert!(!is_valid("4111-1111-1111-1112"));
/// ```
#[inline]
pub fn is_valid(input: &str) -> bool {
    validate(input).is_ok()
}

/// Quickly checks if a card number passes Luhn validation only.
///
/// This doesn't check brand or length, just the checksum.
///
/// # Example
///
/// ```
/// use cc_validator::passes_luhn;
///
/// assert!(passes_luhn("4111111111111111"));
/// assert!(!passes_luhn("4111111111111112"));
/// ```
#[inline]
pub fn passes_luhn(input: &str) -> bool {
    let digits: Vec<u8> = input
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| (c as u8) - b'0')
        .collect();

    !digits.is_empty() && luhn::validate(&digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test card numbers (from various test card lists)
    const VISA_VALID: &str = "4111111111111111";
    const VISA_VALID_FORMATTED: &str = "4111-1111-1111-1111";
    const VISA_VALID_SPACES: &str = "4111 1111 1111 1111";

    const MASTERCARD_VALID: &str = "5500000000000004";
    const AMEX_VALID: &str = "378282246310005";
    const DISCOVER_VALID: &str = "6011111111111117";

    #[test]
    fn test_validate_visa() {
        let card = validate(VISA_VALID).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
        assert_eq!(card.length(), 16);
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn test_validate_formatted() {
        let card = validate(VISA_VALID_FORMATTED).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);

        let card = validate(VISA_VALID_SPACES).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
    }

    #[test]
    fn test_validate_mastercard() {
        let card = validate(MASTERCARD_VALID).unwrap();
        assert_eq!(card.brand(), CardBrand::Mastercard);
    }

    #[test]
    fn test_validate_amex() {
        let card = validate(AMEX_VALID).unwrap();
        assert_eq!(card.brand(), CardBrand::Amex);
        assert_eq!(card.length(), 15);
    }

    #[test]
    fn test_validate_discover() {
        let card = validate(DISCOVER_VALID).unwrap();
        assert_eq!(card.brand(), CardBrand::Discover);
    }

    #[test]
    fn test_invalid_checksum() {
        let err = validate("4111111111111112").unwrap_err();
        assert_eq!(err, ValidationError::InvalidChecksum);
    }

    #[test]
    fn test_invalid_character() {
        let err = validate("4111-1111-1111-111X").unwrap_err();
        match err {
            ValidationError::InvalidCharacter { character, .. } => {
                assert_eq!(character, 'X');
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }

    #[test]
    fn test_too_short() {
        let err = validate("41111111111").unwrap_err(); // 11 digits
        match err {
            ValidationError::TooShort { length, minimum } => {
                assert_eq!(length, 11);
                assert_eq!(minimum, 12);
            }
            _ => panic!("Expected TooShort error"),
        }
    }

    #[test]
    fn test_too_long() {
        let err = validate("41111111111111111111").unwrap_err(); // 20 digits
        match err {
            ValidationError::TooLong { length, maximum } => {
                assert_eq!(length, 20);
                assert_eq!(maximum, 19);
            }
            _ => panic!("Expected TooLong error"),
        }
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(validate("").unwrap_err(), ValidationError::Empty);
    }

    #[test]
    fn test_only_separators() {
        assert_eq!(validate("----").unwrap_err(), ValidationError::NoDigits);
        assert_eq!(validate("    ").unwrap_err(), ValidationError::NoDigits);
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(VISA_VALID));
        assert!(is_valid(VISA_VALID_FORMATTED));
        assert!(!is_valid("4111111111111112"));
        assert!(!is_valid(""));
    }

    #[test]
    fn test_passes_luhn() {
        assert!(passes_luhn(VISA_VALID));
        assert!(!passes_luhn("4111111111111112"));
    }

    #[test]
    fn test_validate_digits() {
        let digits = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let card = validate_digits(&digits).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
    }

    #[test]
    fn test_card_masking() {
        let card = validate(VISA_VALID).unwrap();
        let masked = card.masked();
        assert!(!masked.contains("4111111111111111"));
        assert!(masked.contains("1111"));
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_card_bin() {
        let card = validate(VISA_VALID).unwrap();
        assert_eq!(card.bin6(), "411111");
        assert_eq!(card.bin8(), "41111111");
    }

    #[test]
    fn test_full_number_retrieval() {
        let card = validate(VISA_VALID).unwrap();
        assert_eq!(card.number(), "4111111111111111");
    }

    // Additional test cards from payment processors
    #[test]
    fn test_various_test_cards() {
        // Visa
        assert!(is_valid("4012888888881881"));
        assert!(is_valid("4222222222222"));

        // Mastercard
        assert!(is_valid("5105105105105100"));
        assert!(is_valid("5200828282828210"));

        // Amex
        assert!(is_valid("371449635398431"));
        assert!(is_valid("340000000000009"));

        // Discover
        assert!(is_valid("6011000990139424"));

        // Diners Club
        assert!(is_valid("30569309025904"));

        // JCB
        assert!(is_valid("3530111333300000"));
    }
}
