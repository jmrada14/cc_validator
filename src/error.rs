//! Rich error types for credit card validation.
//!
//! Provides detailed, actionable error messages that explain exactly why validation failed.

use crate::CardBrand;
use std::fmt;

/// Errors that can occur during credit card validation.
///
/// Each variant provides specific details about the validation failure,
/// enabling users to understand and fix the issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The input string was empty.
    Empty,

    /// The card number has too few digits.
    TooShort {
        /// The actual number of digits provided.
        length: usize,
        /// The minimum required digits (12).
        minimum: usize,
    },

    /// The card number has too many digits.
    TooLong {
        /// The actual number of digits provided.
        length: usize,
        /// The maximum allowed digits (19).
        maximum: usize,
    },

    /// An invalid character was found in the input.
    ///
    /// Only digits (0-9), spaces, and hyphens are allowed.
    InvalidCharacter {
        /// The position in the input string (0-indexed).
        position: usize,
        /// The invalid character that was found.
        character: char,
    },

    /// The Luhn checksum validation failed.
    ///
    /// This usually indicates a typo in the card number.
    InvalidChecksum,

    /// The card number length is invalid for the detected brand.
    InvalidLengthForBrand {
        /// The detected card brand.
        brand: CardBrand,
        /// The actual number of digits.
        length: usize,
        /// The valid lengths for this brand.
        valid_lengths: &'static [u8],
    },

    /// Could not detect a known card brand from the BIN/IIN.
    UnknownBrand,

    /// The card number contains only whitespace or separators.
    NoDigits,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "card number is empty"),

            Self::TooShort { length, minimum } => {
                write!(
                    f,
                    "card number too short: got {} digits, minimum is {}",
                    length, minimum
                )
            }

            Self::TooLong { length, maximum } => {
                write!(
                    f,
                    "card number too long: got {} digits, maximum is {}",
                    length, maximum
                )
            }

            Self::InvalidCharacter {
                position,
                character,
            } => {
                write!(
                    f,
                    "invalid character '{}' at position {} (only digits, spaces, and hyphens allowed)",
                    character.escape_default(),
                    position
                )
            }

            Self::InvalidChecksum => {
                write!(f, "invalid checksum (Luhn check failed) - please verify the card number")
            }

            Self::InvalidLengthForBrand {
                brand,
                length,
                valid_lengths,
            } => {
                let valid_str: Vec<String> =
                    valid_lengths.iter().map(|l| l.to_string()).collect();
                write!(
                    f,
                    "{:?} cards must have {} digits, got {}",
                    brand,
                    valid_str.join(" or "),
                    length
                )
            }

            Self::UnknownBrand => {
                write!(f, "unknown card brand - check the card number prefix")
            }

            Self::NoDigits => {
                write!(f, "card number contains no digits")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(ValidationError::Empty.to_string(), "card number is empty");

        assert_eq!(
            ValidationError::TooShort {
                length: 10,
                minimum: 12
            }
            .to_string(),
            "card number too short: got 10 digits, minimum is 12"
        );

        assert_eq!(
            ValidationError::InvalidCharacter {
                position: 5,
                character: 'x'
            }
            .to_string(),
            "invalid character 'x' at position 5 (only digits, spaces, and hyphens allowed)"
        );

        assert_eq!(
            ValidationError::InvalidChecksum.to_string(),
            "invalid checksum (Luhn check failed) - please verify the card number"
        );
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ValidationError>();
    }
}
