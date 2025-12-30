# cc_validator

A credit card validation library for Rust. **Work in progress.**

## What it does

- Validates card numbers using the Luhn algorithm
- Detects card brand from number prefix
- Validates CVV length and expiry dates
- Provides masking utilities for display
- Supports 14 card brands (Visa, Mastercard, Amex, etc.)

## Interfaces

- Rust library
- CLI tool
- REST API server (basic, no auth)
- WebAssembly bindings
- Node.js bindings (via napi-rs)

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Interfaces](#interfaces)
  - [Rust Library](#rust-library)
  - [CLI Tool](#cli-tool)
  - [REST API Server](#rest-api-server)
  - [WebAssembly](#webassembly)
  - [Node.js](#nodejs)
- [API Reference](#api-reference)
- [Supported Card Brands](#supported-card-brands)
- [Feature Flags](#feature-flags)
- [Performance](#performance)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Rust Library (from source)

```bash
git clone https://github.com/jmrada14/cc_validator
cd cc_validator
cargo build --release
```

### CLI Tool

```bash
cargo build --release --features cli
# Binary at target/release/ccvalidator
```

### REST API Server

```bash
cargo build --release --features server
# Binary at target/release/ccvalidator-server
```

## Quick Start

```rust
use cc_validator::{validate, is_valid, CardBrand};

// Validate a card number
let card = validate("4111-1111-1111-1111")?;
assert_eq!(card.brand(), CardBrand::Visa);
assert_eq!(card.last_four(), "1111");

// Safe for logging - never exposes full card number
println!("Processed: {}", card.masked()); // "****-****-****-1111"

// Quick boolean check
if is_valid("4111111111111111") {
    println!("Card is valid!");
}
```

## Interfaces

### Rust Library

Basic usage:

```rust
use cc_validator::{validate, ValidationError};

match validate("4111111111111111") {
    Ok(card) => {
        println!("Brand: {}", card.brand().name());
        println!("Last Four: {}", card.last_four());
        println!("BIN: {}", card.bin6());
    }
    Err(ValidationError::InvalidChecksum) => {
        println!("Card failed Luhn check");
    }
    Err(e) => {
        println!("Validation error: {}", e);
    }
}
```

### CLI Tool

Command-line interface:

```bash
# Validate a card
ccvalidator validate 4111111111111111
# Output:
# Valid: yes
# Brand: Visa
# Last Four: 1111
# Masked: ****-****-****-1111

# JSON output for scripting
ccvalidator validate 4111111111111111 --output json

# Generate test cards
ccvalidator generate --brand visa --count 5

# Format a card number
ccvalidator format 4111111111111111
# Output: 4111 1111 1111 1111

# Validate CVV
ccvalidator cvv 123 --brand visa

# Validate expiry
ccvalidator expiry 12/25

# Detect brand from partial number
ccvalidator detect 4111

# Mask a card number
ccvalidator mask 4111111111111111 --with-bin
```

### REST API Server

Basic HTTP API with Swagger UI. **Note: No authentication or rate limiting - not for production use.**

```bash
# Start server
ccvalidator-server --port 3000

# Server is now available at:
# - API: http://localhost:3000
# - Swagger UI: http://localhost:3000/swagger-ui/
```

**API Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/validate` | Validate a card number |
| `POST` | `/validate/batch` | Validate multiple cards |
| `GET` | `/detect?card=...` | Detect card brand |
| `POST` | `/format` | Format a card number |
| `POST` | `/generate` | Generate test cards |
| `POST` | `/cvv/validate` | Validate CVV |
| `POST` | `/expiry/validate` | Validate expiry |
| `GET` | `/health` | Health check |

**Example Requests:**

```bash
# Validate a card
curl -X POST http://localhost:3000/validate \
  -H "Content-Type: application/json" \
  -d '{"card_number": "4111-1111-1111-1111"}'

# Response:
# {
#   "valid": true,
#   "brand": "Visa",
#   "last_four": "1111",
#   "masked": "****-****-****-1111"
# }

# Batch validation
curl -X POST http://localhost:3000/validate/batch \
  -H "Content-Type: application/json" \
  -d '{"card_numbers": ["4111111111111111", "5500000000000004"]}'

# Generate test cards
curl -X POST http://localhost:3000/generate \
  -H "Content-Type: application/json" \
  -d '{"brand": "visa", "count": 3, "formatted": true}'
```

### WebAssembly

Client-side validation in the browser:

```bash
# Build WASM module
wasm-pack build --features wasm --target web
```

```html
<script type="module">
import init, { validate_card, is_valid, generate_test_card } from './pkg/cc_validator.js';

await init();

// Validate a card
const result = validate_card("4111-1111-1111-1111");
if (result.valid) {
    console.log(`Brand: ${result.brand}`);
    console.log(`Masked: ${result.masked}`);
}

// Quick check
if (is_valid("4111111111111111")) {
    console.log("Valid!");
}

// Generate test card
const testCard = generate_test_card("visa");
</script>
```

**Live Demo:** Run `cd web && python3 -m http.server 8080` and open http://localhost:8080

### Node.js

Node.js bindings using napi-rs:

```bash
cd node
npm install
npm run build
```

```javascript
const {
    validateCard,
    isValid,
    generateTestCard,
    formatCard,
    validateCvv,
    validateExpiry
} = require('cc-validator');

// Validate a card
const result = validateCard("4111-1111-1111-1111");
console.log(result.brand);     // "Visa"
console.log(result.lastFour);  // "1111"
console.log(result.masked);    // "****-****-****-1111"

// Quick check
if (isValid("4111111111111111")) {
    console.log("Valid!");
}

// Generate test cards
const card = generateTestCard("visa");

// Format
console.log(formatCard("4111111111111111")); // "4111 1111 1111 1111"

// Validate CVV
const cvvResult = validateCvv("123");

// Validate expiry
const expiryResult = validateExpiry("12/25");
```

## API Reference

### Core Functions

```rust
// Full validation with detailed result
fn validate(input: &str) -> Result<ValidatedCard, ValidationError>;

// Quick boolean check
fn is_valid(input: &str) -> bool;

// Luhn check only (no brand/length validation)
fn passes_luhn(input: &str) -> bool;

// Validate from digit array
fn validate_digits(digits: &[u8]) -> Result<ValidatedCard, ValidationError>;
```

### ValidatedCard

```rust
impl ValidatedCard {
    fn brand(&self) -> CardBrand;      // Card brand
    fn last_four(&self) -> &str;       // Last 4 digits
    fn bin6(&self) -> &str;            // First 6 digits (BIN)
    fn bin8(&self) -> &str;            // First 8 digits
    fn length(&self) -> usize;         // Total digits
    fn masked(&self) -> String;        // "****-****-****-1234"
}
```

### CVV Validation

```rust
use cc_validator::cvv;

// Validate any CVV (3-4 digits)
let validated = cvv::validate_cvv("123")?;
println!("Length: {}", validated.length());

// Brand-specific validation (Amex requires 4 digits)
let validated = cvv::validate_cvv_for_brand("1234", CardBrand::Amex)?;

// Get expected length
let len = cvv::cvv_length_for_brand(CardBrand::Amex); // 4
let len = cvv::cvv_length_for_brand(CardBrand::Visa); // 3
```

### Expiry Validation

```rust
use cc_validator::expiry;

// Validate (rejects expired cards)
let exp = expiry::validate_expiry("12/25")?;
println!("{}/{}", exp.month(), exp.year());

// Parse without expiry check
let exp = expiry::parse_expiry("12/20")?;

// Check status
if exp.is_expired() {
    println!("Card expired {} months ago", exp.months_until_expiry());
}

// Formatting
println!("{}", exp.format_short()); // "12/25"
println!("{}", exp.format_long());  // "12/2025"
```

### Card Generation

```rust
use cc_validator::generate;

// Random valid card (requires `generate` feature)
let card = generate::generate_card(CardBrand::Visa);
assert!(is_valid(&card));

// Deterministic (no randomness, for testing)
let card = generate::generate_card_deterministic(CardBrand::Visa);

// Custom prefix
let card = generate::generate_card_with_prefix("411111", 16);
```

### Formatting

```rust
use cc_validator::format;

// Auto-format based on brand
format::format_card_number("4111111111111111");    // "4111 1111 1111 1111"
format::format_card_number("378282246310005");     // "3782 822463 10005" (Amex)

// Custom separator
format::format_with_separator("4111111111111111", "-"); // "4111-1111-1111-1111"

// Strip formatting
format::strip_formatting("4111-1111-1111-1111");  // "4111111111111111"

// Partial formatting (for input fields)
format::format_partial("41111111");  // "4111 1111"
```

### Batch Processing

```rust
use cc_validator::{BatchValidator, batch};

let mut validator = BatchValidator::new();
let cards = vec!["4111111111111111", "5500000000000004", "invalid"];

// Validate all
let results = validator.validate_all(&cards);

// Get only valid cards
let valid = validator.validate_valid_only(&cards);

// Count valid/invalid
let (valid_count, invalid_count) = batch::count_valid(&cards);

// Parallel processing (requires `parallel` feature)
#[cfg(feature = "parallel")]
let results = validator.validate_parallel(&cards);
```

### Masking

```rust
use cc_validator::mask;

// Standard masking
let masked = mask::mask_card(&card);  // "****-****-****-1111"

// With BIN visible
let masked = mask::mask_with_bin(&card);  // "411111******1111"
```

## Supported Card Brands

| Brand | Prefix | Length | CVV |
|-------|--------|--------|-----|
| Visa | 4 | 13, 16, 19 | 3 |
| Mastercard | 51-55, 2221-2720 | 16 | 3 |
| American Express | 34, 37 | 15 | 4 |
| Discover | 6011, 644-649, 65 | 16-19 | 3 |
| Diners Club | 36, 38, 300-305 | 14-19 | 3 |
| JCB | 3528-3589 | 16-19 | 3 |
| UnionPay | 62 | 16-19 | 3 |
| Maestro | 50, 56-69 | 12-19 | 3 |
| Mir | 2200-2204 | 16-19 | 3 |
| RuPay | 60, 65, 81, 82 | 16 | 3 |
| Verve | 506, 507 | 16-19 | 3 |
| Elo | 509, 636 | 16 | 3 |
| Troy | 9792 | 16 | 3 |
| BC Card | 94 | 16 | 3 |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `default` | Yes | Core validation only |
| `generate` | No | Test card generation |
| `cli` | No | Command-line tool |
| `server` | No | REST API with Swagger UI |
| `wasm` | No | WebAssembly support |
| `parallel` | No | Rayon-based parallelism |
| `simd` | No | SIMD Luhn (nightly only) |
| `bin-json` | No | JSON BIN database loader |
| `bin-csv` | No | CSV BIN database loader |
| `bin-sqlite` | No | SQLite BIN database |
| `bin-all` | No | All BIN loaders |
| `full` | No | All features except nightly |

## Performance

Run benchmarks to measure performance on your hardware:

```bash
cargo bench
```

Performance depends on your hardware and workload.

## Testing

```bash
# Run all tests
cargo test

# With all features
cargo test --all-features

# Property-based tests
cargo test --test proptest_tests

# Benchmarks
cargo bench

# Fuzz testing (requires nightly)
cargo +nightly fuzz run fuzz_validate -- -max_total_time=60
```

## Project Structure

```
cc_validator/
├── src/
│   ├── lib.rs          # Main library
│   ├── validate.rs     # Core validation
│   ├── luhn.rs         # Luhn algorithm
│   ├── detect.rs       # Brand detection
│   ├── card.rs         # CardBrand, ValidatedCard
│   ├── error.rs        # Error types
│   ├── mask.rs         # Masking utilities
│   ├── format.rs       # Formatting
│   ├── cvv.rs          # CVV validation
│   ├── expiry.rs       # Expiry validation
│   ├── generate.rs     # Card generation
│   ├── batch.rs        # Batch processing
│   ├── stream.rs       # Streaming validation
│   ├── wasm.rs         # WASM bindings
│   └── bin/
│       ├── ccvalidator.rs   # CLI
│       └── server.rs        # REST API
├── web/                # WASM demo
├── node/               # Node.js bindings
├── fuzz/               # Fuzz targets
├── benches/            # Benchmarks
└── tests/              # Integration tests
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License. See [LICENSE](LICENSE).
