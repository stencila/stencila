//! Benchmarks of the performance of the implementation of the `Patchable` trait for `String`s
//!
//! These are mainly implemented to check that there are not operations that
//! are substantially less performant.
//!
//! Only adding/removing/replacing one grapheme in both diffs and in applys,
//! including applying `XMany` operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use node_address::Address;
use node_patch::{apply, diff, value::Values, Operation, Patch, Patchable};

pub fn bench_diff(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("string_diff");

    group.bench_function("none", |bencher| {
        bencher.iter(|| {
            diff(
                black_box(&"string".to_string()),
                black_box(&"string".to_string()),
            )
        })
    });

    group.bench_function("add", |bencher| {
        bencher.iter(|| {
            diff(
                black_box(&"string".to_string()),
                black_box(&"string+".to_string()),
            )
        })
    });

    group.bench_function("remove", |bencher| {
        bencher.iter(|| {
            diff(
                black_box(&"string+".to_string()),
                black_box(&"string".to_string()),
            )
        })
    });

    group.bench_function("replace", |bencher| {
        bencher.iter(|| {
            diff(
                black_box(&"string".to_string()),
                black_box(&"strong".to_string()),
            )
        })
    });

    group.finish();
}

pub fn bench_apply(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("string_apply");

    group.bench_function("add", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::new()),
                black_box(Patch::from_ops(vec![Operation::add(
                    Address::from(0),
                    "a".to_string().to_value(),
                )])),
            )
        })
    });

    group.bench_function("add_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::new()),
                black_box(Patch::from_ops(vec![Operation::add_many(
                    Address::from(0),
                    Values::from_single("a".to_string().to_value()),
                )])),
            )
        })
    });

    group.bench_function("remove", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::from("string")),
                black_box(Patch::from_ops(vec![Operation::remove(Address::from(0))])),
            )
        })
    });

    group.bench_function("remove_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::from("string")),
                black_box(Patch::from_ops(vec![Operation::remove_many(
                    Address::from(0),
                    1,
                )])),
            )
        })
    });

    group.bench_function("replace", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::from("string")),
                black_box(Patch::from_ops(vec![Operation::replace(
                    Address::from(1),
                    "a".to_string().to_value(),
                )])),
            )
        })
    });

    group.bench_function("replace_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut String::from("string")),
                black_box(Patch::from_ops(vec![Operation::replace_many(
                    Address::from(1),
                    1,
                    Values::from_single("a".to_string().to_value()),
                )])),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_diff, bench_apply);
criterion_main!(benches);
