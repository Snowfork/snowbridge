use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use std::{sync::Arc, time::Duration};
use snowbridge_smoketest::contracts::weth9::WETH9;
use subxt::{
    tx::{PairSigner, TxPayload},
    OnlineClient, PolkadotConfig,
};
use snowbridge_smoketest::constants::{ASSET_HUB_WS_URL, ETHEREUM_API};



#[tokio::test]
async fn transfer_token() {
    let ethereum_provider = Provider::<Ws>::connect(ETHEREUM_API)
        .await
        .unwrap()
        .interval(Duration::from_millis(10u64));


    let ethereum_client = Arc::new(ethereum_provider);

    let weth_addr: Address = WETH_CONTRACT.into();
    let weth = WETH9::new(weth_addr, ethereum_client.clone());

    let assethub: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

    let ferdie = dev::ferdie();
}
