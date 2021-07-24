use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::collections::BTreeSet;
use std::path::Path;

pub struct NaiveBpeTokenizer {
    merges_vocab: MergesVocab,
}

impl NaiveBpeTokenizer {
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
}

impl BpeTokenizer for NaiveBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let (text, byte_mapping) = self.pre_process_text(input_text, '\u{2581}');

        let mut symbols = Self::pre_populate_symbols(text.as_str());
        while let Some(best_pair) = self.find_best_merge(&symbols, text.as_str()) {
            self.merge_symbols(&mut symbols, &best_pair.0, &best_pair.1);
        }
        let mut output = Vec::new();
        for symbol in symbols {
            output.push(
                &input_text[byte_mapping[&symbol.start_byte]..byte_mapping[&symbol.end_byte]],
            );
        }
        output
    }
}
