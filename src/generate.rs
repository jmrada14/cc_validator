//! Credit card number generation for testing purposes.
//!
//! This module generates valid credit card numbers that pass Luhn validation.
//! These numbers are intended for testing only and are not real card numbers.
//!
//! # Example
//!
//! ```
//! use cc_validator::generate::{generate_card_deterministic, CardGenerator};
//! use cc_validator::CardBrand;
//!
//! // Generate a deterministic Visa card (no randomness required)
//! let card_number = generate_card_deterministic(CardBrand::Visa);
//! assert!(card_number.starts_with("4"));
//! assert!(cc_validator::is_valid(&card_number));
//!
//! // Use builder pattern
//! let card = CardGenerator::new(CardBrand::Mastercard).generate_deterministic();
//! assert!(card.starts_with("51"));
//! ```
//!
//! # Security Note
//!
//! Generated card numbers are mathematically valid (pass Luhn) but are not
//! connected to real accounts. They should only be used for testing.

use crate::luhn;
use crate::CardBrand;

#[cfg(feature = "generate")]
use rand::Rng;

/// Default prefixes for each card brand.
const VISA_PREFIX: &str = "4";
const MASTERCARD_PREFIX: &str = "51";
const AMEX_PREFIX: &str = "34";
const DISCOVER_PREFIX: &str = "6011";
const DINERS_PREFIX: &str = "36";
const JCB_PREFIX: &str = "3528";
const UNIONPAY_PREFIX: &str = "62";
const MAESTRO_PREFIX: &str = "50";
const MIR_PREFIX: &str = "2200";
const RUPAY_PREFIX: &str = "81";
const VERVE_PREFIX: &str = "506";
const ELO_PREFIX: &str = "509";
const TROY_PREFIX: &str = "9792";
const BCCARD_PREFIX: &str = "94";

/// Default length for each card brand.
const fn default_length(brand: CardBrand) -> usize {
    match brand {
        CardBrand::Visa => 16,
        CardBrand::Mastercard => 16,
        CardBrand::Amex => 15,
        CardBrand::Discover => 16,
        CardBrand::DinersClub => 14,
        CardBrand::Jcb => 16,
        CardBrand::UnionPay => 16,
        CardBrand::Maestro => 16,
        CardBrand::Mir => 16,
        CardBrand::RuPay => 16,
        CardBrand::Verve => 16,
        CardBrand::Elo => 16,
        CardBrand::Troy => 16,
        CardBrand::BcCard => 16,
    }
}

/// Returns the default prefix for a card brand.
pub const fn prefix_for_brand(brand: CardBrand) -> &'static str {
    match brand {
        CardBrand::Visa => VISA_PREFIX,
        CardBrand::Mastercard => MASTERCARD_PREFIX,
        CardBrand::Amex => AMEX_PREFIX,
        CardBrand::Discover => DISCOVER_PREFIX,
        CardBrand::DinersClub => DINERS_PREFIX,
        CardBrand::Jcb => JCB_PREFIX,
        CardBrand::UnionPay => UNIONPAY_PREFIX,
        CardBrand::Maestro => MAESTRO_PREFIX,
        CardBrand::Mir => MIR_PREFIX,
        CardBrand::RuPay => RUPAY_PREFIX,
        CardBrand::Verve => VERVE_PREFIX,
        CardBrand::Elo => ELO_PREFIX,
        CardBrand::Troy => TROY_PREFIX,
        CardBrand::BcCard => BCCARD_PREFIX,
    }
}

/// Generates a valid card number for the given brand using random digits.
///
/// Requires the `generate` feature (which enables the `rand` dependency).
///
/// # Example
///
/// ```
/// use cc_validator::generate::generate_card;
/// use cc_validator::CardBrand;
///
/// let card = generate_card(CardBrand::Visa);
/// assert!(cc_validator::is_valid(&card));
/// ```
#[cfg(feature = "generate")]
pub fn generate_card(brand: CardBrand) -> String {
    let prefix = prefix_for_brand(brand);
    let length = default_length(brand);
    generate_card_with_prefix(prefix, length)
}

/// Generates a valid card number with the given prefix and length.
///
/// Requires the `generate` feature.
///
/// # Arguments
///
/// * `prefix` - The starting digits (BIN/IIN prefix)
/// * `length` - Total length of the card number
///
/// # Panics
///
/// Panics if prefix length >= total length.
///
/// # Example
///
/// ```
/// use cc_validator::generate::generate_card_with_prefix;
///
/// let card = generate_card_with_prefix("411111", 16);
/// assert!(card.starts_with("411111"));
/// assert_eq!(card.len(), 16);
/// assert!(cc_validator::is_valid(&card));
/// ```
#[cfg(feature = "generate")]
pub fn generate_card_with_prefix(prefix: &str, length: usize) -> String {
    let mut rng = rand::thread_rng();
    generate_card_with_rng(prefix, length, &mut rng)
}

/// Generates a valid card number using a provided RNG.
///
/// This is useful for reproducible test generation with seeded RNGs.
#[cfg(feature = "generate")]
pub fn generate_card_with_rng<R: Rng>(prefix: &str, length: usize, rng: &mut R) -> String {
    assert!(
        prefix.len() < length,
        "Prefix length must be less than total length"
    );

    // Convert prefix to digits
    let mut digits: Vec<u8> = prefix
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();

    // Fill with random digits (except last one which will be check digit)
    while digits.len() < length - 1 {
        digits.push(rng.gen_range(0..10));
    }

    // Calculate and append check digit
    let check_digit = luhn::generate_check_digit(&digits);
    digits.push(check_digit);

    // Convert to string
    digits.iter().map(|&d| (b'0' + d) as char).collect()
}

/// Generates a valid card number deterministically (no randomness).
///
/// This version doesn't require the `generate` feature and produces
/// the same output for the same inputs. Useful for tests.
///
/// # Example
///
/// ```
/// use cc_validator::generate::generate_card_deterministic;
/// use cc_validator::CardBrand;
///
/// let card = generate_card_deterministic(CardBrand::Visa);
/// assert!(cc_validator::is_valid(&card));
/// // Same input always produces same output
/// assert_eq!(card, generate_card_deterministic(CardBrand::Visa));
/// ```
pub fn generate_card_deterministic(brand: CardBrand) -> String {
    let prefix = prefix_for_brand(brand);
    let length = default_length(brand);
    generate_card_deterministic_with_prefix(prefix, length)
}

/// Generates a valid card number deterministically with a custom prefix.
///
/// Fills middle digits with zeros and calculates a valid check digit.
pub fn generate_card_deterministic_with_prefix(prefix: &str, length: usize) -> String {
    assert!(
        prefix.len() < length,
        "Prefix length must be less than total length"
    );

    // Convert prefix to digits
    let mut digits: Vec<u8> = prefix
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();

    // Fill with zeros (deterministic)
    while digits.len() < length - 1 {
        digits.push(0);
    }

    // Calculate and append check digit
    let check_digit = luhn::generate_check_digit(&digits);
    digits.push(check_digit);

    // Convert to string
    digits.iter().map(|&d| (b'0' + d) as char).collect()
}

/// Generates multiple valid card numbers for the given brand.
///
/// Requires the `generate` feature.
#[cfg(feature = "generate")]
pub fn generate_cards(brand: CardBrand, count: usize) -> Vec<String> {
    (0..count).map(|_| generate_card(brand)).collect()
}

/// Generates a test card that matches a specific pattern.
///
/// Pattern uses 'X' for random digits, e.g., "4111-XXXX-XXXX-XXXX".
/// Dashes and spaces are stripped from the output.
///
/// Requires the `generate` feature.
///
/// # Example
///
/// ```
/// use cc_validator::generate::generate_from_pattern;
///
/// let card = generate_from_pattern("4111-XXXX-XXXX-XXXX");
/// assert!(card.starts_with("4111"));
/// assert_eq!(card.len(), 16);
/// ```
#[cfg(feature = "generate")]
pub fn generate_from_pattern(pattern: &str) -> String {
    let mut rng = rand::thread_rng();

    // Extract digits and X placeholders
    let mut digits: Vec<u8> = Vec::new();
    let mut has_check_placeholder = false;

    for c in pattern.chars() {
        match c {
            '0'..='9' => digits.push((c as u8) - b'0'),
            'X' | 'x' => {
                digits.push(rng.gen_range(0..10));
                has_check_placeholder = true;
            }
            ' ' | '-' | '.' => continue, // Skip separators
            _ => continue,
        }
    }

    // If the last digit was a placeholder, recalculate check digit
    if has_check_placeholder && !digits.is_empty() {
        digits.pop(); // Remove the random last digit
        let check_digit = luhn::generate_check_digit(&digits);
        digits.push(check_digit);
    }

    digits.iter().map(|&d| (b'0' + d) as char).collect()
}

/// Card generator builder for more complex generation scenarios.
#[derive(Debug, Clone)]
pub struct CardGenerator {
    prefix: String,
    length: usize,
}

impl CardGenerator {
    /// Creates a new card generator for the given brand.
    pub fn new(brand: CardBrand) -> Self {
        Self {
            prefix: prefix_for_brand(brand).to_string(),
            length: default_length(brand),
        }
    }

    /// Creates a new card generator with a custom prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            length: 16,
        }
    }

    /// Sets the card length.
    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Sets the prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Generates a card number deterministically.
    pub fn generate_deterministic(&self) -> String {
        generate_card_deterministic_with_prefix(&self.prefix, self.length)
    }

    /// Generates a card number with randomness.
    #[cfg(feature = "generate")]
    pub fn generate(&self) -> String {
        generate_card_with_prefix(&self.prefix, self.length)
    }

    /// Generates multiple card numbers.
    #[cfg(feature = "generate")]
    pub fn generate_many(&self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.generate()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::is_valid;

    #[test]
    fn test_generate_deterministic_visa() {
        let card = generate_card_deterministic(CardBrand::Visa);
        assert!(card.starts_with("4"));
        assert_eq!(card.len(), 16);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_generate_deterministic_mastercard() {
        let card = generate_card_deterministic(CardBrand::Mastercard);
        assert!(card.starts_with("51"));
        assert_eq!(card.len(), 16);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_generate_deterministic_amex() {
        let card = generate_card_deterministic(CardBrand::Amex);
        assert!(card.starts_with("34"));
        assert_eq!(card.len(), 15);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_generate_deterministic_discover() {
        let card = generate_card_deterministic(CardBrand::Discover);
        assert!(card.starts_with("6011"));
        assert_eq!(card.len(), 16);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_generate_deterministic_is_reproducible() {
        let card1 = generate_card_deterministic(CardBrand::Visa);
        let card2 = generate_card_deterministic(CardBrand::Visa);
        assert_eq!(card1, card2);
    }

    #[test]
    fn test_generate_deterministic_with_prefix() {
        let card = generate_card_deterministic_with_prefix("411111", 16);
        assert!(card.starts_with("411111"));
        assert_eq!(card.len(), 16);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_card_generator_builder() {
        let gen = CardGenerator::new(CardBrand::Visa).length(19);
        let card = gen.generate_deterministic();
        assert!(card.starts_with("4"));
        assert_eq!(card.len(), 19);
        assert!(is_valid(&card));
    }

    #[test]
    fn test_card_generator_with_prefix() {
        let gen = CardGenerator::with_prefix("123456").length(16);
        let card = gen.generate_deterministic();
        assert!(card.starts_with("123456"));
        assert_eq!(card.len(), 16);
        // May not be valid brand but should pass Luhn
        assert!(crate::passes_luhn(&card));
    }

    #[test]
    fn test_prefix_for_all_brands() {
        let brands = [
            CardBrand::Visa,
            CardBrand::Mastercard,
            CardBrand::Amex,
            CardBrand::Discover,
            CardBrand::DinersClub,
            CardBrand::Jcb,
            CardBrand::UnionPay,
            CardBrand::Maestro,
            CardBrand::Mir,
            CardBrand::RuPay,
            CardBrand::Verve,
            CardBrand::Elo,
            CardBrand::Troy,
            CardBrand::BcCard,
        ];

        for brand in brands {
            let card = generate_card_deterministic(brand);
            assert!(
                is_valid(&card) || crate::passes_luhn(&card),
                "Generated card for {:?} should pass validation: {}",
                brand,
                card
            );
        }
    }

    #[cfg(feature = "generate")]
    mod random_tests {
        use super::*;

        #[test]
        fn test_generate_card_visa() {
            let card = generate_card(CardBrand::Visa);
            assert!(card.starts_with("4"));
            assert_eq!(card.len(), 16);
            assert!(is_valid(&card));
        }

        #[test]
        fn test_generate_card_amex() {
            let card = generate_card(CardBrand::Amex);
            assert!(card.starts_with("34") || card.starts_with("37"));
            assert_eq!(card.len(), 15);
            assert!(is_valid(&card));
        }

        #[test]
        fn test_generate_cards_multiple() {
            let cards = generate_cards(CardBrand::Visa, 10);
            assert_eq!(cards.len(), 10);
            for card in cards {
                assert!(is_valid(&card));
            }
        }

        #[test]
        fn test_generate_from_pattern() {
            let card = generate_from_pattern("4111-XXXX-XXXX-XXXX");
            assert!(card.starts_with("4111"));
            assert_eq!(card.len(), 16);
            assert!(is_valid(&card));
        }

        #[test]
        fn test_generate_cards_are_unique() {
            let cards = generate_cards(CardBrand::Visa, 100);
            let mut unique = std::collections::HashSet::new();
            for card in &cards {
                unique.insert(card.clone());
            }
            // With 100 random cards, we should have at least 90 unique
            assert!(unique.len() >= 90);
        }

        #[test]
        fn test_card_generator_random() {
            let gen = CardGenerator::new(CardBrand::Mastercard);
            let cards: Vec<_> = (0..5).map(|_| gen.generate()).collect();
            for card in cards {
                assert!(is_valid(&card));
            }
        }
    }
}
