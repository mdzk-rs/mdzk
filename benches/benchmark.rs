use criterion::{criterion_group, criterion_main, Criterion};
use mdzk::{Note, NoteId, Vault};
use serde_json::json;
use std::path::Path;

fn setup() -> Vault {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("benchsuite");

    mdzk::VaultBuilder::default()
        .source(source.to_owned())
        .build()
        .unwrap()
}

fn bench_incoming_links(c: &mut Criterion) {
    let vault = setup();
    c.bench_function("find incoming links", |b| {
        b.iter(|| {
            let _: Vec<(&NoteId, &Note)> = vault.incoming_arcs(&10479125933004782128).collect();
        })
    });
}

fn bench_serializer(c: &mut Criterion) {
    let vault = setup();
    c.bench_function("serialize vault", |b| b.iter(|| json!(vault)));
}

fn bench_builder(c: &mut Criterion) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("benchsuite");

    c.bench_function("build vault from source", |b| {
        b.iter(|| {
            mdzk::VaultBuilder::default()
                .source(source.to_owned())
                .build()
                .unwrap()
        })
    });
}

fn bench_render_index(c: &mut Criterion) {
    let vault = setup();
    let tmp_dir = std::env::temp_dir().join("mdzk-test");
    let config = mdzk::ssg::config::Config::default();

    std::fs::create_dir_all(&tmp_dir).unwrap();

    c.bench_function("render index", |b| {
        b.iter(|| {
            mdzk::ssg::render_index(&vault, &tmp_dir, &config).unwrap();
        })
    });

    std::fs::remove_dir_all(tmp_dir).unwrap();
}

fn bench_render_notes(c: &mut Criterion) {
    let vault = setup();
    let tmp_dir = std::env::temp_dir().join("mdzk-test");
    let config = mdzk::ssg::config::Config::default();

    std::fs::create_dir_all(&tmp_dir).unwrap();

    c.bench_function("render all notes", |b| {
        b.iter(|| {
            mdzk::ssg::render_notes(&vault, &tmp_dir, &config).unwrap();
        })
    });

    std::fs::remove_dir_all(tmp_dir).unwrap();
}

criterion_group!(
    benches,
    bench_incoming_links,
    bench_serializer,
    bench_builder,
    bench_render_index,
    bench_render_notes,
);

criterion_main!(benches);
