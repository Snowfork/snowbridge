// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{traits::ContainsPair, weights::Weight};
use sp_core::{RuntimeDebug, H160};
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use xcm::v3::{prelude::*, Junction::AccountKey20};
use xcm_executor::traits::ConvertLocation;

const MINIMUM_DEPOSIT: u128 = 1;

/// Messages from Ethereum are versioned. This is because in future,
/// we want to evolve the protocol so that the ethereum side sends XCM messages directly. Instead
/// having BridgeHub transcode the messages into XCM.
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum VersionedMessage {
	V1(MessageV1),
}

/// For V1, the ethereum side sends messages which are transcoded into XCM. These messages are
/// self-contained, in that they can be transcoded using only information in the message.
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct MessageV1 {
	/// EIP-155 chain id of the origin Ethereum network
	pub chain_id: u64,
	/// The gateway-specific message
	pub message: GatewayMessage,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum GatewayMessage {
	UpgradeProxy(UpgradeProxyMessage),
	NativeTokens(NativeTokensMessage),
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum UpgradeProxyMessage {}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum NativeTokensMessage {
	Create {
		origin: H160,
		token: H160,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		create_call_index: [u8; 2],
		set_metadata_call_index: [u8; 2],
	},
	Mint {
		origin: H160,
		token: H160,
		dest: Option<u32>,
		recipient: MultiLocation, // Recipient of funds on final destination
		amount: u128,
	},
}

pub enum ConvertError {
	/// Message is in the wrong format
	BadFormat,
}

impl TryInto<Xcm<()>> for MessageV1 {
	type Error = ConvertError;

	fn try_into(self) -> Result<Xcm<()>, Self::Error> {
		match self.message {
			GatewayMessage::UpgradeProxy(message) => message.convert(self.chain_id),
			GatewayMessage::NativeTokens(message) => message.convert(self.chain_id),
		}
	}
}

impl UpgradeProxyMessage {
	pub fn convert(self, _chain_id: u64) -> Result<Xcm<()>, ConvertError> {
		// The UpgradeProxy gateway doesn't send any messages to Polkadot
		Err(ConvertError::BadFormat)
	}
}

impl NativeTokensMessage {
	pub fn convert(self, chain_id: u64) -> Result<Xcm<()>, ConvertError> {
		let network = NetworkId::Ethereum { chain_id };
		match self {
			NativeTokensMessage::Create {
				origin,
				token,
				name,
				symbol,
				decimals,
				create_call_index,
				set_metadata_call_index,
			} => {
				frame_support::log::trace!(target:"xcm::ali", "{:?}", origin);
				let owner = GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::from_params(
					&chain_id,
					origin.as_fixed_bytes(),
				);
				let buy_execution_fee_amount = 1_000_000_000_000; //WeightToFee::weight_to_fee(&Weight::from_parts(100_000_000, 18_000));
				let buy_execution_fee = MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(buy_execution_fee_amount),
				};

				let asset_id = Self::convert_token_address(network, origin, token);
				let instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(Junction::AccountKey20 { network: None, key: origin.into() })),
					WithdrawAsset(buy_execution_fee.clone().into()),
					BuyExecution { fees: buy_execution_fee.clone(), weight_limit: Unlimited },
					Transact {
						origin_kind: OriginKind::Xcm,
						require_weight_at_most: Weight::from_parts(40_000_000, 10_000),
						call: (create_call_index, asset_id, owner, MINIMUM_DEPOSIT).encode().into(),
					},
					ExpectTransactStatus(MaybeErrorCode::Success),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(60_000_000, 8_000),
						call: (set_metadata_call_index, asset_id, name, symbol, decimals)
							.encode()
							.into(),
					},
					ExpectTransactStatus(MaybeErrorCode::Success),
					RefundSurplus,
				];
				Ok(instructions.into())
			},
			NativeTokensMessage::Mint { origin, token, dest, recipient, amount } => {
				let asset =
					MultiAsset::from((Self::convert_token_address(network, origin, token), amount));

				let mut instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(Junction::AccountKey20 { network: None, key: origin.into() })),
					ReserveAssetDeposited(vec![asset.clone()].into()),
					ClearOrigin,
				];

				match dest {
					Some(para) => {
						let mut fragment: Vec<Instruction<()>> = vec![DepositReserveAsset {
							assets: MultiAssetFilter::Definite(vec![asset.clone()].into()),
							dest: MultiLocation { parents: 1, interior: X1(Parachain(para)) },
							xcm: vec![DepositAsset {
								assets: MultiAssetFilter::Definite(vec![asset.clone()].into()),
								beneficiary: recipient,
							}]
							.into(),
						}];
						instructions.append(&mut fragment);
					},
					None => {
						let mut fragment: Vec<Instruction<()>> = vec![DepositAsset {
							assets: MultiAssetFilter::Definite(vec![asset.clone()].into()),
							beneficiary: recipient,
						}];
						instructions.append(&mut fragment);
					},
				}
				Ok(instructions.into())
			},
		}
	}

	// Convert ERC20 token address to a Multilocation that can be understood by Assets Hub.
	fn convert_token_address(network: NetworkId, origin: H160, token: H160) -> MultiLocation {
		let res = MultiLocation {
			parents: 2,
			interior: X3(
				GlobalConsensus(network),
				AccountKey20 { network: None, key: origin.into() },
				AccountKey20 { network: None, key: token.into() },
			),
		};
		frame_support::log::trace!(target:"xcm::ali", "{:?}", res);
		res
	}
}

pub struct FromEthereumGlobalConsensus;
impl ContainsPair<MultiLocation, MultiLocation> for FromEthereumGlobalConsensus {
	fn contains(a: &MultiLocation, b: &MultiLocation) -> bool {
		let a_network_id = a.interior().global_consensus();
		if let Ok(Ethereum { .. }) = a_network_id {
			b.interior().global_consensus() == a_network_id
		} else {
			false
		}
	}
}

pub struct GlobalConsensusEthereumAccountConvertsFor<AccountId>(PhantomData<AccountId>);
impl<AccountId> ConvertLocation<AccountId> for GlobalConsensusEthereumAccountConvertsFor<AccountId>
where
	AccountId: From<[u8; 32]> + Clone,
{
	fn convert_location(location: &MultiLocation) -> Option<AccountId> {
		if let MultiLocation {
			interior: X2(GlobalConsensus(Ethereum { chain_id }), AccountKey20 { key, .. }),
			..
		} = location
		{
			Some(Self::from_params(chain_id, key).into())
		} else {
			None
		}
	}
}

impl<AccountId> GlobalConsensusEthereumAccountConvertsFor<AccountId> {
	fn from_params(chain_id: &u64, key: &[u8; 20]) -> [u8; 32] {
		(b"ethereum", chain_id, key).using_encoded(blake2_256)
	}
}

#[cfg(test)]
mod tests {
	use super::GlobalConsensusEthereumAccountConvertsFor;
	use hex_literal::hex;
	use xcm_executor::traits::ConvertLocation;
	use sp_core::crypto::Ss58Codec;
	use xcm::v3::prelude::*;

	const CONTRACT_ADDRESS: [u8; 20] = hex!("D184c103F7acc340847eEE82a0B909E3358bc28d");
	const NETWORK: NetworkId = Ethereum { chain_id: 15 };
	const SS58_FORMAT: u16 = 2;
	const EXPECTED_SOVEREIGN_KEY: [u8; 32] =
		hex!("5d6987649e0dac78ddf852eb0f1b1d1bf2be9623d81cb16c17cfa145948bb6dc");
	const EXPECTED_SOVEREIGN_ADDRESS: &'static str =
		"EgoKVgdhGVz41LyP2jckLrmXjnD35xitaX221ktZjQ2Xsxw";

	#[test]
	fn test_contract_location_without_network_converts_successfully() {
		let contract_location = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
			),
		};

		let account =
			GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(&contract_location)
				.unwrap();
		let address = frame_support::sp_runtime::AccountId32::new(account)
			.to_ss58check_with_version(SS58_FORMAT.into());
		assert_eq!(account, EXPECTED_SOVEREIGN_KEY);
		assert_eq!(address, EXPECTED_SOVEREIGN_ADDRESS);

		println!("SS58: {}\nBytes: {:?}", address, account);
	}

	#[test]
	fn test_contract_location_with_network_converts_successfully() {
		let contract_location = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: Some(NETWORK), key: CONTRACT_ADDRESS },
			),
		};

		let account =
			GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(&contract_location)
				.unwrap();
		let address = frame_support::sp_runtime::AccountId32::new(account)
			.to_ss58check_with_version(SS58_FORMAT.into());
		assert_eq!(account, EXPECTED_SOVEREIGN_KEY);
		assert_eq!(address, EXPECTED_SOVEREIGN_ADDRESS);

		println!("SS58: {}\nBytes: {:?}", address, account);
	}

	#[test]
	fn test_contract_location_with_incorrect_location_fails_convert() {
		let contract_location =
			MultiLocation { parents: 2, interior: X2(GlobalConsensus(Polkadot), Parachain(1000)) };

		assert_eq!(
			GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(&contract_location),
			None,
		);
	}
}