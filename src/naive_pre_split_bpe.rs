use crate::bpe_base::{BpeTokenizer, MergesVocab};
use crate::naive_bpe::SymbolArray;
use protobuf::ProtobufError;
use std::path::Path;

pub struct NaivePreSplitBpeTokenizer {
    merges_vocab: MergesVocab,
}

impl NaivePreSplitBpeTokenizer {
    pub fn new(merges_path: &Path) -> Result<Self, ProtobufError> {
        let merges_vocab = Self::read_proto(merges_path)?;
        Ok(Self { merges_vocab })
    }

    fn split_whitespace_punctuation<'a>(
        &self,
        input_string: &'a str,
        whitespace_token: char,
    ) -> Vec<&'a str> {
        let mut output: Vec<&str> = Vec::new();
        let mut start: usize = 0;

        for (c_pos, c) in input_string.char_indices() {
            if c == whitespace_token {
                if start < c_pos {
                    output.push(&input_string[start..c_pos]);
                }
                start = c_pos;
            } else if c.is_ascii_punctuation() {
                if start < c_pos {
                    output.push(&input_string[start..c_pos]);
                }
                output.push(&input_string[c_pos..c_pos + c.len_utf8()]);
                start = c_pos + c.len_utf8();
            }
        }
        if start < input_string.len() {
            output.push(&input_string[start..]);
        }
        output
    }
}

impl BpeTokenizer for NaivePreSplitBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let whitespace_token = '\u{2581}';

        let (text, byte_mapping) = self.pre_process_text(input_text, whitespace_token);
        let split_texts = self.split_whitespace_punctuation(text.as_str(), whitespace_token);

        let mut output = Vec::new();
        let mut offset = 0;
        for split_text in split_texts {
            let mut symbols = SymbolArray::from_text(split_text);
            while let Some(best_pair_index) = symbols.find_best_merge(split_text, self) {
                symbols.merge_symbols(best_pair_index);
            }
            for symbol in symbols.symbols {
                output.push(
                    &input_text[byte_mapping[&(offset + symbol.start_byte)]
                        ..byte_mapping[&(offset + symbol.end_byte)]],
                );
            }
            offset += split_text.len();
        }
        output
    }
}
