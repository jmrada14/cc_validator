//! CVV/CVC/CID validation for credit cards.
//!
//! This module validates Card Verification Values (CVV/CVC/CID):
//!
//! - **CVV** (Card Verification Value) - Visa
//! - **CVC** (Card Verification Code) - Mastercard
//! - **CID** (Card Identification Number) - American Express, Discover
//!
//! # Length Requirements
//!
//! - American Express: 4 digits (printed on front)
//! - All other cards: 3 digits (printed on back)
//!
//! # Example
//!
//! ```
//! use cc_validator::cvv::{validate_cvv, validate_cvv_for_brand};
//! use cc_validator::CardBrand;
//!
//! // Generic validation (3 or 4 digits)
//! assert!(validate_cvv("123").is_ok());
//! assert!(validate_cvv("1234").is_ok());
//!
//! // Brand-specific validation
//! assert!(validate_cvv_for_brand("123", CardBrand::Visa).is_ok());
//! assert!(validate_cvv_for_brand("1234", CardBrand::Amex).is_ok());
//!
//! // Wrong length for brand
//! assert!(validate_cvv_for_brand("1234", CardBrand::Visa).is_err());
//! ```

use crate::CardBrand;
use std::fmt;

/// A validated CVV/CVC code.
#[derive(Clone)]
pub struct ValidatedCvv {
    /// The CVV digits.
    digits: [u8; 4],
    /// Number of digits (3 or 4).
    length: u8,
}

impl ValidatedCvv {
    /// Returns the CVV as a string.
    pub fn as_str(&self) -> String {
        self.digits[..self.length as usize]
            .iter()
            .map(|&d| (b'0' + d) as char)
            .collect()
    }

    /// Returns the number of digits.
    #[inline]
    pub const fn length(&self) -> usize {
        self.length as usize
    }

    /// Returns true if this is a 4-digit CVV (Amex style).
    #[inline]
    pub const fn is_four_digit(&self) -> bool {
        self.length == 4
    }

    /// Returns the CVV digits as a slice.
    pub fn digits(&self) -> &[u8] {
        &self.digits[..self.length as usize]
    }
}

impl fmt::Debug for ValidatedCvv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask CVV in debug output for security
        f.debug_struct("ValidatedCvv")
            .field("value", &"***")
            .field("length", &self.length)
            .finish()
    }
}

impl fmt::Display for ValidatedCvv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask CVV in display for security
        write!(f, "{}", "*".repeat(self.length as usize))
    }
}

impl Drop for ValidatedCvv {
    fn drop(&mut self) {
        // Zero out the CVV on drop
        self.digits = [0; 4];
    }
}

/// Errors that can occur during CVV validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CvvError {
    /// The input is empty.
    Empty,
    /// The CVV contains non-digit characters.
    InvalidCharacter {
        /// The invalid character found.
        character: char,
        /// Position of the invalid character.
        position: usize,
    },
    /// The CVV has an invalid length.
    InvalidLength {
        /// Actual length provided.
        length: usize,
        /// Expected length(s).
        expected: &'static str,
    },
    /// The CVV length doesn't match the card brand requirements.
    WrongLengthForBrand {
        /// The card brand.
        brand: CardBrand,
        /// Actual length provided.
        length: usize,
        /// Expected length for this brand.
        expected: usize,
    },
}

impl fmt::Display for CvvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "CVV is empty"),
            Self::InvalidCharacter { character, position } => {
                write!(f, "invalid character '{}' at position {}", character, position)
            }
            Self::InvalidLength { length, expected } => {
                write!(f, "CVV must be {} digits, got {}", expected, length)
            }
            Self::WrongLengthForBrand {
                brand,
                length,
                expected,
            } => {
                write!(
                    f,
                    "{} cards require {} digit CVV, got {}",
                    brand.name(),
                    expected,
                    length
                )
            }
        }
    }
}

impl std::error::Error for CvvError {}

/// Returns the expected CVV length for a card brand.
///
/// - American Express: 4 digits
/// - All other brands: 3 digits
#[inline]
pub const fn cvv_length_for_brand(brand: CardBrand) -> usize {
    match brand {
        CardBrand::Amex => 4,
        _ => 3,
    }
}

/// Validates a CVV string (accepts 3 or 4 digits).
///
/// This is a generic validation that accepts any valid CVV length.
/// Use `validate_cvv_for_brand` for brand-specific validation.
///
/// # Example
///
/// ```
/// use cc_validator::cvv::validate_cvv;
///
/// let cvv = validate_cvv("123").unwrap();
/// assert_eq!(cvv.length(), 3);
///
/// let cvv = validate_cvv("1234").unwrap();
/// assert_eq!(cvv.length(), 4);
/// ```
pub fn validate_cvv(input: &str) -> Result<ValidatedCvv, CvvError> {
    if input.is_empty() {
        return Err(CvvError::Empty);
    }

    let mut digits = [0u8; 4];
    let mut count = 0;

    for (pos, c) in input.chars().enumerate() {
        if !c.is_ascii_digit() {
            return Err(CvvError::InvalidCharacter {
                character: c,
                position: pos,
            });
        }
        if count >= 4 {
            return Err(CvvError::InvalidLength {
                length: input.len(),
                expected: "3 or 4",
            });
        }
        digits[count] = (c as u8) - b'0';
        count += 1;
    }

    if count < 3 {
        return Err(CvvError::InvalidLength {
            length: count,
            expected: "3 or 4",
        });
    }

    Ok(ValidatedCvv {
        digits,
        length: count as u8,
    })
}

/// Validates a CVV string for a specific card brand.
///
/// Amex cards require 4-digit CVV, all others require 3 digits.
///
/// # Example
///
/// ```
/// use cc_validator::cvv::validate_cvv_for_brand;
/// use cc_validator::CardBrand;
///
/// // Visa requires 3 digits
/// assert!(validate_cvv_for_brand("123", CardBrand::Visa).is_ok());
/// assert!(validate_cvv_for_brand("1234", CardBrand::Visa).is_err());
///
/// // Amex requires 4 digits
/// assert!(validate_cvv_for_brand("1234", CardBrand::Amex).is_ok());
/// assert!(validate_cvv_for_brand("123", CardBrand::Amex).is_err());
/// ```
pub fn validate_cvv_for_brand(input: &str, brand: CardBrand) -> Result<ValidatedCvv, CvvError> {
    let cvv = validate_cvv(input)?;
    let expected = cvv_length_for_brand(brand);

    if cvv.length() != expected {
        return Err(CvvError::WrongLengthForBrand {
            brand,
            length: cvv.length(),
            expected,
        });
    }

    Ok(cvv)
}

/// Checks if a string is a valid CVV (3 or 4 digits).
#[inline]
pub fn is_valid_cvv(input: &str) -> bool {
    validate_cvv(input).is_ok()
}

/// Checks if a string is a valid CVV for a specific card brand.
#[inline]
pub fn is_valid_cvv_for_brand(input: &str, brand: CardBrand) -> bool {
    validate_cvv_for_brand(input, brand).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_3_digit_cvv() {
        let cvv = validate_cvv("123").unwrap();
        assert_eq!(cvv.length(), 3);
        assert_eq!(cvv.as_str(), "123");
        assert!(!cvv.is_four_digit());
    }

    #[test]
    fn test_validate_4_digit_cvv() {
        let cvv = validate_cvv("1234").unwrap();
        assert_eq!(cvv.length(), 4);
        assert_eq!(cvv.as_str(), "1234");
        assert!(cvv.is_four_digit());
    }

    #[test]
    fn test_cvv_with_leading_zeros() {
        let cvv = validate_cvv("007").unwrap();
        assert_eq!(cvv.as_str(), "007");

        let cvv = validate_cvv("0001").unwrap();
        assert_eq!(cvv.as_str(), "0001");
    }

    #[test]
    fn test_invalid_cvv_empty() {
        assert!(matches!(validate_cvv(""), Err(CvvError::Empty)));
    }

    #[test]
    fn test_invalid_cvv_too_short() {
        let result = validate_cvv("12");
        assert!(matches!(
            result,
            Err(CvvError::InvalidLength { length: 2, .. })
        ));
    }

    #[test]
    fn test_invalid_cvv_too_long() {
        let result = validate_cvv("12345");
        assert!(matches!(result, Err(CvvError::InvalidLength { .. })));
    }

    #[test]
    fn test_invalid_cvv_non_digit() {
        let result = validate_cvv("12a");
        assert!(matches!(
            result,
            Err(CvvError::InvalidCharacter {
                character: 'a',
                position: 2
            })
        ));
    }

    #[test]
    fn test_cvv_for_visa() {
        assert!(validate_cvv_for_brand("123", CardBrand::Visa).is_ok());
        assert!(validate_cvv_for_brand("1234", CardBrand::Visa).is_err());
    }

    #[test]
    fn test_cvv_for_mastercard() {
        assert!(validate_cvv_for_brand("123", CardBrand::Mastercard).is_ok());
        assert!(validate_cvv_for_brand("1234", CardBrand::Mastercard).is_err());
    }

    #[test]
    fn test_cvv_for_amex() {
        assert!(validate_cvv_for_brand("1234", CardBrand::Amex).is_ok());
        assert!(validate_cvv_for_brand("123", CardBrand::Amex).is_err());
    }

    #[test]
    fn test_cvv_for_discover() {
        assert!(validate_cvv_for_brand("123", CardBrand::Discover).is_ok());
        assert!(validate_cvv_for_brand("1234", CardBrand::Discover).is_err());
    }

    #[test]
    fn test_cvv_length_for_brand() {
        assert_eq!(cvv_length_for_brand(CardBrand::Amex), 4);
        assert_eq!(cvv_length_for_brand(CardBrand::Visa), 3);
        assert_eq!(cvv_length_for_brand(CardBrand::Mastercard), 3);
        assert_eq!(cvv_length_for_brand(CardBrand::Discover), 3);
    }

    #[test]
    fn test_is_valid_cvv() {
        assert!(is_valid_cvv("123"));
        assert!(is_valid_cvv("1234"));
        assert!(!is_valid_cvv("12"));
        assert!(!is_valid_cvv("12345"));
        assert!(!is_valid_cvv("abc"));
    }

    #[test]
    fn test_cvv_debug_is_masked() {
        let cvv = validate_cvv("123").unwrap();
        let debug = format!("{:?}", cvv);
        assert!(!debug.contains("123"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn test_cvv_display_is_masked() {
        let cvv = validate_cvv("123").unwrap();
        let display = format!("{}", cvv);
        assert_eq!(display, "***");

        let cvv = validate_cvv("1234").unwrap();
        let display = format!("{}", cvv);
        assert_eq!(display, "****");
    }

    #[test]
    fn test_cvv_error_display() {
        let err = CvvError::Empty;
        assert!(err.to_string().contains("empty"));

        let err = CvvError::WrongLengthForBrand {
            brand: CardBrand::Visa,
            length: 4,
            expected: 3,
        };
        assert!(err.to_string().contains("Visa"));
        assert!(err.to_string().contains("3"));
    }
}
