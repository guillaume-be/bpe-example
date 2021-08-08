use crate::bpe_base::{BpeTokenizer, MergesVocab, Symbol, SymbolPair};
use itertools::Itertools;
use protobuf::ProtobufError;
use std::collections::{BTreeSet, BinaryHeap};
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

    fn pre_populate_symbols(input_text: &str) -> BTreeSet<Symbol> {
        let mut symbols = BTreeSet::new();
        for (character_start, character) in input_text.char_indices() {
            symbols.insert(Symbol {
                start_byte: character_start,
                end_byte: character_start + character.len_utf8(),
            });
        }
        symbols
    }

    fn merge_symbols<'a>(
        &self,
        symbols: &'a mut BTreeSet<Symbol>,
        symbol_1: &Symbol,
        symbol_2: &Symbol,
    ) -> Symbol {
        symbols.remove(symbol_1);
        symbols.remove(symbol_2);
        let new_symbol = Symbol {
            start_byte: symbol_1.start_byte,
            end_byte: symbol_2.end_byte,
        };
        symbols.insert(new_symbol);
        new_symbol
    }
}

impl BpeTokenizer for PriorityQueueBpeTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let (text, byte_mapping) = self.pre_process_text(input_text, '\u{2581}');

        let mut symbols: BTreeSet<Symbol> = Self::pre_populate_symbols(text.as_str());
        let mut agenda: BinaryHeap<SymbolPair> = BinaryHeap::new();

        for (left_symbol, right_symbol) in symbols.iter().tuple_windows::<(&Symbol, &Symbol)>() {
            self.maybe_add_pair(left_symbol, right_symbol, text.as_str(), &mut agenda);
        }
        while let Some(symbol_pair) = agenda.pop() {
            let left_symbol = symbols.get(&symbol_pair.left).cloned();
            let right_symbol = symbols.get(&symbol_pair.right).cloned();

            if let (Some(left_symbol), Some(right_symbol)) = (left_symbol, right_symbol) {
                let new_symbol = self.merge_symbols(&mut symbols, &left_symbol, &right_symbol);
                if let Some(next) = symbols.range(new_symbol..).nth(1) {
                    self.maybe_add_pair(&new_symbol, next, text.as_str(), &mut agenda);
                }
                if let Some(prev) = symbols.range(..new_symbol).next_back() {
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
