// Generated, do not edit!
// See README.md for instructions to generate
use frame_support::traits::Get;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	BeaconHeader, ExecutionPayloadHeader, FinalizedHeaderUpdate, HeaderUpdate, InitialUpdate,
	PublicKey, SyncAggregate, SyncCommittee, SyncCommitteeUpdate,
};
use sp_core::U256;
use sp_std::vec;

pub fn initial_sync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>>(
) -> InitialUpdate<SyncCommitteeSize, ProofSize> {
	let time_now = 1675679352; //2023.2.6

	return InitialUpdate{
        header: BeaconHeader{
            slot: 152,
            proposer_index: 2,
            parent_root: hex!("c6e5d2ad0e10309dc895c9f3a37b0dcadf95648f036b42d0ef5084252c69f1ed").into(),
            state_root: hex!("b0f73342ab9c265eae11cc98cee2f82e5974158a6cb6772f5e57991b2f2a3f3a").into(),
            body_root: hex!("d0ae6720fd5cd20d994292f4efa540811f2011b4b8aa466fe1a3c6aa8b1418ad").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: vec![
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        current_sync_committee_branch: vec![
                hex!("ae55686c9aa0cfc3a6a3edf5fcc27f3eafbe2772444f4fae6f1f00e1cee7d0ac").into(),
                hex!("e6d21c47c11698b8acf43778e6e7bdb804f14318bd37460b2274814f1f6cc554").into(),
                hex!("24851e4e01f264323b3bf1373dc8c99c71afa12ec04329df98e6dc1c45555365").into(),
                hex!("f5097ceea15bf7a5f7eabcaacba29af64ba0653e2077287e1d17f98d1a5778ad").into(),
                hex!("ebd5dd152ab8fc8acd6ee0d7e43537c3caedc12a9e283e9079ac17a92f2eb931").into(),
        ].try_into().expect("too many branch proof items"),
        validators_root: hex!("270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69").into(),
        import_time: time_now + 97200, // now + 27 hour sync committee period
    };
}

pub fn sync_committee_update<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> SyncCommitteeUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
	return SyncCommitteeUpdate {
        attested_header: BeaconHeader {
            slot: 144,
            proposer_index: 2,
            parent_root: hex!("d606dedd06b3e00b5c04a80d4d7f4c07c08c6bc1cd6ada47a141f0d7cc4a8032").into(),
            state_root: hex!("1720054de28109ee155947a31ed88649c514e590440e959b071d981d70e57e76").into(),
            body_root: hex!("8624a57d021cf59e367432769b53e4da63f9a613db778e2e89db701f0b068bea").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        next_sync_committee_branch: vec![
            hex!("9aaac2741ce2878a681113933945f318219d798b6d487cc93e8754a2a5a3a965").into(),
            hex!("c31424ccfc25a7c31fdf86b6f96e8fd9968e79fb11b39f8a05c28a91d468705f").into(),
            hex!("f782ef12694554b3080ea1479ae045ffa944b6619ca49f2ab75fc20b43c8980c").into(),
            hex!("360a6db1bac303ac552dc634c7539af9c781e3fc20a7b68ddd95635c6e4cdee5").into(),
            hex!("1e4cad215834a2821fd8cddb69258c0f49506cbd32a2165b774a1860e7e346e1").into(),
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 128,
            proposer_index: 3,
            parent_root: hex!("85c48e098074d4e149c05065af9500cecc802f6ae860be69e33dbd52c5c11299").into(),
            state_root: hex!("5229bec41f0b20ff789212c00466fa41d003ccec477948f1a2aa5acc875dff7a").into(),
            body_root: hex!("abc39f53ae99d6a88afe978320365a6a08ab74167f752b1ca9ec1018e339c7bc").into(),
        },
        finality_branch: vec![
            hex!("1000000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("24477da323967ef1397fb25cde5aa28ed5d7f5a5b437ba4343c9be88041bc119").into(),
            hex!("f782ef12694554b3080ea1479ae045ffa944b6619ca49f2ab75fc20b43c8980c").into(),
            hex!("360a6db1bac303ac552dc634c7539af9c781e3fc20a7b68ddd95635c6e4cdee5").into(),
            hex!("1e4cad215834a2821fd8cddb69258c0f49506cbd32a2165b774a1860e7e346e1").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a6e5309e0439042fe2d98342faf63452f3aed9a0d4f0086a42588e96a4c619d90964575aff9d7c5aa80e6846efcae3ec1825e2b750314fb715692e38c8a74f2fa86ebf2d3343294c87eb7933794e3c82398168cf3fbf9732c4df51bba3e44989").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 2,
        signature_slot: 145,
        block_roots_root: hex!("0367a032243a6c3660e9fe6d1a66c8347767908bff0feef436440e89b35947f5").into(),
        block_roots_branch: vec![
            hex!("bae2da1b2ef7c3d5d08f251a52f28efb26baa02e37c068b9c7c714055eec0985").into(),
            hex!("c046d211fd6f111e77a764a02e4c87ffb2495e23cec971793d2c3b141c23ad19").into(),
            hex!("fffc075fe465a906ba230573b6087e6d582d051c6aa5a6983fee1284818818e8").into(),
            hex!("ce896755e5dd78b5f1d8cff2cfff79745f3a53f3be1cab1faa86c2e4c727af59").into(),
            hex!("06b03718b08a83210a0c3a0ba687e246a701e60c7d5d02cca2fc76d054a78918").into(),
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
            slot: 168,
            proposer_index: 5,
            parent_root: hex!("d567978cf2992cc290f223df75d58369a9893d798c23ed3722ab7a0690bda687").into(),
            state_root: hex!("3e30d9357c88dc78c3bc8ed19685e3f21e90072f809d0d30a4658634e4f5ab41").into(),
            body_root: hex!("581383568a951904285564c960260d21a662919d0f18fb76e001515c7f1496f6").into(),
        },
        finalized_header: BeaconHeader{
            slot: 152,
            proposer_index: 2,
            parent_root: hex!("c6e5d2ad0e10309dc895c9f3a37b0dcadf95648f036b42d0ef5084252c69f1ed").into(),
            state_root: hex!("b0f73342ab9c265eae11cc98cee2f82e5974158a6cb6772f5e57991b2f2a3f3a").into(),
            body_root: hex!("d0ae6720fd5cd20d994292f4efa540811f2011b4b8aa466fe1a3c6aa8b1418ad").into(),
        },
        finality_branch: vec![
            hex!("1300000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("24477da323967ef1397fb25cde5aa28ed5d7f5a5b437ba4343c9be88041bc119").into(),
            hex!("dabc4d1850acd0551d8d30f9d35b3471f88477f56e617cbd48b52a38f0682fd9").into(),
            hex!("58c6354b6ac69908c6306aa6a16ef3fd9d1fcfc9a1ba46732e65635b93d48788").into(),
            hex!("e814c498f7b604b0ee9168053bb905fca91d004c1cb5cd5e8647ce7ceb4328c1").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("b7a1aa69fd2b57308f871dc877215b3521ad336b5af88bc286c5b42f8ecf36cad076c67465fc5c02c72132ee000eeb300e60362fbd5548a6c27e5b0b8d0d3d6b32adfc08673605572b630b005ac07af5cb7617176100ae843fbb86cee4e7c683").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 169,
        block_roots_root: hex!("0ca0cf2867bc9c8361f2c66a926218d0a5817301667e0a974983cd1d87dd6e55").into(),
        block_roots_branch: vec![
            hex!("7265769c5443608ec0ca000bd6d92b98bd74a6ed1d91bdb9745b173bd4bd30b2").into(),
            hex!("63da1a7293f7a8ba9e555312e91aac391847813390b76a51c6b6842a63509151").into(),
            hex!("314eb3af46fb3cd5b6e78ad4e91467a4149c056d1e8d8e76dd2382177d2be464").into(),
            hex!("bc5b324e321445e9ee98c670c563a5c7793b12ecdd79b25be0abee5288d9b410").into(),
            hex!("c1e6c27607e2ec00b123b5e98426100fe38c14e35030505e9cd5c713f194c71d").into(),
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
            slot: 150,
            proposer_index: 5,
            parent_root: hex!("8bdd8b8f9e6887ff2b05b322339fea4ec78a7e6159ec4e54b591a44699662b5b").into(),
            state_root: hex!("0db33febe0d69ec00eeab7961cb3d3e9a16ccc59f1314a5b67d2887be37e624d").into(),
            body_root: hex!("cbfb4d27fd2e98f38921914e5e79b357684769b3adfaa17861b423f7f63f37b1").into(),
        },
        execution_header: ExecutionPayloadHeader {
            parent_hash: hex!("3fe1542912d40d01821fbee41809db8e53a8cbae0d854cb0e5a9d94247fbc031").into(),
            fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
            state_root: hex!("251f215b8f0bb2caceac8cf469f0bf4daf1d0a8e319b506a02ab4a0362e31083").into(),
            receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec().try_into().expect("logs bloom too long"),
            prev_randao: hex!("821b4a7000256c9927032630785533f5fa4f1a40b6ae95eedf86d5910e18549c").into(),
            block_number: 150,
            gas_limit: 69094400,
            gas_used: 0,
            timestamp: 1679133094,
            extra_data: hex!("d983010b02846765746888676f312e31392e358664617277696e").to_vec().try_into().expect("extra data too long"),
            base_fee_per_gas: U256::from(7 as u64),
            block_hash: hex!("d84d6e83a9b4783fe4a290edc0d57e218a42aeb4f4c97dc7c83dacf82a1ec5eb").into(),
            transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
            withdrawals_root: hex!("28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30").into(),
        },
        execution_branch: vec![
            hex!("577ef1f2f9b33764e9a0d56225a4df5890f3b5ac3dc6fec05db54b65f20a8984").into(),
            hex!("336488033fe5f3ef4ccc12af07b9370b92e553e35ecb4a337a1b1c0e4afe1e0e").into(),
            hex!("db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71").into(),
            hex!("1b1451ad43516084cc488a22800d39d3368f0220cbc844957321fdf4dfe8154d").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("b203cba9326023622caad739fe44a50392d85a8bc1cb3f0c64d89319a865dc1dc1d1ff318d939ac24e4866068d8ca442116d4e7badec88015fd2865f97d5d5ab321c6c9a6994a9afee31f03cd015131320324bffc5d18746d232578c321ea61b").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 151,
        block_root_branch: vec![
            hex!("c6e5d2ad0e10309dc895c9f3a37b0dcadf95648f036b42d0ef5084252c69f1ed").into(),
            hex!("86d3344e6df5f9e3b930fe849e4ef99eca85cf455698db3acff0ab5449f0c383").into(),
            hex!("476a0e95a7b8cdef9aa2b8456ad44c46e7d828fd1821dec3dd3f456c21dbd679").into(),
            hex!("0fff87fb108fae09e61eafef5bcf22f4a5267699d5624fab371f661cc5e0c938").into(),
            hex!("76dd343907582c21ec28320c1d6a10349d338ac8ae4bb6e890063ad7d4ce4a6a").into(),
            hex!("7eebb150fd62159c1adf5dbe3e677b9574ea304d752f06de0a9f03f8f0ed6597").into(),
        ].try_into().expect("too many branch proof items"),
        block_root_branch_header_root: hex!("105976df5861b95fa4184564978eac0bcbe872acdd77a0ead552a8b283dabc4f").into(),
    };
}
