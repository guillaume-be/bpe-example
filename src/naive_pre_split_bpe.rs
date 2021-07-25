use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::collections::BTreeSet;
use std::path::Path;

pub struct NaivePreSplitBpeTokenizer {
    merges_vocab: MergesVocab,
}

impl NaivePreSplitBpeTokenizer {
    pub fn new(merges_path: &Path) -> Result<Self, ProtobufError> {
        let merges_vocab = Self::read_proto(merges_path)?;
        Ok(Self { merges_vocab })
    }

    fn find_best_merge<'a>(
        &self,
        symbols: &'a BTreeSet<Symbol>,
        input_text: &str,
    ) -> Option<(Symbol, Symbol)> {
        symbols
            .iter()
            .tuple_windows::<(&'a Symbol, &'a Symbol)>()
            .filter_map(
                |pair| match self.get_merge_score(pair.0, pair.1, input_text) {
                    Some(rank) => Some((pair, rank)),
                    None => None,
                },
            )
            .max_by_key(|(_, rank)| *rank)
            .map(|(pair, _)| (*pair.0, *pair.1))
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
            let mut symbols = Self::pre_populate_symbols(split_text);
            while let Some(best_pair) = self.find_best_merge(&symbols, split_text) {
                self.merge_symbols(&mut symbols, &best_pair.0, &best_pair.1);
            }
            for symbol in symbols {
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
