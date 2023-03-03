// Generated, do not edit!
// See README.md for instructions to generate
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
            slot: 216,
            proposer_index: 1,
            parent_root: hex!("4c626195440e400fdc3a30270dadd75fc1678f601cf87687b545d6a1e23ee685").into(),
            state_root: hex!("056e45a3f966785da94d1579018f09650f7063a04227e20b4c2a648b68988d4c").into(),
            body_root: hex!("49001673147a98309084f5dcd0c8ef48500c5c7593159293029376ff0c70a9b1").into(),
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
                hex!("f8d2de71cc4152339c1101aa4c169ac0caa3e63817f3a1c6ab7f2d61f6c926e0").into(),
                hex!("4de05d85c13cdafe23b8f7d0b12dfe6cf1960f0f01b050a99932d544d47577cb").into(),
                hex!("b86c7fefd5aefc50eaca8665680eff4e322db7bc557334df7ec96dd3ffdb5427").into(),
                hex!("ce95ed5e1749c2ca40ead1998f91f8de8d022458d734993a5a31837d3b27f631").into(),
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
            parent_root: hex!("185c4b5b20d1a8aa6e09fbfa34c7e424c14189677e7e35c91f8a12d9f2fc4c54").into(),
            state_root: hex!("7ca712a4e14fd5411e3674696a8d067c3acb3e31715b5c91f67eecd53eda2b75").into(),
            body_root: hex!("53404be3eb579982a31f14381d3740b62d25049baec06cca7f561c81e5d1c208").into(),
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
            hex!("879d6110ad12fbf45a3e0c252fe82b9e7c9233bbfb9d449bdec61d171bd0c730").into(),
            hex!("3e7e43d069ca0f1a0b07ff084845e473f3a27ced2f9d2986fd359f522f283f58").into(),
            hex!("af8b6c2031f4416d94bd15b5b99c00727f7e6bd5dfdbc72562c652737f787086").into(),
            hex!("24b381ed8ec9d704469da41c4a71cd7c7dd556f6e8c337a74501ab659a92094c").into(),
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 192,
            proposer_index: 5,
            parent_root: hex!("2ef59091ad9d08eca5576d9c6326d91c9cd8410e4d1d57642ff506f219366d74").into(),
            state_root: hex!("eb2c6e75b18c515ba0b7c4d28aef63e434dceff60dcc0bba9bdcab5ef713c4cd").into(),
            body_root: hex!("0c2e222c43ca25d489cabb9853bebb5f6e745215c07de9d0f0fb95551234de9f").into(),
        },
        finality_branch: vec![
            hex!("1800000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("2eba56447b4d9f2467804b7d9165bfc03934bb98f3a4a09f7d6cfd7bb2c08c55").into(),
            hex!("3e7e43d069ca0f1a0b07ff084845e473f3a27ced2f9d2986fd359f522f283f58").into(),
            hex!("af8b6c2031f4416d94bd15b5b99c00727f7e6bd5dfdbc72562c652737f787086").into(),
            hex!("24b381ed8ec9d704469da41c4a71cd7c7dd556f6e8c337a74501ab659a92094c").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a3abb787d4dca690e0f59a1d0d1983b667374bc416b987347030b63fb653c017ef3c562cab15a344acf149ade04307ee16c93f2bc52609b23abd6a6e04192ba924fd30fd683498814112617a925ffe2e1184fc13461249c1ff5ef3bbbb985f6d").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 3,
        signature_slot: 209,
        block_roots_hash: hex!("2bb5d4fcb468b19aaf54b2356810aa6f03e043d0c7a641b11133d17d2402c141").into(),
        block_roots_proof: vec![
            hex!("03c399d8a611cbc47280f6e32d38aa0fee0675093e0c7fa0652e1c1c6c8e63ac").into(),
            hex!("71f874d608a29f108677f1aebd164c2d5e76c6490d9c030fb0490b7ef814d055").into(),
            hex!("e0fc1ad228ce5ec0a2183baf4894a301d7463b844716abebef12a1266d58d699").into(),
            hex!("24e95cd68d21c97fce75ed53754a633c53583b4902b90d055d4b09cb3ed2d3aa").into(),
            hex!("84dc394303560ffb659357af762e2efce64ce619c391892831ef5e68093cb321").into(),
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
            slot: 232,
            proposer_index: 5,
            parent_root: hex!("758c08c7e45b3dd372b7c31cba1c65c084883a885fb2932f566167ce8cad92b5").into(),
            state_root: hex!("3c1591e61f48740994d3682cd21ecde60a4a4b42614c8450470d4d2090418cf3").into(),
            body_root: hex!("eefaf1152971f718cb3d0ead944e4142a52da8f5f081e706d5c750deac0d5978").into(),
        },
        finalized_header: BeaconHeader{
            slot: 216,
            proposer_index: 1,
            parent_root: hex!("4c626195440e400fdc3a30270dadd75fc1678f601cf87687b545d6a1e23ee685").into(),
            state_root: hex!("056e45a3f966785da94d1579018f09650f7063a04227e20b4c2a648b68988d4c").into(),
            body_root: hex!("49001673147a98309084f5dcd0c8ef48500c5c7593159293029376ff0c70a9b1").into(),
        },
        finality_branch: vec![
            hex!("1b00000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("2eba56447b4d9f2467804b7d9165bfc03934bb98f3a4a09f7d6cfd7bb2c08c55").into(),
            hex!("fd208630fcb52072dfec1d5fd88e6eb11b1473141a570df0be22643f20aabfab").into(),
            hex!("7a24beb995c072f82522f18f02e9c28049afa7fca5421a67df5f772559767564").into(),
            hex!("d7fec40856dc66b0a1d9bffd5d73aa7e046e2384456730eb374b0cf95fd4490f").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8697e5adf70d45e41ea3f0f3e15607993c2888ec40eb1553e7ec3acbdb386a6acaf2246a4eebfee26a40432eecea214f08155124980009e9dc929ab2ad962295e9aaa1fe9d57ba0c4630d5bc489680eb5122479a1fabc0faf27de2e953547f75").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 233,
        block_roots_hash: hex!("ed9b426a06683ac4c10d95756da5a583371220d9173cfcf29b2b454447372ba9").into(),
        block_roots_proof: vec![
            hex!("5a37e5ea830c9074d4a7055f51791b17f2f894f801bdd72399480c0e4738e893").into(),
            hex!("ed9a8665714d87df3e991aec903ab951545161be0c1eb18173167cb222c75086").into(),
            hex!("562e789ca67052ccbd0e72e8dbd859a5e1eb162160b5882fa7d123a6a8cec43d").into(),
            hex!("e7fe1dd46977d2e03b94b6ce117e30acb1abd8a2097b3f43e94aa12c57d22f26").into(),
            hex!("eb28790335bfe2e6fc2d43e34416cc7a4be33db80e063f86063af6a99126c591").into(),
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
            slot: 214,
            proposer_index: 3,
            parent_root: hex!("04cf9164c9d06c6d7a6d9fa21f0f199b9bf429794d5549ff11fe867f9f3a903f").into(),
            state_root: hex!("bb297d601613d629f33f84019eeb1759b2e1b0d8e262f9a6892e3331f6d22d80").into(),
            body: Body{
                randao_reveal: hex!("819e25760a789e7a2783f8758aacf6953c5c09464c4cee31d7044b81e6b7946987d75210635dcbcc68c4a3e7910cbc611994adad53ec987e4ebcfa17de7a1c2ebd57a322f7048ced98ebf45346e18f1713df6d00d3e592a7392479340d4309b3").to_vec().try_into().expect("randao reveal too long"),
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
                            slot: 213,
                            index: 0,
                            beacon_block_root: hex!("04cf9164c9d06c6d7a6d9fa21f0f199b9bf429794d5549ff11fe867f9f3a903f").into(),
                            source: Checkpoint{
                                epoch: 25,
                                root: hex!("fa7c738f848376e19c31c91b7abbec142ee0bf102f44716b239b897d2c042cae").into()
                            },
                            target: Checkpoint{
                                epoch: 26,
                                root: hex!("caea346f897947c09303776ce45a89ccd3302ccc8cfa5e3324818f394eccb121").into()
                            },
                        },
                        signature: hex!("b1d92a4a47409ca0115bba864d5a4facdddce8dcf18f66a4081a3442cfbb341f1859e4864aaef018756d34553b347d571574183444c7148f16e948afd8c1422cf7c62dc529e96f2533c9fc7119a20417631e80e8fe0b690b2ce386946d556305").to_vec().try_into().expect("signature too long"),
                    },
                ].try_into().expect("too many attestations"),
                deposits: vec![
                ].try_into().expect("too many deposits"),
                voluntary_exits:vec![
                ].try_into().expect("too many voluntary exits"),
                sync_aggregate: SyncAggregate{
                    sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
                    sync_committee_signature: hex!("9203e8600bb9f6054f775ccdbb38adef0e7c6b8480e715b38cd048ddbc23826bd4b145b6c8f9cfdfb6bb7a9abd06b2b2016cdb4cc89dff1974d6cadb9b6afc6478017f7e1a6fa1f924b760f1590c9e6609be920b861646cdee8ee8bf0957c461").to_vec().try_into().expect("signature too long"),
                },
                execution_payload: ExecutionPayload{
                    parent_hash: hex!("f3db4f1da803d174f67306d688ff5bf9b3922574661c8cf6b4eaaa39507028f6").into(),
                    fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
                    state_root: hex!("6254153699ff00695ae866851b31d888ad555cf64c03d3d4eba72e757552bc7b").into(),
                    receipts_root: hex!("94e76de6c16bedae315aec5eda36ae6605f7679006610b9d1c24720d263354f5").into(),
                    logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000200000000").to_vec().try_into().expect("logs bloom too long"),
                    prev_randao: hex!("0a0fb124d6a965d3268d3e894a8cb888df2750287742cd0c17a67fbd2827399b").into(),
                    block_number: 214,
                    gas_limit: 64906292,
                    gas_used: 106196,
                    timestamp: 1677137686,
                    extra_data: hex!("").to_vec().try_into().expect("extra data too long"),
                    base_fee_per_gas: U256::from(7 as u64),
                    block_hash: hex!("16375e9ec3eba78222284425fbbd4f6e4adf1934f00304e886d9bfb9137a2681").into(),
                    transactions_root: hex!("474ae4fd560957b99d20c81ea03d98ec3d2ef8285602e96d1c50be2b8ddd2397").into(),
                }
            }
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("93f2ba364d3a927f6185f14dc61f0d3cdbefdf1a0106b20a62715538698f3cc0b18e9fb70c3815e49ab43822a28a6d4b0ddcba727ce76b8a8928cbcbd225eadb4cdb47f5f7e14719654e834ad6efbe57d645fb3298e8b8a63eea4c1183e07bc9").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 215,
        block_root_proof: vec![
            hex!("4c626195440e400fdc3a30270dadd75fc1678f601cf87687b545d6a1e23ee685").into(),
            hex!("85f7b9dcec9ca7f03c2b420a1843229db76f3207375e9f23ce95553eb443d04f").into(),
            hex!("0aa90e0879b6fbd708e046957c7f49ac7cb73c6d0bd31eae49a4794a14096df0").into(),
            hex!("e042f5e342ed4c79ff540770526e22399561e2ad5d99cbd97982669e2b59aab9").into(),
            hex!("d391778afd6c5bda5d6acb04356213f4c728bb904d3164087d9940be2d443018").into(),
            hex!("d9fd3a2f11cdd74fc05b8aacdfb59abab320580473be6e9ad2ac243758496eac").into(),
        ].try_into().expect("too many branch proof items"),
        block_root_proof_finalized_header: hex!("d82ea96bca6bc36a1c07cde9bdf05d9777cf4d2d33cb56817f52167ff94e9c24").into(),
    };
}
