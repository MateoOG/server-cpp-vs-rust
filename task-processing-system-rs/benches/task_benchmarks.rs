use criterion::{black_box, criterion_group, criterion_main, Criterion};
use task_processing_system_rs::{Calculator, Operation};

/// Benchmark factorial calculations
fn benchmark_factorial(c: &mut Criterion) {
    c.bench_function("factorial_5", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Factorial), black_box(5)))
    });

    c.bench_function("factorial_10", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Factorial), black_box(10)))
    });

    c.bench_function("factorial_20", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Factorial), black_box(20)))
    });
}

/// Benchmark fibonacci calculations
fn benchmark_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci_10", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Fibonacci), black_box(10)))
    });

    c.bench_function("fibonacci_30", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Fibonacci), black_box(30)))
    });

    c.bench_function("fibonacci_50", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::Fibonacci), black_box(50)))
    });
}

/// Benchmark prime check calculations
fn benchmark_prime_check(c: &mut Criterion) {
    c.bench_function("prime_check_small", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::PrimeCheck), black_box(17)))
    });

    c.bench_function("prime_check_medium", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::PrimeCheck), black_box(982451653)))
    });

    c.bench_function("prime_check_large", |b| {
        b.iter(|| Calculator::calculate(black_box(Operation::PrimeCheck), black_box(9999991)))
    });
}

/// Mixed operations benchmark
fn benchmark_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    
    group.bench_function("factorial_vs_fibonacci", |b| {
        b.iter(|| {
            let _ = Calculator::calculate(black_box(Operation::Factorial), black_box(10));
            let _ = Calculator::calculate(black_box(Operation::Fibonacci), black_box(20));
        })
    });

    group.bench_function("all_operations", |b| {
        b.iter(|| {
            let _ = Calculator::calculate(black_box(Operation::Factorial), black_box(8));
            let _ = Calculator::calculate(black_box(Operation::Fibonacci), black_box(25));
            let _ = Calculator::calculate(black_box(Operation::PrimeCheck), black_box(97));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_factorial,
    benchmark_fibonacci,
    benchmark_prime_check,
    benchmark_mixed_operations
);
criterion_main!(benches);