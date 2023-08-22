use std::ops::Deref;
use std::sync::Arc;
use ethers::abi::{Abi, Token};
use ethers::prelude::{
    Address, Middleware, Provider, Ws,
};
use snowbridge_smoketest::contracts::i_gateway;
use snowbridge_smoketest::helper::*;
use hex_literal::hex;
use snowbridge_smoketest::contracts::hello_world::{HelloWorld, SaidHelloFilter};
use snowbridge_smoketest::constants::*;
use snowbridge_smoketest::parachains::template::{
    api::runtime_types as templateTypes, api::runtime_types::xcm as templateXcm,
};
use snowbridge_smoketest::parachains::template::api::runtime_types::xcm::v3::junction::Junction::AccountKey20;
use snowbridge_smoketest::parachains::template::api::runtime_types::xcm::v3::junction::NetworkId::Ethereum;
use templateTypes::sp_weights::weight_v2::Weight;
use templateXcm::{
    double_encoded::DoubleEncoded,
    v2::OriginKind,
    v3::{
        junctions::Junctions,
        Instruction, Xcm, WeightLimit::Unlimited
    },
    VersionedXcm,
};
use futures::StreamExt;

const HELLO_WORLD_CONTRACT: [u8; 20] = hex!("b1185ede04202fe62d38f5db72f71e38ff3e8305");
const XCM_WEIGHT_REQUIRED: u64 = 3000000000;
const XCM_PROOF_SIZE_REQUIRED: u64 = 18000;

#[tokio::test]
async fn transact() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let agent_id: [u8; 32] =
        hex!("2075b9f5bc236462eb1473c9a6236c3588e33ed19ead53aa3d9c62ed941cb793");
    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let ethereum_client = *(test_clients.ethereum_client.clone());
    let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());
    let agent_address = gateway
        .agent_of(agent_id)
        .await
        .expect("find agent");

    assert!(!agent_address.is_zero(), "agent address not found");

    println!("agent address {}", hex::encode(agent_address));

    fund_account(&test_clients.ethereum_signed_client, agent_address)
        .await
        .expect("fund account");

    let hello_world = HelloWorld::new(HELLO_WORLD_CONTRACT, ethereum_client.clone());
    let contract_abi: Abi = hello_world.abi().clone();
    let function = contract_abi.function("sayHello").unwrap();
    let encoded_data = function
        .encode_input(&[Token::String("Hello, Clara!".to_string())])
        .unwrap();

    println!("data is {}", hex::encode(encoded_data.clone()));

    let contract_location = Junctions::X1(AccountKey20 {
        network: Some(Ethereum { chain_id: 15 }),
        key: HELLO_WORLD_CONTRACT.into(),
    });

    let inner_message = Box::new(Xcm(vec![
        Instruction::UnpaidExecution { weight_limit: Unlimited, check_origin: None },// TODO update to paid
        Instruction::DescendOrigin(contract_location), // TODO not sure if this is right, want to pass the contract address
        Instruction::Transact {
            origin_kind: OriginKind::Xcm,
            require_weight_at_most: Weight {
                ref_time: XCM_WEIGHT_REQUIRED,
                proof_size: XCM_PROOF_SIZE_REQUIRED,
            },
            call: DoubleEncoded {
                encoded: encoded_data,
            },
        },
        Instruction::SetTopic([0; 32]),
    ]));

    let message = Box::new(VersionedXcm::V3(*inner_message));
    let result = send_export_message(&test_clients.template_client, message)
        .await
        .expect("failed to send xcm transact.");

    println!(
        "xcm call issued at block hash {:?}, transaction hash {:?}",
        result.block_hash(),
        result.extrinsic_hash()
    );

    wait_for_arbitrary_ethereum_contract_event(&test_clients.ethereum_client, HELLO_WORLD_CONTRACT).await;
}


pub async fn wait_for_arbitrary_ethereum_contract_event(ethereum_client: &Box<Arc<Provider<Ws>>>, contract_address: [u8; 20]) {
    let addr: Address = contract_address.into();
    let contract = HelloWorld::new(addr, (*ethereum_client).deref().clone());

    let wait_for_blocks = 300;
    let mut stream = ethereum_client
        .subscribe_blocks()
        .await
        .unwrap()
        .take(wait_for_blocks);

    let mut ethereum_event_found = false;
    while let Some(block) = stream.next().await {
        if let Ok(events) = contract
            .event::<SaidHelloFilter>()
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
