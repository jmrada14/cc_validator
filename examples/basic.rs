//! Basic credit card validation example.
//!
//! Run with: `cargo run --example basic`

use cc_validator::{cvv, is_valid, validate, CardBrand, ValidationError};

fn main() {
    println!("=== Basic Credit Card Validation ===\n");

    // Example 1: Validate a Visa card
    let visa_number = "4111-1111-1111-1111";
    println!("Validating: {}", visa_number);

    match validate(visa_number) {
        Ok(card) => {
            println!("  Valid: yes");
            println!("  Brand: {}", card.brand().name());
            println!("  Last Four: {}", card.last_four());
            println!("  BIN (6): {}", card.bin6());
            println!("  BIN (8): {}", card.bin8());
            println!("  Masked: {}", card.masked());
            println!("  Length: {} digits", card.length());
        }
        Err(e) => {
            println!("  Valid: no");
            println!("  Error: {}", e);
        }
    }
    println!();

    // Example 2: Quick boolean check
    let test_cards = [
        ("4111111111111111", "Visa"),
        ("5500000000000004", "Mastercard"),
        ("378282246310005", "Amex"),
        ("6011111111111117", "Discover"),
        ("4111111111111112", "Invalid (bad checksum)"),
    ];

    println!("Quick validation checks:");
    for (number, description) in test_cards {
        let valid = is_valid(number);
        println!(
            "  {} - {}: {}",
            number,
            description,
            if valid { "VALID" } else { "INVALID" }
        );
    }
    println!();

    // Example 3: Handling validation errors
    println!("Error handling examples:");

    let error_cases = [
        ("", "Empty input"),
        ("411111111", "Too short"),
        ("41111111111111111111", "Too long"),
        ("4111-1111-1111-111X", "Invalid character"),
        ("4111111111111112", "Invalid checksum"),
    ];

    for (number, description) in error_cases {
        match validate(number) {
            Ok(_) => println!("  {}: Unexpectedly valid", description),
            Err(e) => {
                let error_type = match e {
                    ValidationError::Empty => "Empty",
                    ValidationError::TooShort { .. } => "TooShort",
                    ValidationError::TooLong { .. } => "TooLong",
                    ValidationError::InvalidCharacter { .. } => "InvalidCharacter",
                    ValidationError::InvalidChecksum => "InvalidChecksum",
                    ValidationError::InvalidLengthForBrand { .. } => "InvalidLengthForBrand",
                    ValidationError::UnknownBrand => "UnknownBrand",
                    ValidationError::NoDigits => "NoDigits",
                };
                println!("  {}: {} - {}", description, error_type, e);
            }
        }
    }
    println!();

    // Example 4: All supported card brands
    println!("Supported card brands:");
    let brands = [
        CardBrand::Visa,
        CardBrand::Mastercard,
        CardBrand::Amex,
        CardBrand::Discover,
        CardBrand::DinersClub,
        CardBrand::Jcb,
        CardBrand::UnionPay,
        CardBrand::Maestro,
        CardBrand::Mir,
        CardBrand::RuPay,
        CardBrand::Verve,
        CardBrand::Elo,
        CardBrand::Troy,
        CardBrand::BcCard,
    ];

    for brand in brands {
        let lengths: Vec<String> = brand
            .valid_lengths()
            .iter()
            .map(|l| l.to_string())
            .collect();
        println!(
            "  {:12} - Lengths: {:15} CVV: {} digits",
            brand.name(),
            lengths.join(", "),
            cvv::cvv_length_for_brand(brand)
        );
    }
}
