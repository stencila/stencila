//! Benchmarks of the performance of methods of `Value`
//!
//! Mainly used to evaluate how well adding variants improves performance
//! and to ensure it does not degrade performance for existing variants.

use criterion::{criterion_group, criterion_main, Criterion};

use common::{maplit::btreemap, serde_json};
use node_patch::{Patchable, Value};
use stencila_schema::*;

pub fn bench_to_from_value(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("value_to_from");

    group.bench_function("null", |bencher| {
        bencher.iter(|| Null::from_value(Primitive::Null(Null {}).to_value()))
    });

    group.bench_function("enum", |bencher| {
        bencher.iter(|| ExecutionStatus::from_value(ExecutionStatus::Running.to_value()))
    });

    group.bench_function("string", |bencher| {
        bencher.iter(|| String::from_value("Hello".to_string().to_value()))
    });

    group.bench_function("primitive_boolean", |bencher| {
        bencher.iter(|| Primitive::from_value(Primitive::Boolean(true).to_value()))
    });

    group.bench_function("primitive_integer", |bencher| {
        bencher.iter(|| Primitive::from_value(Primitive::Integer(123).to_value()))
    });

    group.bench_function("primitive_number", |bencher| {
        bencher.iter(|| Primitive::from_value(Primitive::Number(Number(1.23)).to_value()))
    });

    group.bench_function("primitive_array", |bencher| {
        bencher.iter(|| {
            Primitive::from_value(
                Primitive::Array(vec![
                    Primitive::Boolean(true),
                    Primitive::Integer(123),
                    Primitive::Number(Number(1.23)),
                    Primitive::String("string".to_string()),
                ])
                .to_value(),
            )
        })
    });

    group.bench_function("primitive_object", |bencher| {
        bencher.iter(|| {
            Primitive::from_value(
                Primitive::Object(btreemap! {
                    "a".to_string() => Primitive::Boolean(true),
                    "b".to_string() => Primitive::Integer(123),
                    "c".to_string() => Primitive::Number(Number(1.23)),
                    "d".to_string() => Primitive::String("string".to_string()),
                })
                .to_value(),
            )
        })
    });

    group.bench_function("inline", |bencher| {
        bencher.iter(|| {
            InlineContent::from_value(
                InlineContent::CodeFragment(CodeFragment {
                    code: "Some code".to_string(),
                    ..Default::default()
                })
                .to_value(),
            )
        })
    });

    group.bench_function("block", |bencher| {
        bencher.iter(|| {
            BlockContent::from_value(
                BlockContent::CodeBlock(CodeBlock {
                    code: "Some code".to_string(),
                    ..Default::default()
                })
                .to_value(),
            )
        })
    });

    group.bench_function("node", |bencher| {
        bencher.iter(|| {
            Node::from_value(
                Node::Array(vec![
                    Primitive::Boolean(true),
                    Primitive::Integer(123),
                    Primitive::Number(Number(1.23)),
                    Primitive::String("string".to_string()),
                    Primitive::Object(btreemap! {
                        "a".to_string() => Primitive::Boolean(true),
                        "b".to_string() => Primitive::Integer(123),
                        "c".to_string() => Primitive::Number(Number(1.23)),
                        "d".to_string() => Primitive::String("string".to_string()),
                    }),
                ])
                .to_value(),
            )
        })
    });

    group.finish();
}

pub fn bench_deserialize(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("value_deserialize");

    group.bench_function("string", |bencher| {
        bencher.iter(|| serde_json::from_str::<Value>("string"))
    });

    group.bench_function("inline", |bencher| {
        bencher.iter(|| {
            serde_json::from_str::<Value>(r#"{"type":"CodeFragment","code": "Some code"}"#)
        })
    });

    group.bench_function("block", |bencher| {
        bencher
            .iter(|| serde_json::from_str::<Value>(r#"{"type":"CodeBlock","code": "Some code"}"#))
    });

    group.finish();
}

pub fn bench_serialize(citerion: &mut Criterion) {
    let mut group = citerion.benchmark_group("value_serialize");

    group.bench_function("string", |bencher| {
        bencher.iter(|| serde_json::to_string(&"Hello".to_string().to_value()))
    });

    group.bench_function("inline", |bencher| {
        bencher.iter(|| {
            serde_json::to_string(
                &InlineContent::CodeFragment(CodeFragment {
                    code: "Some code".to_string(),
                    ..Default::default()
                })
                .to_value(),
            )
        })
    });

    group.bench_function("block", |bencher| {
        bencher.iter(|| {
            serde_json::to_string(
                &BlockContent::CodeBlock(CodeBlock {
                    code: "Some code".to_string(),
                    ..Default::default()
                })
                .to_value(),
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_to_from_value,
    bench_deserialize,
    bench_serialize
);
criterion_main!(benches);
