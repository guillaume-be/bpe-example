use crate::proto::sentencepiece_model::ModelProto;
use protobuf::{Message, ProtobufError};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub type MergesVocab = HashMap<String, i64>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Symbol {
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_byte.cmp(&other.start_byte)
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SymbolPair {
    left: Symbol,
    right: Symbol,
    score: i64,
}

impl Ord for SymbolPair {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| other.left.start_byte.cmp(&self.left.start_byte))
    }
}

impl PartialOrd for SymbolPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait BpeTokenizer {
    fn read_proto(merges_path: &Path) -> Result<MergesVocab, ProtobufError> {
        let mut f = File::open(merges_path)?;
        let mut contents = Vec::new();
        let _ = f.read_to_end(&mut contents)?;
        let proto = ModelProto::parse_from_bytes(contents.as_slice())?;

        let mut values = MergesVocab::new();
        for (idx, piece) in proto.get_pieces().iter().enumerate() {
            values.insert(piece.get_piece().to_owned(), idx as i64);
        }
        Ok(values)
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

    fn get_merges_vocab(&self) -> &MergesVocab;

    fn get_merge_score(&self, symbol_1: &Symbol, symbol_2: &Symbol, text: &str) -> Option<i64> {
        self.get_merges_vocab()
            .get(&text[symbol_1.start_byte..symbol_2.end_byte])
            .map(|score| *score)
    }

    fn merge_symbols(&self, symbols: &mut BTreeSet<Symbol>, symbol_1: &Symbol, symbol_2: &Symbol) {
        symbols.remove(symbol_2);
        symbols.replace(Symbol {
            start_byte: symbol_1.start_byte,
            end_byte: symbol_2.end_byte,
        });
    }

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str>;
}
