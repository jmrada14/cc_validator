//! Test card generation example.
//!
//! Run with: `cargo run --example generate --features generate`

use cc_validator::{generate, is_valid, CardBrand};

fn main() {
    println!("=== Test Card Generation ===\n");

    // -------------------------------------------------------------------------
    // Generate cards for each brand
    // -------------------------------------------------------------------------
    println!("--- Generate Cards by Brand ---\n");

    let brands = [
        CardBrand::Visa,
        CardBrand::Mastercard,
        CardBrand::Amex,
        CardBrand::Discover,
        CardBrand::DinersClub,
        CardBrand::Jcb,
        CardBrand::UnionPay,
        CardBrand::Maestro,
    ];

    for brand in brands {
        let card = generate::generate_card(brand);
        let valid = is_valid(&card);
        println!(
            "  {:12}: {} (valid: {})",
            brand.name(),
            card,
            if valid { "yes" } else { "no" }
        );
    }
    println!();

    // -------------------------------------------------------------------------
    // Deterministic generation (for reproducible tests)
    // -------------------------------------------------------------------------
    println!("--- Deterministic Generation ---\n");

    println!("  Generating same card multiple times:");
    for i in 0..3 {
        let card = generate::generate_card_deterministic(CardBrand::Visa);
        println!("    Run {}: {}", i + 1, card);
    }
    println!("  (All cards are identical - deterministic)\n");

    // -------------------------------------------------------------------------
    // Generate with custom prefix
    // -------------------------------------------------------------------------
    println!("--- Custom Prefix Generation ---\n");

    let prefixes = [
        ("411111", 16, "Visa with specific BIN"),
        ("550000", 16, "Mastercard with specific BIN"),
        ("4", 16, "Visa (minimal prefix)"),
        ("37", 15, "Amex"),
    ];

    for (prefix, length, description) in prefixes {
        let card = generate::generate_card_with_prefix(prefix, length);
        let valid = is_valid(&card);
        println!("  {} (prefix: {}, length: {})", description, prefix, length);
        println!("    Generated: {}", card);
        println!("    Valid: {}", if valid { "yes" } else { "no" });
        println!(
            "    Starts with prefix: {}",
            if card.starts_with(prefix) {
                "yes"
            } else {
                "no"
            }
        );
        println!();
    }

    // -------------------------------------------------------------------------
    // Generate multiple cards
    // -------------------------------------------------------------------------
    println!("--- Batch Generation ---\n");

    println!("  Generating 5 Visa cards:");
    for i in 0..5 {
        let card = generate::generate_card(CardBrand::Visa);
        let formatted = cc_validator::format::format_card_number(&card);
        println!("    {}: {}", i + 1, formatted);
    }
    println!();

    // -------------------------------------------------------------------------
    // Verify all generated cards are valid
    // -------------------------------------------------------------------------
    println!("--- Validation Check ---\n");

    let test_count = 1000;
    let mut all_valid = true;

    for brand in brands {
        let mut valid_count = 0;
        for _ in 0..test_count {
            let card = generate::generate_card(brand);
            if is_valid(&card) {
                valid_count += 1;
            }
        }
        let success = valid_count == test_count;
        if !success {
            all_valid = false;
        }
        println!(
            "  {:12}: {}/{} valid ({})",
            brand.name(),
            valid_count,
            test_count,
            if success { "PASS" } else { "FAIL" }
        );
    }
    println!();

    if all_valid {
        println!("  All generated cards pass Luhn validation!");
    } else {
        println!("  WARNING: Some generated cards failed validation!");
    }
}
