use std::{collections::HashMap, fs::OpenOptions};
use std::fs::File;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use std::io::{self, BufReader, SeekFrom};

type ByteString = Vec<u8>; // String in the form of raw bytes

type ByteStr = [u8]; // str in the form of raw bytes

#[derive(Debug, Serialize, Deserialize)] // generate serialized code to write k, v pairs to disk
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64> // mapping b/w keys and file locations
}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .append(true)
                        .open(path)?;
        let index = HashMap::new();

        Ok(ActionKV { f, index })
    }
}
