//! Runtime API definition for the Rialto Channel API

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    pub trait RialtoChannelApi {
        fn get_merkle_roots() -> u64;
    }
}
