//! Fuzz target for expiry date parsing.
//!
//! Tests that expiry parsing never panics on arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use cc_validator::expiry;

fuzz_target!(|data: &str| {
    // These should never panic
    let _ = expiry::parse_expiry(data);
    let _ = expiry::validate_expiry(data);
    let _ = expiry::validate_expiry_with_options(data, true, Some(20));
    let _ = expiry::validate_expiry_with_options(data, false, None);
    let _ = expiry::is_expired(data);

    // If parsing succeeds, test other methods
    if let Ok(exp) = expiry::parse_expiry(data) {
        let _ = exp.is_expired();
        let _ = exp.is_too_far_future(20);
        let _ = exp.months_until_expiry();
        let _ = exp.format_short();
        let _ = exp.format_long();
        let _ = exp.to_string();
    }
});
