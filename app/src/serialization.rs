use serde::{de::DeserializeOwned, Serialize};

pub fn serialize<T: Serialize>(value: &T) -> Option<Vec<u8>> {
    bincode::serialize(value)
        .or_else(|e| {
            println!("Serialization error {}", e);
            Err(e)
        })
        .ok()
}

pub fn serialized_size<T: Serialize>(value: &T) -> Option<u64> {
    bincode::serialized_size(value)
        .or_else(|e| {
            println!("Serialization size error {}", e);
            Err(e)
        })
        .ok()
}

pub fn deserialize<T>(buffer: &[u8]) -> Option<T>
where
    T: DeserializeOwned,
{
    bincode::deserialize_from(buffer)
        .or_else(|e| {
            println!("Deserialization error {}", e);
            Err(e)
        })
        .ok()
}

