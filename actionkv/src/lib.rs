use std::{collections::HashMap, fs::OpenOptions};
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};
use crc::crc32;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use std::io::{self, BufReader, SeekFrom, Seek, Read};

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

    ///
    /// Populates the index by mapping keys to file positions
    /// 
    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&mut self.f);

        loop {
            let position = f.seek(SeekFrom::Current(0))?; // returns the number of bytes from the start of the file which becomes the index

            let maybe_kv = ActionKV::process_record(&mut f);

            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        _ => return Err(err),
                    }
                }
            };
            self.index.insert(kv.key, position);
        }

        Ok(())
    }

    ///
    /// Processes the record by reading 12 bytes that represent three integers: a checksum, the length of the key, and the length of the value
    /// and then use them to read the rest of the data from disk and verify data integrity. 
    /// 
    pub fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        let mut data = ByteString::with_capacity(data_len as usize);

        // f.by_ref() is required because take(n) creates a new Read value
        // using a reference within this short-lived block sidesteps ownership issues.
        {
            f.by_ref()
            .take(data_len as u64)
            .read_to_end(&mut data)?;
        }

        debug_assert_eq!(data.len(), data_len as usize); // test runs only in debug build

        let checksum = crc32::checksum_ieee(&data); // checksum (a number) verifies that the bytes read from disk are the same as what was intended
        if checksum != saved_checksum {
            panic!("data corruption encountered ({:08x} != {:08x})", checksum, saved_checksum);
        }

        let value = data.split_off(key_len as usize);
        let key = data;

        Ok(KeyValuePair { key, value })
    }
}
