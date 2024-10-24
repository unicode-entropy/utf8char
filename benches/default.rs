use core::hint::black_box as bb;
use criterion::{criterion_group, criterion_main, Criterion};
use utf8char::Utf8Char;

pub fn bench(c: &mut Criterion) {
    let rch = bb(char::from_u32(43242).unwrap());
    let mut buf = [0; 4];
    let st = &*bb(rch.encode_utf8(&mut buf));
    let utf8char = Utf8Char::from_char(rch);

    c.bench_function("str.next_char", |c| {
        c.iter(|| unsafe { bb(bb(st).chars().next().unwrap_unchecked()) })
    });

    c.bench_function("utf8char::as_char", |c| {
        c.iter(|| bb(bb(utf8char).to_char()))
    });

    c.bench_function("utf8char::from_char", |c| {
        c.iter(|| bb(Utf8Char::from_char(bb(rch))))
    });

    c.bench_function("encode_unicode::Utf8Char::new", |c| {
        c.iter(|| bb(encode_unicode::Utf8Char::new(bb(rch))))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
