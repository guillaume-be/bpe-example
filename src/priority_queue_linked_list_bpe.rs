use crate::bpe_base::{BpeTokenizer, MergesVocab};
use protobuf::ProtobufError;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Index;
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SymbolNode {
    pub start_byte: usize,
    pub end_byte: usize,
    pub prev: isize,
    pub next: isize,
    pub size: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SymbolNodePair {
    pub left: isize,
    pub right: isize,
    pub score: i64,
    pub pair_size: usize,
}

impl Ord for SymbolNodePair {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| other.left.cmp(&self.left))
    }
}

impl PartialOrd for SymbolNodePair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct SymbolList {
    symbols: Vec<Option<SymbolNode>>,
}

impl Index<usize> for SymbolList {
    type Output = Option<SymbolNode>;

    fn index(&self, index: usize) -> &Option<SymbolNode> {
        self.symbols.index(index)
    }
}

impl IntoIterator for SymbolList {
    type Item = Option<SymbolNode>;
    type IntoIter = <Vec<Option<SymbolNode>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.symbols.into_iter()
    }
}

impl SymbolList {
    pub fn from_text(input_text: &str) -> Self {
        let mut symbols = Vec::with_capacity(input_text.len());
        for (index, (character_start, character)) in input_text.char_indices().enumerate() {
            let next = if index == input_text.char_indices().count() {
                -1
            } else {
                (index + 1) as isize
            };
            symbols.push(Some(SymbolNode {
                start_byte: character_start,
                end_byte: character_start + character.len_utf8(),
                prev: index as isize - 1,
                next,
                size: 1,
            }));
        }
        Self { symbols }
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn merge_symbols(
        &mut self,
        symbol_1_index: usize,
        symbol_2_index: usize,
        size_validation: usize,
    ) -> Option<SymbolNode> {
        if let (Some(left_symbol), Some(right_symbol)) =
            (self[symbol_1_index], self[symbol_2_index])
        {
            if left_symbol.size + right_symbol.size != size_validation {
                return None;
            }
            if right_symbol.next != -1 {
                if let Some(next_next) = self.symbols.get_mut(right_symbol.next as usize).unwrap() {
                    next_next.prev = symbol_1_index as isize;
                }
            }
            let new_symbol = SymbolNode {
                start_byte: left_symbol.start_byte,
                end_byte: right_symbol.end_byte,
                prev: left_symbol.prev,
                next: right_symbol.next,
                size: left_symbol.size + right_symbol.size,
            };
            self.symbols[symbol_2_index] = None;
            self.symbols[symbol_1_index] = Some(new_symbol);
            Some(new_symbol)
        } else {
            None
        }
    }
}

pub struct PriorityQueueBpeLLTokenizer {
    merges_vocab: MergesVocab,
}

impl PriorityQueueBpeLLTokenizer {
    pub fn new(merges_path: &Path) -> Result<Self, ProtobufError> {
        let merges_vocab = Self::read_proto(merges_path)?;
        Ok(Self { merges_vocab })
    }

    fn maybe_add_pair(
        &self,
        left_symbol_index: isize,
        right_symbol_index: isize,
        input_text: &str,
        symbols: &SymbolList,
        agenda: &mut BinaryHeap<SymbolNodePair>,
    ) {
        if left_symbol_index != -1 && right_symbol_index != -1 {
            if let (Some(left_symbol), Some(right_symbol)) = (
                symbols[left_symbol_index as usize],
                symbols[right_symbol_index as usize],
            ) {
                let merged_text = &input_text[left_symbol.start_byte..right_symbol.end_byte];
                if let Some(&score) = self.merges_vocab.get(merged_text) {
                    agenda.push(SymbolNodePair {
                        left: left_symbol_index,
                        right: right_symbol_index,
                        score,
                        pair_size: left_symbol.size + right_symbol.size,
                    })
                }
            }
        }
    }
}

impl BpeTokenizer for PriorityQueueBpeLLTokenizer {
    fn get_merges_vocab(&self) -> &MergesVocab {
        &self.merges_vocab
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str> {
        let (text, byte_mapping) = self.pre_process_text(input_text, '\u{2581}');

        let mut symbols = SymbolList::from_text(text.as_str());
        let mut agenda: BinaryHeap<SymbolNodePair> = BinaryHeap::new();

        for symbol_index in 1..symbols.len() {
            self.maybe_add_pair(
                symbol_index as isize - 1,
                symbol_index as isize,
                text.as_str(),
                &symbols,
                &mut agenda,
            );
        }

        while let Some(symbol_pair) = agenda.pop() {
            let left_symbol_index = symbol_pair.left;
            let right_symbol_index = symbol_pair.right;
            if left_symbol_index != -1 && right_symbol_index != -1 {
                let new_symbol = symbols.merge_symbols(
                    left_symbol_index as usize,
                    right_symbol_index as usize,
                    symbol_pair.pair_size,
                );
                if let Some(new_symbol) = new_symbol {
                    self.maybe_add_pair(
                        new_symbol.prev,
                        left_symbol_index,
                        text.as_str(),
                        &symbols,
                        &mut agenda,
                    );
                    self.maybe_add_pair(
                        left_symbol_index,
                        new_symbol.next,
                        text.as_str(),
                        &symbols,
                        &mut agenda,
                    );
                }
            }
        }

        let mut output = Vec::new();
        for symbol in symbols {
            if let Some(symbol) = symbol {
                output.push(
                    &input_text[byte_mapping[&symbol.start_byte]..byte_mapping[&symbol.end_byte]],
                );
            }
        }
        output
    }
}
