use std::collections::HashMap;

use serde::{Serialize, Deserialize, de::DeserializeOwned};

use crate::{MAX_PACKET_SIZE, serialized_size, serialize, deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct ElementHeader {
    key: String,
    size: u64,
}

impl ElementHeader {
    fn new<T: Serialize>(key: &str, value: &T) -> Option<Self> {
        serialized_size(value).map(|size| Self { key: key.to_string(), size })
    }
}

pub struct HashPacketEncoder(Vec<u8>);

impl HashPacketEncoder {
    pub fn new() -> Self {
        //reserve four bytes at the beginning for storing a u32 that will contain our size
        //Self(vec![0u8; 4])
        Self(Vec::new())
    }

    pub fn add_element<T: Serialize>(&mut self, key: &str, value: T) -> bool {
        //get the size of the element serialized
        let header = match ElementHeader::new(key, &value) {
            Some(header) => header,
            None => return false,
        };

        let serialized_header = match serialize(&header) {
            Some(header) => header,
            None => return false,
        };
        let serialized_value = match serialize(&value) {
            Some(serialized) => serialized,
            None => return false,
        };

        let new_len = self.0.len() + serialized_header.len() + serialized_value.len();

        if new_len < MAX_PACKET_SIZE {
            self.0.extend_from_slice(serialized_header.as_slice());
            self.0.extend_from_slice(serialized_value.as_slice());
            true
        } else {
            false
        }
    }

    pub fn submit(self) -> Vec<u8> {
        self.0
    }
}

pub struct HashPacketDecoder(HashMap<String, Vec<u8>>);

impl HashPacketDecoder {
    pub fn decode(bytes: Vec<u8>) -> Self {
        let mut elements: HashMap<String, Vec<u8>> = HashMap::new();
        let mut offset = 0;

        //iterate our bytes decoding and doing other shit
        //read in our size here
        while offset < bytes.len() {
            if let Some(header) = deserialize::<ElementHeader>(&bytes[offset..]) {
                offset += serialized_size(&header).unwrap() as usize;

                //now in here try and get the next hunk of data
                let element = bytes[offset..(offset + header.size as usize)].to_vec();
                let old_element = elements.insert(header.key, element);
                assert!(old_element == None, "Overwriting packet key!");

                offset += header.size as usize;
            } else {
                break;
            }
        }

        Self(elements)
    }

    pub fn take_element<T: DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        self.0
            .remove(key)
            .map(|bytes| {
                deserialize(&bytes).expect("Failed to deserialize element from packet")
            })
    }
}
