//! WebAssembly bindings for credit card validation.
//!
//! This module provides JavaScript-friendly bindings for the cc_validator library.
//!
//! # Usage from JavaScript
//!
//! ```javascript
//! import init, { validate_card, is_valid, generate_test_card } from 'cc_validator';
//!
//! await init();
//!
//! // Validate a card
//! const result = validate_card("4111-1111-1111-1111");
//! if (result.valid) {
//!     console.log(`Brand: ${result.brand}`);
//!     console.log(`Masked: ${result.masked}`);
//! } else {
//!     console.log(`Error: ${result.error}`);
//! }
//!
//! // Quick validation check
//! if (is_valid("4111111111111111")) {
//!     console.log("Card is valid!");
//! }
//!
//! // Generate test cards
//! const testCard = generate_test_card("visa");
//! ```

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

/// Result of card validation, returned to JavaScript.
#[wasm_bindgen]
pub struct ValidationResult {
    valid: bool,
    brand: Option<String>,
    last_four: Option<String>,
    masked: Option<String>,
    error: Option<String>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn brand(&self) -> Option<String> {
        self.brand.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn last_four(&self) -> Option<String> {
        self.last_four.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn masked(&self) -> Option<String> {
        self.masked.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// Validates a credit card number and returns detailed information.
///
/// Accepts card numbers with spaces, dashes, or no separators.
///
/// # Example
/// ```javascript
/// const result = validate_card("4111-1111-1111-1111");
/// console.log(result.valid);  // true
/// console.log(result.brand);  // "Visa"
/// ```
#[wasm_bindgen]
pub fn validate_card(card_number: &str) -> ValidationResult {
    match crate::validate(card_number) {
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
/// # Example
/// ```javascript
/// if (is_valid("4111111111111111")) {
///     console.log("Valid!");
/// }
/// ```
#[wasm_bindgen]
pub fn is_valid(card_number: &str) -> bool {
    crate::is_valid(card_number)
}

/// Checks if a card number passes the Luhn algorithm.
#[wasm_bindgen]
pub fn passes_luhn(card_number: &str) -> bool {
    crate::passes_luhn(card_number)
}

/// Detects the card brand from a (partial) card number.
///
/// # Example
/// ```javascript
/// const brand = detect_brand("4111");  // "Visa"
/// ```
#[wasm_bindgen]
pub fn detect_brand(card_number: &str) -> Option<String> {
    let digits: Vec<u8> = card_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    crate::detect::detect_brand(&digits).map(|b| b.name().to_string())
}

/// Formats a card number with spaces.
///
/// # Example
/// ```javascript
/// format_card("4111111111111111")  // "4111 1111 1111 1111"
/// ```
#[wasm_bindgen]
pub fn format_card(card_number: &str) -> String {
    crate::format::format_card_number(card_number)
}

/// Formats a card number with a custom separator.
#[wasm_bindgen]
pub fn format_card_with_separator(card_number: &str, separator: &str) -> String {
    crate::format::format_with_separator(card_number, separator)
}

/// Strips all formatting (spaces, dashes) from a card number.
#[wasm_bindgen]
pub fn strip_formatting(card_number: &str) -> String {
    crate::format::strip_formatting(card_number)
}

/// Masks a card number, showing only the last 4 digits.
///
/// # Example
/// ```javascript
/// mask_card("4111111111111111")  // "****-****-****-1111"
/// ```
#[wasm_bindgen]
pub fn mask_card(card_number: &str) -> Result<String, JsValue> {
    match crate::validate(card_number) {
        Ok(card) => Ok(card.masked()),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Generates a valid test card number for the given brand.
///
/// Supported brands: visa, mastercard, amex, discover, jcb, diners, unionpay, maestro
///
/// # Example
/// ```javascript
/// const card = generate_test_card("visa");
/// console.log(card);  // "4539578763621323" (example)
/// ```
#[wasm_bindgen]
pub fn generate_test_card(brand: &str) -> Result<String, JsValue> {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => crate::CardBrand::Visa,
        "mastercard" | "mc" => crate::CardBrand::Mastercard,
        "amex" | "american express" => crate::CardBrand::Amex,
        "discover" => crate::CardBrand::Discover,
        "jcb" => crate::CardBrand::Jcb,
        "diners" | "dinersclub" | "diners club" => crate::CardBrand::DinersClub,
        "unionpay" | "union pay" => crate::CardBrand::UnionPay,
        "maestro" => crate::CardBrand::Maestro,
        "mir" => crate::CardBrand::Mir,
        "rupay" => crate::CardBrand::RuPay,
        "verve" => crate::CardBrand::Verve,
        "elo" => crate::CardBrand::Elo,
        "troy" => crate::CardBrand::Troy,
        "bccard" | "bc card" => crate::CardBrand::BcCard,
        _ => return Err(JsValue::from_str(&format!("Unknown brand: {}", brand))),
    };

    Ok(crate::generate::generate_card_deterministic(card_brand))
}

/// Result of CVV validation.
#[wasm_bindgen]
pub struct CvvResult {
    valid: bool,
    length: Option<u8>,
    error: Option<String>,
}

#[wasm_bindgen]
impl CvvResult {
    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn length(&self) -> Option<u8> {
        self.length
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// Validates a CVV/CVC code.
///
/// # Example
/// ```javascript
/// const result = validate_cvv("123");
/// console.log(result.valid);  // true
/// ```
#[wasm_bindgen]
pub fn validate_cvv(cvv: &str) -> CvvResult {
    match crate::cvv::validate_cvv(cvv) {
        Ok(validated) => CvvResult {
            valid: true,
            length: Some(validated.length() as u8),
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
/// # Example
/// ```javascript
/// const result = validate_cvv_for_brand("1234", "amex");
/// console.log(result.valid);  // true (Amex uses 4-digit CVV)
/// ```
#[wasm_bindgen]
pub fn validate_cvv_for_brand(cvv: &str, brand: &str) -> CvvResult {
    let card_brand = match brand.to_lowercase().as_str() {
        "visa" => crate::CardBrand::Visa,
        "mastercard" | "mc" => crate::CardBrand::Mastercard,
        "amex" | "american express" => crate::CardBrand::Amex,
        "discover" => crate::CardBrand::Discover,
        "jcb" => crate::CardBrand::Jcb,
        "diners" | "dinersclub" => crate::CardBrand::DinersClub,
        _ => {
            return CvvResult {
                valid: false,
                length: None,
                error: Some(format!("Unknown brand: {}", brand)),
            }
        }
    };

    match crate::cvv::validate_cvv_for_brand(cvv, card_brand) {
        Ok(validated) => CvvResult {
            valid: true,
            length: Some(validated.length() as u8),
            error: None,
        },
        Err(e) => CvvResult {
            valid: false,
            length: None,
            error: Some(e.to_string()),
        },
    }
}

/// Result of expiry date validation.
#[wasm_bindgen]
pub struct ExpiryResult {
    valid: bool,
    month: Option<u8>,
    year: Option<u16>,
    expired: Option<bool>,
    formatted: Option<String>,
    error: Option<String>,
}

#[wasm_bindgen]
impl ExpiryResult {
    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn month(&self) -> Option<u8> {
        self.month
    }

    #[wasm_bindgen(getter)]
    pub fn year(&self) -> Option<u16> {
        self.year
    }

    #[wasm_bindgen(getter)]
    pub fn expired(&self) -> Option<bool> {
        self.expired
    }

    #[wasm_bindgen(getter)]
    pub fn formatted(&self) -> Option<String> {
        self.formatted.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// Validates an expiry date.
///
/// Accepts formats: MM/YY, MM/YYYY, MM-YY, MMYY, MMYYYY
///
/// # Example
/// ```javascript
/// const result = validate_expiry("12/25");
/// console.log(result.valid);      // true
/// console.log(result.expired);    // false
/// console.log(result.formatted);  // "12/25"
/// ```
#[wasm_bindgen]
pub fn validate_expiry(date: &str) -> ExpiryResult {
    match crate::expiry::validate_expiry(date) {
        Ok(exp) => ExpiryResult {
            valid: true,
            month: Some(exp.month()),
            year: Some(exp.year()),
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

/// Parses an expiry date without checking if it's expired.
#[wasm_bindgen]
pub fn parse_expiry(date: &str) -> ExpiryResult {
    match crate::expiry::parse_expiry(date) {
        Ok(exp) => ExpiryResult {
            valid: true,
            month: Some(exp.month()),
            year: Some(exp.year()),
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
/// Returns an array of validation results.
///
/// # Example
/// ```javascript
/// const results = validate_batch(["4111111111111111", "5500000000000004"]);
/// results.forEach(r => console.log(r.valid));
/// ```
#[wasm_bindgen]
pub fn validate_batch(card_numbers: js_sys::Array) -> js_sys::Array {
    let results = js_sys::Array::new();

    for card in card_numbers.iter() {
        if let Some(card_str) = card.as_string() {
            results.push(&JsValue::from(validate_card(&card_str)));
        }
    }

    results
}
