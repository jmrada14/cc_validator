//! Node.js bindings for cc_validator.
//!
//! This crate provides native Node.js bindings using napi-rs.
//!
//! # Installation
//!
//! ```bash
//! npm install cc-validator
//! ```
//!
//! # Usage
//!
//! ```javascript
//! const { validateCard, isValid, generateTestCard } = require('cc-validator');
//!
//! // Validate a card
//! const result = validateCard("4111-1111-1111-1111");
//! if (result.valid) {
//!     console.log(`Brand: ${result.brand}`);
//!     console.log(`Masked: ${result.masked}`);
//! }
//!
//! // Quick check
//! if (isValid("4111111111111111")) {
//!     console.log("Valid!");
//! }
//!
//! // Generate test cards
//! const card = generateTestCard("visa");
//! ```

use napi::bindgen_prelude::*;
use napi_derive::napi;

use cc_validator::{
    validate, is_valid as cc_is_valid, passes_luhn as cc_passes_luhn,
    CardBrand, detect, format, expiry, cvv, generate,
};

/// Result of card validation.
#[napi(object)]
pub struct ValidationResult {
    pub valid: bool,
    pub brand: Option<String>,
    pub last_four: Option<String>,
    pub masked: Option<String>,
    pub error: Option<String>,
}

/// Validates a credit card number.
///
/// Accepts card numbers with spaces, dashes, or no separators.
///
/// @param cardNumber - The card number to validate
/// @returns ValidationResult with card details or error
#[napi]
pub fn validate_card(card_number: String) -> ValidationResult {
    match validate(&card_number) {
        Ok(card) => ValidationResult {
            valid: true,
            brand: Some(card.brand().name().to_string()),
            last_four: Some(card.last_four().to_string()),
            masked: Some(card.masked()),
            error: None,
        },
        Err(e) => ValidationResult {
            valid: false,
            brand: None,
            last_four: None,
            masked: None,
            error: Some(e.to_string()),
        },
    }
}

/// Quick check if a card number is valid.
///
/// @param cardNumber - The card number to check
/// @returns true if valid
#[napi]
pub fn is_valid(card_number: String) -> bool {
    cc_is_valid(&card_number)
}

/// Checks if a card passes the Luhn algorithm.
///
/// @param cardNumber - The card number to check
/// @returns true if passes Luhn
#[napi]
pub fn passes_luhn(card_number: String) -> bool {
    cc_passes_luhn(&card_number)
}

/// Detects the card brand from a (partial) card number.
///
/// @param cardNumber - The card number or prefix
/// @returns Brand name or null
#[napi]
pub fn detect_brand(card_number: String) -> Option<String> {
    let digits: Vec<u8> = card_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    detect::detect_brand(&digits).map(|b| b.name().to_string())
}

/// Formats a card number with spaces.
///
/// @param cardNumber - Raw card number
/// @returns Formatted card number
#[napi]
pub fn format_card(card_number: String) -> String {
    format::format_card_number(&card_number)
}

/// Formats a card number with a custom separator.
///
/// @param cardNumber - Raw card number
/// @param separator - Separator to use
/// @returns Formatted card number
#[napi]
pub fn format_card_with_separator(card_number: String, separator: String) -> String {
    format::format_with_separator(&card_number, &separator)
}

/// Strips all formatting from a card number.
///
/// @param cardNumber - Formatted card number
/// @returns Raw digits only
#[napi]
pub fn strip_formatting(card_number: String) -> String {
    format::strip_formatting(&card_number)
}

/// Masks a card number (PCI-DSS compliant).
///
/// @param cardNumber - Card number to mask
/// @returns Masked card number
#[napi]
pub fn mask_card(card_number: String) -> Result<String> {
    match validate(&card_number) {
        Ok(card) => Ok(card.masked()),
        Err(e) => Err(Error::new(Status::InvalidArg, e.to_string())),
    }
}

/// Generates a valid test card number.
///
/// Supported brands: visa, mastercard, amex, discover, jcb, diners, unionpay, maestro
///
/// @param brand - Card brand name
/// @returns Valid card number
#[napi]
pub fn generate_test_card(brand: String) -> Result<String> {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => CardBrand::Visa,
        "mastercard" | "mc" => CardBrand::Mastercard,
        "amex" | "american express" => CardBrand::Amex,
        "discover" => CardBrand::Discover,
        "jcb" => CardBrand::Jcb,
        "diners" | "dinersclub" | "diners club" => CardBrand::DinersClub,
        "unionpay" | "union pay" => CardBrand::UnionPay,
        "maestro" => CardBrand::Maestro,
        "mir" => CardBrand::Mir,
        "rupay" => CardBrand::RuPay,
        "verve" => CardBrand::Verve,
        "elo" => CardBrand::Elo,
        "troy" => CardBrand::Troy,
        "bccard" | "bc card" => CardBrand::BcCard,
        _ => return Err(Error::new(Status::InvalidArg, format!("Unknown brand: {}", brand))),
    };

    Ok(generate::generate_card(card_brand))
}

/// Result of CVV validation.
#[napi(object)]
pub struct CvvResult {
    pub valid: bool,
    pub length: Option<u32>,
    pub error: Option<String>,
}

/// Validates a CVV/CVC code.
///
/// @param cvv - CVV code to validate
/// @returns CvvResult
#[napi]
pub fn validate_cvv(input: String) -> CvvResult {
    match cvv::validate_cvv(&input) {
        Ok(validated) => CvvResult {
            valid: true,
            length: Some(validated.length() as u32),
            error: None,
        },
        Err(e) => CvvResult {
            valid: false,
            length: None,
            error: Some(e.to_string()),
        },
    }
}

/// Validates a CVV for a specific card brand.
///
/// @param cvv - CVV code to validate
/// @param brand - Card brand name
/// @returns CvvResult
#[napi]
pub fn validate_cvv_for_brand(input: String, brand: String) -> CvvResult {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => CardBrand::Visa,
        "mastercard" | "mc" => CardBrand::Mastercard,
        "amex" | "american express" => CardBrand::Amex,
        "discover" => CardBrand::Discover,
        "jcb" => CardBrand::Jcb,
        "diners" | "dinersclub" => CardBrand::DinersClub,
        _ => {
            return CvvResult {
                valid: false,
                length: None,
                error: Some(format!("Unknown brand: {}", brand)),
            }
        }
    };

    match cvv::validate_cvv_for_brand(&input, card_brand) {
        Ok(validated) => CvvResult {
            valid: true,
            length: Some(validated.length() as u32),
            error: None,
        },
        Err(e) => CvvResult {
            valid: false,
            length: None,
            error: Some(e.to_string()),
        },
    }
}

/// Result of expiry validation.
#[napi(object)]
pub struct ExpiryResult {
    pub valid: bool,
    pub month: Option<u32>,
    pub year: Option<u32>,
    pub expired: Option<bool>,
    pub formatted: Option<String>,
    pub error: Option<String>,
}

/// Validates an expiry date.
///
/// Accepts formats: MM/YY, MM/YYYY, MM-YY, MMYY, MMYYYY
///
/// @param date - Expiry date string
/// @returns ExpiryResult
#[napi]
pub fn validate_expiry(date: String) -> ExpiryResult {
    match expiry::validate_expiry(&date) {
        Ok(exp) => ExpiryResult {
            valid: true,
            month: Some(exp.month() as u32),
            year: Some(exp.year() as u32),
            expired: Some(exp.is_expired()),
            formatted: Some(exp.format_short()),
            error: None,
        },
        Err(e) => ExpiryResult {
            valid: false,
            month: None,
            year: None,
            expired: None,
            formatted: None,
            error: Some(e.to_string()),
        },
    }
}

/// Parses an expiry date without checking if expired.
///
/// @param date - Expiry date string
/// @returns ExpiryResult
#[napi]
pub fn parse_expiry(date: String) -> ExpiryResult {
    match expiry::parse_expiry(&date) {
        Ok(exp) => ExpiryResult {
            valid: true,
            month: Some(exp.month() as u32),
            year: Some(exp.year() as u32),
            expired: Some(exp.is_expired()),
            formatted: Some(exp.format_short()),
            error: None,
        },
        Err(e) => ExpiryResult {
            valid: false,
            month: None,
            year: None,
            expired: None,
            formatted: None,
            error: Some(e.to_string()),
        },
    }
}

/// Batch validates multiple card numbers.
///
/// @param cardNumbers - Array of card numbers
/// @returns Array of ValidationResults
#[napi]
pub fn validate_batch(card_numbers: Vec<String>) -> Vec<ValidationResult> {
    card_numbers
        .into_iter()
        .map(|card| validate_card(card))
        .collect()
}

/// Gets the expected CVV length for a card brand.
///
/// @param brand - Card brand name
/// @returns CVV length (3 or 4)
#[napi]
pub fn cvv_length_for_brand(brand: String) -> Result<u32> {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => CardBrand::Visa,
        "mastercard" | "mc" => CardBrand::Mastercard,
        "amex" | "american express" => CardBrand::Amex,
        "discover" => CardBrand::Discover,
        "jcb" => CardBrand::Jcb,
        "diners" | "dinersclub" => CardBrand::DinersClub,
        _ => return Err(Error::new(Status::InvalidArg, format!("Unknown brand: {}", brand))),
    };

    Ok(cvv::cvv_length_for_brand(card_brand) as u32)
}

/// Gets valid card lengths for a brand.
///
/// @param brand - Card brand name
/// @returns Array of valid lengths
#[napi]
pub fn valid_lengths_for_brand(brand: String) -> Result<Vec<u32>> {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => CardBrand::Visa,
        "mastercard" | "mc" => CardBrand::Mastercard,
        "amex" | "american express" => CardBrand::Amex,
        "discover" => CardBrand::Discover,
        "jcb" => CardBrand::Jcb,
        "diners" | "dinersclub" => CardBrand::DinersClub,
        "unionpay" | "union pay" => CardBrand::UnionPay,
        "maestro" => CardBrand::Maestro,
        "mir" => CardBrand::Mir,
        "rupay" => CardBrand::RuPay,
        _ => return Err(Error::new(Status::InvalidArg, format!("Unknown brand: {}", brand))),
    };

    Ok(card_brand.valid_lengths().iter().map(|&l| l as u32).collect())
}
