/// Selector for target application
///
/// This is an opaque byte identifier that can only be decoded by verifiers and
/// target applications.
///
/// For example it could contain an Ethereum contract address.

use sp_std::prelude::*;

pub type AppID = [u8; 32];

/// Message from relayer
pub type Message = Vec<u8>;
