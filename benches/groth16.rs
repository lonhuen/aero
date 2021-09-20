//use criterion::{black_box, criterion_group, criterion_main, Criterion};
//use quail::zksnark::Prover;
//
//fn new_prover() {
//    // read the key
//    //let prover = Prover::new("data/encryption.txt", "data/pk.txt");
//}
//
//fn criterion_benchmark(c: &mut Criterion) {
//    let mut group = c.benchmark_group("new prover");
//    group.significance_level(0.1).sample_size(10);
//    c.bench_function("setup prover", |b| b.iter(|| new_prover()));
//    group.finish();
//}
//
//criterion_group!(benches, criterion_benchmark);
//criterion_main!(benches);
//
