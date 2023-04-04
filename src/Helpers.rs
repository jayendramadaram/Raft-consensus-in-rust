use core::fmt;
use std::fmt::Debug;

pub struct Block {
    pub blocknum : u64,
    pub hash : Vec<u8>,
    pub prev_hash : Vec<u8>,
    pub timestamp : u128,
    pub statechanges : Vec<String>,
}

impl Block {
    pub fn new() -> Self {
        
    }
}

impl  Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Block {} : \n mined at {} with hash {:?} , \n prev hash :  {:?} and \n  following statechanges  {:?}" , &self.blocknum , &self.timestamp , &self.hash , &self.prev_hash , &self.statechanges)
    }
}