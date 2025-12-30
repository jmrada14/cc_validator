/**
 * Enterprise-grade credit card validation library.
 *
 * @example
 * ```javascript
 * const { validateCard, isValid, generateTestCard } = require('cc-validator');
 *
 * const result = validateCard("4111-1111-1111-1111");
 * if (result.valid) {
 *     console.log(`Brand: ${result.brand}`);
 * }
 * ```
 */

/** Result of card validation */
export interface ValidationResult {
  /** Whether the card is valid */
  valid: boolean;
  /** Card brand name (e.g., "Visa", "Mastercard") */
  brand: string | null;
  /** Last 4 digits of the card */
  lastFour: string | null;
  /** Masked card number safe for display */
  masked: string | null;
  /** Error message if validation failed */
  error: string | null;
}

/** Result of CVV validation */
export interface CvvResult {
  /** Whether the CVV is valid */
  valid: boolean;
  /** Length of the CVV (3 or 4) */
  length: number | null;
  /** Error message if validation failed */
  error: string | null;
}

/** Result of expiry date validation */
export interface ExpiryResult {
  /** Whether the expiry is valid */
  valid: boolean;
  /** Month (1-12) */
  month: number | null;
  /** Year (4 digits) */
  year: number | null;
  /** Whether the card is expired */
  expired: boolean | null;
  /** Formatted date (MM/YY) */
  formatted: string | null;
  /** Error message if validation failed */
  error: string | null;
}

/**
 * Validates a credit card number and returns detailed information.
 *
 * @param cardNumber - Card number (can include spaces or dashes)
 * @returns Validation result with card details
 *
 * @example
 * ```javascript
 * const result = validateCard("4111-1111-1111-1111");
 * console.log(result.brand); // "Visa"
 * ```
 */
export function validateCard(cardNumber: string): ValidationResult;

/**
 * Quick check if a card number is valid.
 *
 * @param cardNumber - Card number to validate
 * @returns true if valid
 */
export function isValid(cardNumber: string): boolean;

/**
 * Checks if a card passes the Luhn algorithm.
 *
 * @param cardNumber - Card number to check
 * @returns true if passes Luhn
 */
export function passesLuhn(cardNumber: string): boolean;

/**
 * Detects the card brand from a (partial) card number.
 *
 * @param cardNumber - Card number or prefix
 * @returns Brand name or null if unknown
 */
export function detectBrand(cardNumber: string): string | null;

/**
 * Formats a card number with spaces (brand-aware grouping).
 *
 * @param cardNumber - Raw card number
 * @returns Formatted card (e.g., "4111 1111 1111 1111")
 */
export function formatCard(cardNumber: string): string;

/**
 * Formats a card number with a custom separator.
 *
 * @param cardNumber - Raw card number
 * @param separator - Separator string
 * @returns Formatted card
 */
export function formatCardWithSeparator(cardNumber: string, separator: string): string;

/**
 * Removes all formatting from a card number.
 *
 * @param cardNumber - Formatted card number
 * @returns Raw digits only
 */
export function stripFormatting(cardNumber: string): string;

/**
 * Masks a card number (PCI-DSS compliant).
 *
 * @param cardNumber - Card number to mask
 * @returns Masked card safe for display
 * @throws Error if card is invalid
 */
export function maskCard(cardNumber: string): string;

/**
 * Generates a valid test card number.
 *
 * Supported brands: visa, mastercard, amex, discover, jcb, diners, unionpay, maestro
 *
 * @param brand - Card brand name
 * @returns Valid card number
 * @throws Error if brand is unknown
 */
export function generateTestCard(brand: string): string;

/**
 * Validates a CVV/CVC code.
 *
 * @param cvv - CVV code (3 or 4 digits)
 * @returns Validation result
 */
export function validateCvv(cvv: string): CvvResult;

/**
 * Validates a CVV for a specific card brand.
 *
 * @param cvv - CVV code
 * @param brand - Card brand name
 * @returns Validation result
 */
export function validateCvvForBrand(cvv: string, brand: string): CvvResult;

/**
 * Validates an expiry date (checks if not expired).
 *
 * Accepts: MM/YY, MM/YYYY, MM-YY, MMYY, MMYYYY
 *
 * @param date - Expiry date string
 * @returns Validation result
 */
export function validateExpiry(date: string): ExpiryResult;

/**
 * Parses an expiry date without checking if expired.
 *
 * @param date - Expiry date string
 * @returns Parsed expiry data
 */
export function parseExpiry(date: string): ExpiryResult;

/**
 * Batch validates multiple card numbers.
 *
 * @param cardNumbers - Array of card numbers
 * @returns Array of validation results
 */
export function validateBatch(cardNumbers: string[]): ValidationResult[];

/**
 * Gets the expected CVV length for a card brand.
 *
 * @param brand - Card brand name
 * @returns CVV length (3 or 4)
 * @throws Error if brand is unknown
 */
export function cvvLengthForBrand(brand: string): number;

/**
 * Gets valid card lengths for a brand.
 *
 * @param brand - Card brand name
 * @returns Array of valid lengths
 * @throws Error if brand is unknown
 */
export function validLengthsForBrand(brand: string): number[];
