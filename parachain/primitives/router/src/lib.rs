#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use codec::{Decode, Encode};
use frame_support::{traits::TrackedStorageKey, weights::Weight};
use sp_core::{RuntimeDebug, H160};
use sp_std::prelude::*;
use xcm::v3::prelude::*;

use sp_runtime::traits::Get;

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum Payload {
	NativeTokens(NativeTokensPayload),
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum NativeTokensPayload {
	Create {
		token: H160,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	},
	Mint {
		token: H160,
		dest: Option<u32>,
		recipient: MultiLocation, // Recipient of funds on final destination
		amount: u128,
	},
}

pub trait ConvertMessage {
	/// Convert inbound message to destination and Xcm message
	fn convert(origin: H160, dest: u32, payload: Payload) -> (MultiLocation, Xcm<()>);
}

#[derive(Clone, Eq, PartialEq, Encode)]
pub enum StatemineCall {
	#[codec(index = 53u8)]
	Assets(AssetsCall),
}

#[derive(Clone, Eq, PartialEq, Encode)]
pub enum AssetsCall {
	#[codec(index = 1u8)]
	ForceCreate { asset_id: MultiLocation, owner: [u8; 32], is_sufficient: bool, min_balance: u128 },
	#[codec(index = 17u8)]
	SetMetadata { asset_id: MultiLocation, name: Vec<u8>, symbol: Vec<u8>, decimals: u8 },
}

pub struct InboundMessageConverter<EthereumNetworkId>(PhantomData<EthereumNetworkId>);

impl<EthereumNetworkId> ConvertMessage for InboundMessageConverter<EthereumNetworkId>
where
	EthereumNetworkId: Get<NetworkId>,
{
	fn convert(origin: H160, dest: u32, payload: Payload) -> (MultiLocation, Xcm<()>) {
		let dest = MultiLocation { parents: 1, interior: X1(Parachain(dest)) };
		let xcm = match payload {
			Payload::NativeTokens(inner_payload) =>
				Self::convert_native_tokens_payload(origin, inner_payload),
		};

		(dest, xcm)
	}
}

impl<EthereumNetworkId> InboundMessageConverter<EthereumNetworkId>
where
	EthereumNetworkId: Get<NetworkId>,
{
	fn convert_native_tokens_payload(origin: H160, payload: NativeTokensPayload) -> Xcm<()> {
		let network = EthereumNetworkId::get();

		match payload {
			NativeTokensPayload::Create { token, name, symbols, decimals } => {
				let asset_id = Self::convert_token_address(token);

				let mut instructions: Vec<Instruction<()>> = vec![
					UniversalOrigin(GlobalConsensus(network)),
					DescendOrigin(X1(Junction::AccountKey20 { network: None, key: origin.into() })),
					Transact {
						origin_kind: OriginKind::Native,
						require_weight_at_most: Weight::from_parts(500_000_000, 10000),
						call: StatemineCall::Assets(AssetsCall::ForceCreate {
							asset_id: asset_id.clone(),
							owner: H256::zero().into(),
							is_sufficient: true,
							min_balance: 1,
						})
						.encode()
						.into(),
					},
				];

				instructions.into()
			},
			NativeTokensPayload::Mint { token, dest, recipient, amount } => {
				let asset = MultiAsset::from((Self::convert_token_address(token), amount));

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
				instructions.into()
			},
		}
	}

	fn convert_token_address(token: H160) -> MultiLocation {
		let network = EthereumNetworkId::get();
		return MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(network),
				AccountKey20 { network: None, key: token.into() },
			),
		}
	}
}
