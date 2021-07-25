use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol, SymbolPair};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::collections::BinaryHeap;
use std::path::Path;

pub struct PriorityQueueBpeTokenizer {
    merges_vocab: MergesVocab,
}

impl PriorityQueueBpeTokenizer {
    pub fn new(merges_path: &Path) -> Result<Self, ProtobufError> {
        let merges_vocab = Self::read_proto(merges_path)?;
        Ok(Self { merges_vocab })
    }

    fn maybe_add_pair(
        &self,
        left_symbol: &Symbol,
        right_symbol: &Symbol,
        input_text: &str,
        agenda: &mut BinaryHeap<SymbolPair>,
    ) {
        let merged_text = &input_text[left_symbol.start_byte..right_symbol.end_byte];
        if let Some(&score) = self.merges_vocab.get(merged_text) {
            agenda.push(SymbolPair {
                left: *left_symbol,
                right: *right_symbol,
                score,
            })
        }
    }
}

impl BpeTokenizer for PriorityQueueBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let (text, byte_mapping) = self.pre_process_text(input_text, '\u{2581}');

        let mut symbols = Self::pre_populate_symbols(text.as_str());
        let mut agenda: BinaryHeap<SymbolPair> = BinaryHeap::new();

        for (left_symbol, right_symbol) in symbols.iter().tuple_windows::<(&Symbol, &Symbol)>() {
            self.maybe_add_pair(left_symbol, right_symbol, text.as_str(), &mut agenda);
        }

        while let Some(symbol_pair) = agenda.pop() {
            let left_symbol = symbols.get(&symbol_pair.left).cloned();
            let right_symbol = symbols.get(&symbol_pair.right).cloned();

            if left_symbol.is_none() | right_symbol.is_none() {
                continue;
            } else {
                let new_symbol =
                    self.merge_symbols(&mut symbols, &left_symbol.unwrap(), &right_symbol.unwrap());

                if let Some(next) = symbols.range(..new_symbol).next() {
                    self.maybe_add_pair(&new_symbol, next, text.as_str(), &mut agenda);
                }
                if let Some(prev) = symbols.range(new_symbol..).next_back() {
                    self.maybe_add_pair(prev, &new_symbol, text.as_str(), &mut agenda);
                }
            }
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