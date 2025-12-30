//! Credit card expiry date validation.
//!
//! This module provides utilities for parsing and validating credit card
//! expiration dates in various formats.
//!
//! # Supported Formats
//!
//! - `MM/YY` - e.g., "12/25"
//! - `MM/YYYY` - e.g., "12/2025"
//! - `MMYY` - e.g., "1225"
//! - `MMYYYY` - e.g., "122025"
//! - `MM-YY` - e.g., "12-25"
//! - `MM-YYYY` - e.g., "12-2025"
//!
//! # Example
//!
//! ```
//! use cc_validator::expiry::{parse_expiry, validate_expiry, ExpiryDate};
//!
//! // Parse and validate
//! let expiry = parse_expiry("12/30").unwrap();
//! assert_eq!(expiry.month(), 12);
//! assert_eq!(expiry.year(), 2030);
//!
//! // Check if expired (depends on current date)
//! // assert!(!expiry.is_expired());
//!
//! // Quick validation (use a future date)
//! assert!(validate_expiry("12/30").is_ok());
//! ```

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A validated expiry date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpiryDate {
    /// Month (1-12)
    month: u8,
    /// Four-digit year (e.g., 2025)
    year: u16,
}

impl ExpiryDate {
    /// Creates a new expiry date.
    ///
    /// Returns `None` if the month is invalid (not 1-12).
    pub fn new(month: u8, year: u16) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        Some(Self { month, year })
    }

    /// Returns the month (1-12).
    #[inline]
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Returns the four-digit year.
    #[inline]
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// Returns true if the card has expired.
    ///
    /// A card expires at the end of its expiry month.
    pub fn is_expired(&self) -> bool {
        let (current_year, current_month) = current_year_month();

        if self.year < current_year {
            return true;
        }
        if self.year == current_year && self.month < current_month {
            return true;
        }
        false
    }

    /// Returns true if the expiry date is too far in the future.
    ///
    /// Cards typically aren't issued with expiry dates more than 10 years out.
    pub fn is_too_far_future(&self, max_years: u16) -> bool {
        let (current_year, _) = current_year_month();
        self.year > current_year + max_years
    }

    /// Returns the number of months until expiration.
    ///
    /// Returns 0 if already expired.
    pub fn months_until_expiry(&self) -> u32 {
        let (current_year, current_month) = current_year_month();

        let expiry_months = (self.year as u32) * 12 + (self.month as u32);
        let current_months = (current_year as u32) * 12 + (current_month as u32);

        expiry_months.saturating_sub(current_months)
    }

    /// Formats as MM/YY.
    pub fn format_short(&self) -> String {
        format!("{:02}/{:02}", self.month, self.year % 100)
    }

    /// Formats as MM/YYYY.
    pub fn format_long(&self) -> String {
        format!("{:02}/{:04}", self.month, self.year)
    }
}

impl fmt::Display for ExpiryDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}/{:02}", self.month, self.year % 100)
    }
}

/// Errors that can occur during expiry date parsing/validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpiryError {
    /// The input string is empty.
    Empty,
    /// Invalid format - couldn't parse month/year.
    InvalidFormat,
    /// Month is out of range (must be 1-12).
    InvalidMonth(u8),
    /// Year is in the past (already expired).
    Expired {
        /// The expiry month.
        month: u8,
        /// The expiry year.
        year: u16,
    },
    /// Year is too far in the future.
    TooFarFuture {
        /// The expiry year.
        year: u16,
        /// Maximum allowed year.
        max_year: u16,
    },
}

impl fmt::Display for ExpiryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "expiry date is empty"),
            Self::InvalidFormat => {
                write!(f, "invalid expiry format (expected MM/YY or MM/YYYY)")
            }
            Self::InvalidMonth(m) => {
                write!(f, "invalid month {}: must be 1-12", m)
            }
            Self::Expired { month, year } => {
                write!(f, "card expired ({:02}/{})", month, year)
            }
            Self::TooFarFuture { year, max_year } => {
                write!(
                    f,
                    "expiry year {} is too far in the future (max: {})",
                    year, max_year
                )
            }
        }
    }
}

impl std::error::Error for ExpiryError {}

/// Parses an expiry date string.
///
/// Accepts various formats:
/// - `MM/YY`, `MM/YYYY`
/// - `MM-YY`, `MM-YYYY`
/// - `MMYY`, `MMYYYY`
///
/// # Example
///
/// ```
/// use cc_validator::expiry::parse_expiry;
///
/// let expiry = parse_expiry("12/25").unwrap();
/// assert_eq!(expiry.month(), 12);
/// assert_eq!(expiry.year(), 2025);
///
/// let expiry = parse_expiry("01/2030").unwrap();
/// assert_eq!(expiry.month(), 1);
/// assert_eq!(expiry.year(), 2030);
/// ```
pub fn parse_expiry(input: &str) -> Result<ExpiryDate, ExpiryError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ExpiryError::Empty);
    }

    // Try to parse with separator (/ or -)
    if let Some((month_str, year_str)) = input.split_once('/').or_else(|| input.split_once('-')) {
        return parse_month_year(month_str.trim(), year_str.trim());
    }

    // Try to parse without separator (MMYY or MMYYYY)
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    match digits.len() {
        4 => {
            // MMYY
            parse_month_year(&digits[0..2], &digits[2..4])
        }
        6 => {
            // MMYYYY
            parse_month_year(&digits[0..2], &digits[2..6])
        }
        _ => Err(ExpiryError::InvalidFormat),
    }
}

/// Parses month and year strings.
fn parse_month_year(month_str: &str, year_str: &str) -> Result<ExpiryDate, ExpiryError> {
    let month: u8 = month_str.parse().map_err(|_| ExpiryError::InvalidFormat)?;

    if !(1..=12).contains(&month) {
        return Err(ExpiryError::InvalidMonth(month));
    }

    let year: u16 = match year_str.len() {
        2 => {
            // Two-digit year - assume 2000s
            let yy: u16 = year_str.parse().map_err(|_| ExpiryError::InvalidFormat)?;
            2000 + yy
        }
        4 => year_str.parse().map_err(|_| ExpiryError::InvalidFormat)?,
        _ => return Err(ExpiryError::InvalidFormat),
    };

    Ok(ExpiryDate { month, year })
}

/// Validates an expiry date string.
///
/// This function parses the expiry date and checks that it's not expired
/// and not too far in the future (default: 20 years).
///
/// # Example
///
/// ```
/// use cc_validator::expiry::validate_expiry;
///
/// // Valid future date
/// assert!(validate_expiry("12/30").is_ok());
///
/// // Expired date
/// assert!(validate_expiry("01/20").is_err());
/// ```
pub fn validate_expiry(input: &str) -> Result<ExpiryDate, ExpiryError> {
    validate_expiry_with_options(input, true, Some(20))
}

/// Validates an expiry date with custom options.
///
/// # Arguments
///
/// * `input` - The expiry date string
/// * `check_expired` - Whether to check if the date is expired
/// * `max_years_future` - Maximum years in the future (None to disable check)
pub fn validate_expiry_with_options(
    input: &str,
    check_expired: bool,
    max_years_future: Option<u16>,
) -> Result<ExpiryDate, ExpiryError> {
    let expiry = parse_expiry(input)?;

    if check_expired && expiry.is_expired() {
        return Err(ExpiryError::Expired {
            month: expiry.month,
            year: expiry.year,
        });
    }

    if let Some(max_years) = max_years_future {
        if expiry.is_too_far_future(max_years) {
            let (current_year, _) = current_year_month();
            return Err(ExpiryError::TooFarFuture {
                year: expiry.year,
                max_year: current_year + max_years,
            });
        }
    }

    Ok(expiry)
}

/// Checks if an expiry date string represents an expired card.
///
/// Returns `true` if the card is expired, `false` otherwise.
/// Returns `false` if the input cannot be parsed.
#[inline]
pub fn is_expired(input: &str) -> bool {
    parse_expiry(input).map(|e| e.is_expired()).unwrap_or(false)
}

/// Gets the current year and month.
fn current_year_month() -> (u16, u8) {
    // Calculate from Unix timestamp
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Approximate calculation (good enough for expiry validation)
    // Days since epoch
    let days = secs / 86400;
    // Years since 1970 (approximate, ignoring leap years for simplicity)
    let years = days / 365;
    let year = 1970 + years as u16;

    // Days into current year
    let day_of_year = days % 365;
    // Month (approximate)
    let month = (day_of_year / 30).min(11) as u8 + 1;

    (year, month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mm_yy() {
        let expiry = parse_expiry("12/25").unwrap();
        assert_eq!(expiry.month(), 12);
        assert_eq!(expiry.year(), 2025);
    }

    #[test]
    fn test_parse_mm_yyyy() {
        let expiry = parse_expiry("01/2030").unwrap();
        assert_eq!(expiry.month(), 1);
        assert_eq!(expiry.year(), 2030);
    }

    #[test]
    fn test_parse_with_dash() {
        let expiry = parse_expiry("06-28").unwrap();
        assert_eq!(expiry.month(), 6);
        assert_eq!(expiry.year(), 2028);
    }

    #[test]
    fn test_parse_mmyy() {
        let expiry = parse_expiry("1225").unwrap();
        assert_eq!(expiry.month(), 12);
        assert_eq!(expiry.year(), 2025);
    }

    #[test]
    fn test_parse_mmyyyy() {
        let expiry = parse_expiry("122025").unwrap();
        assert_eq!(expiry.month(), 12);
        assert_eq!(expiry.year(), 2025);
    }

    #[test]
    fn test_parse_with_spaces() {
        let expiry = parse_expiry("  12 / 25  ").unwrap();
        assert_eq!(expiry.month(), 12);
        assert_eq!(expiry.year(), 2025);
    }

    #[test]
    fn test_invalid_month_zero() {
        let result = parse_expiry("00/25");
        assert!(matches!(result, Err(ExpiryError::InvalidMonth(0))));
    }

    #[test]
    fn test_invalid_month_13() {
        let result = parse_expiry("13/25");
        assert!(matches!(result, Err(ExpiryError::InvalidMonth(13))));
    }

    #[test]
    fn test_invalid_format() {
        assert!(matches!(parse_expiry(""), Err(ExpiryError::Empty)));
        assert!(matches!(parse_expiry("abc"), Err(ExpiryError::InvalidFormat)));
        assert!(matches!(parse_expiry("1/2/3"), Err(ExpiryError::InvalidFormat)));
    }

    #[test]
    fn test_is_expired() {
        // Far past date should be expired
        let expiry = ExpiryDate::new(1, 2020).unwrap();
        assert!(expiry.is_expired());

        // Far future date should not be expired
        let expiry = ExpiryDate::new(12, 2099).unwrap();
        assert!(!expiry.is_expired());
    }

    #[test]
    fn test_format() {
        let expiry = ExpiryDate::new(3, 2025).unwrap();
        assert_eq!(expiry.format_short(), "03/25");
        assert_eq!(expiry.format_long(), "03/2025");
        assert_eq!(expiry.to_string(), "03/25");
    }

    #[test]
    fn test_expiry_date_new() {
        assert!(ExpiryDate::new(1, 2025).is_some());
        assert!(ExpiryDate::new(12, 2025).is_some());
        assert!(ExpiryDate::new(0, 2025).is_none());
        assert!(ExpiryDate::new(13, 2025).is_none());
    }

    #[test]
    fn test_validate_expired() {
        let result = validate_expiry("01/20");
        assert!(matches!(result, Err(ExpiryError::Expired { .. })));
    }

    #[test]
    fn test_validate_too_far_future() {
        let result = validate_expiry("01/99");
        assert!(matches!(result, Err(ExpiryError::TooFarFuture { .. })));
    }

    #[test]
    fn test_validate_skip_expired_check() {
        // Should pass even though expired when check is disabled
        let result = validate_expiry_with_options("01/20", false, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_months_until_expiry() {
        // Far future should have many months
        let expiry = ExpiryDate::new(12, 2099).unwrap();
        assert!(expiry.months_until_expiry() > 12);

        // Past should be 0
        let expiry = ExpiryDate::new(1, 2020).unwrap();
        assert_eq!(expiry.months_until_expiry(), 0);
    }

    #[test]
    fn test_is_expired_function() {
        assert!(is_expired("01/20"));
        assert!(!is_expired("12/99"));
        assert!(!is_expired("invalid")); // Returns false on parse error
    }
}
