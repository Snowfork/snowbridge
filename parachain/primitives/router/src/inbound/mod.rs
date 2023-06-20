// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{traits::ContainsPair, weights::Weight};
use sp_core::{Get, RuntimeDebug, H160};
use sp_io::hashing::blake2_256;
use sp_runtime::MultiAddress;
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
		let buy_execution_fee_amount = 2_000_000_000; //WeightToFee::weight_to_fee(&Weight::from_parts(100_000_000, 18_000));
		let buy_execution_fee = MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(buy_execution_fee_amount),
		};

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
				let owner = GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::from_params(
					&chain_id,
					origin.as_fixed_bytes(),
				);

				let origin_location = Junction::AccountKey20 { network: None, key: origin.into() };

				let asset_id = Self::convert_token_address(network, origin, token);
				let instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(origin_location)),
					WithdrawAsset(buy_execution_fee.clone().into()),
					BuyExecution { fees: buy_execution_fee.clone(), weight_limit: Unlimited },
					SetAppendix(
						vec![
							RefundSurplus,
							DepositAsset {
								assets: buy_execution_fee.into(),
								beneficiary: (
									Parent,
									Parent,
									GlobalConsensus(network),
									origin_location,
								)
								.into(),
							},
						]
						.into(),
					),
					Transact {
						origin_kind: OriginKind::Xcm,
						require_weight_at_most: Weight::from_parts(400_000_000, 8_000),
						call: (
							create_call_index,
							asset_id,
							MultiAddress::<[u8; 32], ()>::Id(owner),
							MINIMUM_DEPOSIT,
						)
							.encode()
							.into(),
					},
					ExpectTransactStatus(MaybeErrorCode::Success),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(200_000_000, 8_000),
						call: (set_metadata_call_index, asset_id, name, symbol, decimals)
							.encode()
							.into(),
					},
					ExpectTransactStatus(MaybeErrorCode::Success),
				];
				Ok(instructions.into())
			},
			NativeTokensMessage::Mint { origin, token, dest, recipient, amount } => {
				let asset =
					MultiAsset::from((Self::convert_token_address(network, origin, token), amount));

				let origin_location = Junction::AccountKey20 { network: None, key: origin.into() };

				let mut instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(origin_location)),
					WithdrawAsset(buy_execution_fee.clone().into()),
					BuyExecution { fees: buy_execution_fee.clone(), weight_limit: Unlimited },
					SetAppendix(
						vec![
							RefundSurplus,
							DepositAsset {
								assets: buy_execution_fee.into(),
								beneficiary: (
									Parent,
									Parent,
									GlobalConsensus(network),
									origin_location,
								)
								.into(),
							},
						]
						.into(),
					),
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
		MultiLocation {
			parents: 2,
			interior: X3(
				GlobalConsensus(network),
				AccountKey20 { network: None, key: origin.into() },
				AccountKey20 { network: None, key: token.into() },
			),
		}
	}
}

pub struct FromEthereumGlobalConsensus<EthereumBridgeLocation>(PhantomData<EthereumBridgeLocation>);
impl<EthereumBridgeLocation> ContainsPair<MultiLocation, MultiLocation>
	for FromEthereumGlobalConsensus<EthereumBridgeLocation>
where
	EthereumBridgeLocation: Get<MultiLocation>,
{
	fn contains(asset: &MultiLocation, origin: &MultiLocation) -> bool {
		origin == &EthereumBridgeLocation::get() && asset.starts_with(origin)
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
	use super::{FromEthereumGlobalConsensus, GlobalConsensusEthereumAccountConvertsFor};
	use frame_support::{parameter_types, traits::ContainsPair};
	use hex_literal::hex;
	use sp_core::crypto::Ss58Codec;
	use xcm::v3::prelude::*;
	use xcm_executor::traits::ConvertLocation;

	const CONTRACT_ADDRESS: [u8; 20] = hex!("D184c103F7acc340847eEE82a0B909E3358bc28d");
	const NETWORK: NetworkId = Ethereum { chain_id: 15 };
	const SS58_FORMAT: u16 = 2;
	const EXPECTED_SOVEREIGN_KEY: [u8; 32] =
		hex!("5d6987649e0dac78ddf852eb0f1b1d1bf2be9623d81cb16c17cfa145948bb6dc");
	const EXPECTED_SOVEREIGN_ADDRESS: &'static str =
		"EgoKVgdhGVz41LyP2jckLrmXjnD35xitaX221ktZjQ2Xsxw";

	parameter_types! {
		pub EthereumNetwork: NetworkId = NETWORK;
		pub EthereumLocation: MultiLocation = MultiLocation::new(2, X2(GlobalConsensus(EthereumNetwork::get()), AccountKey20 { network: None, key: CONTRACT_ADDRESS }));
	}

	#[test]
	fn test_contract_location_without_network_converts_successfully() {
		let contract_location = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
			),
		};

		let account = GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(
			&contract_location,
		)
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

		let account = GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(
			&contract_location,
		)
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
			GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::convert_location(
				&contract_location
			),
			None,
		);
	}

	#[test]
	fn test_from_ethereum_global_consensus_with_containing_asset_yields_true() {
		let origin = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
			),
		};
		let asset = MultiLocation {
			parents: 2,
			interior: X3(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
				AccountKey20 {
					network: None,
					key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
				},
			),
		};
		assert!(FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}

	#[test]
	fn test_from_ethereum_global_consensus_without_containing_asset_yields_false() {
		let origin = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
			),
		};
		let asset =
			MultiLocation { parents: 2, interior: X2(GlobalConsensus(Polkadot), Parachain(1000)) };
		assert!(!FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}

	#[test]
	fn test_from_ethereum_global_consensus_without_bridge_origin_yields_false() {
		let origin =
			MultiLocation { parents: 2, interior: X2(GlobalConsensus(Polkadot), Parachain(1000)) };
		let asset = MultiLocation {
			parents: 2,
			interior: X3(
				GlobalConsensus(NETWORK),
				AccountKey20 { network: None, key: CONTRACT_ADDRESS },
				AccountKey20 {
					network: None,
					key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
				},
			),
		};
		assert!(!FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}
}
