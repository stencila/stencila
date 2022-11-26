//! Benchmarks of the performance of the implementation of the `Patchable` trait
//! for `Vec`s of atomics (things that can't be patched) e.g. integers
//!
//! These are mainly implemented to check that there are not operations that
//! are substantially less performant.
//!
//! Only adding/removing/replacing one item in both diffs and in applys,
//! including applying `XMany` operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use node_address::Address;
use node_patch::{apply, diff, value::Values, Operation, Patch, Patchable};

pub fn bench_diff(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("vecs_atomic_diff");

    group.bench_function("none", |bencher| {
        bencher.iter(|| diff(black_box(&vec![1, 2, 3]), black_box(&vec![1, 2, 3])))
    });

    group.bench_function("add", |bencher| {
        bencher.iter(|| diff(black_box(&vec![1, 2, 3]), black_box(&vec![1, 2, 3, 4])))
    });

    group.bench_function("remove", |bencher| {
        bencher.iter(|| diff(black_box(&vec![1, 2, 3, 4]), black_box(&vec![1, 2, 3])))
    });

    group.bench_function("replace", |bencher| {
        bencher.iter(|| diff(black_box(&vec![1, 2, 3]), black_box(&vec![1, 4, 3])))
    });

    group.finish();
}

pub fn bench_apply(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("vecs_atomic_apply");

    group.bench_function("add", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1]),
                black_box(Patch::from_ops(vec![Operation::add(
                    Address::from(0),
                    2.to_value(),
                )])),
            )
        })
    });

    group.bench_function("add_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1]),
                black_box(Patch::from_ops(vec![Operation::add_many(
                    Address::from(0),
                    Values::from_single(2.to_value()),
                )])),
            )
        })
    });

    group.bench_function("remove", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1]),
                black_box(Patch::from_ops(vec![Operation::remove(Address::from(0))])),
            )
        })
    });

    group.bench_function("remove_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1]),
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
                black_box(&mut vec![1]),
                black_box(Patch::from_ops(vec![Operation::replace(
                    Address::from(0),
                    2.to_value(),
                )])),
            )
        })
    });

    group.bench_function("replace_many", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1]),
                black_box(Patch::from_ops(vec![Operation::replace_many(
                    Address::from(0),
                    1,
                    Values::from_single(2.to_value()),
                )])),
            )
        })
    });

    group.bench_function("move", |bencher| {
        bencher.iter(|| {
            apply(
                black_box(&mut vec![1, 2]),
                black_box(Patch::from_ops(vec![Operation::mov(
                    Address::from(0),
                    Address::from(1),
                )])),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_diff, bench_apply);
criterion_main!(benches);
