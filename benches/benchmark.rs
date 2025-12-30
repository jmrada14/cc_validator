//! Benchmarks for cc_validator performance testing.
//!
//! Run with: cargo bench

use cc_validator::{
    batch::{count_valid, validate_batch, BatchValidator},
    luhn,
    stream::ValidateExt,
    validate, validate_digits,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Test card numbers
const VISA_16: &str = "4111111111111111";
const VISA_16_FORMATTED: &str = "4111-1111-1111-1111";
const MASTERCARD: &str = "5500000000000004";
const AMEX: &str = "378282246310005";

const VISA_DIGITS: [u8; 16] = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
const AMEX_DIGITS: [u8; 15] = [3, 7, 8, 2, 8, 2, 2, 4, 6, 3, 1, 0, 0, 0, 5];

/// Benchmark single card validation
fn bench_single_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_validation");

    group.bench_function("visa_16_raw", |b| {
        b.iter(|| validate(black_box(VISA_16)))
    });

    group.bench_function("visa_16_formatted", |b| {
        b.iter(|| validate(black_box(VISA_16_FORMATTED)))
    });

    group.bench_function("mastercard", |b| {
        b.iter(|| validate(black_box(MASTERCARD)))
    });

    group.bench_function("amex_15", |b| {
        b.iter(|| validate(black_box(AMEX)))
    });

    group.finish();
}

/// Benchmark digit-based validation (skip parsing)
fn bench_digit_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("digit_validation");

    group.bench_function("visa_16_digits", |b| {
        b.iter(|| validate_digits(black_box(&VISA_DIGITS)))
    });

    group.bench_function("amex_15_digits", |b| {
        b.iter(|| validate_digits(black_box(&AMEX_DIGITS)))
    });

    group.finish();
}

/// Benchmark Luhn algorithm specifically
fn bench_luhn(c: &mut Criterion) {
    let mut group = c.benchmark_group("luhn");

    group.bench_function("luhn_16_generic", |b| {
        b.iter(|| luhn::validate(black_box(&VISA_DIGITS)))
    });

    group.bench_function("luhn_16_optimized", |b| {
        b.iter(|| luhn::validate_16(black_box(&VISA_DIGITS)))
    });

    group.bench_function("luhn_15_optimized", |b| {
        b.iter(|| luhn::validate_15(black_box(&AMEX_DIGITS)))
    });

    group.finish();
}

/// Benchmark batch validation with various sizes
fn bench_batch_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_validation");

    for size in [10, 100, 1000, 10000].iter() {
        let cards: Vec<&str> = (0..*size)
            .map(|i| match i % 3 {
                0 => VISA_16,
                1 => MASTERCARD,
                _ => AMEX,
            })
            .collect();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("validate_batch", size), &cards, |b, cards| {
            b.iter(|| validate_batch(black_box(cards)))
        });

        group.bench_with_input(BenchmarkId::new("count_valid", size), &cards, |b, cards| {
            b.iter(|| count_valid(black_box(cards)))
        });

        group.bench_with_input(
            BenchmarkId::new("batch_validator", size),
            &cards,
            |b, cards| {
                let mut batch = BatchValidator::new();
                b.iter(|| batch.validate_all(black_box(cards)))
            },
        );
    }

    group.finish();
}

/// Benchmark streaming validation
fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    for size in [100, 1000, 10000].iter() {
        let cards: Vec<&str> = (0..*size).map(|_| VISA_16).collect();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("validate_stream", size),
            &cards,
            |b, cards| {
                b.iter(|| {
                    cards
                        .iter()
                        .copied()
                        .validate_cards()
                        .filter(|r| r.is_ok())
                        .count()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("valid_only_stream", size),
            &cards,
            |b, cards| {
                b.iter(|| cards.iter().copied().validate_valid_only().count())
            },
        );
    }

    group.finish();
}

/// Benchmark memory operations
fn bench_card_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("card_operations");

    let card = validate(VISA_16).unwrap();

    group.bench_function("last_four", |b| {
        b.iter(|| black_box(&card).last_four())
    });

    group.bench_function("bin6", |b| {
        b.iter(|| black_box(&card).bin6())
    });

    group.bench_function("masked", |b| {
        b.iter(|| black_box(&card).masked())
    });

    group.bench_function("masked_with_bin", |b| {
        b.iter(|| black_box(&card).masked_with_bin())
    });

    group.bench_function("number", |b| {
        b.iter(|| black_box(&card).number())
    });

    group.finish();
}

/// Benchmark with mixed valid/invalid cards
fn bench_mixed_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_batch");

    // Mix of valid and invalid cards
    let mixed_cards: Vec<&str> = (0..1000)
        .map(|i| {
            if i % 5 == 0 {
                "4111111111111112" // Invalid (wrong checksum)
            } else if i % 3 == 0 {
                MASTERCARD
            } else if i % 7 == 0 {
                AMEX
            } else {
                VISA_16
            }
        })
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("validate_mixed_batch", |b| {
        b.iter(|| validate_batch(black_box(&mixed_cards)))
    });

    group.bench_function("count_valid_mixed", |b| {
        b.iter(|| count_valid(black_box(&mixed_cards)))
    });

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_parallel(c: &mut Criterion) {
    use cc_validator::batch::{count_valid_parallel, validate_batch_parallel};

    let mut group = c.benchmark_group("parallel");

    for size in [1000, 10000, 100000].iter() {
        let cards: Vec<String> = (0..*size).map(|_| VISA_16.to_string()).collect();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("validate_parallel", size),
            &cards,
            |b, cards| b.iter(|| validate_batch_parallel(black_box(cards))),
        );

        group.bench_with_input(
            BenchmarkId::new("count_parallel", size),
            &cards,
            |b, cards| b.iter(|| count_valid_parallel(black_box(cards))),
        );
    }

    group.finish();
}

#[cfg(not(feature = "parallel"))]
fn bench_parallel(_c: &mut Criterion) {
    // Parallel benchmarks disabled - enable 'parallel' feature
}

criterion_group!(
    benches,
    bench_single_validation,
    bench_digit_validation,
    bench_luhn,
    bench_batch_validation,
    bench_streaming,
    bench_card_operations,
    bench_mixed_batch,
    bench_parallel,
);

criterion_main!(benches);
