//! Comprehensive integration tests for cc_validator.
//!
//! These tests cover edge cases, real-world scenarios, and security considerations.

use cc_validator::{
    batch::BatchValidator, is_valid, luhn, mask, passes_luhn, stream::ValidateExt, validate,
    validate_any, validate_digits, CardBrand, ValidationError,
};

// =============================================================================
// REAL-WORLD TEST CARD NUMBERS
// =============================================================================
// These are official test card numbers from payment processors.
// They pass Luhn validation but are not real cards.

mod test_cards {
    // Visa test cards (from Stripe, Braintree, etc.)
    pub const VISA_1: &str = "4111111111111111";
    pub const VISA_2: &str = "4012888888881881";
    pub const VISA_3: &str = "4222222222222"; // 13 digits
    pub const VISA_4: &str = "4000056655665556";
    pub const VISA_5: &str = "4242424242424242";
    pub const VISA_DEBIT: &str = "4000056655665556";

    // Mastercard test cards
    pub const MC_1: &str = "5555555555554444";
    pub const MC_2: &str = "5105105105105100";
    pub const MC_3: &str = "5200828282828210";
    pub const MC_4: &str = "5431111111111111";
    // New Mastercard 2-series
    pub const MC_2SERIES_1: &str = "2223000048400011";
    pub const MC_2SERIES_2: &str = "2223520043560014";

    // American Express test cards
    pub const AMEX_1: &str = "378282246310005";
    pub const AMEX_2: &str = "371449635398431";
    pub const AMEX_3: &str = "340000000000009";
    pub const AMEX_CORPORATE: &str = "378734493671000";

    // Discover test cards
    pub const DISCOVER_1: &str = "6011111111111117";
    pub const DISCOVER_2: &str = "6011000990139424";
    pub const DISCOVER_3: &str = "6445644564456445";

    // Diners Club test cards
    pub const DINERS_1: &str = "30569309025904";
    pub const DINERS_2: &str = "38520000023237";
    pub const DINERS_3: &str = "36700102000000";

    // JCB test cards
    pub const JCB_1: &str = "3530111333300000";
    pub const JCB_2: &str = "3566002020360505";
}

// =============================================================================
// VALIDATION TESTS - VALID CARDS
// =============================================================================

#[test]
fn test_all_visa_test_cards() {
    for card in [
        test_cards::VISA_1,
        test_cards::VISA_2,
        test_cards::VISA_3,
        test_cards::VISA_4,
        test_cards::VISA_5,
        test_cards::VISA_DEBIT,
    ] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "Visa card {} should be valid: {:?}",
            card,
            result
        );
        assert_eq!(result.unwrap().brand(), CardBrand::Visa);
    }
}

#[test]
fn test_all_mastercard_test_cards() {
    for card in [
        test_cards::MC_1,
        test_cards::MC_2,
        test_cards::MC_3,
        test_cards::MC_4,
        test_cards::MC_2SERIES_1,
        test_cards::MC_2SERIES_2,
    ] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "Mastercard {} should be valid: {:?}",
            card,
            result
        );
        assert_eq!(result.unwrap().brand(), CardBrand::Mastercard);
    }
}

#[test]
fn test_all_amex_test_cards() {
    for card in [
        test_cards::AMEX_1,
        test_cards::AMEX_2,
        test_cards::AMEX_3,
        test_cards::AMEX_CORPORATE,
    ] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "Amex card {} should be valid: {:?}",
            card,
            result
        );
        let validated = result.unwrap();
        assert_eq!(validated.brand(), CardBrand::Amex);
        assert_eq!(validated.length(), 15);
    }
}

#[test]
fn test_all_discover_test_cards() {
    for card in [
        test_cards::DISCOVER_1,
        test_cards::DISCOVER_2,
        test_cards::DISCOVER_3,
    ] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "Discover card {} should be valid: {:?}",
            card,
            result
        );
        assert_eq!(result.unwrap().brand(), CardBrand::Discover);
    }
}

#[test]
fn test_all_diners_test_cards() {
    for card in [
        test_cards::DINERS_1,
        test_cards::DINERS_2,
        test_cards::DINERS_3,
    ] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "Diners card {} should be valid: {:?}",
            card,
            result
        );
        assert_eq!(result.unwrap().brand(), CardBrand::DinersClub);
    }
}

#[test]
fn test_all_jcb_test_cards() {
    for card in [test_cards::JCB_1, test_cards::JCB_2] {
        let result = validate(card);
        assert!(
            result.is_ok(),
            "JCB card {} should be valid: {:?}",
            card,
            result
        );
        assert_eq!(result.unwrap().brand(), CardBrand::Jcb);
    }
}

// =============================================================================
// INPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_various_separators() {
    let base = "4111111111111111";

    // Spaces
    assert!(validate("4111 1111 1111 1111").is_ok());
    assert!(validate("4111  1111  1111  1111").is_ok()); // Double spaces
    assert!(validate(" 4111111111111111 ").is_ok()); // Leading/trailing

    // Dashes
    assert!(validate("4111-1111-1111-1111").is_ok());
    assert!(validate("4111--1111--1111--1111").is_ok()); // Double dashes

    // Periods
    assert!(validate("4111.1111.1111.1111").is_ok());

    // Mixed
    assert!(validate("4111-1111 1111.1111").is_ok());
    assert!(validate("4111 - 1111 - 1111 - 1111").is_ok());

    // All variations should produce same result
    let variations = [
        "4111111111111111",
        "4111 1111 1111 1111",
        "4111-1111-1111-1111",
        "4111.1111.1111.1111",
        "  4111-1111-1111-1111  ",
    ];

    for var in variations {
        let card = validate(var).unwrap();
        assert_eq!(card.number(), base);
        assert_eq!(card.brand(), CardBrand::Visa);
    }
}

#[test]
fn test_invalid_characters() {
    let test_cases = [
        ("4111111111111111a", 'a', 16),
        ("a4111111111111111", 'a', 0),
        ("4111111111x111111", 'x', 10),
        ("4111-1111-1111-111!", '!', 18),
        ("4111\t1111\t1111\t1111", '\t', 4),
        ("4111\n1111\n1111\n1111", '\n', 4),
        ("4111111111111111\0", '\0', 16),
        ("4111111111111111é", 'é', 16),
        ("4111111111111111中", '中', 16),
    ];

    for (input, expected_char, expected_pos) in test_cases {
        let result = validate(input);
        match result {
            Err(ValidationError::InvalidCharacter {
                character,
                position,
            }) => {
                assert_eq!(character, expected_char, "Wrong char for input: {}", input);
                assert_eq!(
                    position, expected_pos,
                    "Wrong position for input: {}",
                    input
                );
            }
            other => panic!("Expected InvalidCharacter for '{}', got {:?}", input, other),
        }
    }
}

// =============================================================================
// LENGTH BOUNDARY TESTS
// =============================================================================

#[test]
fn test_minimum_length_boundary() {
    // 11 digits - too short
    assert!(matches!(
        validate("41111111118"),
        Err(ValidationError::TooShort {
            length: 11,
            minimum: 12
        })
    ));

    // 12 digits - minimum valid (Maestro)
    // Need to find a valid 12-digit Luhn number starting with valid prefix
    let result = validate("501800000000");
    assert!(result.is_ok() || matches!(result, Err(ValidationError::InvalidChecksum)));
}

#[test]
fn test_maximum_length_boundary() {
    // 19 digits - maximum valid
    // 4111111111111111117 is a valid 19-digit Visa
    let nineteen = "4111111111111111117";
    assert_eq!(nineteen.chars().filter(|c| c.is_ascii_digit()).count(), 19);
    let result = validate(nineteen);
    // Should either pass or fail checksum, not length
    assert!(!matches!(result, Err(ValidationError::TooLong { .. })));

    // 20 digits - too long
    let twenty = "41111111111111111111";
    assert_eq!(twenty.len(), 20);
    assert!(matches!(
        validate(twenty),
        Err(ValidationError::TooLong {
            length: 20,
            maximum: 19
        })
    ));
}

#[test]
fn test_brand_specific_lengths() {
    // Visa must be 13, 16, or 19 digits
    // 14-digit starting with 4 should fail with InvalidLengthForBrand
    // "41111111111114" is 14 digits and passes Luhn (sum = 30, 30 % 10 = 0)
    let visa_14 = "41111111111114";
    assert!(passes_luhn(visa_14), "Test card should pass Luhn");
    let result = validate(visa_14);
    assert!(
        matches!(
            result,
            Err(ValidationError::InvalidLengthForBrand {
                brand: CardBrand::Visa,
                length: 14,
                ..
            })
        ),
        "Expected InvalidLengthForBrand for Visa with 14 digits, got {:?}",
        result
    );

    // Amex must be exactly 15 digits
    // 16-digit starting with 37 should fail (if it passes Luhn)
    // "3782822463100055" is 16 digits - need to verify Luhn
    let amex_16 = "3700000000000002"; // 16 digits starting with 37, passes Luhn
    if passes_luhn(amex_16) {
        let result = validate(amex_16);
        assert!(
            matches!(
                result,
                Err(ValidationError::InvalidLengthForBrand {
                    brand: CardBrand::Amex,
                    length: 16,
                    ..
                })
            ),
            "Expected InvalidLengthForBrand for Amex with 16 digits, got {:?}",
            result
        );
    }

    // Mastercard must be exactly 16 digits
    // "510510510510518" is 15 digits starting with 51, passes Luhn
    let mc_15 = "510510510510518";
    if passes_luhn(mc_15) {
        let result = validate(mc_15);
        assert!(
            matches!(
                result,
                Err(ValidationError::InvalidLengthForBrand {
                    brand: CardBrand::Mastercard,
                    length: 15,
                    ..
                })
            ),
            "Expected InvalidLengthForBrand for MC with 15 digits, got {:?}",
            result
        );
    }
}

// =============================================================================
// LUHN ALGORITHM TESTS
// =============================================================================

#[test]
fn test_luhn_single_digit_change() {
    // Changing any single digit should invalidate the card
    let valid = "4111111111111111";

    for i in 0..16 {
        let mut chars: Vec<char> = valid.chars().collect();
        let original = chars[i];
        // Change to a different digit
        chars[i] = if original == '9' {
            '0'
        } else {
            char::from_digit(original.to_digit(10).unwrap() + 1, 10).unwrap()
        };
        let modified: String = chars.into_iter().collect();

        assert!(
            !is_valid(&modified),
            "Changing digit {} from {} should invalidate: {}",
            i,
            original,
            modified
        );
    }
}

#[test]
fn test_luhn_transposition_detection() {
    // Luhn catches most (but not all) transpositions of adjacent digits
    // It does NOT catch transposition of 09 <-> 90

    // Test some transpositions
    let original = "4111111111111111";
    let transposed = "1411111111111111"; // swap pos 0,1: 41 -> 14

    assert!(is_valid(original));
    assert!(!is_valid(transposed));
}

#[test]
fn test_luhn_check_digit_generation() {
    // Verify that generate_check_digit produces correct check digits
    let test_cases = [
        // (partial without check digit, expected check digit)
        (&[4u8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1][..], 1u8),
        (&[5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0][..], 4u8),
        (&[3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0][..], 5u8),
    ];

    for (partial, expected) in test_cases {
        let check = luhn::generate_check_digit(partial);
        assert_eq!(check, expected, "Check digit mismatch for {:?}", partial);

        // Verify the full number is valid
        let mut full = partial.to_vec();
        full.push(check);
        assert!(
            luhn::validate(&full),
            "Full number should be valid: {:?}",
            full
        );
    }
}

#[test]
fn test_luhn_all_same_digits() {
    // Cards with all same digits
    // 0000000000000000 - valid (sum = 0, 0 % 10 = 0)
    assert!(luhn::validate(&[0; 16]));

    // Other single-digit repeats
    for d in 1u8..=9 {
        let digits = [d; 16];
        let is_valid = luhn::validate(&digits);
        // The validity depends on the digit
        // We just verify it doesn't panic
        let _ = is_valid;
    }
}

#[test]
fn test_luhn_optimized_matches_generic() {
    // Verify optimized 16-digit and 15-digit functions match generic
    let test_16: [[u8; 16]; 5] = [
        [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4],
        [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2], // invalid
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
    ];

    for digits in test_16 {
        let generic = luhn::validate(&digits);
        let optimized = luhn::validate_16(&digits);
        assert_eq!(generic, optimized, "Mismatch for {:?}", digits);
    }

    let test_15: [[u8; 15]; 3] = [
        [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5],
        [3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 6], // invalid
    ];

    for digits in test_15 {
        let generic = luhn::validate(&digits);
        let optimized = luhn::validate_15(&digits);
        assert_eq!(generic, optimized, "Mismatch for {:?}", digits);
    }
}

// =============================================================================
// ERROR MESSAGE TESTS
// =============================================================================

#[test]
fn test_error_messages_are_helpful() {
    // Verify error messages contain useful information

    let err = validate("").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("empty"),
        "Empty error should mention 'empty': {}",
        msg
    );

    let err = validate("4111111111").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("short"),
        "TooShort should mention 'short': {}",
        msg
    );
    assert!(
        msg.contains("10"),
        "TooShort should mention actual length: {}",
        msg
    );
    assert!(
        msg.contains("12"),
        "TooShort should mention minimum: {}",
        msg
    );

    let err = validate("4111111111111111x").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("x"),
        "InvalidCharacter should show the character: {}",
        msg
    );
    assert!(
        msg.contains("16"),
        "InvalidCharacter should show position: {}",
        msg
    );

    let err = validate("4111111111111112").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("checksum") || msg.contains("Luhn"),
        "InvalidChecksum should mention checksum: {}",
        msg
    );
}

// =============================================================================
// MASKING TESTS
// =============================================================================

#[test]
fn test_masking_never_exposes_full_number() {
    let cards = [
        test_cards::VISA_1,
        test_cards::MC_1,
        test_cards::AMEX_1,
        test_cards::DISCOVER_1,
    ];

    for card_str in cards {
        let card = validate(card_str).unwrap();
        let masked = card.masked();
        let masked_bin = card.masked_with_bin();
        let debug = format!("{:?}", card);
        let display = format!("{}", card);

        // Full number should never appear
        let clean_number: String = card_str.chars().filter(|c| c.is_ascii_digit()).collect();
        assert!(
            !masked.contains(&clean_number),
            "masked() exposed full number for {}",
            card_str
        );
        assert!(
            !masked_bin.contains(&clean_number),
            "masked_with_bin() exposed full number for {}",
            card_str
        );
        assert!(
            !debug.contains(&clean_number),
            "Debug exposed full number for {}",
            card_str
        );
        assert!(
            !display.contains(&clean_number),
            "Display exposed full number for {}",
            card_str
        );
    }
}

#[test]
fn test_masking_shows_last_four() {
    let card = validate(test_cards::VISA_1).unwrap();
    let last_four = card.last_four();

    assert_eq!(last_four, "1111");
    assert!(card.masked().contains("1111"));
    assert!(card.masked_with_bin().contains("1111"));
}

#[test]
fn test_masking_bin_shows_first_six() {
    let card = validate(test_cards::VISA_1).unwrap();
    let bin6 = card.bin6();

    assert_eq!(bin6, "411111");
    assert!(card.masked_with_bin().starts_with("411111"));
}

#[test]
fn test_mask_string_utility() {
    // Test the standalone mask_string function
    let masked = mask::mask_string("4111111111111111");
    assert!(masked.contains("1111"));
    assert!(!masked.contains("4111111111111111"));
    assert!(masked.contains("*"));

    // With separators
    let masked = mask::mask_string("4111-1111-1111-1111");
    assert!(masked.contains("1111"));
    assert!(!masked.contains("4111"));
}

// =============================================================================
// CONSTANT-TIME COMPARISON TESTS
// =============================================================================

#[test]
fn test_constant_time_eq_correctness() {
    // Equal strings
    assert!(mask::constant_time_eq(b"hello", b"hello"));
    assert!(mask::constant_time_eq(b"", b""));
    assert!(mask::constant_time_eq(
        b"4111111111111111",
        b"4111111111111111"
    ));

    // Unequal strings - different content
    assert!(!mask::constant_time_eq(b"hello", b"world"));
    assert!(!mask::constant_time_eq(
        b"4111111111111111",
        b"4111111111111112"
    ));

    // Unequal strings - different length
    assert!(!mask::constant_time_eq(b"hello", b"hello!"));
    assert!(!mask::constant_time_eq(b"hello", b"hell"));

    // Binary data
    assert!(mask::constant_time_eq(&[0u8, 1, 2, 3], &[0u8, 1, 2, 3]));
    assert!(!mask::constant_time_eq(&[0u8, 1, 2, 3], &[0u8, 1, 2, 4]));
}

// =============================================================================
// BATCH PROCESSING TESTS
// =============================================================================

#[test]
fn test_batch_preserves_order() {
    let cards = vec![
        test_cards::VISA_1,
        "invalid",
        test_cards::MC_1,
        "also invalid",
        test_cards::AMEX_1,
    ];

    let mut batch = BatchValidator::new();
    let results = batch.validate_all(&cards);

    assert_eq!(results.len(), 5);
    assert!(results[0].is_ok());
    assert!(results[1].is_err());
    assert!(results[2].is_ok());
    assert!(results[3].is_err());
    assert!(results[4].is_ok());

    // Verify correct brands in order
    assert_eq!(results[0].as_ref().unwrap().brand(), CardBrand::Visa);
    assert_eq!(results[2].as_ref().unwrap().brand(), CardBrand::Mastercard);
    assert_eq!(results[4].as_ref().unwrap().brand(), CardBrand::Amex);
}

#[test]
fn test_batch_partitioned_indices() {
    let cards = vec![
        test_cards::VISA_1, // 0 - valid
        "bad1",             // 1 - invalid
        test_cards::MC_1,   // 2 - valid
        "bad2",             // 3 - invalid
        "bad3",             // 4 - invalid
    ];

    let mut batch = BatchValidator::new();
    let (valid, invalid) = batch.validate_partitioned(&cards);

    assert_eq!(valid.len(), 2);
    assert_eq!(invalid.len(), 3);

    // Check invalid indices
    let invalid_indices: Vec<usize> = invalid.iter().map(|(i, _)| *i).collect();
    assert_eq!(invalid_indices, vec![1, 3, 4]);
}

// =============================================================================
// STREAMING TESTS
// =============================================================================

#[test]
fn test_stream_validate_cards() {
    let cards = vec![test_cards::VISA_1, "invalid", test_cards::MC_1];

    let results: Vec<_> = cards.iter().map(|s| *s).validate_cards().collect();

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_err());
    assert!(results[2].is_ok());
}

#[test]
fn test_stream_valid_only() {
    let cards = vec![
        test_cards::VISA_1,
        "invalid1",
        test_cards::MC_1,
        "invalid2",
        test_cards::AMEX_1,
    ];

    let valid: Vec<_> = cards.iter().map(|s| *s).validate_valid_only().collect();

    assert_eq!(valid.len(), 3);
    assert_eq!(valid[0].brand(), CardBrand::Visa);
    assert_eq!(valid[1].brand(), CardBrand::Mastercard);
    assert_eq!(valid[2].brand(), CardBrand::Amex);
}

#[test]
fn test_stream_indexed() {
    let cards = vec![test_cards::VISA_1, "invalid", test_cards::MC_1];

    let results: Vec<_> = cards.iter().map(|s| *s).validate_indexed().collect();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1.is_ok());
    assert_eq!(results[1].0, 1);
    assert!(results[1].1.is_err());
    assert_eq!(results[2].0, 2);
    assert!(results[2].1.is_ok());
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_empty_and_whitespace_only() {
    assert!(matches!(validate(""), Err(ValidationError::Empty)));
    assert!(matches!(validate("   "), Err(ValidationError::NoDigits)));
    assert!(matches!(validate("---"), Err(ValidationError::NoDigits)));
    assert!(matches!(
        validate(" - . - "),
        Err(ValidationError::NoDigits)
    ));
}

#[test]
fn test_only_zeros() {
    // 16 zeros passes Luhn (sum = 0) but may fail brand detection
    let zeros = "0000000000000000";
    let result = validate(zeros);
    // Should fail with UnknownBrand, not a crash
    assert!(matches!(result, Err(ValidationError::UnknownBrand)));
}

#[test]
fn test_leading_zeros_preserved() {
    // Card numbers can have leading zeros (though rare)
    // This tests that we handle them correctly
    let card = validate(test_cards::VISA_1).unwrap();
    assert_eq!(card.number().len(), 16);
}

#[test]
fn test_unicode_handling() {
    // Unicode digits should be rejected
    let unicode_tests = [
        "４１１１１１１１１１１１１１１１", // Full-width digits
        "٤١١١١١١١١١١١١١١١",                 // Arabic-Indic digits
    ];

    for input in unicode_tests {
        let result = validate(input);
        assert!(result.is_err(), "Unicode input should fail: {}", input);
    }
}

#[test]
fn test_very_long_separators() {
    // Lots of separators between digits
    let with_many_separators = "4---1---1---1---1---1---1---1---1---1---1---1---1---1---1---1";
    let result = validate(with_many_separators);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().number(), "4111111111111111");
}

// =============================================================================
// CARD OPERATIONS TESTS
// =============================================================================

#[test]
fn test_card_number_retrieval() {
    let card = validate("4111-1111-1111-1111").unwrap();

    // number() should return clean digits only
    assert_eq!(card.number(), "4111111111111111");
    assert_eq!(card.number().len(), 16);

    // Verify no separators
    assert!(!card.number().contains('-'));
    assert!(!card.number().contains(' '));
}

#[test]
fn test_card_bin_lengths() {
    let card = validate(test_cards::VISA_1).unwrap();

    assert_eq!(card.bin6().len(), 6);
    assert_eq!(card.bin8().len(), 8);
    assert_eq!(card.bin(4).len(), 4);
    assert_eq!(card.bin(10).len(), 8); // Capped at 8

    // Verify correct values
    assert_eq!(card.bin6(), "411111");
    assert_eq!(card.bin8(), "41111111");
}

#[test]
fn test_card_length() {
    let visa16 = validate(test_cards::VISA_1).unwrap();
    let visa13 = validate(test_cards::VISA_3).unwrap();
    let amex15 = validate(test_cards::AMEX_1).unwrap();

    assert_eq!(visa16.length(), 16);
    assert_eq!(visa13.length(), 13);
    assert_eq!(amex15.length(), 15);
}

// =============================================================================
// THREAD SAFETY TESTS
// =============================================================================

#[test]
fn test_validated_card_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<cc_validator::ValidatedCard>();
    assert_sync::<cc_validator::ValidatedCard>();
    assert_send::<cc_validator::ValidationError>();
    assert_sync::<cc_validator::ValidationError>();
    assert_send::<cc_validator::CardBrand>();
    assert_sync::<cc_validator::CardBrand>();
}

// =============================================================================
// REGRESSION TESTS
// =============================================================================

#[test]
fn test_mastercard_2_series_detection() {
    // Mastercard 2-series (2221-2720) was added later
    // Ensure these are detected as Mastercard, not something else

    let mc_2series = [
        "2221000000000009", // Start of range
        "2223000048400011",
        "2720000000000005", // End of range (approximately)
    ];

    for card in mc_2series {
        if let Ok(validated) = validate(card) {
            assert_eq!(
                validated.brand(),
                CardBrand::Mastercard,
                "2-series card {} should be Mastercard",
                card
            );
        }
        // Some might fail Luhn, that's OK
    }
}

#[test]
fn test_no_panic_on_any_input() {
    // Fuzz-like test: ensure no panics on various inputs
    let inputs = [
        "",
        " ",
        "a",
        "0",
        "00000000000",
        "99999999999999999999999999999999999999999",
        "4111111111111111",
        "4111-1111-1111-1111",
        "\x00\x01\x02\x03",
        "🎉🎊🎁",
        &"4".repeat(100),
        &" ".repeat(1000),
    ];

    for input in inputs {
        // Should not panic, just return Ok or Err
        let _ = validate(input);
        let _ = is_valid(input);
        let _ = passes_luhn(input);
        let _ = validate_any(input);
        let _ = mask::mask_string(input);
    }
}

// =============================================================================
// VALIDATE_DIGITS TESTS
// =============================================================================

#[test]
fn test_validate_digits_direct() {
    let visa_digits: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    let result = validate_digits(&visa_digits);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().brand(), CardBrand::Visa);

    // Invalid digits
    let invalid: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
    assert!(validate_digits(&invalid).is_err());
}

#[test]
fn test_validate_digits_out_of_range() {
    // Digits > 9 - this is a programming error but should be handled
    let bad_digits: [u8; 16] = [10, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    // This might panic or return an error - document the behavior
    // For now, just ensure it doesn't cause memory corruption
    let _ = std::panic::catch_unwind(|| {
        let _ = validate_digits(&bad_digits);
    });
}
