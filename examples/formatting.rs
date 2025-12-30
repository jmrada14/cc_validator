//! Card number formatting example.
//!
//! Run with: `cargo run --example formatting`

use cc_validator::format;

fn main() {
    println!("=== Card Number Formatting ===\n");

    // -------------------------------------------------------------------------
    // Auto-formatting based on brand
    // -------------------------------------------------------------------------
    println!("--- Auto-formatting (brand-aware) ---\n");

    let cards = [
        ("4111111111111111", "Visa (4-4-4-4)"),
        ("5500000000000004", "Mastercard (4-4-4-4)"),
        ("378282246310005", "Amex (4-6-5)"),
        ("6011111111111117", "Discover (4-4-4-4)"),
        ("30569309025904", "Diners Club (4-6-4)"),
    ];

    for (number, description) in cards {
        let formatted = format::format_card_number(number);
        println!("  {}", description);
        println!("    Input:  {}", number);
        println!("    Output: {}", formatted);
        println!();
    }

    // -------------------------------------------------------------------------
    // Custom separators
    // -------------------------------------------------------------------------
    println!("--- Custom Separators ---\n");

    let number = "4111111111111111";
    let separators = [" ", "-", ".", " - "];

    println!("  Card: {}", number);
    for sep in separators {
        let formatted = format::format_with_separator(number, sep);
        println!("    Separator '{}': {}", sep, formatted);
    }
    println!();

    // -------------------------------------------------------------------------
    // Strip formatting
    // -------------------------------------------------------------------------
    println!("--- Stripping Formatting ---\n");

    let formatted_cards = [
        "4111 1111 1111 1111",
        "4111-1111-1111-1111",
        "4111.1111.1111.1111",
        "4111 - 1111 - 1111 - 1111",
        "  4111  1111  1111  1111  ",
    ];

    for formatted in formatted_cards {
        let stripped = format::strip_formatting(formatted);
        println!("  '{}' -> '{}'", formatted, stripped);
    }
    println!();

    // -------------------------------------------------------------------------
    // Partial formatting (for input fields)
    // -------------------------------------------------------------------------
    println!("--- Partial Formatting (as-you-type) ---\n");

    let partial_inputs = [
        "4",
        "41",
        "411",
        "4111",
        "41111",
        "411111",
        "4111111",
        "41111111",
        "411111111",
        "4111111111",
        "41111111111",
        "411111111111",
        "4111111111111",
        "41111111111111",
        "411111111111111",
        "4111111111111111",
    ];

    println!("  Simulating typing a Visa card:");
    for input in partial_inputs {
        let formatted = format::format_partial(input);
        println!("    {} -> {}", input, formatted);
    }
    println!();

    // Amex partial formatting (4-6-5 grouping)
    let amex_partial = [
        "3", "37", "378", "3782", "37828", "378282", "3782822", "37828224",
    ];

    println!("  Simulating typing an Amex card:");
    for input in amex_partial {
        let formatted = format::format_partial(input);
        println!("    {} -> {}", input, formatted);
    }
    println!();

    // -------------------------------------------------------------------------
    // Round-trip formatting
    // -------------------------------------------------------------------------
    println!("--- Round-trip Formatting ---\n");

    let original = "4111111111111111";
    let formatted = format::format_card_number(original);
    let stripped = format::strip_formatting(&formatted);

    println!("  Original:  {}", original);
    println!("  Formatted: {}", formatted);
    println!("  Stripped:  {}", stripped);
    println!(
        "  Round-trip success: {}",
        if original == stripped { "yes" } else { "no" }
    );
}
