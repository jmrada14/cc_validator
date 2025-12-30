//! Streaming validation for processing cards as they arrive.
//!
//! This module provides iterator adapters for validating card numbers
//! in a streaming fashion, useful for processing large files or network
//! streams without loading everything into memory.
//!
//! # Example
//!
//! ```
//! use cc_validator::stream::ValidateExt;
//!
//! let cards = vec!["4111111111111111", "5500000000000004", "invalid"];
//! let valid_count = cards.iter()
//!     .map(|s| *s)
//!     .validate_cards()
//!     .filter(|r| r.is_ok())
//!     .count();
//!
//! assert_eq!(valid_count, 2);
//! ```

use crate::error::ValidationError;
use crate::validate::validate;
use crate::ValidatedCard;

/// A streaming validator that wraps an iterator of card number strings.
///
/// This struct is created by the `validate_cards` method on iterators.
#[derive(Debug, Clone)]
pub struct ValidateStream<I> {
    inner: I,
}

impl<I> ValidateStream<I> {
    /// Creates a new ValidateStream wrapping the given iterator.
    #[inline]
    pub fn new(inner: I) -> Self {
        Self { inner }
    }

    /// Consumes the stream and returns the inner iterator.
    #[inline]
    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl<I, S> Iterator for ValidateStream<I>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Item = Result<ValidatedCard, ValidationError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| validate(s.as_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, S> ExactSizeIterator for ValidateStream<I>
where
    I: ExactSizeIterator<Item = S>,
    S: AsRef<str>,
{
}

impl<I, S> DoubleEndedIterator for ValidateStream<I>
where
    I: DoubleEndedIterator<Item = S>,
    S: AsRef<str>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|s| validate(s.as_ref()))
    }
}

/// A streaming validator that only yields valid cards.
///
/// Invalid cards are silently skipped.
#[derive(Debug, Clone)]
pub struct ValidOnlyStream<I> {
    inner: I,
}

impl<I> ValidOnlyStream<I> {
    /// Creates a new ValidOnlyStream wrapping the given iterator.
    #[inline]
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I, S> Iterator for ValidOnlyStream<I>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Item = ValidatedCard;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(s) => {
                    if let Ok(card) = validate(s.as_ref()) {
                        return Some(card);
                    }
                    // Invalid card, continue to next
                }
                None => return None,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.inner.size_hint();
        (0, upper) // Lower bound is 0 since all might be invalid
    }
}

/// A streaming validator that yields cards with their original index.
///
/// Useful when you need to track which cards in a batch were valid/invalid.
#[derive(Debug, Clone)]
pub struct IndexedValidateStream<I> {
    inner: I,
    index: usize,
}

impl<I> IndexedValidateStream<I> {
    /// Creates a new IndexedValidateStream.
    #[inline]
    pub fn new(inner: I) -> Self {
        Self { inner, index: 0 }
    }
}

impl<I, S> Iterator for IndexedValidateStream<I>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Item = (usize, Result<ValidatedCard, ValidationError>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| {
            let result = validate(s.as_ref());
            let index = self.index;
            self.index += 1;
            (index, result)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Extension trait for adding card validation to any iterator.
///
/// This trait is automatically implemented for all iterators over
/// string-like types.
pub trait ValidateExt: Iterator + Sized {
    /// Validates each card number yielded by the iterator.
    ///
    /// Returns a new iterator that yields `Result<ValidatedCard, ValidationError>`.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_validator::stream::ValidateExt;
    ///
    /// let cards = ["4111111111111111", "5500000000000004"];
    /// for result in cards.iter().copied().validate_cards() {
    ///     if let Ok(card) = result {
    ///         println!("Valid {} card", card.brand());
    ///     }
    /// }
    /// ```
    fn validate_cards(self) -> ValidateStream<Self>;

    /// Validates and yields only valid cards.
    ///
    /// Invalid cards are silently filtered out.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_validator::stream::ValidateExt;
    ///
    /// let cards = ["4111111111111111", "invalid", "5500000000000004"];
    /// let valid: Vec<_> = cards.iter().copied().validate_valid_only().collect();
    /// assert_eq!(valid.len(), 2);
    /// ```
    fn validate_valid_only(self) -> ValidOnlyStream<Self>;

    /// Validates with index tracking.
    ///
    /// Returns tuples of (index, result) for tracking which cards
    /// succeeded or failed.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_validator::stream::ValidateExt;
    ///
    /// let cards = ["4111111111111111", "invalid"];
    /// for (idx, result) in cards.iter().copied().validate_indexed() {
    ///     match result {
    ///         Ok(card) => println!("Card {} is valid", idx),
    ///         Err(e) => println!("Card {} failed: {}", idx, e),
    ///     }
    /// }
    /// ```
    fn validate_indexed(self) -> IndexedValidateStream<Self>;
}

impl<I: Iterator + Sized> ValidateExt for I {
    #[inline]
    fn validate_cards(self) -> ValidateStream<Self> {
        ValidateStream::new(self)
    }

    #[inline]
    fn validate_valid_only(self) -> ValidOnlyStream<Self> {
        ValidOnlyStream::new(self)
    }

    #[inline]
    fn validate_indexed(self) -> IndexedValidateStream<Self> {
        IndexedValidateStream::new(self)
    }
}

/// Creates a validation stream from a slice of strings.
///
/// Convenience function for creating a stream without using the trait.
#[inline]
pub fn validate_stream<'a, S: AsRef<str> + 'a>(
    cards: &'a [S],
) -> ValidateStream<impl Iterator<Item = &'a S>> {
    ValidateStream::new(cards.iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CardBrand;

    const VALID_VISA: &str = "4111111111111111";
    const VALID_MC: &str = "5500000000000004";
    const INVALID: &str = "1234567890123456";

    #[test]
    fn test_validate_stream() {
        let cards = vec![VALID_VISA, VALID_MC, INVALID];
        let results: Vec<_> = cards.iter().copied().validate_cards().collect();

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_valid_only_stream() {
        let cards = vec![VALID_VISA, INVALID, VALID_MC, "bad"];
        let valid: Vec<_> = cards.iter().copied().validate_valid_only().collect();

        assert_eq!(valid.len(), 2);
        assert_eq!(valid[0].brand(), CardBrand::Visa);
        assert_eq!(valid[1].brand(), CardBrand::Mastercard);
    }

    #[test]
    fn test_indexed_stream() {
        let cards = vec![VALID_VISA, INVALID, VALID_MC];
        let results: Vec<_> = cards.iter().copied().validate_indexed().collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0);
        assert!(results[0].1.is_ok());
        assert_eq!(results[1].0, 1);
        assert!(results[1].1.is_err());
        assert_eq!(results[2].0, 2);
        assert!(results[2].1.is_ok());
    }

    #[test]
    fn test_size_hint() {
        let cards = vec![VALID_VISA, VALID_MC, INVALID];
        let stream = cards.iter().copied().validate_cards();
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_valid_only_size_hint() {
        let cards = vec![VALID_VISA, VALID_MC, INVALID];
        let stream = cards.iter().copied().validate_valid_only();
        // Lower bound is 0 since we don't know how many are valid
        assert_eq!(stream.size_hint(), (0, Some(3)));
    }

    #[test]
    fn test_double_ended() {
        let cards = vec![VALID_VISA, VALID_MC];
        let mut stream = cards.iter().copied().validate_cards();

        let last = stream.next_back().unwrap();
        assert!(last.is_ok());
        assert_eq!(last.unwrap().brand(), CardBrand::Mastercard);

        let first = stream.next().unwrap();
        assert!(first.is_ok());
        assert_eq!(first.unwrap().brand(), CardBrand::Visa);
    }

    #[test]
    fn test_validate_stream_fn() {
        let cards = [VALID_VISA, VALID_MC];
        let count = validate_stream(&cards).filter(|r| r.is_ok()).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_chaining() {
        let cards = vec![VALID_VISA, INVALID, VALID_MC, "bad", "378282246310005"];

        let visa_count = cards
            .iter()
            .copied()
            .validate_cards()
            .filter_map(|r| r.ok())
            .filter(|c| c.brand() == CardBrand::Visa)
            .count();

        assert_eq!(visa_count, 1);
    }

    #[test]
    fn test_with_string_vec() {
        let cards: Vec<String> = vec![
            VALID_VISA.to_string(),
            VALID_MC.to_string(),
        ];

        let results: Vec<_> = cards.iter().validate_cards().collect();
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_empty_stream() {
        let cards: Vec<&str> = vec![];
        let results: Vec<_> = cards.iter().copied().validate_cards().collect();
        assert!(results.is_empty());
    }
}
