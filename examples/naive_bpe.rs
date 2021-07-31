use bpe_example::{download_file_to_cache, BpeTokenizer, NaiveBpeTokenizer};
use std::fs::File;
use std::io;
use std::io::BufRead;

fn main() -> anyhow::Result<()> {
    let hamlet_path = download_file_to_cache(
        "https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt",
        "hamlet.txt")?;
    let model_file = download_file_to_cache(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
        "bpe.model",
    )?;

    let sample_size = 10;
    let file = File::open(hamlet_path)?;
    let mut hamlet = String::new();
    for line in io::BufReader::new(file).lines().take(sample_size) {
        if let Ok(line) = line {
            if !line.is_empty() {
                hamlet.push_str(&line.trim_start());
                hamlet.push(' ');
            }
        }
    }

    let tokenizer = NaiveBpeTokenizer::new(&model_file)?;
    let output = tokenizer.tokenize(hamlet.as_str());
    println!("{:?}", output);
    Ok(())
}
