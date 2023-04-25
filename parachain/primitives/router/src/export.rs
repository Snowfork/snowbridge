use codec::{Decode, Encode};
use frame_support::{ensure, traits::Get};
use snowbridge_core::{ParaId, SubmitMessage};
use sp_core::{RuntimeDebug, H160};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::v3::prelude::*;
use xcm_executor::traits::ExportXcm;

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum OutboundPayload {
    NativeTokensOutbound(NativeTokensOutboundPayload),
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum NativeTokensOutboundPayload {
    Unlock {
        address: H160,
        recipient: H160,
        amount: u128,
    },
}

#[derive(RuntimeDebug)]
enum ParseError {
    XcmMessageTooShort,
    TargetFeeExpected,
    BuyExecutionExpected,
    EndOfXcmMessageExpected,
    ReserveAssetDepositedExpected,
    NoReserveAssets,
    FilterDoesNotConsumeAllReservedAssets,
}

#[derive(RuntimeDebug)]
struct ParseInfo {
    assets: MultiAssets,
    beneficiary: MultiLocation,
    max_target_fee: Option<MultiAsset>,
}

fn parse_xcm(message: &Xcm<()>) -> Result<ParseInfo, ParseError> {
    use ParseError::*;

    let mut it = message.iter();
    let mut next_token = || it.next().ok_or(XcmMessageTooShort);

    // Get target fees if specified.
    let max_target_fee = match next_token()? {
        WithdrawAsset(fee_asset) => match next_token()? {
            BuyExecution {
                fees: execution_fee,
                weight_limit: Unlimited,
            } if fee_asset.len() == 1 && fee_asset.contains(execution_fee) => Some(execution_fee),
            _ => return Err(BuyExecutionExpected),
        },
        UnpaidExecution {
            check_origin: None,
            weight_limit: Unlimited,
        } => None,
        _ => return Err(TargetFeeExpected),
    };

    // Get deposit reserved asset
    let (assets, beneficiary) = if let ReserveAssetDeposited(reserved_assets) = next_token()? {
        if reserved_assets.len() == 0 {
            return Err(NoReserveAssets);
        }
        if let (
            ClearOrigin,
            DepositAsset {
                assets,
                beneficiary,
            },
        ) = (next_token()?, next_token()?)
        {
            if reserved_assets
                .inner()
                .iter()
                .any(|asset| !assets.matches(asset))
            {
                return Err(FilterDoesNotConsumeAllReservedAssets);
            }
            (reserved_assets, beneficiary)
        } else {
            return Err(ReserveAssetDepositedExpected);
        }
    } else {
        return Err(ReserveAssetDepositedExpected);
    };

    if next_token().is_ok() {
        Err(EndOfXcmMessageExpected)
    } else {
        Ok(ParseInfo {
            assets: assets.clone(),
            beneficiary: beneficiary.clone(),
            max_target_fee: max_target_fee.map(|fee| fee.clone()),
        })
    }
}

fn validate_and_encode(parse_info: &ParseInfo) -> Result<Vec<u8>, ()> {
	// We do not support target fees yet
	parse_info.max_target_fee.is_none();
	// We only support a single asset at a time.
	parse_info.assets.len() == 1;
	// We only support ethereum native assets.
    Ok(vec![])
}

#[derive(Encode, Decode)]
struct BridgeMessage(ParaId, u16, Vec<u8>);

pub struct ToBridgeEthereumBlobExporter<RelayNetwork, BridgedNetwork, Submitter>(
    PhantomData<(RelayNetwork, BridgedNetwork, Submitter)>,
);
impl<RelayNetwork: Get<NetworkId>, BridgedNetwork: Get<NetworkId>, Submitter: SubmitMessage>
    ExportXcm for ToBridgeEthereumBlobExporter<RelayNetwork, BridgedNetwork, Submitter>
{
    type Ticket = (Vec<u8>, XcmHash);

    fn validate(
        network: NetworkId,
        _channel: u32,
        universal_source: &mut Option<InteriorMultiLocation>,
        destination: &mut Option<InteriorMultiLocation>,
        message: &mut Option<Xcm<()>>,
    ) -> SendResult<Self::Ticket> {
        let bridged_network = BridgedNetwork::get();
        ensure!(&network == &bridged_network, SendError::NotApplicable);

        let dest = destination.take().ok_or(SendError::MissingArgument)?;
		if let Err((dest, _)) = dest.pushed_front_with(GlobalConsensus(bridged_network)) {
			*destination = Some(dest);
			return Err(SendError::NotApplicable);
		};

        let (local_net, local_sub) = universal_source
            .take()
            .ok_or(SendError::MissingArgument)?
            .split_global()
            .map_err(|()| SendError::Unroutable)?;

        ensure!(local_net == RelayNetwork::get(), SendError::NotApplicable);
        let para_id = match local_sub {
            X1(Parachain(para_id)) => para_id,
            _ => return Err(SendError::MissingArgument),
        };

        let message = message.take().ok_or(SendError::MissingArgument)?;

        let parse_info = parse_xcm(&message).map_err(|_| {
            //TODO: Log
            SendError::Unroutable
        })?;

		let encoded_payload = validate_and_encode(&parse_info).map_err(|_| {
            //TODO: Log
			SendError::Unroutable
		})?;

        //TODO: Log info and trace
        let blob = BridgeMessage(para_id.into(), 0, encoded_payload).encode();
        let hash: [u8; 32] = sp_io::hashing::blake2_256(blob.as_slice());

        // TODO: Fees if any currently returning empty multi assets as cost
        Ok(((blob, hash), MultiAssets::default()))
    }

    fn deliver((blob, hash): (Vec<u8>, XcmHash)) -> Result<XcmHash, SendError> {
        let mut blob = blob.clone();
        let mut input: &[u8] = blob.as_mut();
        let BridgeMessage(source_id, handler, payload) = BridgeMessage::decode(&mut input)
            .map_err(|_| {
                // TODO: Log original error
                SendError::NotApplicable
            })?;
        Submitter::submit(&source_id, handler, payload.as_ref()).map_err(|_| {
            // TODO: Log original error
            SendError::Unroutable
        })?;
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use frame_support::parameter_types;

    use super::*;

    parameter_types! {
        pub const RelayNetwork: NetworkId = Polkadot;
        pub const BridgedNetwork: NetworkId =  Ethereum{ chain_id: 1 };
    }

    struct MockSubmitter;
    impl SubmitMessage for MockSubmitter {
        fn submit(
            _source_id: &snowbridge_core::ParaId,
            _handler: u16,
            _payload: &[u8],
        ) -> sp_runtime::DispatchResult {
            Ok(())
        }
    }

    #[test]
    fn exporter_with_unknown_network_yields_not_applicable() {
        let network = Ethereum { chain_id: 1337 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = None;
        let mut destination: Option<InteriorMultiLocation> = None;
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::NotApplicable));
    }

    #[test]
    fn exporter_with_invalid_destination_yields_missing_argument() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = None;
        let mut destination: Option<InteriorMultiLocation> = None;
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::MissingArgument));
    }

    #[test]
    fn exporter_with_x8_destination_yields_not_applicable() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = None;
        let mut destination: Option<InteriorMultiLocation> = Some(X8(
            OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild,
        ));
        let mut message: Option<Xcm<()>> = None;

        let expected_destination = destination.clone();
        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::NotApplicable));
        assert_eq!(destination, expected_destination);
    }

    #[test]
    fn exporter_without_universal_source_yields_missing_argument() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = None;
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::MissingArgument));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_without_global_universal_location_yields_unroutable() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = Here.into();
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::Unroutable));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_with_remote_universal_source_yields_not_applicable() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> =
            Some(X2(GlobalConsensus(Kusama), Parachain(1000)));
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::NotApplicable));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_without_para_id_in_source_yields_missing_argument() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> =
            Some(X1(GlobalConsensus(Polkadot)));
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::MissingArgument));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_complex_para_id_in_source_yields_missing_argument() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> = Some(X3(
            GlobalConsensus(Polkadot),
            Parachain(1000),
            PalletInstance(12),
        ));
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::MissingArgument));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_without_xcm_message_yields_missing_argument() {
        let network = Ethereum { chain_id: 1 };
        let channel: u32 = 0;
        let mut universal_source: Option<InteriorMultiLocation> =
            Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));
        let mut destination: Option<InteriorMultiLocation> = Here.into();
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::MissingArgument));
        assert_eq!(destination, None);
    }

    #[test]
    fn exporter_test() {
        let network = Ethereum { chain_id: 1 };
        let mut destination: Option<InteriorMultiLocation> = Here.into();

        let mut universal_source: Option<InteriorMultiLocation> =
            Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

        let channel: u32 = 0;
        let mut message: Option<Xcm<()>> = None;

        let result =
            ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
                network,
                channel,
                &mut universal_source,
                &mut destination,
                &mut message,
            );
        assert_eq!(result, Err(SendError::ExceedsMaxMessageSize));
        assert_eq!(destination, None);
    }
}
