// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Converts messages from Ethereum to XCM messages
use codec::{Decode, Encode};
use core::marker::PhantomData;
use derivative::Derivative;
use frame_support::{log, traits::ContainsPair, weights::Weight};
use sp_core::{Get, RuntimeDebug, H160};
use sp_io::hashing::blake2_256;
use sp_runtime::MultiAddress;
use sp_std::{cmp::max, prelude::*};
use xcm::v3::{prelude::*, Junction::AccountKey20};
use xcm_executor::traits::ConvertLocation;

const MINIMUM_DEPOSIT: u128 = 1;
pub const LOG_TARGET: &str = "xcm::snowbridge-router";

/// Messages from Ethereum are versioned. This is because in future,
/// we may want to evolve the protocol so that the ethereum side sends XCM messages directly.
/// Instead having BridgeHub transcode the messages into XCM.
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
	/// The fee to cover xcm buy_execution
	pub fee: u128,
	/// The command originating from the Gateway contract
	pub message: Command,
}

#[derive(Derivative, Encode, Decode)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum Command {
	/// Register a wrapped token on the AssetHub `ForeignAssets` pallet
	RegisterToken {
		/// The address of the gateway
		gateway: H160,
		/// The address of the ERC20 token to be bridged over to AssetHub
		token: H160,
		/// The stable ID of the `ForeignAssets::create` extrinsic
		create_call_index: [u8; 2],
	},
	/// Send a token to AssetHub or another parachain
	SendToken {
		/// The address of the gateway
		gateway: H160,
		/// The address of the ERC20 token to be bridged over to AssetHub
		token: H160,
		/// The destination for the transfer
		destination: Destination,
		/// Amount to transfer
		amount: u128,
	},
	/// call arbitrary transact in another parachain
	Transact {
		/// The address of the sender
		sender: H160,
		/// OriginKind
		origin_kind: OriginKind,
		/// The payload of the transact
		payload: Vec<u8>,
		/// The ref_time part of weight
		ref_time: u64,
		/// The proof_size part of weight
		proof_size: u64,
	},
}

/// Destination for bridged tokens
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum Destination {
	/// The funds will be deposited into account `id` on AssetHub
	AccountId32 { id: [u8; 32] },
	/// The funds will deposited into the sovereign account of destination parachain `para_id` on
	/// AssetHub, Account `id` on the destination parachain will receive the funds via a
	/// reserve-backed transfer. See https://github.com/paritytech/xcm-format#depositreserveasset
	ForeignAccountId32 { para_id: u32, id: [u8; 32] },
	/// The funds will deposited into the sovereign account of destination parachain `para_id` on
	/// AssetHub, Account `id` on the destination parachain will receive the funds via a
	/// reserve-backed transfer. See https://github.com/paritytech/xcm-format#depositreserveasset
	ForeignAccountId20 { para_id: u32, id: [u8; 20] },
}

impl From<MessageV1> for Xcm<()> {
	fn from(val: MessageV1) -> Self {
		val.message.convert(val.chain_id, val.fee)
	}
}

impl Command {
	pub fn convert(self, chain_id: u64, fee: u128) -> Xcm<()> {
		log::debug!(target: LOG_TARGET,"chain_id: {}, fee: {}, command: {:?},", chain_id, fee, self);
		let network = Ethereum { chain_id };
		// Reference from https://coincodex.com/convert/ethereum/polkadot/
		const SWAP_RATE: u128 = 367;
		// Sanity base fee applies to most of the xcm calls
		const BASE_FEE: u128 = 2_000_000_000;

		let buy_execution_fee_amount =
			max(BASE_FEE, fee.saturating_div(1000000u128.saturating_div(SWAP_RATE)));

		let buy_execution_fee = MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(buy_execution_fee_amount),
		};

		match self {
			Command::Transact { sender, origin_kind, payload, ref_time, proof_size } => {
				log::debug!(target: LOG_TARGET, "transact sender {:?}, payload {:?}", sender, payload);

				let origin_location = AccountKey20 { network: None, key: sender.into() };

				let weight_limit: Weight = Weight::from_parts(ref_time, proof_size);

				let instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(origin_location)),
					WithdrawAsset(buy_execution_fee.clone().into()),
					BuyExecution { fees: buy_execution_fee, weight_limit: Unlimited },
					SetAppendix(
						vec![
							RefundSurplus,
							DepositAsset {
								assets: AllCounted(1).into(),
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
						origin_kind,
						require_weight_at_most: weight_limit,
						call: payload.into(),
					},
					ExpectTransactStatus(MaybeErrorCode::Success),
				];
				instructions.into()
			},
			Command::RegisterToken { gateway, token, create_call_index } => {
				let owner = GlobalConsensusEthereumAccountConvertsFor::<[u8; 32]>::from_params(
					&chain_id,
					gateway.as_fixed_bytes(),
				);

				let origin_location = Junction::AccountKey20 { network: None, key: gateway.into() };

				let asset_id = Self::convert_token_address(network, gateway, token);
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
				];
				instructions.into()
			},
			Command::SendToken { gateway, token, destination, amount } => {
				let asset = MultiAsset::from((
					Self::convert_token_address(network, gateway, token),
					amount,
				));

				let origin_location = Junction::AccountKey20 { network: None, key: gateway.into() };

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

				let (dest_para_id, beneficiary) = match destination {
					Destination::AccountId32 { id } => (
						None,
						MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { network: None, id }),
						},
					),
					Destination::ForeignAccountId32 { para_id, id } => (
						Some(para_id),
						MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { network: None, id }),
						},
					),
					Destination::ForeignAccountId20 { para_id, id } => (
						Some(para_id),
						MultiLocation {
							parents: 0,
							interior: X1(AccountKey20 { network: None, key: id }),
						},
					),
				};

				let assets = MultiAssetFilter::Definite(vec![asset].into());

				let mut fragment: Vec<Instruction<()>> = match dest_para_id {
					Some(dest_para_id) => {
						vec![DepositReserveAsset {
							assets: assets.clone(),
							dest: MultiLocation {
								parents: 1,
								interior: X1(Parachain(dest_para_id)),
							},
							xcm: vec![DepositAsset { assets, beneficiary }].into(),
						}]
					},
					None => {
						vec![DepositAsset { assets, beneficiary }]
					},
				};
				instructions.append(&mut fragment);
				instructions.into()
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

	const CONTRACT_ADDRESS: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");
	const NETWORK: NetworkId = Ethereum { chain_id: 15 };
	const SS58_FORMAT: u16 = 2;
	const EXPECTED_SOVEREIGN_KEY: [u8; 32] =
		hex!("c9794dd8013efb2ad83f668845c62b373c16ad33971745731408058e4d0c6ff5");
	const EXPECTED_SOVEREIGN_ADDRESS: &'static str =
		"H8VBFC4LG91ByxMG6GwsCcAacjitnzGmGbqnvSEQFBywJEL";

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

		println!("SS58: {}\nBytes: {:?}", address, account);

		assert_eq!(account, EXPECTED_SOVEREIGN_KEY);
		assert_eq!(address, EXPECTED_SOVEREIGN_ADDRESS);
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
				AccountKey20 { network: None, key: [0; 20] },
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
				AccountKey20 { network: None, key: [0; 20] },
			),
		};
		assert!(!FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}
}
