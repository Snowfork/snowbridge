#![allow(dead_code)]
#![allow(unused_variables)]

pub use primitive_types::H160;

type Address = H160;
type RawMessage = Vec<u8>;
type Signature = Vec<u8>;

fn verify(message: RawMessage, signature: Signature) -> bool {
    false
}
