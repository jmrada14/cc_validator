//! Fuzz target for CVV validation.
//!
//! Tests that CVV functions never panic on arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use cc_validator::{cvv, CardBrand};

fuzz_target!(|data: &str| {
    // These should never panic
    let _ = cvv::validate_cvv(data);
    let _ = cvv::is_valid_cvv(data);

    // Test with all brands
    let brands = [
        CardBrand::Visa,
        CardBrand::Mastercard,
        CardBrand::Amex,
        CardBrand::Discover,
        CardBrand::DinersClub,
        CardBrand::Jcb,
    ];

    for brand in brands {
        let _ = cvv::validate_cvv_for_brand(data, brand);
        let _ = cvv::is_valid_cvv_for_brand(data, brand);
    }

    // If validation succeeds, test other methods
    if let Ok(validated) = cvv::validate_cvv(data) {
        let _ = validated.as_str();
        let _ = validated.length();
        let _ = validated.is_four_digit();
        let _ = validated.digits();
        let _ = format!("{:?}", validated);
        let _ = format!("{}", validated);
    }
});
