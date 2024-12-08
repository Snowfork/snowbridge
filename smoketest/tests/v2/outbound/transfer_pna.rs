mod transfer_pna {
	use assethub::api::polkadot_xcm::calls::TransactionApi;
	use ethers::{
		prelude::Middleware,
		providers::{Provider, Ws},
		types::Address,
	};
	use futures::StreamExt;
	use snowbridge_smoketest::{
		constants::*,
		contracts::{token, token::TransferFilter},
		helper::AssetHubConfig,
		parachains::assethub::{
			api::runtime_types::{
				sp_weights::weight_v2::Weight,
				staging_xcm::v5::{
					asset::{
						Asset,
						AssetFilter::{Definite, Wild},
						AssetId, AssetTransferFilter, Assets,
						Fungibility::Fungible,
						WildAsset::AllCounted,
					},
					junction::{
						Junction::{AccountKey20, GlobalConsensus},
						NetworkId,
					},
					junctions::Junctions,
					location::Location,
					Instruction::*,
					Xcm,
				},
				xcm::VersionedXcm,
			},
			{self},
		},
	};
	use std::{sync::Arc, time::Duration};
	use subxt::OnlineClient;
	use subxt_signer::sr25519::dev;

	#[tokio::test]
	async fn transfer_pna() {
		let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
			.await
			.unwrap()
			.interval(Duration::from_millis(10u64));

		let ethereum_client = Arc::new(ethereum_provider);

		let assethub: OnlineClient<AssetHubConfig> =
			OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

		let destination = Location {
			parents: 2,
			interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
				chain_id: ETHEREUM_CHAIN_ID,
			})]),
		};

		let beneficiary = Location {
			parents: 0,
			interior: Junctions::X1([AccountKey20 { network: None, key: ETHEREUM_ADDRESS.into() }]),
		};

		// Local fee amount(in DOT) should cover
		// 1. execution cost on AH
		// 2. delivery cost to BH
		// 3. execution cost on BH
		let local_fee_amount = 200_000_000_000;
		// Remote fee amount(in WETH) should cover execution cost on Ethereum
		let remote_fee_amount = 4_000_000_000;

		const TOKEN_AMOUNT: u128 = 100_000_000_000;

		let weth_asset_location: Location = Location {
			parents: 2,
			interior: Junctions::X2([
				GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
				AccountKey20 { network: None, key: *WETH_CONTRACT },
			]),
		};

		let local_fee_asset = Asset {
			id: AssetId(Location { parents: 1, interior: Junctions::Here }),
			fun: Fungible(local_fee_amount),
		};
		let remote_fee_asset =
			Asset { id: AssetId(weth_asset_location), fun: Fungible(remote_fee_amount) };

		let assets = vec![
			Asset {
				id: AssetId(Location { parents: 1, interior: Junctions::Here }),
				fun: Fungible(TOKEN_AMOUNT + local_fee_amount),
			},
			remote_fee_asset.clone(),
		];

		let xcm = VersionedXcm::V5(Xcm(vec![
			WithdrawAsset(Assets(assets.into())),
			PayFees { asset: local_fee_asset },
			InitiateTransfer {
				destination,
				remote_fees: Some(AssetTransferFilter::ReserveWithdraw(Definite(Assets(
					vec![remote_fee_asset.clone()].into(),
				)))),
				preserve_origin: true,
				assets: vec![AssetTransferFilter::ReserveDeposit(Definite(Assets(vec![Asset {
					id: AssetId(Location { parents: 1, interior: Junctions::Here }),
					fun: Fungible(TOKEN_AMOUNT),
				}])))],
				remote_xcm: Xcm(vec![DepositAsset { assets: Wild(AllCounted(2)), beneficiary }]),
			},
		]));

		let signer = dev::bob();

		let token_transfer_call =
			TransactionApi.execute(xcm, Weight { ref_time: 8_000_000_000, proof_size: 80_000 });

		let _ = assethub
			.tx()
			.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
			.await
			.expect("call success");

		let erc20_dot_address: Address = ERC20_DOT_CONTRACT.into();
		let erc20_dot = token::Token::new(erc20_dot_address, ethereum_client.clone());

		let wait_for_blocks = 500;
		let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

		let mut transfer_event_found = false;
		while let Some(block) = stream.next().await {
			println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
			if let Ok(transfers) = erc20_dot
				.event::<TransferFilter>()
				.at_block_hash(block.hash.unwrap())
				.query()
				.await
			{
				for transfer in transfers {
					println!("Transfer event found at ethereum block {:?}", block.number.unwrap());
					println!("from {:?}", transfer.from);
					println!("to {:?}", transfer.to);
					assert_eq!(transfer.value, TOKEN_AMOUNT.into());
					transfer_event_found = true;
				}
			}
			if transfer_event_found {
				break
			}
		}
		assert!(transfer_event_found);
	}
}
