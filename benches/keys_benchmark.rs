use bytes::{Bytes, BytesMut};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use redis_proto::asyncresp::RespParser;
use tokio_util::codec::Decoder;

fn bench_parsing(c: &mut Criterion) {
    let buf: String = std::iter::repeat("a").take(100).collect();
    let mut decoder = RespParser::default();
    let mut group = c.benchmark_group("decoding");
    group.throughput(Throughput::Bytes(buf.len() as u64 + 3));
    group.bench_function("simple_string", |b| {
        let _ = b.iter(|| {
            let mut buf = BytesMut::from(format!("+{}\r\n", buf).as_str());
            decoder
                .decode(black_box(&mut buf))
                .expect("parsing to work");
        });
    });
    group.finish();
}

fn bench_translate(c: &mut Criterion) {
    let values: Bytes = std::iter::repeat("a").take(200).collect::<String>().into();
    
}

criterion_group!(
    benches,
    bench_parsing,
);

criterion_main!(benches);