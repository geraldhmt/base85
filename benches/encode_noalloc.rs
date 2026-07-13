// start bench with
// cargo bench --message-format=short --bench=encode_noalloc
// cargo bench --message-format=short --bench=encode_noalloc --features only_safe

use base85::*;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::hint::black_box;

fn encode_noalloc_benchmark(c: &mut Criterion) {
    let mut testdata = vec![0; 0x100000];
    ThreadRng::default().fill_bytes(&mut testdata);
    let mut encoded_resudata = Vec::with_capacity(calc_encode_len(testdata.len()));
    encoded_resudata.resize(encoded_resudata.capacity(), 0);
    let mut resudata = Vec::with_capacity(encoded_resudata.capacity());
    resudata.resize(resudata.capacity(), 0);
    let _encoded = match encode_noalloc(&testdata, &mut encoded_resudata) {
        Ok(encoded) => encoded,
        Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
    };

    c.bench_function("encoder_noalloc", |b| {
        b.iter(|| {
            let _ = match encode_noalloc(black_box(&testdata), black_box(&mut resudata)) {
                Ok(_) => {}
                Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
            };
        })
    });

    c.bench_function("encoder_noalloc_prime", |b| {
        b.iter(|| {
            let _ = match encode_noalloc(black_box(&testdata[..100003]), black_box(&mut resudata)) {
                Ok(_) => {}
                Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
            };
        })
    });

    c.bench_function("encoder_noalloc_1K", |b| {
        b.iter(|| {
            let _ = match encode_noalloc(black_box(&testdata[..1024]), black_box(&mut resudata)) {
                Ok(_) => {}
                Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
            };
        })
    });

    c.bench_function("encoder_noalloc_32B", |b| {
        b.iter(|| {
            let _ = match encode_noalloc(black_box(&testdata[..32]), black_box(&mut resudata)) {
                Ok(_) => {}
                Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
            };
        })
    });

    c.bench_function("decoder_noalloc", |b| {
        b.iter(|| {
            match decode_noalloc(black_box(&resudata), black_box(&mut testdata)) {
                Ok(_) => {}
                Err(e) => panic!("Error decoding test data: {e} at line: {}", line!()),
            };
        })
    });
}

criterion_group!(benches, encode_noalloc_benchmark);
criterion_main!(benches);
