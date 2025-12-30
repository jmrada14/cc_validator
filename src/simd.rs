//! SIMD-accelerated Luhn algorithm implementation.
//!
//! This module provides a vectorized implementation of the Luhn checksum
//! algorithm using Rust's portable SIMD API.
//!
//! # Feature
//!
//! Requires the `simd` feature and a nightly Rust compiler:
//!
//! ```toml
//! [features]
//! simd = []
//! ```
//!
//! ```rust,ignore
//! #![feature(portable_simd)]
//! ```
//!
//! # Performance
//!
//! The SIMD implementation processes 16 digits at once, providing
//! significant speedup for 16+ digit card numbers on supported hardware.

#[cfg(feature = "simd")]
use std::simd::{cmp::SimdPartialOrd, u8x16};

/// Validates a 16-digit card number using SIMD.
///
/// This is optimized for the most common card length (16 digits).
///
/// # Arguments
///
/// * `digits` - Exactly 16 digits (0-9 values, not ASCII)
///
/// # Returns
///
/// `true` if the Luhn checksum is valid, `false` otherwise.
///
/// # Safety
///
/// The input must be exactly 16 digits, each in range 0-9.
#[cfg(feature = "simd")]
#[inline]
pub fn validate_16_simd(digits: &[u8; 16]) -> bool {
    // Load digits into SIMD register
    let v = u8x16::from_slice(digits);

    // Create mask for positions that need doubling
    // In a 16-digit card, counting from RIGHT: position 0 is check digit (not doubled),
    // position 1 is doubled, etc. From LEFT: positions 0,2,4,6,8,10,12,14 get doubled.
    let double_mask = u8x16::from_array([1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]);

    // Double the appropriate digits
    let doubled = v + v;

    // Subtract 9 from values > 9
    let nine = u8x16::splat(9);
    let needs_sub = doubled.simd_gt(nine);
    let subtracted = doubled - nine;

    // Select: use subtracted where needs_sub is true, else use doubled
    let doubled_adjusted = needs_sub.select(subtracted, doubled);

    // Select: use doubled_adjusted where double_mask is 1, else use original
    let mask_bool = double_mask.simd_gt(u8x16::splat(0));
    let final_values = mask_bool.select(doubled_adjusted, v);

    // Sum all values
    let sum: u32 = final_values.as_array().iter().map(|&x| x as u32).sum();

    sum % 10 == 0
}

/// Validates any length card number using SIMD where possible.
///
/// Falls back to scalar implementation for cards shorter than 16 digits.
///
/// # Arguments
///
/// * `digits` - Card digits (0-9 values)
///
/// # Returns
///
/// `true` if the Luhn checksum is valid, `false` otherwise.
#[cfg(feature = "simd")]
#[inline]
pub fn validate_simd(digits: &[u8]) -> bool {
    match digits.len() {
        16 => {
            let arr: [u8; 16] = digits.try_into().unwrap();
            validate_16_simd(&arr)
        }
        // For other lengths, use scalar implementation
        _ => crate::luhn::validate(digits),
    }
}

/// SIMD-accelerated batch validation.
///
/// Validates multiple 16-digit cards in parallel using SIMD.
/// Cards of other lengths are processed with the scalar implementation.
#[cfg(feature = "simd")]
pub fn validate_batch_simd(cards: &[&[u8]]) -> Vec<bool> {
    cards
        .iter()
        .map(|digits| {
            if digits.len() == 16 {
                let arr: [u8; 16] = (*digits).try_into().unwrap();
                validate_16_simd(&arr)
            } else {
                crate::luhn::validate(digits)
            }
        })
        .collect()
}

// Provide stub implementations when SIMD is not enabled
// These fall back to scalar implementations

/// Validates a 16-digit card number.
///
/// This is a stub that falls back to the scalar implementation
/// when the `simd` feature is not enabled.
#[cfg(not(feature = "simd"))]
#[inline]
pub fn validate_16_simd(digits: &[u8; 16]) -> bool {
    crate::luhn::validate_16(digits)
}

/// Validates any length card number.
///
/// This is a stub that falls back to the scalar implementation
/// when the `simd` feature is not enabled.
#[cfg(not(feature = "simd"))]
#[inline]
pub fn validate_simd(digits: &[u8]) -> bool {
    crate::luhn::validate(digits)
}

/// Batch validates multiple card numbers.
///
/// This is a stub that falls back to the scalar implementation
/// when the `simd` feature is not enabled.
#[cfg(not(feature = "simd"))]
pub fn validate_batch_simd(cards: &[&[u8]]) -> Vec<bool> {
    cards
        .iter()
        .map(|digits| crate::luhn::validate(digits))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_16_simd_valid() {
        // Valid Visa test card
        let digits: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert!(validate_16_simd(&digits));

        // Valid Mastercard
        let digits: [u8; 16] = [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4];
        assert!(validate_16_simd(&digits));
    }

    #[test]
    fn test_validate_16_simd_invalid() {
        // Invalid - changed last digit
        let digits: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
        assert!(!validate_16_simd(&digits));
    }

    #[test]
    fn test_validate_simd_various_lengths() {
        // 16 digits
        let visa16: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert!(validate_simd(&visa16));

        // 15 digits (Amex)
        let amex: [u8; 15] = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5];
        assert!(validate_simd(&amex));

        // 19 digits (Visa)
        let visa19: [u8; 19] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0];
        assert!(validate_simd(&visa19));
    }

    #[test]
    fn test_batch_simd() {
        let card1: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let card2: [u8; 16] = [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4];
        let card3: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]; // Invalid

        let cards: Vec<&[u8]> = vec![&card1[..], &card2[..], &card3[..]];
        let results = validate_batch_simd(&cards);

        assert_eq!(results, vec![true, true, false]);
    }

    #[test]
    fn test_simd_matches_scalar() {
        // Ensure SIMD gives same results as scalar
        let test_cards: [[u8; 16]; 4] = [
            [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4],
            [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
            [5, 1, 0, 5, 1, 0, 5, 1, 0, 5, 1, 0, 5, 1, 0, 0],
        ];

        for card in &test_cards {
            let simd_result = validate_16_simd(card);
            let scalar_result = crate::luhn::validate_16(card);
            assert_eq!(
                simd_result, scalar_result,
                "SIMD and scalar disagree on {:?}",
                card
            );
        }
    }
}
