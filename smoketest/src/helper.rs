use crate::constants::*;
use crate::contracts::i_gateway;
use crate::parachains::bridgehub::{self};
use crate::parachains::template::api::runtime_types::xcm as templateXcm;
use crate::parachains::template::{self};
use ethers::prelude::{
    Address, EthEvent, LocalWallet, Middleware, Provider, Signer, SignerMiddleware,
    TransactionRequest, Ws, U256,
};
use ethers::providers::Http;
use futures::StreamExt;
use sp_core::{sr25519::Pair, Pair as PairT, H160};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use subxt::blocks::ExtrinsicEvents;
use subxt::events::StaticEvent;
use subxt::tx::{PairSigner, TxPayload};
use subxt::{Config, OnlineClient, PolkadotConfig};
use templateXcm::{
    v3::{junction::Junction, junctions::Junctions, multilocation::MultiLocation},
    VersionedMultiLocation, VersionedXcm,
};

/// Custom config that works with Statemint
pub enum TemplateConfig {}

impl Config for TemplateConfig {
    type Index = <PolkadotConfig as Config>::Index;
    type Hash = <PolkadotConfig as Config>::Hash;
    type AccountId = <PolkadotConfig as Config>::AccountId;
    type Address = <PolkadotConfig as Config>::Address;
    type Signature = <PolkadotConfig as Config>::Signature;
    type Hasher = <PolkadotConfig as Config>::Hasher;
    type Header = <PolkadotConfig as Config>::Header;
    type ExtrinsicParams = <PolkadotConfig as Config>::ExtrinsicParams;
}

pub struct TestClients {
    pub bridge_hub_client: Box<OnlineClient<PolkadotConfig>>,
    pub template_client: Box<OnlineClient<TemplateConfig>>,
    pub ethereum_client: Box<Arc<Provider<Ws>>>,
    pub ethereum_signed_client: Box<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

pub async fn initial_clients() -> Result<TestClients, Box<dyn std::error::Error>> {
    let bridge_hub_client: OnlineClient<PolkadotConfig> = OnlineClient::from_url(BRIDGE_HUB_WS_URL)
        .await
        .expect("can not connect to assethub");

    let template_client: OnlineClient<TemplateConfig> =
        OnlineClient::from_url(TEMPLATE_NODE_WS_URL)
            .await
            .expect("can not connect to assethub");

    let ethereum_provider = Provider::<Ws>::connect(ETHEREUM_API)
        .await
        .unwrap()
        .interval(Duration::from_millis(10u64));

    let ethereum_client = Arc::new(ethereum_provider);

    let ethereum_signed_client = initialize_wallet().await.expect("initialize wallet");

    Ok(TestClients {
        bridge_hub_client: Box::new(bridge_hub_client),
        template_client: Box::new(template_client),
        ethereum_client: Box::new(ethereum_client),
        ethereum_signed_client: Box::new(Arc::new(ethereum_signed_client)),
    })
}

pub async fn wait_for_bridgehub_event<Ev: StaticEvent>(
    bridge_hub_client: &Box<OnlineClient<PolkadotConfig>>,
) {
    let mut blocks = bridge_hub_client
        .blocks()
        .subscribe_finalized()
        .await
        .expect("block subscription")
        .take(5);

    let mut substrate_event_found = false;
    while let Some(Ok(block)) = blocks.next().await {
        println!(
            "Polling bridgehub block {} for expected event.",
            block.number()
        );
        let events = block.events().await.expect("read block events");
        for event in events.find::<Ev>() {
            let _ = event.expect("expect upgrade");
            println!("Event found at bridgehub block {}.", block.number());
            substrate_event_found = true;
            break;
        }
        if substrate_event_found {
            break;
        }
    }
    assert!(substrate_event_found);
}

pub async fn wait_for_ethereum_event<Ev: EthEvent>(ethereum_client: &Box<Arc<Provider<Ws>>>) {
    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let gateway = i_gateway::IGateway::new(gateway_addr, (*ethereum_client).deref().clone());

    let wait_for_blocks = 300;
    let mut stream = ethereum_client
        .subscribe_blocks()
        .await
        .unwrap()
        .take(wait_for_blocks);

    let mut ethereum_event_found = false;
    while let Some(block) = stream.next().await {
        println!(
            "Polling ethereum block {:?} for expected event",
            block.number.unwrap()
        );
        if let Ok(events) = gateway
            .event::<Ev>()
            .at_block_hash(block.hash.unwrap())
            .query()
            .await
        {
            for _ in events {
                println!("Event found at ethereum block {:?}", block.number.unwrap());
                ethereum_event_found = true;
                break;
            }
        }
        if ethereum_event_found {
            break;
        }
    }
    assert!(ethereum_event_found);
}

pub async fn send_xcm_transact(
    template_client: &Box<OnlineClient<TemplateConfig>>,
    message: Box<VersionedXcm>,
) -> Result<ExtrinsicEvents<TemplateConfig>, Box<dyn std::error::Error>> {
    let dest = Box::new(VersionedMultiLocation::V3(MultiLocation {
        parents: 1,
        interior: Junctions::X1(Junction::Parachain(BRIDGE_HUB_PARA_ID)),
    }));

    let xcm_call = template::api::polkadot_xcm::calls::TransactionApi.send(*dest, *message);

    let owner: Pair = Pair::from_string("//Alice", None).expect("cannot create keypair");

    let signer: PairSigner<TemplateConfig, _> = PairSigner::new(owner);

    let result = template_client
        .tx()
        .sign_and_submit_then_watch_default(&xcm_call, &signer)
        .await
        .expect("send through xcm call.")
        .wait_for_finalized_success()
        .await
        .expect("xcm call failed");

    Ok(result)
}

pub async fn initialize_wallet(
) -> Result<SignerMiddleware<Provider<Http>, LocalWallet>, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(ETHEREUM_HTTP_API)
        .unwrap()
        .interval(Duration::from_millis(10u64));

    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(ETHEREUM_CHAIN_ID);

    Ok(SignerMiddleware::new(provider.clone(), wallet.clone()))
}

pub async fn get_balance(
    client: &Box<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    who: Address,
) -> Result<U256, Box<dyn std::error::Error>> {
    let balance = client.get_balance(who, None).await?;

    Ok(balance)
}

pub async fn fund_account(
    client: &Box<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    address_to: Address,
) -> Result<(), Box<dyn std::error::Error>> {
    let tx = TransactionRequest::new()
        .to(address_to)
        .from(client.address())
        .value(U256::from(ethers::utils::parse_ether(1)?));
    let tx = client.send_transaction(tx, None).await?.await?;
    assert_eq!(tx.clone().unwrap().status.unwrap().as_u64(), 1u64);
    println!("receipt: {:#?}", hex::encode(tx.unwrap().transaction_hash));
    Ok(())
}

pub async fn construct_create_agent_call(
    bridge_hub_client: &Box<OnlineClient<PolkadotConfig>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let call = bridgehub::api::ethereum_control::calls::TransactionApi
        .create_agent()
        .encode_call_data(&bridge_hub_client.metadata())?;

    Ok(call)
}

pub async fn construct_create_channel_call(
    bridge_hub_client: &Box<OnlineClient<PolkadotConfig>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let call = bridgehub::api::ethereum_control::calls::TransactionApi
        .create_channel()
        .encode_call_data(&bridge_hub_client.metadata())?;

    Ok(call)
}

pub async fn construct_transfer_native_from_agent_call(
    bridge_hub_client: &Box<OnlineClient<PolkadotConfig>>,
    recipient: H160,
    amount: u128,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let call = bridgehub::api::ethereum_control::calls::TransactionApi
        .transfer_native_from_agent(recipient, amount)
        .encode_call_data(&bridge_hub_client.metadata())?;

    Ok(call)
}
