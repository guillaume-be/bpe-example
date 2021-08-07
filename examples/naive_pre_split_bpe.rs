use bpe_example::{
    download_file_to_cache, get_file_header, BpeTokenizer, NaivePreSplitBpeTokenizer,
};

fn main() -> anyhow::Result<()> {
    let hamlet_path = download_file_to_cache(
        "https://gist.githubusercontent.com/provpup/2fc41686eab7400b796b/raw/b575bd01a58494dfddc1d6429ef0167e709abf9b/hamlet.txt",
        "hamlet.txt")?;
    let model_file = download_file_to_cache(
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model",
        "bpe.model",
    )?;

    let sample_size = 10;
    let hamlet = get_file_header(&hamlet_path, sample_size)?;

    let tokenizer = NaivePreSplitBpeTokenizer::new(&model_file)?;
    let output = tokenizer.tokenize(hamlet.as_str());
    println!("{:?}", output);
    Ok(())
}
