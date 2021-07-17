use std::fs;
use std::fs::File;
use std::io::{copy, Read};
use std::path::{Path, PathBuf};

mod proto;

use crate::proto::sentencepiece_model::ModelProto;
use protobuf::{Message, ProtobufError};
use std::collections::HashMap;

/// Download a file target to a cache location
pub fn download_file_to_cache(src: &str, target: &str) -> Result<PathBuf, ureq::Error> {
    let mut home = dirs::home_dir().unwrap();
    home.push(".cache");
    home.push(target);
    if !home.exists() {
        let mut response = ureq::get(src).call()?.into_reader();
        fs::create_dir_all(home.parent().unwrap()).unwrap();
        let mut dest = File::create(&home).unwrap();
        copy(&mut response, &mut dest).unwrap();
    }
    Ok(home)
}

pub trait SentencePieceLoader {
    fn read_proto(src: &Path) -> Result<HashMap<String, i64>, ProtobufError> {
        let mut f = File::open(src)?;
        let mut contents = Vec::new();
        let _ = f.read_to_end(&mut contents)?;
        let proto = ModelProto::parse_from_bytes(contents.as_slice())?;

        let mut values = HashMap::new();
        for (idx, piece) in proto.get_pieces().iter().enumerate() {
            values.insert(piece.get_piece().to_owned(), idx as i64);
        }
        Ok(values)
    }
}
