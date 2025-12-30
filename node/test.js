/**
 * Simple test script for cc-validator Node.js bindings.
 *
 * Run: npm run build && node test.js
 */

const {
  validateCard,
  isValid,
  passesLuhn,
  detectBrand,
  formatCard,
  formatCardWithSeparator,
  stripFormatting,
  maskCard,
  generateTestCard,
  validateCvv,
  validateCvvForBrand,
  validateExpiry,
  parseExpiry,
  validateBatch,
  cvvLengthForBrand,
  validLengthsForBrand,
} = require('./index.js');

console.log('Testing cc-validator Node.js bindings\n');

// Test validateCard
console.log('=== validateCard ===');
const result = validateCard('4111-1111-1111-1111');
console.log('validateCard("4111-1111-1111-1111"):', result);
console.assert(result.valid === true, 'Should be valid');
console.assert(result.brand === 'Visa', 'Should be Visa');

const invalid = validateCard('4111111111111112');
console.log('validateCard("4111111111111112"):', invalid);
console.assert(invalid.valid === false, 'Should be invalid');

// Test isValid
console.log('\n=== isValid ===');
console.log('isValid("4111111111111111"):', isValid('4111111111111111'));
console.assert(isValid('4111111111111111') === true, 'Should be valid');
console.assert(isValid('4111111111111112') === false, 'Should be invalid');

// Test passesLuhn
console.log('\n=== passesLuhn ===');
console.log('passesLuhn("4111111111111111"):', passesLuhn('4111111111111111'));
console.assert(passesLuhn('4111111111111111') === true, 'Should pass Luhn');

// Test detectBrand
console.log('\n=== detectBrand ===');
console.log('detectBrand("4111"):', detectBrand('4111'));
console.log('detectBrand("5500"):', detectBrand('5500'));
console.log('detectBrand("3782"):', detectBrand('3782'));
console.assert(detectBrand('4111') === 'Visa', 'Should be Visa');
console.assert(detectBrand('5500') === 'Mastercard', 'Should be Mastercard');
console.assert(detectBrand('3782') === 'American Express', 'Should be Amex');

// Test formatCard
console.log('\n=== formatCard ===');
console.log('formatCard("4111111111111111"):', formatCard('4111111111111111'));
console.log('formatCard("378282246310005"):', formatCard('378282246310005'));

// Test formatCardWithSeparator
console.log('\n=== formatCardWithSeparator ===');
console.log('formatCardWithSeparator("4111111111111111", "-"):', formatCardWithSeparator('4111111111111111', '-'));

// Test stripFormatting
console.log('\n=== stripFormatting ===');
console.log('stripFormatting("4111 1111 1111 1111"):', stripFormatting('4111 1111 1111 1111'));
console.assert(stripFormatting('4111-1111-1111-1111') === '4111111111111111', 'Should strip formatting');

// Test maskCard
console.log('\n=== maskCard ===');
console.log('maskCard("4111111111111111"):', maskCard('4111111111111111'));

// Test generateTestCard
console.log('\n=== generateTestCard ===');
const visaCard = generateTestCard('visa');
console.log('generateTestCard("visa"):', visaCard);
console.assert(isValid(visaCard), 'Generated card should be valid');
console.assert(detectBrand(visaCard) === 'Visa', 'Should be Visa');

const amexCard = generateTestCard('amex');
console.log('generateTestCard("amex"):', amexCard);
console.assert(detectBrand(amexCard) === 'American Express', 'Should be Amex');

// Test validateCvv
console.log('\n=== validateCvv ===');
console.log('validateCvv("123"):', validateCvv('123'));
console.log('validateCvv("1234"):', validateCvv('1234'));
console.assert(validateCvv('123').valid === true, '3-digit CVV should be valid');
console.assert(validateCvv('1234').valid === true, '4-digit CVV should be valid');
console.assert(validateCvv('12').valid === false, '2-digit CVV should be invalid');

// Test validateCvvForBrand
console.log('\n=== validateCvvForBrand ===');
console.log('validateCvvForBrand("1234", "amex"):', validateCvvForBrand('1234', 'amex'));
console.log('validateCvvForBrand("123", "visa"):', validateCvvForBrand('123', 'visa'));
console.assert(validateCvvForBrand('1234', 'amex').valid === true, 'Amex 4-digit CVV should be valid');
console.assert(validateCvvForBrand('123', 'amex').valid === false, 'Amex 3-digit CVV should be invalid');

// Test validateExpiry
console.log('\n=== validateExpiry ===');
console.log('validateExpiry("12/30"):', validateExpiry('12/30'));
console.log('validateExpiry("01/20"):', validateExpiry('01/20'));
console.assert(validateExpiry('12/30').valid === true, 'Future date should be valid');
console.assert(validateExpiry('01/20').valid === false, 'Past date should be invalid');

// Test parseExpiry
console.log('\n=== parseExpiry ===');
console.log('parseExpiry("12/30"):', parseExpiry('12/30'));
const parsed = parseExpiry('12/30');
console.assert(parsed.month === 12, 'Month should be 12');
console.assert(parsed.year === 2030, 'Year should be 2030');

// Test validateBatch
console.log('\n=== validateBatch ===');
const batchResults = validateBatch([
  '4111111111111111',
  '5500000000000004',
  'invalid',
  '378282246310005',
]);
console.log('validateBatch results:', batchResults.map(r => ({ valid: r.valid, brand: r.brand })));
console.assert(batchResults.length === 4, 'Should have 4 results');
console.assert(batchResults[0].valid === true, 'First should be valid');
console.assert(batchResults[2].valid === false, 'Third should be invalid');

// Test cvvLengthForBrand
console.log('\n=== cvvLengthForBrand ===');
console.log('cvvLengthForBrand("amex"):', cvvLengthForBrand('amex'));
console.log('cvvLengthForBrand("visa"):', cvvLengthForBrand('visa'));
console.assert(cvvLengthForBrand('amex') === 4, 'Amex CVV should be 4');
console.assert(cvvLengthForBrand('visa') === 3, 'Visa CVV should be 3');

// Test validLengthsForBrand
console.log('\n=== validLengthsForBrand ===');
console.log('validLengthsForBrand("visa"):', validLengthsForBrand('visa'));
console.log('validLengthsForBrand("amex"):', validLengthsForBrand('amex'));

console.log('\n\x1b[32mAll tests passed!\x1b[0m');
