// start bench with
// cargo bench --message-format=short --bench=encode
// cargo bench --message-format=short --bench=encode --features only_safe

use base85::*;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::ThreadRng, Rng};
use std::hint::black_box;

fn encode_benchmark(c: &mut Criterion) {
    let mut testdata = vec![0; 0x100000];
    ThreadRng::default().fill_bytes(&mut testdata);
    let encoded = encode(&testdata);
    c.bench_function("encoder", |b| {
        b.iter(|| {
            let _ = encode(black_box(&testdata));
        })
    });

    c.bench_function("encoder_prime", |b| {
        b.iter(|| {
            let _ = encode(black_box(&testdata[..100003]));
        })
    });

    c.bench_function("encoder_1K", |b| {
        b.iter(|| {
            let _ = encode(black_box(&testdata[..1024]));
        })
    });

    c.bench_function("encoder_32B", |b| {
        b.iter(|| {
            let _ = encode(black_box(&testdata[..32]));
        })
    });

    c.bench_function("decoder", |b| {
        b.iter(|| {
            let _ = decode(black_box(&encoded.as_bytes()));
        })
    });
}

criterion_group!(benches, encode_benchmark);
criterion_main!(benches);
