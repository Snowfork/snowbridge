use sp_std::vec::Vec;

/// Selector for target application
///
/// This is an opaque byte identifier that can only be decoded by verifiers and
/// target applications.
///
/// For example it could contain an Ethereum contract address.
pub type AppID = [u8; 32];

/// Raw message from relayer
pub type Message = Vec<u8>;

