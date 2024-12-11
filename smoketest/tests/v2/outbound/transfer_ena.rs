mod transfer_ena {
	use assethub::api::polkadot_xcm::calls::TransactionApi;
	use ethers::{
		prelude::Middleware,
		providers::{Provider, Ws},
		types::Address,
	};
	use futures::StreamExt;
	use snowbridge_smoketest::{
		constants::*,
		contracts::{
			i_gateway_v2::IGatewayV2,
			weth9::{TransferFilter, WETH9},
		},
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
					junction::{Junction, NetworkId},
					junctions::Junctions,
					location::Location,
					Instruction::{DepositAsset, InitiateTransfer, PayFees, WithdrawAsset},
					Xcm,
				},
				xcm::VersionedXcm,
			},
			{self},
		},
	};
	use std::{str::FromStr, sync::Arc, time::Duration};
	use subxt::OnlineClient;
	use subxt_signer::{sr25519, SecretUri};

	#[tokio::test]
	async fn transfer_ena() {
		let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
			.await
			.unwrap()
			.interval(Duration::from_millis(10u64));

		let ethereum_client = Arc::new(ethereum_provider);

		let weth_addr: Address = (*WETH_CONTRACT).into();
		let weth = WETH9::new(weth_addr, ethereum_client.clone());

		let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
		let gateway = IGatewayV2::new(gateway_addr, ethereum_client.clone());

		let agent_src =
			gateway.agent_of(ASSET_HUB_AGENT_ID).await.expect("could not get agent address");
		println!("agent_src: {:?}", agent_src);

		let assethub: OnlineClient<AssetHubConfig> =
			OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

		let destination = Location {
			parents: 2,
			interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
				chain_id: ETHEREUM_CHAIN_ID,
			})]),
		};

		let beneficiary = Location {
			parents: 0,
			interior: Junctions::X1([Junction::AccountKey20 {
				network: None,
				key: (*ETHEREUM_RECEIVER).into(),
			}]),
		};

		let local_fee_amount: u128 = 800_000_000_000;
		let local_fee_asset = Asset {
			id: AssetId(Location { parents: 1, interior: Junctions::Here }),
			fun: Fungible(local_fee_amount),
		};
		let amount: u128 = 1_000_000_000;
		let asset_location = Location {
			parents: 2,
			interior: Junctions::X2([
				Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
				Junction::AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
			]),
		};
		let remote_fee_asset =
			Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount / 2) };
		let reserved_asset =
			Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount / 2) };

		let assets = vec![
			local_fee_asset.clone(),
			Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount) },
		];

		let xcm = VersionedXcm::V5(Xcm(vec![
			WithdrawAsset(Assets(assets.into())),
			PayFees { asset: local_fee_asset.clone() },
			InitiateTransfer {
				destination,
				remote_fees: Some(AssetTransferFilter::ReserveWithdraw(Definite(Assets(
					vec![remote_fee_asset.clone()].into(),
				)))),
				preserve_origin: true,
				assets: vec![AssetTransferFilter::ReserveWithdraw(Definite(Assets(vec![
					reserved_asset.clone(),
				])))],
				remote_xcm: Xcm(vec![DepositAsset { assets: Wild(AllCounted(2)), beneficiary }]),
			},
		]));

		let suri = SecretUri::from_str(&SUBSTRATE_KEY).expect("Parse SURI");

		let signer = sr25519::Keypair::from_uri(&suri).expect("valid keypair");

		let token_transfer_call =
			TransactionApi.execute(xcm, Weight { ref_time: 8_000_000_000, proof_size: 80_000 });

		let _ = assethub
			.tx()
			.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
			.await
			.expect("call success");

		let wait_for_blocks = 500;
		let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

		let mut transfer_event_found = false;
		while let Some(block) = stream.next().await {
			println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
			if let Ok(transfers) =
				weth.event::<TransferFilter>().at_block_hash(block.hash.unwrap()).query().await
			{
				for transfer in transfers {
					if transfer.src.eq(&agent_src) {
						println!(
							"Transfer event found at ethereum block {:?}",
							block.number.unwrap()
						);
						assert_eq!(transfer.src, agent_src.into());
						assert_eq!(transfer.dst, (*ETHEREUM_RECEIVER).into());
						assert_eq!(transfer.wad, amount.into());
						transfer_event_found = true;
					}
				}
			}
			if transfer_event_found {
				break
			}
		}
		assert!(transfer_event_found);
	}
}
