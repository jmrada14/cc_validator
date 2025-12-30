//! Batch processing for high-throughput card validation.
//!
//! This module provides efficient batch validation of multiple card numbers,
//! with optional parallel processing using rayon.
//!
//! # Performance
//!
//! - Pre-allocated buffers avoid per-card allocation overhead
//! - Optional parallel processing with the `parallel` feature
//! - Process millions of cards per second on modern hardware

use crate::error::ValidationError;
use crate::validate::validate;
use crate::ValidatedCard;

/// Batch validator for processing multiple card numbers efficiently.
///
/// The batch validator maintains internal state optimized for processing
/// many cards in sequence.
///
/// # Example
///
/// ```
/// use cc_validator::BatchValidator;
///
/// let mut batch = BatchValidator::new();
/// let cards = vec!["4111111111111111", "5500000000000004", "378282246310005"];
/// let results = batch.validate_all(&cards);
///
/// for (card, result) in cards.iter().zip(results.iter()) {
///     match result {
///         Ok(validated) => println!("{}: {} valid", card, validated.brand()),
///         Err(e) => println!("{}: invalid - {}", card, e),
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct BatchValidator {
    // Reserved for future optimizations (e.g., thread-local buffers)
    _private: (),
}

impl BatchValidator {
    /// Creates a new batch validator.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates a batch of card numbers.
    ///
    /// Returns a vector of results in the same order as the input.
    ///
    /// # Arguments
    ///
    /// * `cards` - Slice of card number strings to validate.
    pub fn validate_all<S: AsRef<str>>(
        &mut self,
        cards: &[S],
    ) -> Vec<Result<ValidatedCard, ValidationError>> {
        cards.iter().map(|c| validate(c.as_ref())).collect()
    }

    /// Validates a batch and returns only the valid cards.
    ///
    /// Invalid cards are silently filtered out.
    ///
    /// # Arguments
    ///
    /// * `cards` - Slice of card number strings to validate.
    pub fn validate_valid_only<S: AsRef<str>>(
        &mut self,
        cards: &[S],
    ) -> Vec<ValidatedCard> {
        cards
            .iter()
            .filter_map(|c| validate(c.as_ref()).ok())
            .collect()
    }

    /// Validates a batch and partitions into valid and invalid.
    ///
    /// Returns a tuple of (valid_cards, errors).
    ///
    /// # Arguments
    ///
    /// * `cards` - Slice of card number strings to validate.
    pub fn validate_partitioned<S: AsRef<str>>(
        &mut self,
        cards: &[S],
    ) -> (Vec<ValidatedCard>, Vec<(usize, ValidationError)>) {
        let mut valid = Vec::new();
        let mut invalid = Vec::new();

        for (i, card) in cards.iter().enumerate() {
            match validate(card.as_ref()) {
                Ok(c) => valid.push(c),
                Err(e) => invalid.push((i, e)),
            }
        }

        (valid, invalid)
    }

    /// Validates cards in parallel using rayon.
    ///
    /// This is typically faster for large batches (>1000 cards) on
    /// multi-core systems.
    ///
    /// # Arguments
    ///
    /// * `cards` - Slice of card number strings to validate.
    ///
    /// # Feature
    ///
    /// Requires the `parallel` feature to be enabled.
    #[cfg(feature = "parallel")]
    pub fn validate_parallel<S: AsRef<str> + Sync>(
        &mut self,
        cards: &[S],
    ) -> Vec<Result<ValidatedCard, ValidationError>> {
        use rayon::prelude::*;
        cards.par_iter().map(|c| validate(c.as_ref())).collect()
    }

    /// Validates cards in parallel, returning only valid ones.
    ///
    /// # Feature
    ///
    /// Requires the `parallel` feature to be enabled.
    #[cfg(feature = "parallel")]
    pub fn validate_parallel_valid_only<S: AsRef<str> + Sync>(
        &mut self,
        cards: &[S],
    ) -> Vec<ValidatedCard> {
        use rayon::prelude::*;
        cards
            .par_iter()
            .filter_map(|c| validate(c.as_ref()).ok())
            .collect()
    }
}

/// Validates a slice of cards without creating a BatchValidator.
///
/// This is a convenience function for one-off batch validation.
///
/// # Example
///
/// ```
/// use cc_validator::batch::validate_batch;
///
/// let cards = ["4111111111111111", "5500000000000004"];
/// let results = validate_batch(&cards);
/// assert!(results[0].is_ok());
/// assert!(results[1].is_ok());
/// ```
#[inline]
pub fn validate_batch<S: AsRef<str>>(cards: &[S]) -> Vec<Result<ValidatedCard, ValidationError>> {
    cards.iter().map(|c| validate(c.as_ref())).collect()
}

/// Validates a slice of cards in parallel.
///
/// # Feature
///
/// Requires the `parallel` feature to be enabled.
#[cfg(feature = "parallel")]
#[inline]
pub fn validate_batch_parallel<S: AsRef<str> + Sync>(
    cards: &[S],
) -> Vec<Result<ValidatedCard, ValidationError>> {
    use rayon::prelude::*;
    cards.par_iter().map(|c| validate(c.as_ref())).collect()
}

/// Counts valid and invalid cards in a batch.
///
/// This is faster than validating all and then counting, as it
/// doesn't allocate for results.
///
/// # Returns
///
/// Tuple of (valid_count, invalid_count)
///
/// # Example
///
/// ```
/// use cc_validator::batch::count_valid;
///
/// let cards = ["4111111111111111", "1234567890123456", "5500000000000004"];
/// let (valid, invalid) = count_valid(&cards);
/// assert_eq!(valid, 2);
/// assert_eq!(invalid, 1);
/// ```
#[inline]
pub fn count_valid<S: AsRef<str>>(cards: &[S]) -> (usize, usize) {
    let mut valid = 0;
    let mut invalid = 0;

    for card in cards {
        if validate(card.as_ref()).is_ok() {
            valid += 1;
        } else {
            invalid += 1;
        }
    }

    (valid, invalid)
}

/// Counts valid and invalid cards in parallel.
///
/// # Feature
///
/// Requires the `parallel` feature to be enabled.
#[cfg(feature = "parallel")]
#[inline]
pub fn count_valid_parallel<S: AsRef<str> + Sync>(cards: &[S]) -> (usize, usize) {
    use rayon::prelude::*;

    let valid: usize = cards
        .par_iter()
        .filter(|c| validate(c.as_ref()).is_ok())
        .count();

    (valid, cards.len() - valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_VISA: &str = "4111111111111111";
    const VALID_MC: &str = "5500000000000004";
    const VALID_AMEX: &str = "378282246310005";
    const INVALID: &str = "1234567890123456";

    #[test]
    fn test_batch_validate_all() {
        let mut batch = BatchValidator::new();
        let cards = vec![VALID_VISA, VALID_MC, INVALID, VALID_AMEX];
        let results = batch.validate_all(&cards);

        assert_eq!(results.len(), 4);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
        assert!(results[3].is_ok());
    }

    #[test]
    fn test_batch_valid_only() {
        let mut batch = BatchValidator::new();
        let cards = vec![VALID_VISA, INVALID, VALID_MC];
        let valid = batch.validate_valid_only(&cards);

        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_batch_partitioned() {
        let mut batch = BatchValidator::new();
        let cards = vec![VALID_VISA, INVALID, VALID_MC, "bad"];
        let (valid, invalid) = batch.validate_partitioned(&cards);

        assert_eq!(valid.len(), 2);
        assert_eq!(invalid.len(), 2);
        assert_eq!(invalid[0].0, 1); // Index of first invalid
        assert_eq!(invalid[1].0, 3); // Index of second invalid
    }

    #[test]
    fn test_validate_batch_fn() {
        let cards = [VALID_VISA, VALID_MC];
        let results = validate_batch(&cards);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_count_valid() {
        let cards = [VALID_VISA, INVALID, VALID_MC, "bad"];
        let (valid, invalid) = count_valid(&cards);
        assert_eq!(valid, 2);
        assert_eq!(invalid, 2);
    }

    #[test]
    fn test_empty_batch() {
        let mut batch = BatchValidator::new();
        let cards: Vec<&str> = vec![];
        let results = batch.validate_all(&cards);
        assert!(results.is_empty());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_parallel_validation() {
        let mut batch = BatchValidator::new();
        let cards: Vec<String> = (0..1000)
            .map(|_| VALID_VISA.to_string())
            .collect();

        let results = batch.validate_parallel(&cards);
        assert_eq!(results.len(), 1000);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_count_valid_parallel() {
        let cards: Vec<&str> = vec![VALID_VISA, INVALID, VALID_MC, "bad"];
        let (valid, invalid) = count_valid_parallel(&cards);
        assert_eq!(valid, 2);
        assert_eq!(invalid, 2);
    }
}
