//! Fuzz target for card formatting.
//!
//! Tests that formatting functions never panic on arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use cc_validator::{format, CardBrand};

fuzz_target!(|data: &str| {
    // These should never panic
    let _ = format::format_card_number(data);
    let _ = format::format_with_separator(data, " ");
    let _ = format::format_with_separator(data, "-");
    let _ = format::format_with_separator(data, "");
    let _ = format::strip_formatting(data);
    let _ = format::format_partial(data);
    let _ = format::split_into_groups(data);
    let _ = format::is_valid_format(data);

    // Test with all brands
    let brands = [
        CardBrand::Visa,
        CardBrand::Mastercard,
        CardBrand::Amex,
        CardBrand::Discover,
        CardBrand::DinersClub,
    ];

    for brand in brands {
        let _ = format::format_for_brand(data, brand);
        let _ = format::format_for_brand_with_separator(data, brand, "-");
    }

    // Verify roundtrip property
    let formatted = format::format_card_number(data);
    let stripped = format::strip_formatting(&formatted);
    let original_digits = format::strip_formatting(data);
    assert_eq!(stripped, original_digits, "Format roundtrip should preserve digits");
});
