// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Converts messages from Ethereum to XCM messages
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{traits::ContainsPair, weights::Weight, PalletError};
use scale_info::TypeInfo;
use sp_core::{Get, RuntimeDebug, H160};
use sp_io::hashing::blake2_256;
use sp_runtime::MultiAddress;
use sp_std::prelude::*;
use xcm::prelude::{Junction::AccountKey20, *};
use xcm_executor::traits::ConvertLocation;

const MINIMUM_DEPOSIT: u128 = 1;

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
	/// The command originating from the Gateway contract
	pub command: Command,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum Command {
	/// Register a wrapped token on the AssetHub `ForeignAssets` pallet
	RegisterToken {
		/// The address of the ERC20 token to be bridged over to AssetHub
		token: H160,
	},
	/// Send a token to AssetHub or another parachain
	SendToken {
		/// The address of the ERC20 token to be bridged over to AssetHub
		token: H160,
		/// The destination for the transfer
		destination: Destination,
		/// Amount to transfer
		amount: u128,
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

pub struct MessageToXcm<
	CreateAssetCall,
	CreateAssetExecutionFee,
	CreateAssetDeposit,
	SendTokenExecutionFee,
> where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
{
	_phantom: PhantomData<(
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
	)>,
}

/// Reason why a message conversion failed.
#[derive(Copy, Clone, TypeInfo, PalletError, Encode, Decode, RuntimeDebug)]
pub enum ConvertMessageError {
	/// The message version is not supported for conversion.
	UnsupportedVersion,
}

/// convert the inbound message to xcm which will be forwarded to the destination chain
pub trait ConvertMessage {
	/// Converts a versioned message into an XCM message and an optional topicID
	fn convert(message: VersionedMessage) -> Result<Xcm<()>, ConvertMessageError>;
}

pub type CallIndex = [u8; 2];

impl<CreateAssetCall, CreateAssetExecutionFee, CreateAssetDeposit, SendTokenExecutionFee>
	ConvertMessage
	for MessageToXcm<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
	> where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
{
	fn convert(message: VersionedMessage) -> Result<Xcm<()>, ConvertMessageError> {
		use Command::*;
		use VersionedMessage::*;
		match message {
			V1(MessageV1 { chain_id, command: RegisterToken { token } }) =>
				Ok(Self::convert_register_token(chain_id, token)),
			V1(MessageV1 { chain_id, command: SendToken { token, destination, amount } }) =>
				Ok(Self::convert_send_token(chain_id, token, destination, amount)),
		}
	}
}

impl<CreateAssetCall, CreateAssetExecutionFee, CreateAssetDeposit, SendTokenExecutionFee>
	MessageToXcm<CreateAssetCall, CreateAssetExecutionFee, CreateAssetDeposit, SendTokenExecutionFee>
where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
{
	fn convert_register_token(chain_id: u64, token: H160) -> Xcm<()> {
		let network = Ethereum { chain_id };
		let buy_execution_fee = MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(CreateAssetExecutionFee::get()),
		};
		let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);
		let asset_id = Self::convert_token_address(network, token);
		let create_call_index: [u8; 2] = CreateAssetCall::get();

		Xcm(vec![
			//ReceiveTeleportedAsset(buy_execution_fee.clone().into()),
			UniversalOrigin(GlobalConsensus(network)),
			WithdrawAsset(buy_execution_fee.clone().into()),
			BuyExecution { fees: buy_execution_fee, weight_limit: Unlimited },
			SetAppendix(
				vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(AllCounted(1)),
						beneficiary: (Parent, Parent, GlobalConsensus(network)).into(),
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
		])
	}

	fn convert_send_token(
		chain_id: u64,
		token: H160,
		destination: Destination,
		amount: u128,
	) -> Xcm<()> {
		let network = Ethereum { chain_id };
		let buy_execution_fee = MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(SendTokenExecutionFee::get()),
		};
		let asset = MultiAsset::from((Self::convert_token_address(network, token), amount));

		let (dest_para_id, beneficiary) = match destination {
			Destination::AccountId32 { id } => (
				None,
				MultiLocation { parents: 0, interior: X1(AccountId32 { network: None, id }) },
			),
			Destination::ForeignAccountId32 { para_id, id } => (
				Some(para_id),
				MultiLocation { parents: 0, interior: X1(AccountId32 { network: None, id }) },
			),
			Destination::ForeignAccountId20 { para_id, id } => (
				Some(para_id),
				MultiLocation { parents: 0, interior: X1(AccountKey20 { network: None, key: id }) },
			),
		};

		let assets = Definite(vec![asset.clone()].into());

		Xcm(vec![
			//ReceiveTeleportedAsset(buy_execution_fee.clone().into()),
			UniversalOrigin(GlobalConsensus(network)),
			WithdrawAsset(buy_execution_fee.clone().into()),
			BuyExecution { fees: buy_execution_fee, weight_limit: Unlimited },
			SetAppendix(
				vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(AllCounted(1)),
						beneficiary: (Parent, Parent, GlobalConsensus(network)).into(),
					},
				]
				.into(),
			),
			ReserveAssetDeposited(asset.into()),
			ClearOrigin,
		]
		.into_iter()
		.chain(match dest_para_id {
			Some(dest_para_id) => vec![DepositReserveAsset {
				assets: assets.clone(),
				dest: MultiLocation { parents: 1, interior: X1(Parachain(dest_para_id)) },
				xcm: vec![DepositAsset { assets, beneficiary }].into(),
			}],
			None => vec![DepositAsset { assets, beneficiary }],
		})
		.collect())
	}

	// Convert ERC20 token address to a Multilocation that can be understood by Assets Hub.
	fn convert_token_address(network: NetworkId, token: H160) -> MultiLocation {
		MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(network),
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

pub struct GlobalConsensusEthereumConvertsFor<AccountId>(PhantomData<AccountId>);
impl<AccountId> ConvertLocation<AccountId> for GlobalConsensusEthereumConvertsFor<AccountId>
where
	AccountId: From<[u8; 32]> + Clone,
{
	fn convert_location(location: &MultiLocation) -> Option<AccountId> {
		if let MultiLocation { interior: X1(GlobalConsensus(Ethereum { chain_id })), .. } = location
		{
			Some(Self::from_chain_id(chain_id).into())
		} else {
			None
		}
	}
}

impl<AccountId> GlobalConsensusEthereumConvertsFor<AccountId> {
	pub fn from_chain_id(chain_id: &u64) -> [u8; 32] {
		(b"ethereum-chain", chain_id).using_encoded(blake2_256)
	}
}

#[cfg(test)]
mod tests {
	use super::{FromEthereumGlobalConsensus, GlobalConsensusEthereumConvertsFor};
	use frame_support::{parameter_types, traits::ContainsPair};
	use hex_literal::hex;
	use sp_core::crypto::Ss58Codec;
	use xcm::v3::prelude::*;
	use xcm_executor::traits::ConvertLocation;

	const NETWORK: NetworkId = Ethereum { chain_id: 15 };
	const SS58_FORMAT: u16 = 2;
	const EXPECTED_SOVEREIGN_KEY: [u8; 32] =
		hex!("da4d66c3651dc151264eee5460493210338e41a7bbfca91a520e438daf180bf5");
	const EXPECTED_SOVEREIGN_ADDRESS: &'static str =
		"HWYx2xgcdpSjJQicUUZFRR1EJNPVEQoUDSUB29rfxF617nv";

	parameter_types! {
		pub EthereumNetwork: NetworkId = NETWORK;
		pub EthereumLocation: MultiLocation = MultiLocation::new(2, X1(GlobalConsensus(EthereumNetwork::get())));
	}

	#[test]
	fn test_contract_location_without_network_converts_successfully() {
		let contract_location =
			MultiLocation { parents: 2, interior: X1(GlobalConsensus(NETWORK)) };

		let account =
			GlobalConsensusEthereumConvertsFor::<[u8; 32]>::convert_location(&contract_location)
				.unwrap();
		let address = frame_support::sp_runtime::AccountId32::new(account)
			.to_ss58check_with_version(SS58_FORMAT.into());

		println!("SS58: {}\nBytes: {:?}", address, account);

		assert_eq!(account, EXPECTED_SOVEREIGN_KEY);
		assert_eq!(address, EXPECTED_SOVEREIGN_ADDRESS);
	}

	#[test]
	fn test_contract_location_with_network_converts_successfully() {
		let contract_location =
			MultiLocation { parents: 2, interior: X1(GlobalConsensus(NETWORK)) };

		let account =
			GlobalConsensusEthereumConvertsFor::<[u8; 32]>::convert_location(&contract_location)
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
			GlobalConsensusEthereumConvertsFor::<[u8; 32]>::convert_location(&contract_location),
			None,
		);
	}

	#[test]
	fn test_from_ethereum_global_consensus_with_containing_asset_yields_true() {
		let origin = MultiLocation { parents: 2, interior: X1(GlobalConsensus(NETWORK)) };
		let asset = MultiLocation {
			parents: 2,
			interior: X2(GlobalConsensus(NETWORK), AccountKey20 { network: None, key: [0; 20] }),
		};
		assert!(FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}

	#[test]
	fn test_from_ethereum_global_consensus_without_containing_asset_yields_false() {
		let origin = MultiLocation { parents: 2, interior: X1(GlobalConsensus(NETWORK)) };
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
			interior: X2(GlobalConsensus(NETWORK), AccountKey20 { network: None, key: [0; 20] }),
		};
		assert!(!FromEthereumGlobalConsensus::<EthereumLocation>::contains(&asset, &origin));
	}
}
