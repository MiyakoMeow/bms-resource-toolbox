use bms_resource_toolbox::fs::walk::has_chart_file;
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn benchmark_async_has_chart_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = std::env::temp_dir();

    c.bench_function("async_has_chart_file", |b| {
        b.iter(|| rt.block_on(has_chart_file(&temp_dir)))
    });
}

criterion_group!(benches, benchmark_async_has_chart_file);
criterion_main!(benches);
