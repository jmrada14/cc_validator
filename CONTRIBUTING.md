# Contributing to cc_validator

Thank you for considering contributing to cc_validator!

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/cc_validator
   cd cc_validator
   ```

2. Install Rust (1.70+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Pull Request Process

1. Fork the repository and create your branch from `main`
2. Add tests for any new functionality
3. Ensure all tests pass: `cargo test --all-features`
4. Run clippy: `cargo clippy -- -D warnings`
5. Format code: `cargo fmt`
6. Update documentation if needed
7. Submit a pull request

## Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation for public APIs
- Add tests for new functionality

## Testing

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Run fuzz tests (requires nightly)
cargo +nightly fuzz run fuzz_validate -- -max_total_time=60
```

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).
