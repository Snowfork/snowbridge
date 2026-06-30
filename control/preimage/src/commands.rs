use crate::helpers::calculate_delivery_fee;
use crate::{
    constants::*, Context, ForceCheckpointArgs, GatewayAddressArgs, GatewayOperatingModeEnum,
    OperatingModeEnum, PricingParametersArgs, RebalanceSovAccountsArgs, RegisterEtherArgs,
    UpdateAssetArgs, UpgradeArgs,
};
use alloy_primitives::{utils::format_units, U256};
use codec::Encode;
use snowbridge_router_primitives::inbound::GlobalConsensusEthereumConvertsFor;
use sp_arithmetic::FixedU128;
use sp_crypto_hashing::twox_128;
use std::{fs::File, io::Read};
use subxt::utils::MultiAddress;
use subxt::utils::Static;

type CheckpointUpdate = snowbridge_beacon_primitives::CheckpointUpdate<512>;

use crate::asset_hub_runtime::runtime_types::pallet_assets;
use crate::asset_hub_runtime::RuntimeCall as AssetHubRuntimeCall;

use crate::bridge_hub_runtime::runtime_types::{
    snowbridge_core::{
        operating_mode::BasicOperatingMode,
        pricing::{PricingParameters, Rewards},
    },
    snowbridge_outbound_queue_primitives::{v1::message::Initializer, OperatingMode},
    snowbridge_pallet_ethereum_client, snowbridge_pallet_inbound_queue,
    snowbridge_pallet_inbound_queue_v2, snowbridge_pallet_outbound_queue,
    snowbridge_pallet_system, snowbridge_pallet_system_v2,
};
use crate::bridge_hub_runtime::RuntimeCall as BridgeHubRuntimeCall;

use crate::asset_hub_runtime::runtime_types::{
    snowbridge_core::operating_mode::BasicOperatingMode as AssetHubBasicOperatingMode,
    snowbridge_pallet_system_frontend,
};
#[cfg(feature = "polkadot")]
pub mod asset_hub_polkadot_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
        location::Location,
    };

    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
            parents: 2,
            interior: X2([
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ]),
        };
    }
}

#[cfg(feature = "paseo")]
pub mod asset_hub_paseo_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
        location::Location,
    };

    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
            parents: 2,
            interior: X2([
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ]),
        };
    }
}

#[cfg(feature = "westend")]
pub mod asset_hub_westend_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
        location::Location,
    };
    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
            parents: 2,
            interior: X2([
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ]),
        };
    }
}

pub fn gateway_operating_mode(operating_mode: &GatewayOperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match operating_mode {
        GatewayOperatingModeEnum::Normal => OperatingMode::Normal,
        GatewayOperatingModeEnum::RejectingOutboundMessages => {
            OperatingMode::RejectingOutboundMessages
        }
    };
    BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::set_operating_mode { mode },
    )
}

pub fn inbound_queue_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumInboundQueue(
        snowbridge_pallet_inbound_queue::pallet::Call::set_operating_mode { mode },
    )
}

pub fn ethereum_client_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::set_operating_mode { mode },
    )
}

pub fn outbound_queue_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumOutboundQueue(
        snowbridge_pallet_outbound_queue::pallet::Call::set_operating_mode { mode },
    )
}

// V2 variant: halts the inbound-queue-v2 pallet's `submit` extrinsic, blocking
// processing of V2 Ethereum -> Polkadot messages on BridgeHub.
pub fn inbound_queue_v2_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumInboundQueueV2(
        snowbridge_pallet_inbound_queue_v2::pallet::Call::set_operating_mode { mode },
    )
}

// V2 variant: sends `Command::SetOperatingMode` to the Gateway via the V2 outbound
// queue. Sets the same Gateway `$.mode` storage as the V1 variant; both are kept
// so governance can halt via whichever outbound path is live.
pub fn gateway_operating_mode_v2(
    operating_mode: &GatewayOperatingModeEnum,
) -> BridgeHubRuntimeCall {
    let mode = match operating_mode {
        GatewayOperatingModeEnum::Normal => OperatingMode::Normal,
        GatewayOperatingModeEnum::RejectingOutboundMessages => {
            OperatingMode::RejectingOutboundMessages
        }
    };
    BridgeHubRuntimeCall::EthereumSystemV2(
        snowbridge_pallet_system_v2::pallet::Call::set_operating_mode { mode },
    )
}

// AssetHub-side: halts the system-frontend pallet. The `PausableExporter` wrapping
// the AssetHub->Ethereum XcmRouter consults `SnowbridgeSystemFrontend::is_paused()`
// and returns `SendError::NotApplicable` when halted, short-circuiting every
// AssetHub->Ethereum export (V1 and V2 share the same wrapper). This is the
// primary outbound halt lever for V2 because `outbound-queue-v2` has no local
// operating-mode storage.
pub fn system_frontend_operating_mode(param: &OperatingModeEnum) -> AssetHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => AssetHubBasicOperatingMode::Normal,
        OperatingModeEnum::Halted => AssetHubBasicOperatingMode::Halted,
    };
    AssetHubRuntimeCall::SnowbridgeSystemFrontend(
        snowbridge_pallet_system_frontend::pallet::Call::set_operating_mode { mode },
    )
}

pub fn upgrade(params: &UpgradeArgs) -> BridgeHubRuntimeCall {
    BridgeHubRuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::upgrade {
        impl_address: params.logic_address.into_array().into(),
        impl_code_hash: params.logic_code_hash.0.into(),
        initializer: Some(Initializer {
            params: params.initializer_params.clone().into(),
            maximum_required_gas: params.initializer_gas,
        }),
    })
}

pub async fn pricing_parameters(
    context: &Context,
    params: &PricingParametersArgs,
) -> Result<(BridgeHubRuntimeCall, AssetHubRuntimeCall), Box<dyn std::error::Error>> {
    // BridgeHub parameters
    let pricing_params: PricingParameters<u128> = PricingParameters {
        exchange_rate: Static(FixedU128::from_rational(
            params.exchange_rate_numerator.into(),
            params.exchange_rate_denominator.into(),
        )),
        multiplier: Static(FixedU128::from_rational(
            params.multiplier_numerator.into(),
            params.multiplier_denominator.into(),
        )),
        fee_per_gas: crate::bridge_hub_runtime::runtime_types::primitive_types::U256(
            params.fee_per_gas.into_limbs(),
        ),
        rewards: Rewards {
            local: params.local_reward.to::<u128>(),
            remote: crate::bridge_hub_runtime::runtime_types::primitive_types::U256(
                params.remote_reward.into_limbs(),
            ),
        },
    };

    let outbound_delivery_fee =
        calculate_delivery_fee(&context.bridge_hub_api, &pricing_params).await?;

    let total_outbound_fee = outbound_delivery_fee.local + outbound_delivery_fee.remote;

    // Adjust outbound fee up by 10% as a buffer
    let total_outbound_fee_adjusted = total_outbound_fee.saturating_add(total_outbound_fee / 10);

    eprintln!("BridgeHub:");
    eprintln!(
        "  ExchangeRate: {} ETH/{}",
        params.exchange_rate_numerator as f64 / params.exchange_rate_denominator as f64,
        POLKADOT_SYMBOL
    );
    eprintln!(
        "  FeePerGas: {} GWEI",
        format_units(params.fee_per_gas, "gwei").unwrap(),
    );
    eprintln!(
        "  LocalReward: {} {} [{} PLANCK]",
        format_units(U256::from(params.local_reward), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        params.local_reward,
    );
    eprintln!(
        "  RemoteReward: {} ETH [{} WEI]",
        format_units(params.remote_reward, "eth").unwrap(),
        params.remote_reward
    );
    eprintln!("AssetHub:");
    eprintln!(
        "  BaseFee: {} {}, [{} PLANCK]",
        format_units(U256::from(total_outbound_fee_adjusted), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        total_outbound_fee_adjusted
    );

    // AssetHub parameters
    let asset_hub_outbound_fee_storage_key: Vec<u8> =
        twox_128(b":BridgeHubEthereumBaseFee:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = total_outbound_fee_adjusted.encode();

    eprintln!(
        "Storage key for 'BridgeHubEthereumBaseFee': 0x{}",
        hex::encode(&asset_hub_outbound_fee_storage_key)
    );

    Ok((
        BridgeHubRuntimeCall::EthereumSystem(
            snowbridge_pallet_system::pallet::Call::set_pricing_parameters {
                params: pricing_params,
            },
        ),
        AssetHubRuntimeCall::System(
            crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
                items: vec![(
                    asset_hub_outbound_fee_storage_key,
                    asset_hub_outbound_fee_encoded,
                )],
            },
        ),
    ))
}

pub fn set_assethub_fee(fee: u128) -> AssetHubRuntimeCall {
    let asset_hub_outbound_fee_storage_key: Vec<u8> =
        twox_128(b":BridgeHubEthereumBaseFee:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = fee.encode();

    eprintln!(
        "Storage key for 'BridgeHubEthereumBaseFee': 0x{}",
        hex::encode(&asset_hub_outbound_fee_storage_key)
    );

    AssetHubRuntimeCall::System(
        crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(
                asset_hub_outbound_fee_storage_key,
                asset_hub_outbound_fee_encoded,
            )],
        },
    )
}

pub fn set_assethub_fee_v2(fee: u128) -> AssetHubRuntimeCall {
    let asset_hub_outbound_fee_storage_key: Vec<u8> =
        twox_128(b":BridgeHubEthereumBaseFeeV2:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = fee.encode();

    eprintln!(
        "Storage key for 'BridgeHubEthereumBaseFeeV2': 0x{}",
        hex::encode(&asset_hub_outbound_fee_storage_key)
    );

    AssetHubRuntimeCall::System(
        crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(
                asset_hub_outbound_fee_storage_key,
                asset_hub_outbound_fee_encoded,
            )],
        },
    )
}

pub fn force_checkpoint(params: &ForceCheckpointArgs) -> BridgeHubRuntimeCall {
    let mut file = File::open(params.checkpoint.clone()).expect("File not found");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Failed to read the file");
    let checkpoint: CheckpointUpdate = serde_json::from_str(&data).unwrap();
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::force_checkpoint {
            update: Box::new(Static(checkpoint)),
        },
    )
}

pub fn set_gateway_address(params: &GatewayAddressArgs) -> BridgeHubRuntimeCall {
    let storage_key = sp_crypto_hashing::twox_128(b":EthereumGatewayAddress:").to_vec();
    let storage_value = params.gateway_address.into_array().encode();
    BridgeHubRuntimeCall::System(
        crate::bridge_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(storage_key, storage_value)],
        },
    )
}

/// Quote the DOT needed to receive exactly `eth_out` wei of Ether via AssetConversion.
/// `include_fee = true` includes the 0.3% LP fee; the difference between the two is the fee.
#[cfg(feature = "polkadot")]
async fn quote_dot_for_exact_eth(
    context: &Context,
    eth_out: u128,
    include_fee: bool,
) -> Result<u128, Box<dyn std::error::Error>> {
    use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junctions::Junctions, location::Location,
    };
    let dot = Location {
        parents: 1,
        interior: Junctions::Here,
    };
    let eth = crate::commands::asset_hub_polkadot_types::get_ether_id(
        crate::bridge_hub_runtime::CHAIN_ID,
    );
    context
        .asset_hub_api
        .runtime_api()
        .at_latest()
        .await?
        .call(
            crate::asset_hub_runtime::apis()
                .asset_conversion_api()
                .quote_price_tokens_for_exact_tokens(dot, eth, eth_out, include_fee),
        )
        .await?
        .ok_or_else(|| "AssetConversionApi returned no DOT quote for the requested ETH amount".into())
}

#[cfg(feature = "polkadot")]
pub async fn rebalance_sov_accounts(
    context: &Context,
    params: &RebalanceSovAccountsArgs,
) -> Result<AssetHubRuntimeCall, Box<dyn std::error::Error>> {
    use crate::asset_hub_runtime::runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        pallet_xcm,
        sp_weights::weight_v2::Weight,
        staging_xcm::v5::{
            asset::{
                Asset, AssetFilter, AssetId, AssetTransferFilter, Assets, Fungibility, WildAsset,
            },
            junction::{Junction, NetworkId},
            junctions::Junctions,
            location::Location,
            Instruction::*,
            Xcm,
        },
        xcm::v3::WeightLimit,
        xcm::VersionedXcm,
    };

    const GATEWAY_PROXY: [u8; 20] = hex_literal::hex!("27ca963c279c93801941e1eb8799c23f407d68e7");
    // Asset Hub Treasury account = `PalletId(*b"py/trsry").into_account_truncating()`
    // = b"modl" ++ b"py/trsry" ++ [0u8; 20]. SS58 13UVJyLnbVp9RBZYFwFGyDvVd1y27Tt8tkntv6Q7JVPhFsTB.
    // This is where Snowbridge/XCM delivery fees historically accrue (pre-DAP FeeManager sink).
    const TREASURY_ACCOUNT: [u8; 32] =
        hex_literal::hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000");

    if params.eth_swap_price_pad < 0.0 || params.eth_swap_slippage_pad < 0.0 {
        return Err("swap pads must be non-negative decimals (e.g. 0.1 for 10%)".into());
    }

    let gateway_eth: u128 = params.eth_amount.try_into().map_err(|_| {
        format!("--eth-amount {} exceeds the u128 supported by XCM", params.eth_amount)
    })?;
    let bridge_fee_eth: u128 = params.bridge_fee_eth.try_into().map_err(|_| {
        format!("--bridge-fee-eth {} exceeds the u128 supported by XCM", params.bridge_fee_eth)
    })?;
    let dot_amount = params.dot_amount.to::<u128>();

    // The swap must produce exactly the gateway top-up plus the Ethereum-leg bridge fee.
    let eth_out = gateway_eth
        .checked_add(bridge_fee_eth)
        .ok_or("ETH out (eth-amount + bridge-fee-eth) overflowed u128")?;

    // `include_fee=true` is the real exact-out cost (constant-product price impact + 0.3% LP fee);
    // `include_fee=false` is the linear spot estimate (no impact, no fee). swap_base is what the
    // swap actually spends, so `give` is built from it.
    let swap_base = quote_dot_for_exact_eth(context, eth_out, true).await?;
    let dot_no_fee = quote_dot_for_exact_eth(context, eth_out, false).await?;
    // Split the gap honestly: the 0.3% LP fee is applied to the input, the remainder is the
    // pool price impact (which is large when the DOT/Ether pool is shallow).
    let lp_fee = swap_base.saturating_mul(3) / 1000;
    let price_impact = swap_base.saturating_sub(dot_no_fee).saturating_sub(lp_fee);

    // Display split of the spot DOT across the two ETH buckets (proportional by ETH).
    let gateway_dot = (dot_no_fee as u128).saturating_mul(gateway_eth) / eth_out;
    let bridge_fee_dot = dot_no_fee.saturating_sub(gateway_dot);

    // Pads (decimals, e.g. 0.10) applied to the full swap cost, in parts-per-billion for precision.
    let ppb = 1_000_000_000u128;
    let price_ppb = (params.eth_swap_price_pad * ppb as f64).round() as u128;
    let slippage_ppb = (params.eth_swap_slippage_pad * ppb as f64).round() as u128;
    let price_pad = swap_base.saturating_mul(price_ppb) / ppb;
    let slippage_pad = swap_base.saturating_mul(slippage_ppb) / ppb;

    // `give` is the max DOT the exact-out swap may spend; unused DOT flows to the AH sovereign.
    let give_dot = swap_base
        .checked_add(price_pad)
        .and_then(|v| v.checked_add(slippage_pad))
        .ok_or("swap give amount overflowed u128")?;
    // Bridge Hub charges for executing the inbound teleport + deposit of the remaining DOT into the
    // Asset Hub sovereign account. The unpaid-execution path is not available to us here (the
    // origin after the teleport is neither cleared-and-allowed nor an unpaid-allowed location), so
    // we pay BH execution with teleported DOT. Draw it from the Treasury on top of `dot_amount` so
    // the sovereign still nets the full amount; any unused remainder is refunded into the deposit.
    let bridge_hub_fee_dot = params.bridge_hub_fee_dot.to::<u128>();
    let withdraw_dot = dot_amount
        .checked_add(give_dot)
        .and_then(|v| v.checked_add(bridge_hub_fee_dot))
        .ok_or("total DOT withdraw overflowed u128")?;

    let dot = |x: u128| format_units(U256::from(x), POLKADOT_DECIMALS).unwrap();
    let eth = |x: u128| format_units(U256::from(x), "eth").unwrap();
    let pct = |p: f64| {
        let s = format!("{:.4}", p * 100.0);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    let to_dot = |x: u128| x as f64 / 10f64.powi(POLKADOT_DECIMALS as i32);
    let to_eth = |x: u128| x as f64 / 10f64.powi(18);
    let spot_rate = to_dot(dot_no_fee) / to_eth(eth_out);
    let eff_rate = to_dot(swap_base) / to_eth(eth_out);

    eprintln!("Rebalance Sovereign Accounts (Polkadot)");
    eprintln!(
        "Exchange rate = {:.3} DOT/ETH spot, {:.3} DOT/ETH effective ({} ETH out)",
        spot_rate, eff_rate, eth(eth_out)
    );
    eprintln!("Total Withdraw from Treasury = {} DOT", dot(withdraw_dot));
    eprintln!("  Bridge Hub Sovereign     = {} DOT", dot(dot_amount));
    eprintln!("  Snowbridge Gateway       = {} DOT ({} ETH)", dot(gateway_dot), eth(gateway_eth));
    eprintln!("  Bridging fee             = {} DOT ({} ETH)", dot(bridge_fee_dot), eth(bridge_fee_eth));
    eprintln!("  Pool price impact        = {} DOT", dot(price_impact));
    eprintln!("  LP fee (0.3%)            = {} DOT", dot(lp_fee));
    eprintln!("  Price fluctuation pad    = {} DOT ({}% pad)", dot(price_pad), pct(params.eth_swap_price_pad));
    eprintln!("  Slippage pad             = {} DOT ({}% pad)", dot(slippage_pad), pct(params.eth_swap_slippage_pad));
    eprintln!("  Bridge Hub exec fee      = {} DOT (refunded into sovereign if unused)", dot(bridge_hub_fee_dot));

    let dot_location = Location {
        parents: 1,
        interior: Junctions::Here,
    };
    let eth_location = Location {
        parents: 2,
        interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
            chain_id: crate::bridge_hub_runtime::CHAIN_ID,
        })]),
    };
    let treasury_location = Location {
        parents: 0,
        interior: Junctions::X1([Junction::AccountId32 {
            network: None,
            id: TREASURY_ACCOUNT,
        }]),
    };

    let dot_for_withdraw = asset(dot_location.clone(), withdraw_dot);
    let dot_for_swap = asset(dot_location.clone(), give_dot);
    let eth_out_asset = asset(eth_location.clone(), eth_out);
    let eth_for_gateway = asset(eth_location.clone(), gateway_eth);
    let eth_for_fee = asset(eth_location.clone(), bridge_fee_eth);
    let dot_for_bh_fee = asset(dot_location.clone(), bridge_hub_fee_dot);

    // The Snowbridge V2 outbound converter requires the exported message to end
    // with SetTopic (otherwise XcmConverterError::SetTopicExpected => the BridgeHub
    // export to Ethereum can't be converted and the whole message is Unroutable).
    // Derive a deterministic topic from the transfer parameters so the preimage
    // stays reproducible while distinct rebalances get distinct message ids.
    let topic: [u8; 32] = sp_crypto_hashing::blake2_256(
        &(gateway_eth, bridge_fee_eth, withdraw_dot, give_dot).encode(),
    );

    let message = Xcm(vec![
        UnpaidExecution {
            weight_limit: WeightLimit::Unlimited,
            check_origin: None,
        },
        // Act as the Treasury so the withdraw debits its balance.
        AliasOrigin(treasury_location.clone()),
        // Pay onward-message delivery fees by withdrawing from the Treasury (origin) directly,
        // not from holding. Otherwise the final "all remaining DOT" teleport drains holding
        // before the BridgeHub send charges its delivery fee, failing with NotHoldingFees.
        SetFeesMode { jit_withdraw: true },
        WithdrawAsset(Assets(vec![dot_for_withdraw])),
        // Set early so any leftover/dust (and any error path) refunds to the Treasury.
        SetAppendix(Xcm(vec![
            RefundSurplus,
            DepositAsset {
                assets: AssetFilter::Wild(WildAsset::All),
                beneficiary: treasury_location,
            },
        ])),
        // Swap for the EXACT ETH out (gateway + bridge fee), spending up to `give_dot`; fail otherwise.
        ExchangeAsset {
            give: AssetFilter::Definite(Assets(vec![dot_for_swap])),
            want: Assets(vec![eth_out_asset]),
            maximal: false,
        },
        // Bridge the exact gateway ETH to Ethereum, paying the Ethereum-side fee in ETH.
        InitiateTransfer {
            destination: eth_location.clone(),
            remote_fees: Some(AssetTransferFilter::ReserveWithdraw(AssetFilter::Definite(
                Assets(vec![eth_for_fee]),
            ))),
            preserve_origin: true,
            assets: BoundedVec(vec![AssetTransferFilter::ReserveWithdraw(
                AssetFilter::Definite(Assets(vec![eth_for_gateway])),
            )]),
            remote_xcm: Xcm(vec![
                DepositAsset {
                    // The Snowbridge V2 converter evaluates this in Ethereum context, where the
                    // gateway ETH reanchors to `Here`. An `AllOf { id: <Ethereum location> }`
                    // filter would not match that id (FilterDoesNotConsumeAllAssets => Unroutable),
                    // so deposit by count like the canonical builders do.
                    assets: AssetFilter::Wild(WildAsset::AllCounted(1)),
                    beneficiary: Location {
                        parents: 0,
                        interior: Junctions::X1([Junction::AccountKey20 {
                            network: None,
                            key: GATEWAY_PROXY,
                        }]),
                    },
                },
                // Required by the Snowbridge V2 export converter (SetTopicExpected).
                SetTopic(topic),
            ]),
        },
        // Send ALL remaining DOT to the Asset Hub sovereign account on Bridge Hub.
        // DOT moves between system parachains by teleport (the relay is its reserve, not BH),
        // so ReserveWithdraw here fails with UntrustedReserveLocation.
        //
        // We must pay for BH execution: `remote_fees: None` would append `UnpaidExecution`, which
        // BH's barrier rejects here (it disallows `ClearOrigin` on the unpaid path, and an
        // `AliasOrigin` to the Treasury is not an unpaid-allowed origin). Paying with a teleported
        // DOT fee instead routes the message through BH's paid-execution barrier, which accepts the
        // sibling Asset Hub origin and tolerates the subsequent `ClearOrigin`.
        InitiateTransfer {
            destination: Location {
                parents: 1,
                interior: Junctions::X1([Junction::Parachain(BRIDGE_HUB_ID)]),
            },
            remote_fees: Some(AssetTransferFilter::Teleport(AssetFilter::Definite(Assets(
                vec![dot_for_bh_fee],
            )))),
            // A plain deposit to the sovereign needs no origin; ClearOrigin keeps the paid barrier happy.
            preserve_origin: false,
            assets: BoundedVec(vec![AssetTransferFilter::Teleport(AssetFilter::Wild(
                WildAsset::AllCounted(1),
            ))]),
            remote_xcm: Xcm(vec![
                // Refund the unused execution fee back into holding so it lands in the sovereign too.
                RefundSurplus,
                DepositAsset {
                    assets: AssetFilter::Wild(WildAsset::All),
                    beneficiary: Location {
                        parents: 1,
                        interior: Junctions::X1([Junction::Parachain(ASSET_HUB_ID)]),
                    },
                },
            ]),
        },
        // Correlate the AH-side message with the bridged Ethereum message for tracing.
        SetTopic(topic),
    ]);

    fn asset(id: Location, amount: u128) -> Asset {
        Asset {
            id: AssetId(id),
            fun: Fungibility::Fungible(amount),
        }
    }

    Ok(AssetHubRuntimeCall::PolkadotXcm(
        pallet_xcm::pallet::Call::execute {
            message: Box::new(VersionedXcm::V5(message)),
            max_weight: Weight {
                ref_time: 500_000_000_000 - 1,
                proof_size: 3 * 1024 * 1024 - 1,
            },
        },
    ))
}

#[cfg(not(feature = "polkadot"))]
pub async fn rebalance_sov_accounts(
    _context: &Context,
    _params: &RebalanceSovAccountsArgs,
) -> Result<AssetHubRuntimeCall, Box<dyn std::error::Error>> {
    panic!("RebalanceSovAccounts only for polkadot runtime.");
}

pub fn make_asset_sufficient(params: &UpdateAssetArgs) -> AssetHubRuntimeCall {
    use subxt::utils::AccountId32;
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;
    let asset_id = get_asset_id(chain_id, params.contract_id.into_array().into());
    let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);
    AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_asset_status {
        id: asset_id,
        owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        issuer: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        admin: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        freezer: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        min_balance: params.min_balance,
        is_sufficient: params.is_sufficient,
        is_frozen: params.is_frozen,
    })
}

pub fn force_set_metadata(params: &UpdateAssetArgs) -> AssetHubRuntimeCall {
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;
    let asset_id = get_asset_id(chain_id, params.contract_id.into_array().into());
    AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_set_metadata {
        id: asset_id,
        name: params.name.as_bytes().to_vec(),
        symbol: params.symbol.as_bytes().to_vec(),
        decimals: params.decimals,
        is_frozen: params.is_frozen,
    })
}

pub fn register_ether(params: &RegisterEtherArgs) -> (AssetHubRuntimeCall, AssetHubRuntimeCall) {
    use subxt::utils::AccountId32;
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;

    let asset_id = get_ether_id(chain_id);
    let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);

    let force_register =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_create {
            id: asset_id.clone(),
            min_balance: params.ether_min_balance,
            is_sufficient: true,
            owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        });
    let metadata =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_set_metadata {
            id: asset_id,
            name: params.ether_name.as_bytes().to_vec(),
            symbol: params.ether_symbol.as_bytes().to_vec(),
            decimals: params.ether_decimals,
            is_frozen: false,
        });

    return (force_register, metadata);
}

#[cfg(feature = "polkadot")]
fn register_polkadot_native_asset(
    location: crate::bridge_hub_runtime::runtime_types::xcm::VersionedLocation,
    name: &'static str,
    symbol: &'static str,
    decimals: u8,
) -> BridgeHubRuntimeCall {
    use crate::bridge_hub_runtime::runtime_types::{bounded_collections, snowbridge_core};

    let call = BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::register_token {
            location: location.into(),
            metadata: snowbridge_core::AssetMetadata {
                name: bounded_collections::bounded_vec::BoundedVec(name.as_bytes().to_vec()),
                symbol: bounded_collections::bounded_vec::BoundedVec(symbol.as_bytes().to_vec()),
                decimals,
            },
        },
    );

    return call;
}

#[cfg(feature = "polkadot")]
pub fn register_erc20_token_metadata() -> Vec<AssetHubRuntimeCall> {
    use alloy_primitives::Address;
    use hex_literal::hex;

    let tokens = vec![
        (
            hex!("9d39a5de30e57443bff2a8307a4256c8797a3497"),
            "Staked USDe",
            "sUSDe",
            18,
        ),
        (
            hex!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
            "Wrapped Ether",
            "WETH",
            18,
        ),
        (
            hex!("6982508145454ce325ddbe47a25d4ec3d2311933"),
            "Pepe",
            "PEPE",
            18,
        ),
        (
            hex!("5a98fcbea516cf06857215779fd812ca3bef1b32"),
            "Lido DAO Token",
            "LDO",
            18,
        ),
        (
            hex!("a3931d71877c0e7a3148cb7eb4463524fec27fbd"),
            "Savings USDS",
            "sUSDS",
            18,
        ),
        (
            hex!("8236a87084f8b84306f72007f36f2618a5634494"),
            "Lombard Staked Bitcoin",
            "LBTC",
            8,
        ),
        (
            hex!("1abaea1f7c830bd89acc67ec4af516284b1bc33c"),
            "Euro Coin",
            "EURC",
            6,
        ),
        (
            hex!("56072c95faa701256059aa122697b133aded9279"),
            "SKY Governance Token",
            "SKY",
            18,
        ),
        (
            hex!("ba41ddf06b7ffd89d1267b5a93bfef2424eb2003"),
            "Mythos",
            "MYTH",
            18,
        ),
        (
            hex!("0e186357c323c806c1efdad36d217f7a54b63d18"),
            "Curio Gas Token",
            "CGT2.0",
            18,
        ),
        (
            hex!("aa7a9ca87d3694b5755f213b5d04094b8d0f0a6f"),
            "OriginTrail TRAC",
            "TRAC",
            18,
        ),
        (
            hex!("18084fba666a33d37592fa2633fd49a74dd93a88"),
            "tBTC v2",
            "tBTC",
            18,
        ),
        (
            hex!("7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0"),
            "Wrapped liquid staked Ether 2.0",
            "wstETH",
            18,
        ),
        (
            hex!("582d872a1b094fc48f5de31d3b73f2d9be47def1"),
            "Wrapped TON Coin",
            "TONCOIN",
            9,
        ),
        (
            hex!("6b175474e89094c44da98b954eedeac495271d0f"),
            "Dai Stablecoin",
            "DAI",
            18,
        ),
        (
            hex!("95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce"),
            "SHIBA INU",
            "SHIB",
            18,
        ),
        (
            hex!("7de91b204c1c737bcee6f000aaa6569cf7061cb7"),
            "Robonomics",
            "XRT",
            9,
        ),
        (
            hex!("2260fac5e5542a773aa44fbcfedf7c193bc2c599"),
            "Wrapped BTC",
            "WBTC",
            8,
        ),
        (
            hex!("8daebade922df735c38c80c7ebd708af50815faa"),
            "tBTC",
            "TBTC",
            18,
        ),
        (
            hex!("5d3d01fd6d2ad1169b17918eb4f153c6616288eb"),
            "KILT",
            "KILT",
            15,
        ),
        (
            hex!("514910771af9ca656af840dff83e8264ecf986ca"),
            "ChainLink Token",
            "LINK",
            18,
        ),
        (
            hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9"),
            "Aave Token",
            "AAVE",
            18,
        ),
    ];

    tokens
        .into_iter()
        .map(|(contract_address, name, symbol, decimals)| {
            let params = UpdateAssetArgs {
                contract_id: Address::from(contract_address),
                name: name.to_string(),
                symbol: symbol.to_string(),
                decimals,
                min_balance: 1,
                is_sufficient: false,
                is_frozen: false,
            };
            force_set_metadata(&params)
        })
        .collect()
}

#[cfg(feature = "polkadot")]
pub fn token_registrations() -> Vec<BridgeHubRuntimeCall> {
    use crate::bridge_hub_runtime::runtime_types::{
        staging_xcm::v5::{
            junction::Junction::*, junction::NetworkId::*, junctions::Junctions::*,
            location::Location,
        },
        xcm::VersionedLocation,
    };
    use hex_literal::hex;
    return vec![
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: Here,
            }),
            "Polkadot",
            "DOT",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 2,
                interior: X1([GlobalConsensus(Kusama)]),
            }),
            "Kusama",
            "KSM",
            12u8,
        ),
        /*
         * Parachains
         */
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([Parachain(2004), PalletInstance(10)]),
            }),
            "Glimmer",
            "GLMR",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2030),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0001000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Bifrost Native Token",
            "BNC",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2030),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0900000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Voucher DOT",
            "vDOT",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([Parachain(2034), GeneralIndex(0)]),
            }),
            "Hydration",
            "HDX",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2039)]),
            }),
            "Integritee TEER",
            "TEER",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2051)]),
            }),
            "Ajuna Polkadot AJUN",
            "AJUN",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(3344)]),
            }),
            "Polimec",
            "PLMC",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(3370)]),
            }),
            "LAOS",
            "LAOS",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2086)]),
            }),
            "KILT Spiritnet",
            "KILT",
            15u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2006)]),
            }),
            "Astar",
            "ASTR",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2031),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0001000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Centrifuge",
            "CFG",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2101)]),
            }),
            "Subsocial",
            "SUB",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2035)]),
            }),
            "Phala Token",
            "PHA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2012),
                    GeneralKey {
                        length: 4,
                        data: hex!(
                            "5041524100000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Parallel",
            "PARA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2008)]),
            }),
            "Crust Parachain Native Token",
            "CRU",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X1([Parachain(2104)]),
            }),
            "Manta",
            "MANTA",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2000),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0000000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Acala",
            "ACA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X2([
                    Parachain(2000),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0003000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Liquid DOT",
            "LDOT",
            10u8,
        ),
        /*
         * Meme coins
         */
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(30)]),
            }),
            "DED",
            "DED",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(23)]),
            }),
            "PINK",
            "PINK",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(86)]),
            }),
            "Kolkadot",
            "KOL",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V5(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(31337)]),
            }),
            "GAVUN WUD",
            "WUD",
            10u8,
        ),
    ];
}

pub fn replay_sep_2025_xcm() -> crate::asset_hub_runtime::RuntimeCall {
    use crate::asset_hub_runtime::runtime_types::{
        pallet_xcm,
        staging_xcm::v5::{
            asset::{Asset, AssetId, Assets, Fungibility, WildAsset},
            junction::Junction,
            junctions::Junctions,
            location::Location,
            Instruction::*,
            Xcm,
        },
        xcm::{v3::WeightLimit, VersionedLocation, VersionedXcm},
    };
    use hex_literal::hex;

    enum AssetType {
        Erc20([u8; 20]),
        Eth,
    }

    // Failed XCM messages to replay - each contains asset, amount, beneficiary, and topic
    let failed_messages: Vec<(AssetType, u128, [u8; 20], [u8; 32])> = vec![
        // SKY token transfer
        (
            AssetType::Erc20(hex!("56072c95faa701256059aa122697b133aded9279")),
            90413710543975890000000,
            hex!("601d579ecd0464a1a090ceef81a703465a1679cd"),
            hex!("f701fb349a04e4c923e26aab4e0288975d904507cdc32a3d3bdab8105507c736"),
        ),
        // sUSDe token transfer
        (
            AssetType::Erc20(hex!("9d39a5de30e57443bff2a8307a4256c8797a3497")),
            16716000000000000000000,
            hex!("9117900a3794ad6d167dd97853f82a1aa07f9bbc"),
            hex!("e4cff6bf2217eb4cf9332d2daee1ada70b405402414a2249a6e9b42ab759f93f"),
        ),
        // tBTC v2 token transfer
        (
            AssetType::Erc20(hex!("18084fba666a33d37592fa2633fd49a74dd93a88")),
            250830765728855800,
            hex!("601d579ecd0464a1a090ceef81a703465a1679cd"),
            hex!("d289d29c0ccbca0fe47be2a0bf8d09af3a90d719ce62129d75714a342750b6e4"),
        ),
        // AAVE token transfer 1
        (
            AssetType::Erc20(hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9")),
            33044703802651993696,
            hex!("2265a7503597ab32bab72eaa186e6329fb7b68f3"),
            hex!("c8864869cd4ed5921d5cc251290357ffb24f905e1a475a2ba6c9ecd96c55df71"),
        ),
        // AAVE token transfer 2
        (
            AssetType::Erc20(hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9")),
            212116067921877821839,
            hex!("a9c415d6881e1a992861a7fa6bef3ed4736152c2"),
            hex!("e259e4fb1c24f7cf4e6d2a9d50e13794de5fd6863083addc3d55ddff3b3d58cd"),
        ),
        // ETH transfer
        (
            AssetType::Eth,
            350000000000000000,
            hex!("ad8d4c544a6ce24b89841354b2738e026a12bca4"),
            hex!("1ae83a0cba8f448c466fb0863fc25827b6978b7c3b3f93785184412cb2632e31"),
        ),
    ];

    let mut all_instructions = vec![UnpaidExecution {
        weight_limit: WeightLimit::Unlimited,
        check_origin: None,
    }];

    // Add all failed messages as separate ExportMessage instructions
    for (asset_type, amount, beneficiary_address, topic) in failed_messages.iter() {
        let asset_location = match asset_type {
            AssetType::Erc20(token_address) => Location {
                parents: 0,
                interior: Junctions::X1([Junction::AccountKey20 {
                    network: None,
                    key: *token_address,
                }]),
            },
            AssetType::Eth => Location {
                parents: 0,
                interior: Junctions::Here,
            },
        };

        all_instructions.push(ExportMessage {
            network: crate::asset_hub_runtime::runtime_types::staging_xcm::v5::junction::NetworkId::Ethereum {
                chain_id: crate::bridge_hub_runtime::CHAIN_ID,
            },
            destination: Junctions::Here,
            xcm: Xcm(vec![
                WithdrawAsset(Assets(vec![Asset {
                    id: AssetId(asset_location.clone()),
                    fun: Fungibility::Fungible(*amount),
                }])),
                ClearOrigin,
                BuyExecution {
                    fees: Asset {
                        id: AssetId(asset_location.clone()),
                        fun: Fungibility::Fungible(1),
                    },
                    weight_limit: WeightLimit::Unlimited,
                },
                DepositAsset {
                    assets: crate::asset_hub_runtime::runtime_types::staging_xcm::v5::asset::AssetFilter::Wild(WildAsset::AllCounted(1)),
                    beneficiary: Location {
                        parents: 0,
                        interior: Junctions::X1([Junction::AccountKey20 {
                            network: None,
                            key: *beneficiary_address,
                        }]),
                    },
                },
                SetTopic(*topic),
            ]),
        });
        all_instructions.push(SetTopic(*topic));
    }

    let asset_hub_xcm =
        crate::asset_hub_runtime::RuntimeCall::PolkadotXcm(pallet_xcm::pallet::Call::send {
            dest: Box::new(VersionedLocation::V5(Location {
                parents: 1,
                interior: Junctions::X1([Junction::Parachain(crate::constants::BRIDGE_HUB_ID)]),
            })),
            message: Box::new(VersionedXcm::V5(Xcm(all_instructions))),
        });
    asset_hub_xcm
}

/// Mint refund for a failed Hydration→Ethereum USDT transfer (Feb 2026).
///
/// Transaction: 0xcce3ccdd216ad59c2c602987fe7e8e77ab68dbe83a4555dff630ea346a512c2a
/// The transfer failed on BridgeHub and the user's USDT was burnt on AssetHub.
/// This sends XCM from BridgeHub→AssetHub with ReserveAssetDeposited to mint
/// the USDT back to the beneficiary.
pub fn mint_feb_2026_xcm() -> BridgeHubRuntimeCall {
    use crate::bridge_hub_runtime::runtime_types::{
        pallet_xcm,
        staging_xcm::v5::{
            asset::{Asset, AssetFilter, AssetId, Assets, Fungibility, WildAsset},
            junction::Junction,
            junctions::Junctions,
            location::Location,
            Instruction::*,
            Xcm,
        },
        xcm::{v3::WeightLimit, VersionedLocation, VersionedXcm},
    };
    use hex_literal::hex;

    // USDT (Tether) ERC20 on Ethereum Mainnet
    let usdt_address: [u8; 20] = hex!("dac17f958d2ee523a2206206994597c13d831ec7");
    // 499.739459 USDT (6 decimals) - exact on-chain amount from extrinsic 11369277-3
    let usdt_amount: u128 = 499_739_459;

    // Beneficiary: 16AQJHpSRMh5X1mULm4dCgYxrQLsrnK3uwCQ436iitYk1ru7 (Hydration sender)
    let beneficiary: [u8; 32] =
        hex!("e458cde73940bd29d637face28de378dfb933f21734cfb1d24a0edfb4b81f31c");

    // USDT location: Ethereum ERC20 foreign asset
    let usdt_location = Location {
        parents: 2,
        interior: Junctions::X2([
            Junction::GlobalConsensus(
                crate::bridge_hub_runtime::runtime_types::staging_xcm::v5::junction::NetworkId::Ethereum {
                    chain_id: crate::bridge_hub_runtime::CHAIN_ID,
                },
            ),
            Junction::AccountKey20 {
                network: None,
                key: usdt_address,
            },
        ]),
    };

    // XCM message from BridgeHub to AssetHub:
    // BridgeHub is the reserve for Ethereum-bridged assets, so ReserveAssetDeposited
    // will mint the USDT on AssetHub.
    let instructions = vec![
        ReserveAssetDeposited(Assets(vec![Asset {
            id: AssetId(usdt_location.clone()),
            fun: Fungibility::Fungible(usdt_amount),
        }])),
        ClearOrigin,
        BuyExecution {
            fees: Asset {
                id: AssetId(usdt_location),
                fun: Fungibility::Fungible(usdt_amount),
            },
            weight_limit: WeightLimit::Unlimited,
        },
        DepositAsset {
            assets: AssetFilter::Wild(WildAsset::AllCounted(1)),
            beneficiary: Location {
                parents: 0,
                interior: Junctions::X1([Junction::AccountId32 {
                    network: None,
                    id: beneficiary,
                }]),
            },
        },
        SetTopic(hex!(
            "cce3ccdd216ad59c2c602987fe7e8e77ab68dbe83a4555dff630ea346a512c2a"
        )),
    ];

    // BridgeHub sends XCM to AssetHub via pallet_xcm::send
    BridgeHubRuntimeCall::PolkadotXcm(pallet_xcm::pallet::Call::send {
        dest: Box::new(VersionedLocation::V5(Location {
            parents: 1,
            interior: Junctions::X1([Junction::Parachain(crate::constants::ASSET_HUB_ID)]),
        })),
        message: Box::new(VersionedXcm::V5(Xcm(instructions))),
    })
}

#[cfg(feature = "polkadot")]
pub fn frequency_token_registrations() -> Vec<BridgeHubRuntimeCall> {
    use crate::bridge_hub_runtime::runtime_types::{
        staging_xcm::v5::{junction::Junction::*, junctions::Junctions::*, location::Location},
        xcm::VersionedLocation,
    };
    return vec![register_polkadot_native_asset(
        VersionedLocation::V5(Location {
            parents: 1,
            interior: X1([Parachain(2091)]),
        }),
        "Frequency",
        "FRQCY",
        8u8,
    )];
}
