#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct StoredLeaves(pub Vec<Vec<u8>>);
