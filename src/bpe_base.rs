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
        self.start_byte
            .cmp(&other.start_byte)
            .then_with(|| self.end_byte.cmp(&other.end_byte))
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SymbolPair {
    pub left: Symbol,
    pub right: Symbol,
    pub score: i64,
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

    fn pre_process_text(
        &self,
        input_text: &str,
        whitespace_token: char,
    ) -> (String, HashMap<usize, usize>) {
        let mut byte_mapping: HashMap<usize, usize> = HashMap::new();
        let whitespace_token_len = whitespace_token.len_utf8();
        let mut pre_processed_text = Vec::new();
        let mut offset = 0;

        if !input_text.starts_with(whitespace_token) {
            pre_processed_text.push(whitespace_token);
            byte_mapping.insert(0, 0);
            offset += whitespace_token_len;
        };

        for (character_start, character) in input_text.char_indices() {
            byte_mapping.insert(character_start + offset, character_start);
            if character.is_whitespace() {
                pre_processed_text.push(whitespace_token);
                offset += whitespace_token_len - 1;
            } else {
                pre_processed_text.push(character);
            }
        }
        let pre_processed_text = pre_processed_text.iter().collect::<String>();
        byte_mapping.insert(pre_processed_text.len(), input_text.len());

        (pre_processed_text, byte_mapping)
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
            .get(&text[symbol_1.start_byte..symbol_2.end_byte]).copied()
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

    fn tokenize<'a>(&self, input_text: &'a str) -> Vec<&'a str>;
}
