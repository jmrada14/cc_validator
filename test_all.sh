#!/bin/bash
# Comprehensive test script for cc_validator

set -e  # Exit on error

echo "========================================"
echo "cc_validator - Full Test Suite"
echo "========================================"

cd "$(dirname "$0")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

pass() { echo -e "${GREEN}✓ $1${NC}"; }
fail() { echo -e "${RED}✗ $1${NC}"; exit 1; }
info() { echo -e "${BLUE}→ $1${NC}"; }

# 1. Rust Tests
echo ""
info "Running Rust unit tests..."
cargo test --lib --quiet && pass "Unit tests" || fail "Unit tests"

info "Running integration tests..."
cargo test --test '*' --quiet && pass "Integration tests" || fail "Integration tests"

info "Running doc tests..."
cargo test --doc --quiet && pass "Doc tests" || fail "Doc tests"

# 2. CLI Tests
echo ""
info "Building CLI..."
cargo build --features cli --quiet && pass "CLI build" || fail "CLI build"

CLI="./target/debug/ccvalidator"

info "Testing CLI commands..."

# Validate command
$CLI validate 4111111111111111 > /dev/null && pass "CLI: validate" || fail "CLI: validate"

# Validate with JSON output
$CLI validate 4111111111111111 --output json | grep -q '"valid": true' && pass "CLI: validate JSON" || fail "CLI: validate JSON"

# Generate command
GENERATED=$($CLI generate --brand visa)
$CLI validate "$GENERATED" > /dev/null && pass "CLI: generate" || fail "CLI: generate"

# Format command
$CLI format 4111111111111111 | grep -q "4111 1111 1111 1111" && pass "CLI: format" || fail "CLI: format"

# CVV command
$CLI cvv 123 | grep -q "Valid: yes" && pass "CLI: cvv" || fail "CLI: cvv"

# Expiry command
$CLI expiry 12/30 | grep -q "Valid: yes" && pass "CLI: expiry" || fail "CLI: expiry"

# Mask command
$CLI mask 4111111111111111 | grep -q "1111" && pass "CLI: mask" || fail "CLI: mask"

# Luhn command
$CLI luhn 4111111111111111 | grep -q "PASS" && pass "CLI: luhn" || fail "CLI: luhn"

# Detect command
$CLI detect 4111 | grep -q "Visa" && pass "CLI: detect" || fail "CLI: detect"

# 3. WASM Build Test
echo ""
info "Checking WASM feature compiles..."
cargo check --features wasm --quiet && pass "WASM build" || fail "WASM build"

# 4. Node.js Bindings Test
echo ""
if [ -d "node" ]; then
    info "Checking Node.js bindings compile..."
    (cd node && cargo check --quiet) && pass "Node.js bindings build" || fail "Node.js bindings build"
fi

# 5. Feature combinations
echo ""
info "Testing feature combinations..."
cargo check --features "bin-json" --quiet && pass "Feature: bin-json" || fail "Feature: bin-json"
cargo check --features "bin-csv" --quiet && pass "Feature: bin-csv" || fail "Feature: bin-csv"
cargo check --features "parallel" --quiet && pass "Feature: parallel" || fail "Feature: parallel"
cargo check --features "generate" --quiet && pass "Feature: generate" || fail "Feature: generate"

# 6. Benchmarks compile
echo ""
info "Checking benchmarks compile..."
cargo check --benches --quiet && pass "Benchmarks" || fail "Benchmarks"

# Summary
echo ""
echo "========================================"
echo -e "${GREEN}All tests passed!${NC}"
echo "========================================"
echo ""
echo "Additional manual tests:"
echo "  - WASM: wasm-pack build --features wasm"
echo "  - Node.js: cd node && npm install && npm run build && npm test"
echo "  - Fuzzing: cargo +nightly fuzz run fuzz_validate"
