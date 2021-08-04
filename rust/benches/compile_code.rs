use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stencila::{graphs::resources, methods::compile::code::compile};

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("compile r", |bencher| {
        bencher.iter(|| {
            compile(
                black_box(&PathBuf::from("path")),
                &resources::file(&PathBuf::from("path")),
                black_box("library(pkg)"),
                "r",
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
