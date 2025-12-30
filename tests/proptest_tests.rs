//! Property-based tests using proptest.
//!
//! These tests verify invariants that should hold for all inputs,
//! helping discover edge cases that manual tests might miss.

use proptest::prelude::*;
use cc_validator::{
    validate, is_valid, passes_luhn, CardBrand,
    luhn, format, mask, expiry, cvv,
    generate::{generate_card_deterministic, CardGenerator},
};

// =============================================================================
// STRATEGIES
// =============================================================================

/// Generates a valid card number for any brand.
fn valid_card_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
        Just(CardBrand::Discover),
        Just(CardBrand::DinersClub),
        Just(CardBrand::Jcb),
    ]
    .prop_map(|brand| generate_card_deterministic(brand))
}

/// Generates a random digit string of a given length.
fn digit_string(len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(prop::char::range('0', '9'), len)
        .prop_map(|chars| chars.into_iter().collect())
}

/// Generates a random digit string of a length within range.
fn digit_string_range(range: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = String> {
    range.prop_flat_map(|len| digit_string(len))
}

/// Generates a string with separators (spaces, dashes) mixed in.
fn card_with_separators(card: String) -> impl Strategy<Value = String> {
    let len = card.len();
    proptest::collection::vec(
        prop_oneof![
            Just(""),
            Just(" "),
            Just("-"),
            Just("  "),
            Just(" - "),
        ],
        len + 1
    )
    .prop_map(move |seps| {
        let mut result = String::new();
        for (i, c) in card.chars().enumerate() {
            result.push_str(seps.get(i).unwrap_or(&""));
            result.push(c);
        }
        result.push_str(seps.last().unwrap_or(&""));
        result
    })
}

// =============================================================================
// LUHN ALGORITHM PROPERTIES
// =============================================================================

proptest! {
    /// Property: Generated cards always pass Luhn validation.
    #[test]
    fn generated_cards_always_pass_luhn(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
        Just(CardBrand::Discover),
    ]) {
        let card = generate_card_deterministic(brand);
        prop_assert!(passes_luhn(&card), "Generated card should pass Luhn: {}", card);
    }

    /// Property: Adding a check digit makes any digit sequence valid.
    #[test]
    fn check_digit_makes_valid(
        prefix in digit_string_range(11..=18)
    ) {
        let digits: Vec<u8> = prefix.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();
        let check = luhn::generate_check_digit(&digits);
        let mut full = digits.clone();
        full.push(check);
        prop_assert!(luhn::validate(&full), "Adding check digit should make sequence valid");
    }

    /// Property: Changing any single digit invalidates Luhn.
    #[test]
    fn single_digit_change_invalidates_luhn(
        brand in prop_oneof![Just(CardBrand::Visa), Just(CardBrand::Mastercard)],
        change_pos in 0usize..16usize,
        delta in 1u8..=9u8,
    ) {
        let card = generate_card_deterministic(brand);
        let digits: Vec<u8> = card.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();

        if change_pos < digits.len() {
            let mut modified = digits.clone();
            modified[change_pos] = (modified[change_pos] + delta) % 10;

            // If we changed to the same digit (delta = 10), skip
            if modified[change_pos] != digits[change_pos] {
                prop_assert!(!luhn::validate(&modified),
                    "Changing digit at position {} should invalidate Luhn", change_pos);
            }
        }
    }

    /// Property: All zeros of valid length passes Luhn (sum = 0).
    #[test]
    fn all_zeros_passes_luhn(len in 12usize..=19usize) {
        let zeros: Vec<u8> = vec![0; len];
        prop_assert!(luhn::validate(&zeros), "All zeros should pass Luhn");
    }
}

// =============================================================================
// VALIDATION PROPERTIES
// =============================================================================

proptest! {
    /// Property: Valid cards are always detected correctly.
    #[test]
    fn valid_cards_validate_successfully(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
        Just(CardBrand::Discover),
        Just(CardBrand::DinersClub),
        Just(CardBrand::Jcb),
    ]) {
        let card = generate_card_deterministic(brand);
        let result = validate(&card);
        prop_assert!(result.is_ok(), "Generated card should validate: {:?}", result);
        prop_assert_eq!(result.unwrap().brand(), brand);
    }

    /// Property: is_valid is consistent with validate.
    #[test]
    fn is_valid_consistent_with_validate(input in ".*") {
        let is_valid_result = is_valid(&input);
        let validate_result = validate(&input);
        prop_assert_eq!(is_valid_result, validate_result.is_ok());
    }

    /// Property: Separators don't affect validation result.
    #[test]
    fn separators_dont_affect_validation(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        // Add various separators
        let with_spaces = format!("{} {} {} {}",
            &card[0..4], &card[4..8], &card[8..12], &card[12..]);
        let with_dashes = format!("{}-{}-{}-{}",
            &card[0..4], &card[4..8], &card[8..12], &card[12..]);
        let with_dots = format!("{}.{}.{}.{}",
            &card[0..4], &card[4..8], &card[8..12], &card[12..]);

        prop_assert!(validate(&with_spaces).is_ok());
        prop_assert!(validate(&with_dashes).is_ok());
        prop_assert!(validate(&with_dots).is_ok());

        // All should produce same card number
        let clean: String = card.chars().filter(|c| c.is_ascii_digit()).collect();
        prop_assert_eq!(validate(&with_spaces).unwrap().number(), clean);
    }

    /// Property: validate never panics on any input.
    #[test]
    fn validate_never_panics(input in ".*") {
        let _ = validate(&input);
        let _ = is_valid(&input);
        let _ = passes_luhn(&input);
    }
}

// =============================================================================
// FORMATTING PROPERTIES
// =============================================================================

proptest! {
    /// Property: format then strip produces original digits.
    #[test]
    fn format_roundtrip(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        let formatted = format::format_card_number(&card);
        let stripped = format::strip_formatting(&formatted);
        prop_assert_eq!(stripped, card);
    }

    /// Property: Formatting always adds separators.
    #[test]
    fn formatting_adds_separators(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
    ]) {
        let card = generate_card_deterministic(brand);
        let formatted = format::format_card_number(&card);
        prop_assert!(formatted.contains(' '), "Formatted card should contain spaces");
        prop_assert!(formatted.len() > card.len(), "Formatted should be longer");
    }

    /// Property: strip_formatting produces only digits.
    #[test]
    fn strip_formatting_only_digits(input in "[0-9 \\-\\.]{0,30}") {
        let stripped = format::strip_formatting(&input);
        prop_assert!(stripped.chars().all(|c| c.is_ascii_digit()),
            "Stripped output should contain only digits");
    }

    /// Property: split_into_groups and join equals original.
    #[test]
    fn split_join_roundtrip(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        let groups = format::split_into_groups(&card);
        let joined: String = groups.join("");
        prop_assert_eq!(joined, card);
    }
}

// =============================================================================
// MASKING PROPERTIES
// =============================================================================

proptest! {
    /// Property: Masked output never contains full card number.
    #[test]
    fn masked_never_exposes_full_number(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
    ]) {
        let card = generate_card_deterministic(brand);
        let validated = validate(&card).unwrap();
        let masked = validated.masked();
        let masked_bin = validated.masked_with_bin();

        prop_assert!(!masked.contains(&card), "masked() should not contain full number");
        prop_assert!(!masked_bin.contains(&card), "masked_with_bin() should not contain full number");
    }

    /// Property: Masked output always shows last four digits.
    #[test]
    fn masked_shows_last_four(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        let validated = validate(&card).unwrap();
        let last_four = validated.last_four();
        let masked = validated.masked();

        prop_assert!(masked.contains(&last_four), "masked() should show last four: {}", masked);
    }

    /// Property: mask_string always contains asterisks.
    #[test]
    fn mask_string_contains_asterisks(card in digit_string_range(12..=19)) {
        if is_valid(&card) {
            let masked = mask::mask_string(&card);
            prop_assert!(masked.contains('*'), "Masked string should contain asterisks");
        }
    }
}

// =============================================================================
// EXPIRY DATE PROPERTIES
// =============================================================================

proptest! {
    /// Property: Valid months parse correctly.
    #[test]
    fn valid_month_parses(month in 1u8..=12u8, year in 25u16..=99u16) {
        let input = format!("{:02}/{:02}", month, year);
        let result = expiry::parse_expiry(&input);
        prop_assert!(result.is_ok(), "Valid month/year should parse: {}", input);
        let expiry = result.unwrap();
        prop_assert_eq!(expiry.month(), month);
        prop_assert_eq!(expiry.year(), 2000 + year);
    }

    /// Property: Invalid months are rejected.
    #[test]
    fn invalid_month_rejected(month in prop_oneof![Just(0u8), 13u8..=99u8], year in 25u16..=99u16) {
        let input = format!("{:02}/{:02}", month, year);
        let result = expiry::parse_expiry(&input);
        prop_assert!(result.is_err() || month == 0 || month > 12,
            "Invalid month should be rejected: {}", input);
    }

    /// Property: Formatting roundtrips.
    #[test]
    fn expiry_format_roundtrip(month in 1u8..=12u8, year in 2025u16..=2050u16) {
        let expiry = expiry::ExpiryDate::new(month, year).unwrap();
        let short = expiry.format_short();
        let parsed = expiry::parse_expiry(&short).unwrap();
        prop_assert_eq!(parsed.month(), month);
        prop_assert_eq!(parsed.year(), year);
    }
}

// =============================================================================
// CVV PROPERTIES
// =============================================================================

proptest! {
    /// Property: 3-digit CVV is valid.
    #[test]
    fn three_digit_cvv_valid(d1 in 0u8..=9, d2 in 0u8..=9, d3 in 0u8..=9) {
        let cvv_str = format!("{}{}{}", d1, d2, d3);
        let result = cvv::validate_cvv(&cvv_str);
        prop_assert!(result.is_ok(), "3-digit CVV should be valid: {}", cvv_str);
        prop_assert_eq!(result.unwrap().length(), 3);
    }

    /// Property: 4-digit CVV is valid.
    #[test]
    fn four_digit_cvv_valid(d1 in 0u8..=9, d2 in 0u8..=9, d3 in 0u8..=9, d4 in 0u8..=9) {
        let cvv_str = format!("{}{}{}{}", d1, d2, d3, d4);
        let result = cvv::validate_cvv(&cvv_str);
        prop_assert!(result.is_ok(), "4-digit CVV should be valid: {}", cvv_str);
        prop_assert_eq!(result.unwrap().length(), 4);
    }

    /// Property: CVV for Amex requires 4 digits.
    #[test]
    fn amex_cvv_length(d1 in 0u8..=9, d2 in 0u8..=9, d3 in 0u8..=9) {
        let cvv_str = format!("{}{}{}", d1, d2, d3);
        let result = cvv::validate_cvv_for_brand(&cvv_str, CardBrand::Amex);
        prop_assert!(result.is_err(), "Amex should require 4-digit CVV");
    }

    /// Property: CVV for Visa requires 3 digits.
    #[test]
    fn visa_cvv_length(d1 in 0u8..=9, d2 in 0u8..=9, d3 in 0u8..=9, d4 in 0u8..=9) {
        let cvv_str = format!("{}{}{}{}", d1, d2, d3, d4);
        let result = cvv::validate_cvv_for_brand(&cvv_str, CardBrand::Visa);
        prop_assert!(result.is_err(), "Visa should require 3-digit CVV");
    }
}

// =============================================================================
// GENERATOR PROPERTIES
// =============================================================================

proptest! {
    /// Property: Generated cards always have correct length for brand.
    #[test]
    fn generated_cards_correct_length(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
        Just(CardBrand::Amex),
        Just(CardBrand::Discover),
    ]) {
        let card = generate_card_deterministic(brand);
        let expected_len = match brand {
            CardBrand::Amex => 15,
            CardBrand::DinersClub => 14,
            _ => 16,
        };
        prop_assert_eq!(card.len(), expected_len, "Card length should match brand");
    }

    /// Property: CardGenerator produces valid cards.
    #[test]
    fn card_generator_produces_valid(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let gen = CardGenerator::new(brand);
        let card = gen.generate_deterministic();
        prop_assert!(is_valid(&card), "CardGenerator should produce valid cards");
    }

    /// Property: Generated cards start with correct prefix.
    #[test]
    fn generated_cards_correct_prefix(brand in prop_oneof![
        (Just(CardBrand::Visa), Just("4")),
        (Just(CardBrand::Amex), Just("34")),
        (Just(CardBrand::Discover), Just("6011")),
    ]) {
        let (b, expected_prefix) = brand;
        let card = generate_card_deterministic(b);
        prop_assert!(card.starts_with(expected_prefix),
            "Card should start with {}: {}", expected_prefix, card);
    }
}

// =============================================================================
// SECURITY PROPERTIES
// =============================================================================

proptest! {
    /// Property: Debug output never exposes full card number.
    #[test]
    fn debug_never_exposes_card(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        let validated = validate(&card).unwrap();
        let debug = format!("{:?}", validated);
        prop_assert!(!debug.contains(&card), "Debug should not expose full card: {}", debug);
    }

    /// Property: Display output never exposes full card number.
    #[test]
    fn display_never_exposes_card(brand in prop_oneof![
        Just(CardBrand::Visa),
        Just(CardBrand::Mastercard),
    ]) {
        let card = generate_card_deterministic(brand);
        let validated = validate(&card).unwrap();
        let display = format!("{}", validated);
        prop_assert!(!display.contains(&card), "Display should not expose full card: {}", display);
    }

    /// Property: CVV debug never exposes value.
    #[test]
    fn cvv_debug_never_exposes(d1 in 0u8..=9, d2 in 0u8..=9, d3 in 0u8..=9) {
        let cvv_str = format!("{}{}{}", d1, d2, d3);
        let cvv = cvv::validate_cvv(&cvv_str).unwrap();
        let debug = format!("{:?}", cvv);
        prop_assert!(!debug.contains(&cvv_str), "CVV debug should not expose value");
    }
}
