use codec::{Decode, Encode};
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

pub struct ToBridgeEthereumBlobExporter<Submitter, SourceId>(
	PhantomData<Submitter>,
	PhantomData<SourceId>,
);
impl<Submitter: SubmitMessage<SourceId>, SourceId> ExportXcm
	for ToBridgeEthereumBlobExporter<Submitter, SourceId>
{
	type Ticket = (Vec<u8>, XcmHash);

	fn validate(
		_network: NetworkId,
		_channel: u32,
		_universal_source: &mut Option<InteriorMultiLocation>,
		_destination: &mut Option<InteriorMultiLocation>,
		_message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		todo!()
	}

	fn deliver(_ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		todo!()
	}
}
