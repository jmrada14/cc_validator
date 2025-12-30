//! Fuzz target for card validation.
//!
//! Tests that validate() never panics on arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use cc_validator::{validate, is_valid, passes_luhn, validate_any, validate_digits};

fuzz_target!(|data: &str| {
    // These should never panic, regardless of input
    let _ = validate(data);
    let _ = is_valid(data);
    let _ = passes_luhn(data);
    let _ = validate_any(data);

    // Also test with raw bytes interpreted as digits
    let digits: Vec<u8> = data.bytes().map(|b| b % 10).collect();
    if !digits.is_empty() && digits.len() <= 19 {
        let _ = validate_digits(&digits);
    }
});
