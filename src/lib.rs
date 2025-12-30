//! # cc_validator
//!
//! Credit card validation library for Rust. Work in progress.
//!
//! ## Features
//!
//! - Luhn algorithm validation
//! - Card brand detection (14 brands)
//! - CVV and expiry date validation
//! - Card number masking for display
//! - Multiple interfaces: library, CLI, REST API, WASM, Node.js
//!
//! ## Quick Start
//!
//! ```rust
//! use cc_validator::{validate, is_valid, CardBrand};
//!
//! // Validate a card number
//! let card = validate("4111-1111-1111-1111").unwrap();
//! assert_eq!(card.brand(), CardBrand::Visa);
//! assert_eq!(card.last_four(), "1111");
//!
//! // Safe for logging - never exposes full card number
//! println!("Card: {}", card.masked()); // "****-****-****-1111"
//!
//! // Quick boolean check
//! assert!(is_valid("4111111111111111"));
//! assert!(!is_valid("4111111111111112"));
//! ```
//!
//! ## CVV Validation
//!
//! ```rust
//! use cc_validator::{cvv, CardBrand};
//!
//! // Validate any CVV (3-4 digits)
//! let validated = cvv::validate_cvv("123").unwrap();
//! assert_eq!(validated.length(), 3);
//!
//! // Brand-specific (Amex requires 4 digits)
//! let validated = cvv::validate_cvv_for_brand("1234", CardBrand::Amex).unwrap();
//! assert!(cvv::validate_cvv_for_brand("123", CardBrand::Amex).is_err());
//! ```
//!
//! ## Expiry Validation
//!
//! ```rust
//! use cc_validator::expiry;
//!
//! // Validate expiry (rejects expired cards)
//! let exp = expiry::validate_expiry("12/30").unwrap();
//! assert_eq!(exp.month(), 12);
//! assert_eq!(exp.year(), 2030);
//! assert!(!exp.is_expired());
//!
//! // Parse without expiry check
//! let exp = expiry::parse_expiry("01/20").unwrap();
//! assert!(exp.is_expired());
//! ```
//!
//! ## Card Formatting
//!
//! ```rust
//! use cc_validator::format;
//!
//! // Auto-format based on brand
//! let formatted = format::format_card_number("4111111111111111");
//! assert_eq!(formatted, "4111 1111 1111 1111");
//!
//! // Amex uses 4-6-5 grouping
//! let formatted = format::format_card_number("378282246310005");
//! assert_eq!(formatted, "3782 822463 10005");
//!
//! // Custom separator
//! let formatted = format::format_with_separator("4111111111111111", "-");
//! assert_eq!(formatted, "4111-1111-1111-1111");
//!
//! // Strip formatting
//! let digits = format::strip_formatting("4111-1111-1111-1111");
//! assert_eq!(digits, "4111111111111111");
//! ```
//!
//! ## Test Card Generation
//!
//! ```rust
//! use cc_validator::{generate, is_valid, CardBrand};
//!
//! // Generate valid test card (deterministic, no randomness)
//! let card = generate::generate_card_deterministic(CardBrand::Visa);
//! assert!(is_valid(&card));
//! assert!(card.starts_with("4"));
//! ```
//!
//! ## Batch Processing
//!
//! ```rust
//! use cc_validator::{BatchValidator, batch};
//!
//! let mut batch_validator = BatchValidator::new();
//! let cards = vec!["4111111111111111", "5500000000000004", "invalid"];
//!
//! // Get all results
//! let results = batch_validator.validate_all(&cards);
//! assert_eq!(results.len(), 3);
//!
//! // Get only valid cards
//! let valid = batch_validator.validate_valid_only(&cards);
//! assert_eq!(valid.len(), 2);
//!
//! // Count valid
//! let (valid_count, _) = batch::count_valid(&cards);
//! assert_eq!(valid_count, 2);
//! ```
//!
//! ## Streaming Validation
//!
//! ```rust
//! use cc_validator::stream::ValidateExt;
//!
//! let cards = vec!["4111111111111111", "invalid", "5500000000000004"];
//! let valid_cards: Vec<_> = cards.iter()
//!     .copied()
//!     .validate_valid_only()
//!     .collect();
//!
//! assert_eq!(valid_cards.len(), 2);
//! ```
//!
//! ## Supported Card Brands
//!
//! | Brand | Prefix | Length | CVV |
//! |-------|--------|--------|-----|
//! | Visa | 4 | 13, 16, 19 | 3 |
//! | Mastercard | 51-55, 2221-2720 | 16 | 3 |
//! | American Express | 34, 37 | 15 | 4 |
//! | Discover | 6011, 644-649, 65 | 16-19 | 3 |
//! | Diners Club | 36, 38, 300-305 | 14-19 | 3 |
//! | JCB | 3528-3589 | 16-19 | 3 |
//! | UnionPay | 62 | 16-19 | 3 |
//! | Maestro | 50, 56-69 | 12-19 | 3 |
//! | Mir | 2200-2204 | 16-19 | 3 |
//! | RuPay | 60, 65, 81, 82 | 16 | 3 |
//! | Verve | 506, 507 | 16-19 | 3 |
//! | Elo | 509, 636 | 16 | 3 |
//! | Troy | 9792 | 16 | 3 |
//! | BC Card | 94 | 16 | 3 |
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `generate` | Test card generation |
//! | `cli` | Command-line tool |
//! | `server` | REST API with Swagger UI |
//! | `wasm` | WebAssembly support |
//! | `parallel` | Rayon-based parallelism |
//! | `simd` | SIMD Luhn (nightly only) |
//! | `bin-json` | JSON BIN database loader |
//! | `bin-csv` | CSV BIN database loader |
//! | `bin-sqlite` | SQLite BIN database |
//!
//! ## Security
//!
//! This library is designed with PCI-DSS compliance in mind:
//!
//! - Card numbers stored in fixed-size arrays, not heap strings
//! - Automatic memory zeroization when `ValidatedCard` is dropped
//! - `Debug` and `Display` show masked numbers only
//! - Constant-time comparison for sensitive operations
//! - No unsafe code (`#![deny(unsafe_code)]`)

#![cfg_attr(feature = "simd", feature(portable_simd))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod batch;
#[path = "bindb/mod.rs"]
pub mod bin;
pub mod card;
pub mod cvv;
pub mod detect;
pub mod error;
pub mod expiry;
pub mod format;
pub mod generate;
pub mod luhn;
pub mod mask;
pub mod simd;
pub mod stream;
pub mod validate;

#[cfg(feature = "wasm")]
mod wasm;

// Re-export main types at crate root
pub use batch::BatchValidator;
pub use card::{CardBrand, ValidatedCard, MAX_CARD_DIGITS, MIN_CARD_DIGITS};
pub use error::ValidationError;
pub use validate::{is_valid, passes_luhn, validate, validate_any, validate_digits};

// Re-export mask utilities
pub use mask::{constant_time_eq, constant_time_eq_str, mask_string};

#[cfg(test)]
mod tests {
    use super::*;

    // Standard test card numbers from payment processors
    const VISA_16: &str = "4111111111111111";
    const VISA_13: &str = "4222222222222";
    const MASTERCARD: &str = "5500000000000004";
    const MASTERCARD_2: &str = "5105105105105100";
    const AMEX: &str = "378282246310005";
    const AMEX_2: &str = "371449635398431";
    const DISCOVER: &str = "6011111111111117";
    const DINERS: &str = "30569309025904";
    const JCB: &str = "3530111333300000";

    #[test]
    fn test_visa_validation() {
        let card = validate(VISA_16).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
        assert_eq!(card.length(), 16);
        assert_eq!(card.last_four(), "1111");

        let card = validate(VISA_13).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
        assert_eq!(card.length(), 13);
    }

    #[test]
    fn test_mastercard_validation() {
        let card = validate(MASTERCARD).unwrap();
        assert_eq!(card.brand(), CardBrand::Mastercard);

        let card = validate(MASTERCARD_2).unwrap();
        assert_eq!(card.brand(), CardBrand::Mastercard);
    }

    #[test]
    fn test_amex_validation() {
        let card = validate(AMEX).unwrap();
        assert_eq!(card.brand(), CardBrand::Amex);
        assert_eq!(card.length(), 15);

        let card = validate(AMEX_2).unwrap();
        assert_eq!(card.brand(), CardBrand::Amex);
    }

    #[test]
    fn test_discover_validation() {
        let card = validate(DISCOVER).unwrap();
        assert_eq!(card.brand(), CardBrand::Discover);
    }

    #[test]
    fn test_diners_validation() {
        let card = validate(DINERS).unwrap();
        assert_eq!(card.brand(), CardBrand::DinersClub);
    }

    #[test]
    fn test_jcb_validation() {
        let card = validate(JCB).unwrap();
        assert_eq!(card.brand(), CardBrand::Jcb);
    }

    #[test]
    fn test_formatted_input() {
        // With dashes
        let card = validate("4111-1111-1111-1111").unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);

        // With spaces
        let card = validate("4111 1111 1111 1111").unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);

        // Mixed
        let card = validate("4111-1111 1111-1111").unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
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
            _ => panic!("Expected InvalidCharacter"),
        }
    }

    #[test]
    fn test_too_short() {
        let err = validate("41111111111").unwrap_err();
        match err {
            ValidationError::TooShort { length, minimum } => {
                assert_eq!(length, 11);
                assert_eq!(minimum, MIN_CARD_DIGITS);
            }
            _ => panic!("Expected TooShort"),
        }
    }

    #[test]
    fn test_too_long() {
        let err = validate("41111111111111111111").unwrap_err();
        match err {
            ValidationError::TooLong { length, maximum } => {
                assert_eq!(length, 20);
                assert_eq!(maximum, MAX_CARD_DIGITS);
            }
            _ => panic!("Expected TooLong"),
        }
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(VISA_16));
        assert!(is_valid(MASTERCARD));
        assert!(is_valid(AMEX));
        assert!(!is_valid("4111111111111112"));
        assert!(!is_valid(""));
    }

    #[test]
    fn test_passes_luhn() {
        assert!(passes_luhn(VISA_16));
        assert!(!passes_luhn("4111111111111112"));
    }

    #[test]
    fn test_masking() {
        let card = validate(VISA_16).unwrap();
        let masked = card.masked();

        // Should not contain full number
        assert!(!masked.contains("4111111111111111"));
        // Should contain last 4
        assert!(masked.contains("1111"));
        // Should contain mask characters
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_bin_retrieval() {
        let card = validate(VISA_16).unwrap();
        assert_eq!(card.bin6(), "411111");
        assert_eq!(card.bin8(), "41111111");
    }

    #[test]
    fn test_display() {
        let card = validate(VISA_16).unwrap();
        let display = format!("{}", card);

        // Should show brand and masked number
        assert!(display.contains("Visa"));
        assert!(display.contains("*"));
        assert!(!display.contains("4111111111111111"));
    }

    #[test]
    fn test_debug_is_safe() {
        let card = validate(VISA_16).unwrap();
        let debug = format!("{:?}", card);

        // Debug should not expose full card number
        assert!(!debug.contains("4111111111111111"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_mask_string() {
        let masked = mask_string("4111111111111111");
        assert!(!masked.contains("4111111111111111"));
        assert!(masked.contains("1111"));
    }

    #[test]
    fn test_validate_digits() {
        let digits = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let card = validate_digits(&digits).unwrap();
        assert_eq!(card.brand(), CardBrand::Visa);
    }

    #[test]
    fn test_thread_safety() {
        // Ensure types are Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ValidatedCard>();
        assert_send_sync::<ValidationError>();
        assert_send_sync::<CardBrand>();
        assert_send_sync::<BatchValidator>();
    }
}
