//! Luhn algorithm implementation for credit card validation.
//!
//! The Luhn algorithm (also known as the "modulus 10" algorithm) is a checksum
//! formula used to validate credit card numbers and other identification numbers.
//!
//! # Performance
//!
//! This implementation uses a lookup table for the doubling operation,
//! making it O(n) with minimal overhead. For SIMD acceleration on nightly
//! Rust, enable the `simd` feature.

/// Lookup table for doubled digits: double the value, subtract 9 if >= 10.
/// This avoids the branch and division in the inner loop.
/// Index is the digit (0-9), value is the transformed result.
const DOUBLE_TABLE: [u8; 10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];

/// Validates a card number using the Luhn algorithm.
///
/// # Arguments
///
/// * `digits` - A slice of digits (0-9) representing the card number.
///
/// # Returns
///
/// `true` if the checksum is valid, `false` otherwise.
///
/// # Algorithm
///
/// 1. Starting from the rightmost digit (check digit), moving left
/// 2. Double every second digit
/// 3. If doubling results in a number > 9, subtract 9
/// 4. Sum all digits
/// 5. If the sum is divisible by 10, the number is valid
///
/// # Example
///
/// ```
/// use cc_validator::luhn::validate;
///
/// // Valid Visa test card
/// let digits = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
/// assert!(validate(&digits));
///
/// // Invalid card (changed last digit)
/// let invalid = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
/// assert!(!validate(&invalid));
/// ```
#[inline]
pub fn validate(digits: &[u8]) -> bool {
    if digits.is_empty() {
        return false;
    }

    let sum = compute_checksum(digits);
    sum % 10 == 0
}

/// Computes the Luhn checksum for a sequence of digits.
///
/// This is the core algorithm used by both validation and check digit generation.
///
/// # Arguments
///
/// * `digits` - A slice of digits (0-9).
///
/// # Returns
///
/// The Luhn sum (not modulo 10).
#[inline]
pub fn compute_checksum(digits: &[u8]) -> u32 {
    let len = digits.len();
    let mut sum: u32 = 0;

    // Process from right to left
    // The rightmost digit is position 0 (not doubled)
    // Position 1 is doubled, position 2 is not, etc.
    let mut i = 0;
    while i < len {
        let idx = len - 1 - i;
        let digit = digits[idx];

        if i % 2 == 1 {
            // Double this digit (positions 1, 3, 5, ...)
            sum += DOUBLE_TABLE[digit as usize] as u32;
        } else {
            // Don't double (positions 0, 2, 4, ...)
            sum += digit as u32;
        }
        i += 1;
    }

    sum
}

/// Generates the check digit for a partial card number.
///
/// Given digits without the check digit, computes what the check digit
/// should be for the number to pass Luhn validation.
///
/// # Arguments
///
/// * `digits` - A slice of digits without the check digit.
///
/// # Returns
///
/// The check digit (0-9) that makes the full number valid.
///
/// # Example
///
/// ```
/// use cc_validator::luhn::generate_check_digit;
///
/// // Visa test card without check digit
/// let partial = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
/// assert_eq!(generate_check_digit(&partial), 1);
/// ```
#[inline]
pub fn generate_check_digit(digits: &[u8]) -> u8 {
    // When computing for check digit generation, the digits we have
    // will all be shifted one position to the left (position-wise)
    // compared to the final number.
    //
    // In the final number:
    // - Check digit is at position 0 (not doubled)
    // - Current last digit will be at position 1 (doubled)
    // - etc.

    let len = digits.len();
    let mut sum: u32 = 0;

    // Process from right to left
    // Since check digit will be at position 0, current position i
    // from right will be at position i+1 in final number
    let mut i = 0;
    while i < len {
        let idx = len - 1 - i;
        let digit = digits[idx];

        // Position in final number will be i+1
        // If (i+1) is odd, we double
        if i % 2 == 0 {
            // i is even, so i+1 is odd -> double
            sum += DOUBLE_TABLE[digit as usize] as u32;
        } else {
            // i is odd, so i+1 is even -> don't double
            sum += digit as u32;
        }
        i += 1;
    }

    // Check digit is (10 - (sum % 10)) % 10
    ((10 - (sum % 10)) % 10) as u8
}

/// Validates digits using an optimized unrolled loop for 16-digit cards.
///
/// This is the most common card length, so we optimize for it.
#[inline]
pub fn validate_16(digits: &[u8; 16]) -> bool {
    // Unrolled loop for 16 digits
    // Positions from right: 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15
    // Double positions: 1,3,5,7,9,11,13,15 (indices 14,12,10,8,6,4,2,0)
    // Keep positions: 0,2,4,6,8,10,12,14 (indices 15,13,11,9,7,5,3,1)

    let sum = digits[15] as u32
        + DOUBLE_TABLE[digits[14] as usize] as u32
        + digits[13] as u32
        + DOUBLE_TABLE[digits[12] as usize] as u32
        + digits[11] as u32
        + DOUBLE_TABLE[digits[10] as usize] as u32
        + digits[9] as u32
        + DOUBLE_TABLE[digits[8] as usize] as u32
        + digits[7] as u32
        + DOUBLE_TABLE[digits[6] as usize] as u32
        + digits[5] as u32
        + DOUBLE_TABLE[digits[4] as usize] as u32
        + digits[3] as u32
        + DOUBLE_TABLE[digits[2] as usize] as u32
        + digits[1] as u32
        + DOUBLE_TABLE[digits[0] as usize] as u32;

    sum % 10 == 0
}

/// Validates digits using an optimized unrolled loop for 15-digit cards (Amex).
#[inline]
pub fn validate_15(digits: &[u8; 15]) -> bool {
    // 15 digits - odd length
    // Positions from right: 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14
    // Double positions: 1,3,5,7,9,11,13 (indices 13,11,9,7,5,3,1)
    // Keep positions: 0,2,4,6,8,10,12,14 (indices 14,12,10,8,6,4,2,0)

    let sum = digits[14] as u32
        + DOUBLE_TABLE[digits[13] as usize] as u32
        + digits[12] as u32
        + DOUBLE_TABLE[digits[11] as usize] as u32
        + digits[10] as u32
        + DOUBLE_TABLE[digits[9] as usize] as u32
        + digits[8] as u32
        + DOUBLE_TABLE[digits[7] as usize] as u32
        + digits[6] as u32
        + DOUBLE_TABLE[digits[5] as usize] as u32
        + digits[4] as u32
        + DOUBLE_TABLE[digits[3] as usize] as u32
        + digits[2] as u32
        + DOUBLE_TABLE[digits[1] as usize] as u32
        + digits[0] as u32;

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cards() {
        // Visa test cards
        assert!(validate(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        assert!(validate(&[4, 0, 1, 2, 8, 8, 8, 8, 8, 8, 8, 8, 1, 8, 8, 1]));

        // Mastercard test card
        assert!(validate(&[5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]));
        assert!(validate(&[5, 1, 0, 5, 1, 0, 5, 1, 0, 5, 1, 0, 5, 1, 0, 0]));

        // Amex test card
        assert!(validate(&[3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5]));
        assert!(validate(&[3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));

        // Discover test card
        assert!(validate(&[6, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 7]));

        // Diners Club
        assert!(validate(&[3, 0, 5, 6, 9, 3, 0, 9, 0, 2, 5, 9, 0, 4]));
    }

    #[test]
    fn test_invalid_cards() {
        // Changed last digit
        assert!(!validate(&[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]));

        // Changed first digit
        assert!(!validate(&[5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));

        // Random invalid
        assert!(!validate(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_generate_check_digit() {
        // Visa
        let partial = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(generate_check_digit(&partial), 1);

        // Mastercard
        let partial = [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(generate_check_digit(&partial), 4);

        // Amex
        let partial = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0];
        assert_eq!(generate_check_digit(&partial), 5);
    }

    #[test]
    fn test_validate_16() {
        let valid: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert!(validate_16(&valid));

        let invalid: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
        assert!(!validate_16(&invalid));
    }

    #[test]
    fn test_validate_15() {
        let valid: [u8; 15] = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5];
        assert!(validate_15(&valid));

        let invalid: [u8; 15] = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 6];
        assert!(!validate_15(&invalid));
    }

    #[test]
    fn test_empty_input() {
        assert!(!validate(&[]));
    }

    #[test]
    fn test_single_digit() {
        // Single 0 should be valid (0 % 10 == 0)
        assert!(validate(&[0]));
        // Other single digits should be invalid (unless they're 0)
        assert!(!validate(&[1]));
        assert!(!validate(&[5]));
    }

    #[test]
    fn test_double_table_values() {
        // Verify the lookup table is correct
        for i in 0..10 {
            let doubled = i * 2;
            let expected = if doubled > 9 { doubled - 9 } else { doubled };
            assert_eq!(DOUBLE_TABLE[i], expected as u8);
        }
    }
}
