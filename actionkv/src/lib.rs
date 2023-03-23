use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{self, BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use serde_derive::{Deserialize, Serialize};

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
    /// Opens a file at the specified path and returns a new instance of ActionKV 
    /// initialized with the file and an empty index.
    ///
    /// # Arguments
    ///
    /// * path - A reference to a Path type representing the path to the file to open.
    ///
    /// # Returns
    ///
    /// An io::Result containing a new instance of ActionKV initialized with the file at path and an empty index if the
    /// operation was successful.
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

    /// Reads a key-value pair from a file and returns it as a KeyValuePair.
    /// The function reads the checksum, key length, value length, and data from f, verifies that the checksum
    /// matches the computed checksum of the data.
    ///
    /// # Type parameters
    ///
    /// * R - A generic type that implements the Read trait.
    ///
    /// # Arguments
    ///
    /// * f - A mutable reference to a generic Read type representing the file to read from.
    ///
    /// # Returns
    ///
    /// An io::Result containing a KeyValuePair representing the key-value pair read from the file if the operation
    /// was successful. 
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

    /// Reads key-value pairs from a file and creates an index for each key-value pair.
    ///
    /// # Arguments
    ///
    /// None.
    ///
    /// # Returns
    ///
    /// An io::Result that indicates whether the operation was successful.
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

    /// Reads a key-value pair from the file at the specified byte offset position 
    /// and returns it as a KeyValuePair.
    ///
    /// # Arguments
    ///
    /// * position - An unsigned 64-bit integer representing the byte offset in the file where the key-value pair is stored.
    ///
    /// # Returns
    ///
    /// An io::Result containing the key-value pair stored at the specified byte offset position if the operation was successful.
    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&mut self.f);
        f.seek(SeekFrom::Start(position))?; // seek to position
        let kv = ActionKV::process_record(&mut f)?;

        Ok(kv)
    }

    /// Retrieves a value from the database given a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key that the value is associated with.
    ///
    /// # Returns
    ///
    /// If the key exists in the database, returns `Ok(Some(ByteString))`, where `ByteString` is the value associated with the key.
    ///
    /// If the key does not exist in the database, returns `Ok(None)`.
    ///
    /// If there is an I/O error, returns `Err(io::Error)`.
    ///
    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(position) => *position,
        };

        let kv = self.get_at(position)?;

        Ok(Some(kv.value))
    }

    /// Inserts a key-value pair into a file, ignoring the index. 
    ///
    /// # Arguments
    ///
    /// * key - A mutable reference to a ByteStr representing the key to insert into the file.
    /// * value - A mutable reference to a ByteStr representing the value to insert into the file.
    ///
    /// # Returns
    ///
    /// An io::Result containing a u64 representing the current position of the cursor within the file if the
    /// operation was successful.
    pub fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        let mut f = BufWriter::new(&mut self.f);
        let key_len = key.len();
        let value_len = value.len();
        let mut tmp = ByteString::with_capacity(key_len + value_len);

        for byte in key {
            tmp.push(*byte);
        }

        for byte in value {
            tmp.push(*byte);
        }

        let checksum = crc32::checksum_ieee(&tmp);

        let next_byte = SeekFrom::End(0); // position of end of the file
        let current_position = f.seek(SeekFrom::Current(0))?; // current position
        f.seek(next_byte)?; // seek to end of the file
        
        // write bytes
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(value_len as u32)?;
        f.write_all(&mut tmp)?;

        Ok(current_position)
    }

    /// Inserts a key-value pair into a file and creates an index for the key in the Hashmap. 
    ///
    /// # Arguments
    ///
    /// * key - A mutable reference to a ByteStr representing the key to insert into the file and index.
    /// * value - A mutable reference to a ByteStr representing the value to insert into the file.
    ///
    /// # Returns
    ///
    /// An io::Result that indicates whether the operation was successful.
    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        let position = self.insert_but_ignore_index(key, value)?;

        self.index.insert(key.to_vec(), position); // key.to_vec() converts the &ByteStr to a ByteString

        Ok(())
    }
}
