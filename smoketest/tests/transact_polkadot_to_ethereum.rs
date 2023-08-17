use ethers::abi::{Abi, Token};
use ethers::prelude::Address;
use snowbridge_smoketest::contracts::i_gateway;
use snowbridge_smoketest::helper::*;
use hex_literal::hex;
//use snowbridge_smoketest::contracts::hello_world::{HelloWorld, SaidHelloFilter};
use snowbridge_smoketest::contracts::hello_world::HelloWorld;
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

const HELLO_WORLD_CONTRACT: [u8; 20] = hex!("EE9170ABFbf9421Ad6DD07F6BDec9D89F2B581E0");
//const BRIDGE_HUB_FEE_REQUIRED: u128 = 1000000000;
const XCM_WEIGHT_REQUIRED: u64 = 3000000000;
const XCM_PROOF_SIZE_REQUIRED: u64 = 18000;

#[tokio::test]
async fn transact() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let ethereum_client = *(test_clients.ethereum_client.clone());
    let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());
    let agent_address = gateway
        .agent_of(SIBLING_AGENT_ID)
        .await
        .expect("find agent");

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

   // wait_for_ethereum_event::<SaidHelloFilter>(&test_clients.ethereum_client, HELLO_WORLD_CONTRACT).await;
}
