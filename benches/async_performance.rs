//! Benchmark for async filesystem operations.

#![allow(missing_docs)]

use bms_resource_toolbox::fs::pack_move::is_dir_having_file;
use criterion::{Criterion, criterion_group, criterion_main};
use tokio::runtime::Runtime;

fn benchmark_async_is_dir_having_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = std::env::temp_dir();

    c.bench_function("async_is_dir_having_file", |b| {
        b.iter(|| rt.block_on(is_dir_having_file(&temp_dir)));
    });
}

criterion_group!(benches, benchmark_async_is_dir_having_file);
criterion_main!(benches);
