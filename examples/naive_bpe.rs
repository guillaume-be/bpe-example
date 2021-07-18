use bpe_example::{download_file_to_cache, BpeTokenizer, NaiveBpeTokenizer};

fn main() -> anyhow::Result<()> {
    let _hamlet_path = download_file_to_cache(
        "https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt",
        "hamlet.txt")?;
    let model_file = download_file_to_cache(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
        "bpe.model",
    )?;

    let tokenizer = NaiveBpeTokenizer::new(&model_file)?;
    let input_text = "Hello, world!";
    let output = tokenizer.tokenize(input_text);
    Ok(())
}
