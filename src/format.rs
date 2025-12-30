//! Card number formatting utilities.
//!
//! This module provides functions to format credit card numbers for display,
//! following standard grouping conventions for each card brand.
//!
//! # Format Conventions
//!
//! - **Visa/Mastercard/Discover** (16 digits): `XXXX XXXX XXXX XXXX`
//! - **American Express** (15 digits): `XXXX XXXXXX XXXXX`
//! - **Diners Club** (14 digits): `XXXX XXXXXX XXXX`
//! - **Other**: Groups of 4 with remainder at end
//!
//! # Example
//!
//! ```
//! use cc_validator::format::{format_card_number, format_with_separator};
//!
//! // Default formatting with spaces
//! assert_eq!(format_card_number("4111111111111111"), "4111 1111 1111 1111");
//!
//! // Amex formatting
//! assert_eq!(format_card_number("378282246310005"), "3782 822463 10005");
//!
//! // Custom separator
//! assert_eq!(format_with_separator("4111111111111111", "-"), "4111-1111-1111-1111");
//! ```

use crate::CardBrand;
use crate::detect::detect_brand;

/// Formats a credit card number with standard grouping.
///
/// Uses space as the default separator. Groups are determined by the
/// detected card brand.
///
/// # Example
///
/// ```
/// use cc_validator::format::format_card_number;
///
/// assert_eq!(format_card_number("4111111111111111"), "4111 1111 1111 1111");
/// assert_eq!(format_card_number("378282246310005"), "3782 822463 10005");
/// ```
pub fn format_card_number(input: &str) -> String {
    format_with_separator(input, " ")
}

/// Formats a credit card number with a custom separator.
///
/// # Example
///
/// ```
/// use cc_validator::format::format_with_separator;
///
/// assert_eq!(format_with_separator("4111111111111111", "-"), "4111-1111-1111-1111");
/// assert_eq!(format_with_separator("4111111111111111", " - "), "4111 - 1111 - 1111 - 1111");
/// ```
pub fn format_with_separator(input: &str, separator: &str) -> String {
    // Extract digits only
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return String::new();
    }

    // Detect brand to determine grouping
    let digit_values: Vec<u8> = digits
        .iter()
        .map(|&c| (c as u8) - b'0')
        .collect();
    let brand = detect_brand(&digit_values);

    // Get grouping pattern
    let groups = grouping_for_brand(brand, digits.len());

    // Apply grouping
    let mut result = String::with_capacity(digits.len() + groups.len() * separator.len());
    let mut pos = 0;

    for (i, &group_size) in groups.iter().enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        for _ in 0..group_size {
            if pos < digits.len() {
                result.push(digits[pos]);
                pos += 1;
            }
        }
    }

    // Add any remaining digits
    if pos < digits.len() {
        if !result.is_empty() {
            result.push_str(separator);
        }
        for c in &digits[pos..] {
            result.push(*c);
        }
    }

    result
}

/// Formats a card number for a specific brand.
///
/// This is useful when you know the brand and want to ensure correct formatting
/// even if the prefix doesn't match typical patterns.
///
/// # Example
///
/// ```
/// use cc_validator::format::format_for_brand;
/// use cc_validator::CardBrand;
///
/// assert_eq!(
///     format_for_brand("378282246310005", CardBrand::Amex),
///     "3782 822463 10005"
/// );
/// ```
pub fn format_for_brand(input: &str, brand: CardBrand) -> String {
    format_for_brand_with_separator(input, brand, " ")
}

/// Formats a card number for a specific brand with a custom separator.
pub fn format_for_brand_with_separator(input: &str, brand: CardBrand, separator: &str) -> String {
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return String::new();
    }

    let groups = grouping_for_brand(Some(brand), digits.len());

    let mut result = String::with_capacity(digits.len() + groups.len() * separator.len());
    let mut pos = 0;

    for (i, &group_size) in groups.iter().enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        for _ in 0..group_size {
            if pos < digits.len() {
                result.push(digits[pos]);
                pos += 1;
            }
        }
    }

    if pos < digits.len() {
        if !result.is_empty() {
            result.push_str(separator);
        }
        for c in &digits[pos..] {
            result.push(*c);
        }
    }

    result
}

/// Returns the digit grouping pattern for a card brand.
fn grouping_for_brand(brand: Option<CardBrand>, length: usize) -> Vec<usize> {
    match brand {
        Some(CardBrand::Amex) => {
            // Amex: 4-6-5
            vec![4, 6, 5]
        }
        Some(CardBrand::DinersClub) if length == 14 => {
            // Diners 14-digit: 4-6-4
            vec![4, 6, 4]
        }
        _ => {
            // Standard: groups of 4
            let full_groups = length / 4;
            let remainder = length % 4;

            let mut groups = vec![4; full_groups];
            if remainder > 0 {
                groups.push(remainder);
            }
            groups
        }
    }
}

/// Strips all formatting from a card number, leaving only digits.
///
/// # Example
///
/// ```
/// use cc_validator::format::strip_formatting;
///
/// assert_eq!(strip_formatting("4111 1111 1111 1111"), "4111111111111111");
/// assert_eq!(strip_formatting("4111-1111-1111-1111"), "4111111111111111");
/// ```
pub fn strip_formatting(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Formats a partial card number as the user types.
///
/// This is useful for real-time formatting in input fields.
/// It formats what's been entered so far according to brand conventions.
///
/// # Example
///
/// ```
/// use cc_validator::format::format_partial;
///
/// assert_eq!(format_partial("4111"), "4111");
/// assert_eq!(format_partial("41111"), "4111 1");
/// assert_eq!(format_partial("411111111111"), "4111 1111 1111");
/// ```
pub fn format_partial(input: &str) -> String {
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return String::new();
    }

    // For partial input, just use groups of 4
    let mut result = String::with_capacity(digits.len() + digits.len() / 4);

    for (i, c) in digits.iter().enumerate() {
        if i > 0 && i % 4 == 0 {
            result.push(' ');
        }
        result.push(*c);
    }

    result
}

/// Formats the card number into chunks for display.
///
/// Returns a vector of digit groups for flexible rendering.
///
/// # Example
///
/// ```
/// use cc_validator::format::split_into_groups;
///
/// let groups = split_into_groups("4111111111111111");
/// assert_eq!(groups, vec!["4111", "1111", "1111", "1111"]);
///
/// let groups = split_into_groups("378282246310005");
/// assert_eq!(groups, vec!["3782", "822463", "10005"]);
/// ```
pub fn split_into_groups(input: &str) -> Vec<String> {
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return vec![];
    }

    let digit_values: Vec<u8> = digits.iter().map(|&c| (c as u8) - b'0').collect();
    let brand = detect_brand(&digit_values);
    let group_sizes = grouping_for_brand(brand, digits.len());

    let mut groups = Vec::with_capacity(group_sizes.len());
    let mut pos = 0;

    for &size in &group_sizes {
        if pos < digits.len() {
            let end = (pos + size).min(digits.len());
            let group: String = digits[pos..end].iter().collect();
            if !group.is_empty() {
                groups.push(group);
            }
            pos = end;
        }
    }

    // Handle any remaining digits
    if pos < digits.len() {
        let group: String = digits[pos..].iter().collect();
        groups.push(group);
    }

    groups
}

/// Validates that a string contains only valid card number characters.
///
/// Valid characters are: digits (0-9), spaces, dashes, and periods.
///
/// # Example
///
/// ```
/// use cc_validator::format::is_valid_format;
///
/// assert!(is_valid_format("4111 1111 1111 1111"));
/// assert!(is_valid_format("4111-1111-1111-1111"));
/// assert!(!is_valid_format("4111a1111-1111-1111"));
/// ```
pub fn is_valid_format(input: &str) -> bool {
    input.chars().all(|c| c.is_ascii_digit() || c == ' ' || c == '-' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_visa_16() {
        assert_eq!(
            format_card_number("4111111111111111"),
            "4111 1111 1111 1111"
        );
    }

    #[test]
    fn test_format_visa_13() {
        assert_eq!(
            format_card_number("4222222222222"),
            "4222 2222 2222 2"
        );
    }

    #[test]
    fn test_format_amex() {
        assert_eq!(
            format_card_number("378282246310005"),
            "3782 822463 10005"
        );
    }

    #[test]
    fn test_format_diners_14() {
        assert_eq!(
            format_card_number("30569309025904"),
            "3056 930902 5904"
        );
    }

    #[test]
    fn test_format_with_dashes() {
        assert_eq!(
            format_with_separator("4111111111111111", "-"),
            "4111-1111-1111-1111"
        );
    }

    #[test]
    fn test_format_already_formatted() {
        // Should strip and reformat
        assert_eq!(
            format_card_number("4111-1111-1111-1111"),
            "4111 1111 1111 1111"
        );
    }

    #[test]
    fn test_format_empty() {
        assert_eq!(format_card_number(""), "");
        assert_eq!(format_card_number("   "), "");
    }

    #[test]
    fn test_strip_formatting() {
        assert_eq!(strip_formatting("4111 1111 1111 1111"), "4111111111111111");
        assert_eq!(strip_formatting("4111-1111-1111-1111"), "4111111111111111");
        assert_eq!(strip_formatting("4111.1111.1111.1111"), "4111111111111111");
    }

    #[test]
    fn test_format_partial() {
        assert_eq!(format_partial("4"), "4");
        assert_eq!(format_partial("41"), "41");
        assert_eq!(format_partial("411"), "411");
        assert_eq!(format_partial("4111"), "4111");
        assert_eq!(format_partial("41111"), "4111 1");
        assert_eq!(format_partial("411111"), "4111 11");
        assert_eq!(format_partial("4111111111111111"), "4111 1111 1111 1111");
    }

    #[test]
    fn test_split_into_groups() {
        let groups = split_into_groups("4111111111111111");
        assert_eq!(groups, vec!["4111", "1111", "1111", "1111"]);

        let groups = split_into_groups("378282246310005");
        assert_eq!(groups, vec!["3782", "822463", "10005"]);
    }

    #[test]
    fn test_is_valid_format() {
        assert!(is_valid_format("4111111111111111"));
        assert!(is_valid_format("4111 1111 1111 1111"));
        assert!(is_valid_format("4111-1111-1111-1111"));
        assert!(is_valid_format("4111.1111.1111.1111"));
        assert!(!is_valid_format("4111a111111111111"));
        assert!(!is_valid_format("4111_1111_1111_1111"));
    }

    #[test]
    fn test_format_for_brand() {
        assert_eq!(
            format_for_brand("378282246310005", CardBrand::Amex),
            "3782 822463 10005"
        );

        assert_eq!(
            format_for_brand("4111111111111111", CardBrand::Visa),
            "4111 1111 1111 1111"
        );
    }

    #[test]
    fn test_format_for_brand_with_separator() {
        assert_eq!(
            format_for_brand_with_separator("4111111111111111", CardBrand::Visa, "-"),
            "4111-1111-1111-1111"
        );
    }

    #[test]
    fn test_grouping_standard_19_digit() {
        // 19 digit card: 4-4-4-4-3
        let groups = grouping_for_brand(Some(CardBrand::Visa), 19);
        assert_eq!(groups, vec![4, 4, 4, 4, 3]);
    }

    #[test]
    fn test_format_19_digit() {
        let card = format_card_number("4111111111111111111");
        assert_eq!(card, "4111 1111 1111 1111 111");
    }
}
