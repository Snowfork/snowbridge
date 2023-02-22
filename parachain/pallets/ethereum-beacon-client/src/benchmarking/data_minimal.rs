// Generated, do not edit!
// To generate, run from snowbridge dir:
// clear && mage -d relayer build && relayer/build/snowbridge-relay generate-beacon-benchmark-data
use frame_support::traits::Get;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	Attestation, AttestationData, BeaconBlock, BeaconHeader, BlockUpdate, Body, Checkpoint,
	Eth1Data, ExecutionPayload, FinalizedHeaderUpdate, InitialSync, PublicKey, SyncAggregate,
	SyncCommittee, SyncCommitteePeriodUpdate,
};
use sp_core::U256;
use sp_std::vec;

pub fn initial_sync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>>(
) -> InitialSync<SyncCommitteeSize, ProofSize> {
	let time_now = 1675679352; //2023.2.6

	return InitialSync{
        header: BeaconHeader{
            slot: 4320,
            proposer_index: 6,
            parent_root: hex!("2517b387eb00483e3e0902b3248e06dd842de4933321c3e2736754bc7ce363ff").into(),
            state_root: hex!("31608b3b90081fbdab866fb4749c0d1e217140a2cd467ff4f1d49ce23d295989").into(),
            body_root: hex!("4d03e2b189d4a6c391326316880688febed441b02c469803adf8c26a59cbad87").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: vec![
                    PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                    PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                    PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                    PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                    PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                    PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                    PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                    PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                    PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                    PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                    PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                    PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                    PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                    PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                    PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                    PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                    PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                    PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                    PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                    PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                    PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                    PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                    PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                    PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                    PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                    PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                    PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                    PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                    PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                    PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                    PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                    PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        current_sync_committee_branch: vec![
                hex!("7f2e568b4f4aa2f1557744a7f0c441c5308defcaeaff620e9ab6c89ee16cc7ce").into(),
                hex!("c4b77c52210edef1cf2887107baf44dd040294384ddee23de95a4a819d0f2ec5").into(),
                hex!("5a8effe0486dfe88ad677ad6774cb58619ee4ea444c3a09a1249dadea4bbd6b8").into(),
                hex!("4e5574309f87838b497e456d0f7510373a24307fe6152cd55b05dfa5d39ae68b").into(),
                hex!("e969d71ce6bb68887c205c0f2c0e84e95fbc70911585b63abfe45ce472eea63e").into(),
        ].try_into().expect("too many branch proof items"),
        validators_root: hex!("270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69").into(),
        import_time: time_now + 97200, // now + 27 hour sync committee period
    };
}

pub fn sync_committee_update<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> SyncCommitteePeriodUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
	return SyncCommitteePeriodUpdate {
        attested_header: BeaconHeader {
            slot: 4304,
            proposer_index: 6,
            parent_root: hex!("b12738c16673201010c878617ef41897ff473ecc1142c4cd550886248722534b").into(),
            state_root: hex!("09c8840f2e4d343419795013eb206b9fb40ab8701bbfb1daa6fcc3d7b2021b4b").into(),
            body_root: hex!("07eac4f9731ae365574c99252dec1a45534cce283aba95d5485a59fb2c5ff2f1").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        next_sync_committee_branch: vec![
            hex!("7ba44032b68620539b1bac45e5202dd530af5f6b669a5a496ba0fcfb3f0b8da3").into(),
            hex!("9cfee9bd3c56d615fef406b18ba9df9c445f1054598de268bed7f626db2b88d8").into(),
            hex!("b889b0ccf1097246980be855d5be5c1ad1defd5753576c4c3ebc24d2aae3d6f3").into(),
            hex!("a7c60c441e03acbe5642c3bb27068a301cf1273e5e44c44ac1f6adb557ac89c1").into(),
            hex!("77f15a909912fa009f44e959eb8e858565b0c113fb3fb726909e9b1275e7a71c").into(),
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 4288,
            proposer_index: 6,
            parent_root: hex!("f9957df8c463f67e8a0aa3e3dcd06dafe595edc5563de2ac3a0035cecbb9fbd6").into(),
            state_root: hex!("aeb704dd0c20e01d29de3110b6802737ec80edcbd6312bf86340fcb727e013b8").into(),
            body_root: hex!("c1bedcf935e56f7a0b7aea1434b657ec070fb57bcc394961239d20581691b7be").into(),
        },
        finality_branch: vec![
            hex!("1802000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("0387a805c263663787e72f37d5ef283a8cbcdd550b5f7cc836867891f91307cf").into(),
            hex!("b889b0ccf1097246980be855d5be5c1ad1defd5753576c4c3ebc24d2aae3d6f3").into(),
            hex!("a7c60c441e03acbe5642c3bb27068a301cf1273e5e44c44ac1f6adb557ac89c1").into(),
            hex!("77f15a909912fa009f44e959eb8e858565b0c113fb3fb726909e9b1275e7a71c").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a698ffb9b168f27371aee2c77f30308e9475d46173c317988f99988a4badb404b90c17a9383de66e3b2ac284fc74654e066b2b098dbb4bd3f84e8bbab2bc91c61c39a261803fc20b44886852a85dec58fb65da7da6753acfb7a304f52638cb92").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 67,
        signature_slot: 4305,
        block_roots_hash: hex!("f5e05e6ce8326908b7e16cdb0ec4dca94a3ca7a99b76f5cf5b8ae28d62dce343").into(),
        block_roots_proof: vec![
            hex!("6edd89a3e95d01e2e1754d1d8ce43c3de074da7a8be326f701ed3dbb6101a3bd").into(),
            hex!("071121eea09b532393679d8451a5e986b1de736066c1cc96e07e5aa20d83e841").into(),
            hex!("4602a1ba0d287bb5045a2f5c13a622bea5461feb4efa3e608cdaa0a92a6dea3c").into(),
            hex!("45d2c1a649dbf52a46a6ad761cbc5afc6b3f98b140ab0baf24f12f621a4d0c33").into(),
            hex!("5d4969dc5d385bd6fc782ec6ddae76d55699a54a2680a1af633a218712753618").into(),
        ].try_into().expect("too many branch proof items"),
    };
}

pub fn finalized_header_update<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> FinalizedHeaderUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
	return FinalizedHeaderUpdate{
        attested_header: BeaconHeader {
            slot: 4336,
            proposer_index: 4,
            parent_root: hex!("310d780d2298529d1f8b394c4609c46389c588cf7be1cfa5b92bb3688cc9da3f").into(),
            state_root: hex!("03f2a208a7ff68f05065dbabf150f7e5928141094b58aabb9deda700d16814d8").into(),
            body_root: hex!("195c285389416eb633551efef0143e658405876d074398f4aa56e9efd98ae608").into(),
        },
        finalized_header: BeaconHeader{
            slot: 4320,
            proposer_index: 6,
            parent_root: hex!("2517b387eb00483e3e0902b3248e06dd842de4933321c3e2736754bc7ce363ff").into(),
            state_root: hex!("31608b3b90081fbdab866fb4749c0d1e217140a2cd467ff4f1d49ce23d295989").into(),
            body_root: hex!("4d03e2b189d4a6c391326316880688febed441b02c469803adf8c26a59cbad87").into(),
        },
        finality_branch: vec![
            hex!("1c02000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("0387a805c263663787e72f37d5ef283a8cbcdd550b5f7cc836867891f91307cf").into(),
            hex!("9a38891ce8f1ecb32a63e9a65f4046c089d9cbc4d999524c85ef2a161b470554").into(),
            hex!("301445f0efe5e8bb451314b211d87193bb4c4efc328b48c8556a9c60054d0127").into(),
            hex!("44e8665fd8760c9ea45b013021c394cd40ea6ba02cd31d9f8b53918491c7601e").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("94894b5dbf329bf8eb6a6f4535c5d7fc14a13e276e2030847199cc60af8c2b440740ec0b89557d7551173c39b736e6df0dbd240798d75771349c4411e1becb6b0922cf85df2c1f7ee3ffbd4fe07ced430a66d29a7de658c8a62651fb6839b609").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 4337,
        block_roots_hash: hex!("5d28daf85f8c42979ab9a3bfd7db6f29ef0692b27c2b6d1bf73fe2c746dc028c").into(),
        block_roots_proof: vec![
            hex!("38d682cf7559159f6b103902173efafa4790e5dbb42ea784c48434072cfdec2f").into(),
            hex!("2b9e307f79e9f53bf000cf3bf53f32e5a3cee06810083aec3d2aacb5f4bf942d").into(),
            hex!("a07140d8dc510852d8b992a2791077718f6eaeb364b87fdcb9cef50316dd54c6").into(),
            hex!("0f8003a78beef60e0ed9fd5b4baea6dd130a1253235fc31e1f4eeebf1bcf019a").into(),
            hex!("068351f45cb391dde74b994b64982385adbd352d134b7876a59750d28e52dd64").into(),
        ].try_into().expect("too many branch proof items")
    };
}

pub fn block_update<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
	DepositDataSize: Get<u32>,
	PublicKeySize: Get<u32>,
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	ProposerSlashingSize: Get<u32>,
	AttesterSlashingSize: Get<u32>,
	VoluntaryExitSize: Get<u32>,
	AttestationSize: Get<u32>,
	ValidatorCommitteeSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> BlockUpdate<
	FeeRecipientSize,
	LogsBloomSize,
	ExtraDataSize,
	DepositDataSize,
	PublicKeySize,
	SignatureSize,
	ProofSize,
	ProposerSlashingSize,
	AttesterSlashingSize,
	VoluntaryExitSize,
	AttestationSize,
	ValidatorCommitteeSize,
	SyncCommitteeSize,
> {
	return BlockUpdate{
        block: BeaconBlock{
            slot: 4318,
            proposer_index: 5,
            parent_root: hex!("6f054ea7ce689df194425798937714e1a35cfffe954b75efcad81db32972ae17").into(),
            state_root: hex!("75915ac3f37ba189576f2f3cf3ca3e9763244aa29f1173d188b494bee9722ade").into(),
            body: Body{
                randao_reveal: hex!("89dcaa27c0b62d269621e26e03860cf1f5da5866757502d6c4556739f42d6a6824bcb201fdf24b260b1196fa217935470e7182c7c53fadebddb8fe3379b9f5f855b9f2e30f98b574ae77457b3cae68a7d43c535aa98ee8bc34d3a9faa8ce0aee").to_vec().try_into().expect("randao reveal too long"),
                eth1_data: Eth1Data{
                    deposit_root: hex!("6a0f9d6cb0868daa22c365563bb113b05f7568ef9ee65fdfeb49a319eaf708cf").into(),
                    deposit_count: 8,
                    block_hash: hex!("6f13d8e3bc84f30d3cd546587cde300b4e60890e5ee14c9529fd27b83cf09361").into(),
                },
                graffiti: hex!("4c6f6465737461722d76312e342e322f636c6172612f534e4f2d3338392d6164").into(),
                proposer_slashings: vec![
                ].try_into().expect("too many proposer slashings"),
                attester_slashings: vec![
                ].try_into().expect("too many attester slashings"),
                attestations: vec![
                    Attestation{
                        aggregation_bits: hex!("03").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 4317,
                            index: 0,
                            beacon_block_root: hex!("6f054ea7ce689df194425798937714e1a35cfffe954b75efcad81db32972ae17").into(),
                            source: Checkpoint{
                                epoch: 538,
                                root: hex!("d81b7781246ffd772f08294ae6d593399a903b059ec770dd176f00f45b3d7be2").into()
                            },
                            target: Checkpoint{
                                epoch: 539,
                                root: hex!("8241bc63edb3d425ce6d175513106f002d5f4f603ae7cb1cd25fffd24584f15f").into()
                            },
                        },
                        signature: hex!("b4d0e769388742a365bc59f820a336af9e87883285a659a5ec6321892e559ddada929f4d583bdb7efd9eca26523b39d7074be1ee9aa6da56506a89aad4e0c8d69f5c38184eac31885236bb517510132881579304fe6eee2b510656c195ef80bc").to_vec().try_into().expect("signature too long"),
                    },
                ].try_into().expect("too many attestations"),
                deposits: vec![
                ].try_into().expect("too many deposits"),
                voluntary_exits:vec![
                ].try_into().expect("too many voluntary exits"),
                sync_aggregate: SyncAggregate{
                    sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
                    sync_committee_signature: hex!("b2aa795ca0140510eee21279d9bbc3695853c62906d23e36669e4d2d989846f70740e22c976dac8f4811be8b9e46bb5a062d6aebc46c884a4606b768263f12a8e1b7d6b1b6c129baafed3a508260d7bf2a0da51e83ee57e25e7e7b25d1856fd7").to_vec().try_into().expect("signature too long"),
                },
                execution_payload: ExecutionPayload{
                    parent_hash: hex!("48516d7969e010750fcfeb96205abfc1409bc58d7be546135bfebddd01c18751").into(),
                    fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
                    state_root: hex!("2c988c2df2d94e57532ff5917134efc06d701b13209ab1f3584d9134ff162577").into(),
                    receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
                    logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec().try_into().expect("logs bloom too long"),
                    prev_randao: hex!("9e4f90822e09fc0089eeaad4727b04af7fe86a291ba86f82f6c8bf234a27c1f9").into(),
                    block_number: 4318,
                    gas_limit: 30000000,
                    gas_used: 0,
                    timestamp: 1676898944,
                    extra_data: hex!("").to_vec().try_into().expect("extra data too long"),
                    base_fee_per_gas: U256::from(7 as u32),
                    block_hash: hex!("76e697cb4e9e9e221721cbe0e7ba3bbe5333a2751745101da1412c2b96ea9567").into(),
                    transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
                }
            }
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8d89b3d6d0e9a2fc3ba6485651df62b06c1a2673f5a8f32deb87e32f5738f3a04999f738f15ab72d30d9ce7126cc82d90b23245f48e36667ccd5534773a0d84e6f62780b60819d4ea8d7f19ed279f13f68b3dff309da065c134a7a0c6b3991dc").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 4319,
        block_root_proof: vec![
            hex!("2517b387eb00483e3e0902b3248e06dd842de4933321c3e2736754bc7ce363ff").into(),
            hex!("f9d364c4caaee03c5bf1234cec7c55186a67ac9725db0974d9de64f13f0632a1").into(),
            hex!("42108a785a9b551309e6e745417a481e7a20eb7491bb0dd3195d731c1469e5ce").into(),
            hex!("2cab4643850ff635a77616f1c71f4bce9965e2d1531d87e1ffebf782185ce0b6").into(),
            hex!("9fcfea3a14016a5670332d5f79db08f7664a16fadda531dc5c74d426472b86e6").into(),
            hex!("bc2d284650afe3369675e9d0c2d9171b79b3f92e1c76f36fede13dcb22d0701d").into(),
        ].try_into().expect("too many branch proof items"),
        block_root_proof_finalized_header: hex!("52134169df12aab5540f07cb316e1c91ebbf12df83800e11c5a325fc85fc8728").into(),
    };
}
