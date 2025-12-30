//! CVV and expiry date validation example.
//!
//! Run with: `cargo run --example cvv_expiry`

use cc_validator::{cvv, expiry, CardBrand};

fn main() {
    println!("=== CVV and Expiry Date Validation ===\n");

    // -------------------------------------------------------------------------
    // CVV Validation
    // -------------------------------------------------------------------------
    println!("--- CVV Validation ---\n");

    // Basic CVV validation (accepts 3-4 digits)
    let cvv_examples = ["123", "1234", "12", "12345", "abc", ""];

    println!("Basic CVV validation:");
    for code in cvv_examples {
        match cvv::validate_cvv(code) {
            Ok(validated) => {
                println!("  '{}': Valid ({} digits)", code, validated.length());
            }
            Err(e) => {
                println!("  '{}': Invalid - {}", code, e);
            }
        }
    }
    println!();

    // Brand-specific CVV validation
    println!("Brand-specific CVV validation:");
    let brands_and_cvvs = [
        (CardBrand::Visa, "123", "3-digit for Visa"),
        (CardBrand::Visa, "1234", "4-digit for Visa"),
        (CardBrand::Amex, "1234", "4-digit for Amex"),
        (CardBrand::Amex, "123", "3-digit for Amex"),
        (CardBrand::Mastercard, "123", "3-digit for Mastercard"),
    ];

    for (brand, code, description) in brands_and_cvvs {
        match cvv::validate_cvv_for_brand(code, brand) {
            Ok(validated) => {
                println!("  {} - Valid ({} digits)", description, validated.length());
            }
            Err(e) => {
                println!("  {} - Invalid: {}", description, e);
            }
        }
    }
    println!();

    // Expected CVV lengths by brand
    println!("Expected CVV lengths by brand:");
    let brands = [
        CardBrand::Visa,
        CardBrand::Mastercard,
        CardBrand::Amex,
        CardBrand::Discover,
    ];
    for brand in brands {
        println!(
            "  {}: {} digits",
            brand.name(),
            cvv::cvv_length_for_brand(brand)
        );
    }
    println!();

    // -------------------------------------------------------------------------
    // Expiry Date Validation
    // -------------------------------------------------------------------------
    println!("--- Expiry Date Validation ---\n");

    // Various expiry date formats
    let expiry_formats = [
        "12/25",   // MM/YY
        "12/2025", // MM/YYYY
        "1225",    // MMYY
        "12-25",   // MM-YY
        "12 25",   // MM YY
        "01/30",   // Future date
        "12/20",   // Past date (will be rejected by validate_expiry)
    ];

    println!("Expiry date validation (rejects expired):");
    for date in expiry_formats {
        match expiry::validate_expiry(date) {
            Ok(exp) => {
                println!(
                    "  '{}': Valid - {}/{} (expires in {} months)",
                    date,
                    exp.month(),
                    exp.year(),
                    exp.months_until_expiry()
                );
            }
            Err(e) => {
                println!("  '{}': Invalid - {}", date, e);
            }
        }
    }
    println!();

    // Parse without expiry check (allows expired dates)
    println!("Parsing without expiry check:");
    let parse_examples = ["12/20", "01/19", "12/30"];
    for date in parse_examples {
        match expiry::parse_expiry(date) {
            Ok(exp) => {
                let status = if exp.is_expired() {
                    "EXPIRED".to_string()
                } else {
                    format!("valid for {} more months", exp.months_until_expiry())
                };
                println!("  '{}': {}/{} - {}", date, exp.month(), exp.year(), status);
            }
            Err(e) => {
                println!("  '{}': Parse error - {}", date, e);
            }
        }
    }
    println!();

    // Expiry formatting
    println!("Expiry date formatting:");
    if let Ok(exp) = expiry::parse_expiry("12/25") {
        println!("  Short format: {}", exp.format_short()); // "12/25"
        println!("  Long format: {}", exp.format_long()); // "December 2025"
    }
    println!();

    // Invalid expiry dates
    println!("Invalid expiry date examples:");
    let invalid_dates = ["13/25", "00/25", "12/99", "ab/cd", "", "1"];
    for date in invalid_dates {
        match expiry::parse_expiry(date) {
            Ok(_) => println!("  '{}': Unexpectedly valid", date),
            Err(e) => println!("  '{}': {}", date, e),
        }
    }
    println!();

    // Custom validation options
    println!("Custom validation options:");
    // Only accept cards expiring within 5 years
    let test_dates = ["12/26", "12/35"];
    for date in test_dates {
        match expiry::validate_expiry_with_options(date, true, Some(5)) {
            Ok(_exp) => println!("  '{}': Valid (within 5 years)", date),
            Err(e) => println!("  '{}': {} (max 5 years)", date, e),
        }
    }
}
