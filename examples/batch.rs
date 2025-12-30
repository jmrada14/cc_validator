//! Batch processing example.
//!
//! Run with: `cargo run --example batch`

use cc_validator::{batch, stream::ValidateExt, BatchValidator};

fn main() {
    println!("=== Batch Credit Card Validation ===\n");

    // Sample card numbers (mix of valid and invalid)
    let cards = vec![
        "4111111111111111", // Valid Visa
        "5500000000000004", // Valid Mastercard
        "378282246310005",  // Valid Amex
        "6011111111111117", // Valid Discover
        "4111111111111112", // Invalid (bad checksum)
        "invalid",          // Invalid (not a number)
        "30569309025904",   // Valid Diners Club
        "3530111333300000", // Valid JCB
    ];

    // Method 1: BatchValidator
    println!("Using BatchValidator:");
    let mut batch = BatchValidator::new();

    // Get all results
    let results = batch.validate_all(&cards);
    println!("  Total cards: {}", results.len());
    println!(
        "  Valid: {}",
        results.iter().filter(|r| r.is_ok()).count()
    );
    println!(
        "  Invalid: {}",
        results.iter().filter(|r| r.is_err()).count()
    );
    println!();

    // Get only valid cards
    let valid_cards = batch.validate_valid_only(&cards);
    println!("  Valid cards:");
    for card in &valid_cards {
        println!("    {} - {}", card.masked(), card.brand().name());
    }
    println!();

    // Count valid (efficient - stops early on failures)
    let (count, _) = batch::count_valid(&cards);
    println!("  Quick count of valid cards: {}", count);
    println!();

    // Method 2: Streaming validation
    println!("Using Streaming Validation:");

    // Filter to only valid cards
    let valid_only: Vec<_> = cards.iter().copied().validate_valid_only().collect();
    println!("  Valid cards (streaming): {}", valid_only.len());
    for card in &valid_only {
        println!("    {} - {}", card.masked(), card.brand().name());
    }
    println!();

    // Get all results as iterator
    let all_results: Vec<_> = cards.iter().copied().validate_cards().collect();
    println!("  All results (streaming):");
    for (i, result) in all_results.iter().enumerate() {
        match result {
            Ok(card) => println!("    [{}] Valid: {} - {}", i, card.masked(), card.brand().name()),
            Err(e) => println!("    [{}] Invalid: {}", i, e),
        }
    }
    println!();

    // Method 3: Standard iterator methods
    println!("Using Standard Iterator Methods:");
    let valid_visa_count = cards
        .iter()
        .copied()
        .validate_valid_only()
        .filter(|c| c.brand() == cc_validator::CardBrand::Visa)
        .count();
    println!("  Valid Visa cards: {}", valid_visa_count);

    let brands: Vec<_> = cards
        .iter()
        .copied()
        .validate_valid_only()
        .map(|c| c.brand().name().to_string())
        .collect();
    println!("  Brands found: {:?}", brands);
    println!();

    // Performance demonstration with larger dataset
    println!("Performance Test:");
    let large_dataset: Vec<&str> = cards.iter().copied().cycle().take(10000).collect();

    let start = std::time::Instant::now();
    let (count, _) = batch::count_valid(&large_dataset);
    let elapsed = start.elapsed();

    println!(
        "  Validated {} cards in {:?}",
        large_dataset.len(),
        elapsed
    );
    println!("  Valid: {}", count);
    println!(
        "  Rate: {:.2} cards/sec",
        large_dataset.len() as f64 / elapsed.as_secs_f64()
    );
}
