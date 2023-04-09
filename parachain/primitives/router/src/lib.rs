#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use codec::{Decode, Encode};
use sp_core::{RuntimeDebug, H160};
use sp_std::prelude::*;
use xcm::v3::prelude::*;

use sp_runtime::traits::{Convert, Get};

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

pub struct ConvertTokenAddress<EthereumNetwork>(PhantomData<EthereumNetwork>);
impl<EthereumNetwork: Get<NetworkId>> Convert<H160, MultiLocation>
	for ConvertTokenAddress<EthereumNetwork>
{
	fn convert(a: H160) -> MultiLocation {
		let network = EthereumNetwork::get();
		return MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(network),
				AccountKey20 { network: Some(network), key: a.into() },
			),
		}
	}
}

pub trait ConvertMessage {
	/// Convert inbound message to destination and Xcm message
	fn convert(origin: H160, dest: u32, payload: Payload) -> (MultiLocation, Xcm<()>);
}

pub struct InboundMessageConverter<ConvertTokenAddress>(PhantomData<ConvertTokenAddress>);
impl<ConvertTokenAddress> ConvertMessage for InboundMessageConverter<ConvertTokenAddress>
where
	ConvertTokenAddress: Convert<H160, MultiLocation>,
{
	fn convert(origin: H160, dest: u32, payload: Payload) -> (MultiLocation, Xcm<()>) {
		let dest = MultiLocation { parents: 1, interior: X1(Parachain(dest)) };
		let xcm = match payload {
			Payload::NativeTokens(inner_payload) =>
				convert_native_tokens_payload::<ConvertTokenAddress>(origin, inner_payload),
		};

		(dest, xcm)
	}
}

fn convert_native_tokens_payload<ConvertTokenAddress>(
	origin: H160,
	payload: NativeTokensPayload,
) -> Xcm<()>
where
	ConvertTokenAddress: Convert<H160, MultiLocation>,
{
	let network = NetworkId::Ethereum { chain_id: 1 };

	match payload {
		NativeTokensPayload::Create { .. } => Vec::new().into(),
		NativeTokensPayload::Mint { token, dest, recipient, amount } => {
			let asset = MultiAsset::from((ConvertTokenAddress::convert(token), amount));

			let mut instructions: Vec<Instruction<()>> = vec![
				UniversalOrigin(GlobalConsensus(network)),
				DescendOrigin(X1(Junction::AccountKey20 {
					network: Some(network),
					key: origin.into(),
				})),
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
