//! Fuzz target for Luhn algorithm.
//!
//! Tests that luhn functions never panic and maintain invariants.

#![no_main]

use libfuzzer_sys::fuzz_target;
use cc_validator::luhn;

fuzz_target!(|data: &[u8]| {
    // Clamp values to valid digit range
    let digits: Vec<u8> = data.iter().map(|&b| b % 10).collect();

    if digits.is_empty() {
        return;
    }

    // Test validate
    let _ = luhn::validate(&digits);

    // Test specialized validators if length matches
    if digits.len() == 16 {
        let arr: [u8; 16] = digits.clone().try_into().unwrap();
        let generic = luhn::validate(&digits);
        let specialized = luhn::validate_16(&arr);
        assert_eq!(generic, specialized, "16-digit validation mismatch");
    }

    if digits.len() == 15 {
        let arr: [u8; 15] = digits.clone().try_into().unwrap();
        let generic = luhn::validate(&digits);
        let specialized = luhn::validate_15(&arr);
        assert_eq!(generic, specialized, "15-digit validation mismatch");
    }

    // Test check digit generation
    if digits.len() >= 1 && digits.len() <= 18 {
        let check = luhn::generate_check_digit(&digits);
        assert!(check <= 9, "Check digit should be 0-9");

        // Adding check digit should make it valid
        let mut with_check = digits.clone();
        with_check.push(check);
        assert!(luhn::validate(&with_check), "Adding check digit should make valid");
    }
});
