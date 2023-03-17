// Generated, do not edit!
// See README.md for instructions to generate
use frame_support::traits::Get;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	BeaconHeader, ExecutionPayload, FinalizedHeaderUpdate, HeaderUpdate, InitialSync, PublicKey,
	SyncAggregate, SyncCommittee, SyncCommitteePeriodUpdate, VersionedExecutionPayload,
};
use sp_core::U256;
use sp_std::vec;

pub fn initial_sync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>>(
) -> InitialSync<SyncCommitteeSize, ProofSize> {
	let time_now = 1675679352; //2023.2.6

	return InitialSync{
        header: BeaconHeader{
            slot: 200,
            proposer_index: 2,
            parent_root: hex!("ab567bdc9d0d0ee3aff95dc66511af7351d91b02bda2c3f7a6c4192e6075f976").into(),
            state_root: hex!("73cb8a053c734d92da7ba6eed9210bb099ac56e74c7c5a7269310898b80226be").into(),
            body_root: hex!("5c2029e17e0e074502f9e81b37610f704cc0b6a803c99ca66eba5a1332c72cae").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: vec![
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        current_sync_committee_branch: vec![
                hex!("6e3b291c7aa80c720a25e601cdb538dda7155fe7330d2b5aa1f33401c63cf8b9").into(),
                hex!("c6b55d6b09cfa95f704029a868cbd0cf134eb760f320405752081ef4fa4c18e9").into(),
                hex!("04b2bc5912e7c803a7ac48ccbdaa5533d424a14ca873aaf87145fd211d28ff7e").into(),
                hex!("4658e5975fd9c7ff18ce8202289d4114fde3a7d985eaec2a281e7fadedd67eac").into(),
                hex!("f230f50ffcb833561d6732695d178f80f0bd99ec5b9b26ceae253825f7699ced").into(),
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
            slot: 208,
            proposer_index: 4,
            parent_root: hex!("b316179f69cf2be27ee7044df0b9f44b792deb91c7664c939956d0d017db421e").into(),
            state_root: hex!("d575509adcd7a596da7ce306eaa41e04b9699ee1f26f7dd2fc44c17b7d84f96f").into(),
            body_root: hex!("0d2d1b55270bce3232c56afe54b679a625d73eb1dd310a92b266d24d54b38125").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        next_sync_committee_branch: vec![
            hex!("c4f39befd9b88e6e3f73e13c1d97cddbc6d6d7a57301f2f8de8c78894ba2947b").into(),
            hex!("db604d9db0f3ece524ccc55294062dd538efe1674bf6d1c6725bad70127eec35").into(),
            hex!("7f3b6f9d1d413e91fdc814ed8a1448aeb3934cc98d499496a80c50bd1ba5d86b").into(),
            hex!("34181f50415dc2af5efeffae79cbb99316b5efcf5fdae5f07267c6db5cda1ad3").into(),
            hex!("15a182f6a09df7ee2c91003d0077de5150878ade3fb802d3a07100c95fe7b328").into(),
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 192,
            proposer_index: 5,
            parent_root: hex!("497c585329b632e1b143b637a4d50332fe64537009716274aac718e3e1f71167").into(),
            state_root: hex!("a167e61746e5b7e03270e7d436c17bf122d72f58d29e21f7fadafcd47fa55271").into(),
            body_root: hex!("771b0681987c94ad90048c6c140c877b0c82b9c93dc2465ac9533b4fbe4bb7f2").into(),
        },
        finality_branch: vec![
            hex!("1800000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("2eba56447b4d9f2467804b7d9165bfc03934bb98f3a4a09f7d6cfd7bb2c08c55").into(),
            hex!("7f3b6f9d1d413e91fdc814ed8a1448aeb3934cc98d499496a80c50bd1ba5d86b").into(),
            hex!("34181f50415dc2af5efeffae79cbb99316b5efcf5fdae5f07267c6db5cda1ad3").into(),
            hex!("15a182f6a09df7ee2c91003d0077de5150878ade3fb802d3a07100c95fe7b328").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a9fc120c6294911bf03c73faf81dd8e0f0f8ae1f93f86d532e8cbffaa01379f43314e6f339f721d9b261837946a17e060f8e318ce81942a79107c988ec0ae7867f22ee7d1baf9b982d3499f4bf87cbf6ef6fd2362a7711d10dd193ca637e33b9").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 3,
        signature_slot: 209,
        block_roots_root: hex!("f4d7598c249fba7c65f26bc2ddae4fef09a4b200fc4d91143c7d0b38e99ff376").into(),
        block_roots_branch: vec![
            hex!("f0772725d68f543e36e2cd0b07c88be5a799ddbc7672ab59900d4ab296c67197").into(),
            hex!("9b0dae7ad28ddb6f78514e711527a477cc2eacbef755a92ee6e5add41403888a").into(),
            hex!("77128194df217d1a536785d1004dd64bbcf24f06e4b28a010b5acca487c7e252").into(),
            hex!("24e95cd68d21c97fce75ed53754a633c53583b4902b90d055d4b09cb3ed2d3aa").into(),
            hex!("9a97ae266eb3bcc182faad3724913b69dba4c838928f56d1a72710691a661b64").into(),
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
            slot: 216,
            proposer_index: 1,
            parent_root: hex!("4020995a48095741a25a7f1e6d584a82fd612f64859dca246bc9b3f6581d12aa").into(),
            state_root: hex!("bc32a12c080feed42674fb2266ef6630f21a3bb27d2e3eeeb8b9a805eaae300b").into(),
            body_root: hex!("bb159dd6d107afb139e1e8ed51c9a832ea90b6c42528d120ff5ba4bcadc634cd").into(),
        },
        finalized_header: BeaconHeader{
            slot: 200,
            proposer_index: 2,
            parent_root: hex!("ab567bdc9d0d0ee3aff95dc66511af7351d91b02bda2c3f7a6c4192e6075f976").into(),
            state_root: hex!("73cb8a053c734d92da7ba6eed9210bb099ac56e74c7c5a7269310898b80226be").into(),
            body_root: hex!("5c2029e17e0e074502f9e81b37610f704cc0b6a803c99ca66eba5a1332c72cae").into(),
        },
        finality_branch: vec![
            hex!("1900000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("2eba56447b4d9f2467804b7d9165bfc03934bb98f3a4a09f7d6cfd7bb2c08c55").into(),
            hex!("bfc2809829f9da224e6846933fe0f1addf6ce66702fab8be33e53887cc8242e4").into(),
            hex!("327c4336daa90bf3c6733281f93ef87dd66ca87fc5731509b72926301a1bf4d7").into(),
            hex!("82e90a0a6764bf036a50c170d62a335ec9e16c43f21396dbf2ba4c2e6889ae85").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8951893b2b640e71429656fcd76c32ead866521e28abd543e863e3581f463f067d47841a61fb007390d5656ab6c8719803d09bcc6b8e7d5e9037527a130284ca12cd6b57adadf845dadee0df3b94189de19f2026347688f63ac8b3ef3d8c5353").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 217,
        block_roots_root: hex!("cad4079987df885cf6abfc9a9b9f0246a5b3c1f0482984e0a56ef45c7d61d0f8").into(),
        block_roots_branch: vec![
            hex!("5083c8e8187533037a9651ca553b84c95bb0926526a63d0a998d3e224e1fe73a").into(),
            hex!("f4f97a1ce9eab3921d12a717a03e0be24653526fff7ae56a431ed149aba54efc").into(),
            hex!("2f4117e62050ee570b92bbe8fe8a4698225dea683287e1ab74bca230da2be698").into(),
            hex!("ea18f0e7e613d0e6ba96f2220331878c67069901d6a4aefc6c67ee195fc2c9b4").into(),
            hex!("56c631e11f081d15d8ef709720942e26469c2d5681211a632f3b3adfa59b4f2e").into(),
        ].try_into().expect("too many branch proof items")
    };
}

pub fn header_update<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> HeaderUpdate<
	FeeRecipientSize,
	LogsBloomSize,
	ExtraDataSize,
	SignatureSize,
	ProofSize,
	SyncCommitteeSize,
> {
	return HeaderUpdate{
        beacon_header: BeaconHeader{
            slot: 198,
            proposer_index: 3,
            parent_root: hex!("c1099800778c36b1c3c81aed5e106d284b8d7dd7b595f54f448fb8d3a49842d8").into(),
            state_root: hex!("d4b15f3e012f030600478a2e2f926d2a4bdc11e5a272358774b6ade313741c7c").into(),
            body_root: hex!("f6774295e2367becc1194183039734ba1ad8d635e6cb8bd8e823af268d7e4460").into(),
        },
        execution_header: VersionedExecutionPayload::Bellatrix(ExecutionPayload{
            parent_hash: hex!("86ab41761054378f348183cb6c35d9775c3c9975c0139bca8d67661f228bfba3").into(),
            fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
            state_root: hex!("06f05ca47fff28a16c76564d8f2af2f8469dc64719bd2ec639300c84325758dc").into(),
            receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec().try_into().expect("logs bloom too long"),
            prev_randao: hex!("eb0207e113d9c93fae653b4b7fc8a73c1a462c66f30c91623791d0e11bccc273").into(),
            block_number: 198,
            gas_limit: 65928898,
            gas_used: 0,
            timestamp: 1678359385,
            extra_data: hex!("").to_vec().try_into().expect("extra data too long"),
            base_fee_per_gas: U256::from(7 as u64),
            block_hash: hex!("1cde65ff8185db267ad62e60940dedc3fc284ae8aa70fb1a23f85f7e3cb42f82").into(),
            transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
        }),
        execution_branch: vec![
            hex!("4e4c59c066d8211a499e5ae32adfab353faff1bc59d779eabe3f4a0ee2e212b7").into(),
            hex!("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b").into(),
            hex!("db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71").into(),
            hex!("6e31b6d6b26f260e9cb14e972ffac5fde67e3193ddcbe9ce756d7b94a4a94560").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("b6921f93d481e1294f47ce11d4a49d9cfd8cf7d57b3031f5f9e3f68180d2519904f0e2cba0d8e9213a2e56061d91136414b3e1ec3a414af1c1c6b0f0c47c0eec644538019990b5413083f2ceeea13a5739a3b0332bf55dc22bbfbc19b662e107").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 199,
        block_root_branch: vec![
            hex!("ab567bdc9d0d0ee3aff95dc66511af7351d91b02bda2c3f7a6c4192e6075f976").into(),
            hex!("d19186b993d9550b1246a00cfaf7c6335347ceb2ce0e329f9554bb56793e27dc").into(),
            hex!("4dc98a6388a37d7a715a812e22b0afbe7d414074b88d1e0b4744566a82b9986e").into(),
            hex!("2ab20473cf89877a4002fcd21745753874f32f5cb33d11f7937e4772c28b8e30").into(),
            hex!("1681bc3367bcb133d24066ca9e46e898ec9bb34e2dac3989ed9b6f1cd6b9b5cf").into(),
            hex!("82900d73e1753f4e82f484de7987f05557720d040dedfbe193180c56754d7210").into(),
        ].try_into().expect("too many branch proof items"),
        block_root_branch_header_root: hex!("485f6e00c2bdede47af56e235ca39ed794a30273d005f0d9c7770cbdb5346da5").into(),
    };
}
