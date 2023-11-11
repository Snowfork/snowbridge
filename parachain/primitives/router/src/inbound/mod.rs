// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Converts messages from Ethereum to XCM messages
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
	traits::{tokens::Balance as BalanceT, ContainsPair},
	weights::Weight,
	PalletError,
};
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
	RefundAccount,
	AccountId,
	Balance,
> where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
	RefundAccount: Get<AccountId>,
	Balance: BalanceT,
{
	_phantom: PhantomData<(
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		AccountId,
		Balance,
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
	type Balance: BalanceT + From<u128>;
	type AccountId;
	/// Converts a versioned message into an XCM message and an optional topicID
	fn convert(message: VersionedMessage) -> Result<(Xcm<()>, Self::Balance), ConvertMessageError>;
}

pub type CallIndex = [u8; 2];

impl<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		AccountId,
		Balance,
	> ConvertMessage
	for MessageToXcm<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		AccountId,
		Balance,
	> where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
	RefundAccount: Get<AccountId>,
	Balance: BalanceT + From<u128>,
	AccountId: Into<[u8; 32]>,
{
	type Balance = Balance;
	type AccountId = AccountId;

	fn convert(message: VersionedMessage) -> Result<(Xcm<()>, Self::Balance), ConvertMessageError> {
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

impl<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		AccountId,
		Balance,
	>
	MessageToXcm<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		AccountId,
		Balance,
	> where
	CreateAssetCall: Get<CallIndex>,
	CreateAssetExecutionFee: Get<u128>,
	CreateAssetDeposit: Get<u128>,
	SendTokenExecutionFee: Get<u128>,
	RefundAccount: Get<AccountId>,
	Balance: BalanceT + From<u128>,
	AccountId: Into<[u8; 32]>,
{
	fn convert_register_token(chain_id: u64, token: H160) -> (Xcm<()>, Balance) {
		let network = Ethereum { chain_id };
		let fee: MultiAsset = (MultiLocation::parent(), CreateAssetExecutionFee::get()).into();
		let deposit: MultiAsset = (MultiLocation::parent(), CreateAssetDeposit::get()).into();

		let total_amount = CreateAssetExecutionFee::get() + CreateAssetDeposit::get();
		let total: MultiAsset = (MultiLocation::parent(), total_amount).into();

		let bridge_location: MultiLocation = (Parent, Parent, GlobalConsensus(network)).into();
		let fee_refund_location: MultiLocation =
			(AccountId32 { network: None, id: RefundAccount::get().into() }).into();

		let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);
		let asset_id = Self::convert_token_address(network, token);
		let create_call_index: [u8; 2] = CreateAssetCall::get();

		let xcm: Xcm<()> = vec![
			// Teleport required fees.
			ReceiveTeleportedAsset(total.into()),
			// Pay for execution.
			BuyExecution { fees: fee, weight_limit: Unlimited },
			// Fund the snowbridge sovereign with the required deposit for creation.
			DepositAsset { assets: Definite(deposit.into()), beneficiary: bridge_location },
			// Change origin to the bridge.
			UniversalOrigin(GlobalConsensus(network)),
			// Call create_asset on foreign assets pallet.
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
			// Refund any surplus execution from transact.
			RefundSurplus,
			// Send any remaining fees to the destination parachain.
			DepositAsset { assets: Wild(All), beneficiary: fee_refund_location },
		]
		.into();

		(xcm, total_amount.into())
	}

	fn convert_send_token(
		chain_id: u64,
		token: H160,
		destination: Destination,
		amount: u128,
	) -> (Xcm<()>, Balance) {
		let network = Ethereum { chain_id };
		let fee_amount = SendTokenExecutionFee::get();
		let fee: MultiAsset = (MultiLocation::parent(), fee_amount).into();
		let asset: MultiAsset = (Self::convert_token_address(network, token), amount).into();

		let fee_refund_location: MultiLocation =
			(AccountId32 { network: None, id: RefundAccount::get().into() }).into();

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

		let mut total_fee_amount = fee_amount;
		let mut instructions = vec![
			ReceiveTeleportedAsset(fee.clone().into()),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			UniversalOrigin(GlobalConsensus(network)),
			ReserveAssetDeposited(asset.clone().into()),
			ClearOrigin,
		];

		match dest_para_id {
			Some(dest_para_id) => {
				instructions.extend(vec![
					// Perform a deposit reserve to send to destination chain.
					DepositReserveAsset {
						assets: Definite(asset.clone().into()),
						dest: MultiLocation { parents: 1, interior: X1(Parachain(dest_para_id)) },
						xcm: vec![
							// Receive fees.
							ReceiveTeleportedAsset(fee.clone().into()),
							// Buy execution on target.
							BuyExecution { fees: fee, weight_limit: Unlimited },
							// Deposit asset to benificiary.
							DepositAsset { assets: Definite(asset.into()), beneficiary },
							// Deposit remaining fees to destination.
							DepositAsset { assets: Wild(All), beneficiary: fee_refund_location },
						]
						.into(),
					},
				]);
				total_fee_amount += fee_amount;
			},
			None => {
				instructions.extend(vec![
					// Deposit asset to benificiary.
					DepositAsset { assets: Definite(asset.into()), beneficiary },
					// Deposit remaining fees to destination.
					DepositAsset { assets: Wild(All), beneficiary: fee_refund_location },
				]);
			},
		}

		(instructions.into(), total_fee_amount.into())
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
	use crate::inbound::{
		CallIndex, Command, ConvertMessage, Destination, MessageToXcm, MessageV1, MultiAddress,
		VersionedMessage, H160, MINIMUM_DEPOSIT,
	};
	use codec::Encode;
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

		pub const CreateAssetCall: CallIndex = [1, 1];
		pub const CreateAssetExecutionFee: u128 = 123;
		pub const CreateAssetDeposit: u128 = 891;
		pub const SendTokenExecutionFee: u128 = 592;
		pub RefundAccount: [u8; 32] = [5; 32];
	}

	type Converter = MessageToXcm<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		RefundAccount,
		[u8; 32],
		u128,
	>;

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

	#[test]
	fn test_xcm_converter_send_token_local_returns_execution_fee() {
		let amount = 100;
		let id = [1; 32];
		let token = [2; 20];
		let chain_id = 15;
		let refund_id = RefundAccount::get();
		let destination = Destination::AccountId32 { id };
		let command = Command::SendToken { token: H160(token), destination, amount };

		let message = VersionedMessage::V1(MessageV1 { chain_id, command });
		let expected_fee = SendTokenExecutionFee::get();
		let expected = Xcm(vec![
			ReceiveTeleportedAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(expected_fee) }
					.into(),
			),
			BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(expected_fee),
				},
				weight_limit: Unlimited,
			},
			UniversalOrigin(GlobalConsensus(Ethereum { chain_id })),
			ReserveAssetDeposited(
				MultiAsset {
					id: Concrete(MultiLocation {
						parents: 2,
						interior: X2(
							GlobalConsensus(Ethereum { chain_id }),
							AccountKey20 { network: None, key: token },
						),
					}),
					fun: Fungible(amount),
				}
				.into(),
			),
			ClearOrigin,
			DepositAsset {
				assets: Definite(
					MultiAsset {
						id: Concrete(MultiLocation {
							parents: 2,
							interior: X2(
								GlobalConsensus(Ethereum { chain_id }),
								AccountKey20 { network: None, key: token },
							),
						}),
						fun: Fungible(100),
					}
					.into(),
				),
				beneficiary: MultiLocation {
					parents: 0,
					interior: X1(AccountId32 { network: None, id }),
				},
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: MultiLocation {
					parents: 0,
					interior: X1(AccountId32 { network: None, id: refund_id }),
				},
			},
		]);
		let Ok((xcm, fee)) = Converter::convert(message) else {panic!("unreachable");};
		assert_eq!(fee, expected_fee);
		assert_eq!(xcm, expected);
	}

	#[test]
	fn test_xcm_converter_send_token_remote_account_id_returns_double_execution_fee() {
		let amount = 100;
		let para_id = 1001;
		let id = [1; 32];
		let token = [2; 20];
		let chain_id = 15;
		let refund_id = RefundAccount::get();
		let destination = Destination::ForeignAccountId32 { para_id, id };
		let command = Command::SendToken { token: H160(token), destination, amount };

		let message = VersionedMessage::V1(MessageV1 { chain_id, command });
		let expected_fee = SendTokenExecutionFee::get();
		let expected = Xcm(vec![
			ReceiveTeleportedAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(expected_fee) }
					.into(),
			),
			BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(expected_fee),
				},
				weight_limit: Unlimited,
			},
			UniversalOrigin(GlobalConsensus(Ethereum { chain_id })),
			ReserveAssetDeposited(
				MultiAsset {
					id: Concrete(MultiLocation {
						parents: 2,
						interior: X2(
							GlobalConsensus(Ethereum { chain_id }),
							AccountKey20 { network: None, key: token },
						),
					}),
					fun: Fungible(100),
				}
				.into(),
			),
			ClearOrigin,
			DepositReserveAsset {
				assets: Definite(
					MultiAsset {
						id: Concrete(MultiLocation {
							parents: 2,
							interior: X2(
								GlobalConsensus(Ethereum { chain_id }),
								AccountKey20 { network: None, key: token },
							),
						}),
						fun: Fungible(100),
					}
					.into(),
				),
				dest: MultiLocation { parents: 1, interior: X1(Parachain(para_id)) },
				xcm: Xcm(vec![
					ReceiveTeleportedAsset(
						MultiAsset {
							id: Concrete(MultiLocation::parent()),
							fun: Fungible(expected_fee),
						}
						.into(),
					),
					BuyExecution {
						fees: MultiAsset {
							id: Concrete(MultiLocation::parent()),
							fun: Fungible(expected_fee),
						},
						weight_limit: Unlimited,
					},
					DepositAsset {
						assets: Definite(
							MultiAsset {
								id: Concrete(MultiLocation {
									parents: 2,
									interior: X2(
										GlobalConsensus(Ethereum { chain_id }),
										AccountKey20 { network: None, key: token },
									),
								}),
								fun: Fungible(amount),
							}
							.into(),
						),
						beneficiary: MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { network: None, id }),
						},
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary: MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { network: None, id: refund_id }),
						},
					},
				]),
			},
		]);
		let Ok((xcm, fee)) = Converter::convert(message) else {panic!("unreachable");};
		assert_eq!(fee, expected_fee * 2);
		assert_eq!(xcm, expected);
	}

	#[test]
	fn test_xcm_converter_send_token_remote_account_key_returns_double_execution_fee() {
		let amount = 100;
		let para_id = 1001;
		let id = [1; 20];
		let token = [2; 20];
		let chain_id = 15;
		let refund_id = RefundAccount::get();
		let destination = Destination::ForeignAccountId20 { para_id, id };
		let command = Command::SendToken { token: H160(token), destination, amount };

		let message = VersionedMessage::V1(MessageV1 { chain_id, command });
		let expected_fee = SendTokenExecutionFee::get();
		let expected = Xcm(vec![
			ReceiveTeleportedAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(expected_fee) }
					.into(),
			),
			BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(expected_fee),
				},
				weight_limit: Unlimited,
			},
			UniversalOrigin(GlobalConsensus(Ethereum { chain_id })),
			ReserveAssetDeposited(
				MultiAsset {
					id: Concrete(MultiLocation {
						parents: 2,
						interior: X2(
							GlobalConsensus(Ethereum { chain_id }),
							AccountKey20 { network: None, key: token },
						),
					}),
					fun: Fungible(amount),
				}
				.into(),
			),
			ClearOrigin,
			DepositReserveAsset {
				assets: Definite(
					MultiAsset {
						id: Concrete(MultiLocation {
							parents: 2,
							interior: X2(
								GlobalConsensus(Ethereum { chain_id }),
								AccountKey20 { network: None, key: token },
							),
						}),
						fun: Fungible(100),
					}
					.into(),
				),
				dest: MultiLocation { parents: 1, interior: X1(Parachain(para_id)) },
				xcm: Xcm(vec![
					ReceiveTeleportedAsset(
						MultiAsset {
							id: Concrete(MultiLocation::parent()),
							fun: Fungible(expected_fee),
						}
						.into(),
					),
					BuyExecution {
						fees: MultiAsset {
							id: Concrete(MultiLocation::parent()),
							fun: Fungible(expected_fee),
						},
						weight_limit: Unlimited,
					},
					DepositAsset {
						assets: Definite(
							MultiAsset {
								id: Concrete(MultiLocation {
									parents: 2,
									interior: X2(
										GlobalConsensus(Ethereum { chain_id }),
										AccountKey20 { network: None, key: token },
									),
								}),
								fun: Fungible(100),
							}
							.into(),
						),
						beneficiary: MultiLocation {
							parents: 0,
							interior: X1(AccountKey20 { network: None, key: id }),
						},
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary: MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { network: None, id: refund_id }),
						},
					},
				]),
			},
		]);
		let Ok((xcm, fee)) = Converter::convert(message) else {panic!("unreachable");};
		assert_eq!(fee, expected_fee * 2);
		assert_eq!(xcm, expected);
	}

	#[test]
	fn test_xcm_converter_register_token_fee_includes_deposit_and_execution() {
		let chain_id = 15;
		let token = [3; 20];
		let refund_id = RefundAccount::get();
		let command = Command::RegisterToken { token: token.into() };
		let message = VersionedMessage::V1(MessageV1 { chain_id, command });
		let execution_fee = CreateAssetExecutionFee::get();
		let deposit = CreateAssetDeposit::get();

		let create_call_index: [u8; 2] = CreateAssetCall::get();
		let expected_asset_id = MultiLocation {
			parents: 2,
			interior: X2(
				GlobalConsensus(Ethereum { chain_id }),
				AccountKey20 { network: None, key: token.into() },
			),
		};
		let expected = Xcm(vec![
			ReceiveTeleportedAsset(
				MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(execution_fee + deposit),
				}
				.into(),
			),
			BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(execution_fee),
				},
				weight_limit: Unlimited,
			},
			DepositAsset {
				assets: Definite(
					MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(deposit) }
						.into(),
				),
				beneficiary: MultiLocation {
					parents: 2,
					interior: X1(GlobalConsensus(Ethereum { chain_id })),
				},
			},
			UniversalOrigin(GlobalConsensus(Ethereum { chain_id })),
			Transact {
				origin_kind: OriginKind::Xcm,
				require_weight_at_most: Weight::from_parts(400000000, 8000),
				call: (
					create_call_index,
					expected_asset_id,
					MultiAddress::<[u8; 32], ()>::Id(EXPECTED_SOVEREIGN_KEY),
					MINIMUM_DEPOSIT,
				)
					.encode()
					.into(),
			},
			RefundSurplus,
			DepositAsset {
				assets: Wild(All),
				beneficiary: MultiLocation {
					parents: 0,
					interior: X1(AccountId32 { network: None, id: refund_id }),
				},
			},
		]);
		let Ok((xcm, fee)) = Converter::convert(message) else {panic!("unreachable");};
		assert_eq!(fee, execution_fee + deposit);
		assert_eq!(xcm, expected);
	}
}
