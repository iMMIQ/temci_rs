use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use temci::run::stats::{Sample, Statistics};

fn bench_statistics_from_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    for size in [10, 100, 1000, 10000].iter() {
        let data: Vec<f64> = (1..=*size).map(|i| i as f64).collect();
        let sample = Sample::new(data);

        group.bench_with_input(BenchmarkId::from_parameter(size), &sample, |b, s| {
            b.iter(|| Statistics::from_sample(black_box(s)))
        });
    }

    group.finish();
}

fn bench_outlier_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("outliers");

    for size in [10, 100, 1000].iter() {
        let data: Vec<f64> = (1..=*size).map(|i| i as f64).chain(vec![10000.0]).collect();
        let sample = Sample::new(data);

        group.bench_with_input(BenchmarkId::from_parameter(size), &sample, |b, s| {
            b.iter(|| s.detect_outliers(black_box(1.5)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_statistics_from_sample,
    bench_outlier_detection
);
criterion_main!(benches);
