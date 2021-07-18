use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol, SymbolPair};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::borrow::BorrowMut;
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
    ) -> Option<(&'a Symbol, &'a Symbol)> {
        symbols
            .iter()
            .tuple_windows::<(&'a Symbol, &'a Symbol)>()
            .filter_map(
                |pair| match self.get_merge_score(pair.0, pair.0, input_text) {
                    Some(rank) => Some((pair, rank)),
                    None => None,
                },
            )
            .max_by_key(|(pair, rank)| *rank)
            .map(|(pair, _)| (pair.0, pair.1))
    }
}

impl BpeTokenizer for NaiveBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize(&self, input_text: &str) -> Vec<&str> {
        let mut symbols = Self::pre_populate_symbols(input_text);
        while let Some(best_pair) = self.find_best_merge(&symbols, input_text) {
            self.merge_symbols(&mut symbols, best_pair.0, best_pair.1);
        }
        println!("{:?}", symbols);
        Vec::new()
    }
}
