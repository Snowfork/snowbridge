// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use frame_benchmarking::v2::*;
use crate::Pallet as InboundQueue;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use snowbridge_core::Proof;
    use super::*;
    use hex_literal::hex;
    //use crate::benchmarking::benchmarks::OUTBOUND_QUEUE_ADDRESS;

    //const OUTBOUND_QUEUE_ADDRESS: [u8; 20] = hex!["87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d"];

    #[benchmark]
    fn submit() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        let payload = hex!("f9011c94ee9170abfbf9421ad6dd07f6bdec9d89f2b581e0f863a01b11dcf133cc240f682dab2d3a8e4cd35c5da8c9cf99adac4336f8512584c5ada000000000000000000000000000000000000000000000000000000000000003e8a00000000000000000000000000000000000000000000000000000000000000001b8a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004b000f000000000000000100d184c103f7acc340847eee82a0b909e3358bc28d440edffa1352b13227e8ee646f3ea37456dec701345772617070656420457468657210574554481235003511000000000000000000000000000000000000000000");

        let message = Message {
            data: payload.into(),
            proof: Proof {
                block_hash: Default::default(),
                tx_index: Default::default(),
                data: Default::default(),
            },
        };

        #[block]
        {
            let _ = InboundQueue::<T>::submit(RawOrigin::Signed(caller.clone()).into(), message);
        }

        Ok(())
    }

    impl_benchmark_test_suite!(InboundQueue, crate::test::new_tester(Default::default()), crate::test::Test,);
}
