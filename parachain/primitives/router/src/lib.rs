#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use codec::{Decode, Encode};
use sp_core::{RuntimeDebug, H160};
use sp_std::prelude::*;
use xcm::v3::prelude::*;

use sp_runtime::traits::Get;

pub mod export;

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
			NativeTokensPayload::Create { .. } => Vec::new().into(),
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
