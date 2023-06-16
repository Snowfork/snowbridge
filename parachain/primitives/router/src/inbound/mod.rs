// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use codec::{Decode, Encode};
use frame_support::weights::Weight;

use sp_core::{RuntimeDebug, H160};
use sp_std::prelude::*;
use xcm::v3::prelude::*;

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
				let asset_id = Self::convert_token_address(network, origin, token);
				let instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(Junction::AccountKey20 { network: None, key: origin.into() })),
					Transact {
						origin_kind: OriginKind::Xcm,
						require_weight_at_most: Weight::from_parts(40_000_000_000, 8000),
						call: (create_call_index, asset_id, [7u8; 32], 1u128).encode().into(),
					},
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(20_000_000_000, 8000),
						call: (set_metadata_call_index, asset_id, name, symbol, decimals)
							.encode()
							.into(),
					},
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
		return MultiLocation {
			parents: 2,
			interior: X3(
				GlobalConsensus(network),
				AccountKey20 { network: None, key: origin.into() },
				AccountKey20 { network: None, key: token.into() },
			),
		};
	}
}
