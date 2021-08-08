use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::path::Path;

#[derive(Debug)]
pub struct SymbolArray {
    pub symbols: Vec<Symbol>,
}

impl IntoIterator for SymbolArray {
    type Item = Symbol;
    type IntoIter = <Vec<Symbol> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.symbols.into_iter()
    }
}

impl SymbolArray {
    pub fn from_text(input_text: &str) -> Self {
        let mut symbols = Vec::new();
        for (character_start, character) in input_text.char_indices() {
            symbols.push(Symbol {
                start_byte: character_start,
                end_byte: character_start + character.len_utf8(),
            });
        }
        Self { symbols }
    }

    pub fn find_best_merge<T>(&self, input_text: &str, tokenizer: &T) -> Option<usize>
    where
        T: BpeTokenizer,
    {
        self.symbols
            .iter()
            .tuple_windows::<(&Symbol, &Symbol)>()
            .enumerate()
            .filter_map(|(pos, (first, second))| {
                tokenizer
                    .get_merge_score(first, second, input_text)
                    .map(|rank| (pos, rank))
            })
            .min_by_key(|(_, rank)| *rank)
            .map(|(pos, _)| pos)
    }

    pub fn merge_symbols(&mut self, best_pair_index: usize) -> Symbol {
        let new_symbol = Symbol {
            start_byte: self.symbols[best_pair_index].start_byte,
            end_byte: self.symbols[best_pair_index + 1].end_byte,
        };
        self.symbols.remove(best_pair_index + 1);
        self.symbols.remove(best_pair_index);
        self.symbols.insert(best_pair_index, new_symbol);
        new_symbol
    }
}

pub struct NaiveBpeTokenizer {
    merges_vocab: MergesVocab,
}

impl NaiveBpeTokenizer {
    pub fn new(merges_path: &Path) -> Result<Self, ProtobufError> {
        let merges_vocab = Self::read_proto(merges_path)?;
        Ok(Self { merges_vocab })
    }
}

impl BpeTokenizer for NaiveBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let (text, byte_mapping) = self.pre_process_text(input_text, '\u{2581}');

        let mut symbols = SymbolArray::from_text(text.as_str());
        while let Some(best_pair_index) = symbols.find_best_merge(text.as_str(), self) {
            symbols.merge_symbols(best_pair_index);
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
