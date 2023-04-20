use codec::{Decode, Encode};
use frame_support::{ensure, traits::Get};
use snowbridge_core::SubmitMessage;
use sp_core::{RuntimeDebug, H160};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::v3::prelude::*;
use xcm_executor::traits::ExportXcm;

pub enum OutboundPayload {
	NativeTokensOutbound(NativeTokensOutboundPayload),
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum NativeTokensOutboundPayload {
	Unlock { address: H160, recipient: H160, amount: u128 },
}

pub struct ToBridgeEthereumBlobExporter<BridgedNetwork, Submitter>(
	PhantomData<(BridgedNetwork, Submitter)>,
);
impl<BridgedNetwork: Get<NetworkId>, Submitter: SubmitMessage> ExportXcm
	for ToBridgeEthereumBlobExporter<BridgedNetwork, Submitter>
{
	type Ticket = (Vec<u8>, XcmHash);

	fn validate(
		network: NetworkId,
		_channel: u32,
		_universal_source: &mut Option<InteriorMultiLocation>,
		_destination: &mut Option<InteriorMultiLocation>,
		_message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let bridged_network = BridgedNetwork::get();
		ensure!(&network == &bridged_network, SendError::NotApplicable);
		todo!()
	}

	fn deliver(_ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;

	use super::*;

	parameter_types! {
		pub const BridgedNetwork: NetworkId =  Ethereum{ chain_id: 1 };
	}

	struct MockSubmitter;
	impl SubmitMessage for MockSubmitter {
		fn submit(
			_source_id: &snowbridge_core::ParaId,
			_handler: u16,
			_payload: &[u8],
		) -> sp_runtime::DispatchResult {
			return Ok(())
		}
	}

	#[test]
	fn exporter_with_unknown_network_yields_not_applicable() {
		let network = Ethereum { chain_id: 1337 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result = ToBridgeEthereumBlobExporter::<BridgedNetwork, MockSubmitter>::validate(
			network,
			channel,
			&mut universal_source,
			&mut destination,
			&mut message,
		);
		assert_eq!(result, Err(SendError::NotApplicable));
	}
}
