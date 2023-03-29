pub use cheat_codes::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod cheat_codes {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"difficulty\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"message\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectRevert\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"prank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"roll\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"warp\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static CHEATCODES_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct CheatCodes<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for CheatCodes<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for CheatCodes<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for CheatCodes<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for CheatCodes<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(CheatCodes)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> CheatCodes<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    CHEATCODES_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `difficulty` (0x46cc92d9) function
        pub fn difficulty(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([70, 204, 146, 217], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectRevert` (0xc31eb0e0) function
        pub fn expect_revert(
            &self,
            message: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([195, 30, 176, 224], message)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `prank` (0xca669fa7) function
        pub fn prank(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([202, 102, 159, 167], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `roll` (0x1f7b4f30) function
        pub fn roll(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 123, 79, 48], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `warp` (0xe5d6bf02) function
        pub fn warp(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([229, 214, 191, 2], p0)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for CheatCodes<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `difficulty` function with signature `difficulty(uint256)` and selector `0x46cc92d9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "difficulty", abi = "difficulty(uint256)")]
    pub struct DifficultyCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `expectRevert` function with signature `expectRevert(bytes4)` and selector `0xc31eb0e0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert(bytes4)")]
    pub struct ExpectRevertCall {
        pub message: [u8; 4],
    }
    ///Container type for all input parameters for the `prank` function with signature `prank(address)` and selector `0xca669fa7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "prank", abi = "prank(address)")]
    pub struct PrankCall(pub ::ethers::core::types::Address);
    ///Container type for all input parameters for the `roll` function with signature `roll(uint256)` and selector `0x1f7b4f30`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "roll", abi = "roll(uint256)")]
    pub struct RollCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `warp` function with signature `warp(uint256)` and selector `0xe5d6bf02`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "warp", abi = "warp(uint256)")]
    pub struct WarpCall(pub ::ethers::core::types::U256);
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum CheatCodesCalls {
        Difficulty(DifficultyCall),
        ExpectRevert(ExpectRevertCall),
        Prank(PrankCall),
        Roll(RollCall),
        Warp(WarpCall),
    }
    impl ::ethers::core::abi::AbiDecode for CheatCodesCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <DifficultyCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Difficulty(decoded));
            }
            if let Ok(decoded)
                = <ExpectRevertCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectRevert(decoded));
            }
            if let Ok(decoded)
                = <PrankCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Prank(decoded));
            }
            if let Ok(decoded)
                = <RollCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Roll(decoded));
            }
            if let Ok(decoded)
                = <WarpCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Warp(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for CheatCodesCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Difficulty(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExpectRevert(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Prank(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Roll(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Warp(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for CheatCodesCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Difficulty(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectRevert(element) => ::core::fmt::Display::fmt(element, f),
                Self::Prank(element) => ::core::fmt::Display::fmt(element, f),
                Self::Roll(element) => ::core::fmt::Display::fmt(element, f),
                Self::Warp(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<DifficultyCall> for CheatCodesCalls {
        fn from(value: DifficultyCall) -> Self {
            Self::Difficulty(value)
        }
    }
    impl ::core::convert::From<ExpectRevertCall> for CheatCodesCalls {
        fn from(value: ExpectRevertCall) -> Self {
            Self::ExpectRevert(value)
        }
    }
    impl ::core::convert::From<PrankCall> for CheatCodesCalls {
        fn from(value: PrankCall) -> Self {
            Self::Prank(value)
        }
    }
    impl ::core::convert::From<RollCall> for CheatCodesCalls {
        fn from(value: RollCall) -> Self {
            Self::Roll(value)
        }
    }
    impl ::core::convert::From<WarpCall> for CheatCodesCalls {
        fn from(value: WarpCall) -> Self {
            Self::Warp(value)
        }
    }
}
