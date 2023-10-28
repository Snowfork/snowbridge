//! Implementation for [`frame_support::traits::ProcessMessage`]
use super::*;
use crate::weights::WeightInfo;
use frame_support::{
	traits::{ProcessMessage, ProcessMessageError},
	weights::WeightMeter,
};
use snowbridge_core::outbound::AggregateMessageOrigin;

impl<T: Config> ProcessMessage for Pallet<T> {
	type Origin = AggregateMessageOrigin;
	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		_: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		let weight = T::WeightInfo::do_process_message();
		if !meter.check_accrue(weight) {
			return Err(ProcessMessageError::Overweight(weight))
		}
		Self::do_process_message(origin, message)
	}
}
