#[macro_use]
extern crate criterion;

use bpe_example::{download_file_to_cache, get_file_header, BpeTokenizer, NaiveBpeTokenizer};
use criterion::{black_box, Criterion};
use std::time::{Duration, Instant};

fn get_tokenizer(model_url: &str) -> NaiveBpeTokenizer {
    let model_file = download_file_to_cache(model_url, "bpe.model").unwrap();
    NaiveBpeTokenizer::new(&model_file).unwrap()
}

fn get_corpus(corpus_url: &str, n_lines: usize) -> String {
    let corpus_file = download_file_to_cache(corpus_url, "corpus.txt").unwrap();
    get_file_header(&corpus_file, n_lines).unwrap()
}

fn tokenize(iters: u64, tokenizer: &NaiveBpeTokenizer, corpus: &str) -> Duration {
    let mut duration = Duration::new(0, 0);
    for _i in 0..iters {
        let start = Instant::now();
        let _ = tokenizer.tokenize(corpus);
        duration = duration.checked_add(start.elapsed()).unwrap();
    }
    duration
}

fn bench_tokenization_1(c: &mut Criterion) {
    let sample_size = 1;

    let corpus = get_corpus("https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt", sample_size);
    let tokenizer = get_tokenizer(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
    );

    c.bench_function("Tokenization 1 lines", |b| {
        b.iter_custom(|iters| black_box(tokenize(iters, &tokenizer, corpus.as_str())))
    });
}

fn bench_tokenization_10(c: &mut Criterion) {
    let sample_size = 10;

    let corpus = get_corpus("https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt", sample_size);
    let tokenizer = get_tokenizer(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
    );

    c.bench_function("Tokenization 10 lines", |b| {
        b.iter_custom(|iters| black_box(tokenize(iters, &tokenizer, corpus.as_str())))
    });
}

fn bench_tokenization_100(c: &mut Criterion) {
    let sample_size = 100;

    let corpus = get_corpus("https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt", sample_size);
    let tokenizer = get_tokenizer(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
    );

    c.bench_function("Tokenization 100 lines", |b| {
        b.iter_custom(|iters| black_box(tokenize(iters, &tokenizer, corpus.as_str())))
    });
}

fn bench_tokenization_1000(c: &mut Criterion) {
    let sample_size = 1000;

    let corpus = get_corpus("https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt", sample_size);
    let tokenizer = get_tokenizer(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
    );

    c.bench_function("Tokenization 1000 lines", |b| {
        b.iter_custom(|iters| black_box(tokenize(iters, &tokenizer, corpus.as_str())))
    });
}

criterion_group! {
name = benches;
config = Criterion::default();
targets = bench_tokenization_1, bench_tokenization_10, bench_tokenization_100, bench_tokenization_1000
}

criterion_main!(benches);
