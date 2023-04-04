use core::fmt;
use std::{fmt::Debug};
use std::io::Write; // import the Write trait
use byteorder::{WriteBytesExt, LittleEndian};
use std::io::Cursor;
use sha256::digest;


pub struct Block {
    pub blocknum : u64,
    pub hash : String,
    pub prev_hash : String,
    pub timestamp : u128,
    pub statechanges : String,
}

impl Block {
    pub fn new(blocknum: u64, hash: String, prev_hash: String, timestamp: u128, statechanges: String) -> Self {
        Block { blocknum, hash, prev_hash, timestamp, statechanges }
    }
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.blocknum.to_be_bytes());
        bytes.extend_from_slice(self.statechanges.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(self.statechanges.as_bytes());
        bytes
    }

    pub fn hash(&self) -> String {
        // Convert whole BLOCK struct into bytes then hash it

        // let mut vec = Vec::new();
        // let mut cursor = Cursor::new(&mut vec);
        // cursor.write_u64::<LittleEndian>(self.blocknum).unwrap();
        // // Write the hash as 32 bytes (assuming it's always 32 bytes long)
        // cursor.write_all(&self.hash).unwrap();
        // // Write the prev_hash as 32 bytes (assuming it's always 32 bytes long)
        // cursor.write_all(&self.prev_hash).unwrap();
        // // Write the timestamp as 16 bytes in little endian order
        // cursor.write_u128::<LittleEndian>(self.timestamp).unwrap();
        // // Extend the vec with the bytes of the statechanges string
        // vec.extend_from_slice(self.statechanges.as_bytes());
        // // Return the vec
        // vec
        // Create a longer lived value for the vector
    let bytes = self.as_bytes();
    // Take a slice of it
    let slice = bytes.as_slice();
    digest(slice)

    }
}

impl  Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Block {} : \nmined at {} with hash {:?} , \nprev hash :  {:?} and \nfollowing statechanges  {:?}" , &self.blocknum , &self.timestamp , &self.hash , &self.prev_hash , &self.statechanges)
    }
}

pub struct  BlockChain {
    pub chain : Vec<Block>
}