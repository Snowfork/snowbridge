// Generated, do not edit!
// See README.md for instructions to generate
use frame_support::traits::Get;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
    BeaconHeader, ExecutionPayloadHeaderCapella, FinalizedHeaderUpdate, HeaderUpdate, InitialUpdate, PublicKey,
    SyncAggregate, SyncCommittee, SyncCommitteePeriodUpdate,
};
use sp_core::U256;
use sp_std::vec;

pub fn initial_sync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>>(
    ) -> InitialUpdate<SyncCommitteeSize, ProofSize> {
    let time_now = 1675679352; //2023.2.6

    return InitialUpdate{
        header: BeaconHeader{
            slot: ,
            proposer_index: ,
            parent_root: hex!("").into(),
            state_root: hex!("").into(),
            body_root: hex!("").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: [
            ],
            aggregate_pubkey: hex!("").into(),
        },
        current_sync_committee_branch: vec![
        ],
        validators_root: hex!("").into(),
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
            slot: 80,
            proposer_index: 2,
            parent_root: hex!("8c772b8cae6fcf05326d0baf2aa35f62721407764a4c40ebc6496f236f2996b6").into(),
            state_root: hex!("b8e9fb79fa9e5bf6f16f799a8db2d44f9c174f2938c8cc34168f95e1e597c955").into(),
            body_root: hex!("042581f5e4b50a56f5258e8f58358e4cb36819c76157fedde33fff7ab71e968a").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
            ],
            aggregate_pubkey: hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into(),
        },
        next_sync_committee_branch: vec![
            hex!("c83ff57424470a11c94cdd1ddd9acf552852c8f6f378bb58e9eb00b13f92f7ea").into(),
            hex!("952f981c1b064e7f348f085ed0b9a7adb6dae3c91a83436c9c7bb3033d9ac5b9").into(),
            hex!("6aecda95663b9e2e1e63dfca55fe0372ba982040541bd0f46f8188bb977a923d").into(),
            hex!("b569d651a3035ccf987b096c0a709f3675c1d51b4887930cc17dbde7b5622ec1").into(),
            hex!("fc1d7df4f17e5d20edae1433e8268b0974287df80a91d8431829b626ade1f89a").into(),
        ],
        finalized_header: BeaconHeader{
            slot: 64,
            proposer_index: 6,
            parent_root: hex!("081a74140bfb2dfa9bfbfd9dfb4cf16273c7af830342a09a33a024e3ab61cfc2").into(),
            state_root: hex!("12530419828feb4acb6792916be98dbb2511bf1db0d1dd53af4b92b88ecf2de0").into(),
            body_root: hex!("fef2914c7b983f033f143ffe5a997b5c28f87f9737fee7a84554fdd7011ca643").into(),
        },
        finality_branch: vec![
            hex!("0800000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("82948c086478d3cd9d4440c395e7c919fa33ce91abceeba81437478dc289acdf").into(),
            hex!("6aecda95663b9e2e1e63dfca55fe0372ba982040541bd0f46f8188bb977a923d").into(),
            hex!("b569d651a3035ccf987b096c0a709f3675c1d51b4887930cc17dbde7b5622ec1").into(),
            hex!("fc1d7df4f17e5d20edae1433e8268b0974287df80a91d8431829b626ade1f89a").into(),
        ],
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("a5a7e1f249b6de746e154cce33656ce1e02cec33e54c883a4fc80af657da26b2e2f79c817cf7d17b558a90012c984bcf03043988bbf086392eb24ed417f1d1f9c4dd09bfa61fec9419b292676cbffd124afd89a775f49c9716f5bb4586485554").into(),
        },
        sync_committee_period: 1,
        signature_slot: 81,
        block_roots_root: hex!("fb8e30a463796805bf1d26d4343c1f44bf2ef40990c6899910adfb0620acb4b3").into(),
        block_roots_branch: vec![
            hex!("4db88d411c4c4a74161590c0d74a90b1ef90c00c11357c6785fb370b48180c3a").into(),
            hex!("aa68a5ab2d4b7b602b8248f672066adef102e3b1a30199fdbe49a1256aaa07d9").into(),
            hex!("aa325bab0688c4876ff936d5ad9be143f5b221e317d67d6c20ff8c698043bb59").into(),
            hex!("7eeee99ba2d0ee2979898849e0b71bec91c3e5652e20dc08fbe71f3256bd7fed").into(),
            hex!("58eaba2309bd9ec1b77d9589b0e5caac686a9928434862feb1fa51d4dae3a0bc").into(),
        ],
    };
}

pub fn finalized_header_update<
    SignatureSize: Get<u32>,
    ProofSize: Get<u32>,
    SyncCommitteeSize: Get<u32>,
>() -> FinalizedHeaderUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
    return FinalizedHeaderUpdate{
        attested_header: BeaconHeader {
            slot: 88,
            proposer_index: 4,
            parent_root: hex!("3c5d78ac911dffd15db68532fb2636e0bdbc3ae2949019c00b6393270518c241").into(),
            state_root: hex!("2da21762825d7dda284dcf5abd4775a73f862b19ba5f242485ece9c13022d85d").into(),
            body_root: hex!("c23de36f009769dc4a7286f4d4bf3cba9ecd81451bbff95f8eb41dd6184be1e7").into(),
        },
        finalized_header: BeaconHeader{
            slot: 72,
            proposer_index: 2,
            parent_root: hex!("5629f3117ab668ceaa7a73850a0963737a9ade6b4dc933f33f89cca29d4afe18").into(),
            state_root: hex!("135ad596ce3d49f2e77f983e3974d57203dc73a072bd6b39df0f8597379538fb").into(),
            body_root: hex!("3f5981bf6bda829a629d1b363a1eefdba35ffdfb11b52a86e4e3bfb7e587d402").into(),
        },
        finality_branch: vec![
            hex!("0900000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("82948c086478d3cd9d4440c395e7c919fa33ce91abceeba81437478dc289acdf").into(),
            hex!("51f5d2fdf77f840922a620300a9331378f6ee2359b4b531594edd05940cf2ca3").into(),
            hex!("dd9fdd6452906fe49bacc0289348af8378e4696059bc96f8101f51dc2978ccdc").into(),
            hex!("12a5e4daa84e527a9d385a353b3ce39466b0c5fab3d64900bdba82b3a596b68b").into(),
        ],
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("95a4470526294e5361ec25fa7d4614e9c7ab7d8fbbcc581f1e857559f53355013a4fb1c957c160735816d4cab737f39501b49c9c41641604175a2cc4fd39544da8ed6361fe3a2377cf072adfb91aeb3dd0d9063cd2f58dc9816089e59f569fad").into(),
        },
        signature_slot: 89,
        block_roots_root: hex!("3c24cf1eab10af300fb99827ccdf210738e44200c20d8740416534a775218f97").into(),
        block_roots_branch: vec![
            hex!("b6c272185a6ac2b7312fe852e9ae9788ddfb3a6c2277ee4c6d523d0da4770b74").into(),
            hex!("7be44999084960dddb5224beb5d170deecf56fd837cc467d1f19ab5e9db835f2").into(),
            hex!("80f2585a32b329c05067ab6eea4f70549020188abfa926b5c6a4bcade3734a36").into(),
            hex!("7117ac2c53d67ecd44dfe218c6c3e2f204a1584a83bb1d1f3750bd8ec635dbdc").into(),
            hex!("a13f0e75fe185d630a7f3e6fd1ab0887da47e542bd274d3834e22f030b830819").into(),
        ]
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
            slot: 70,
            proposer_index: 0,
            parent_root: hex!("8d5bd407701a7c2a7783060aa32523947e8dee84587596dc865bfaaea7b88cb4").into(),
            state_root: hex!("76a774cdb872ae29238bea2b38283cfc9bf0724521042f48b47bad2ed7c13d1c").into(),
            body_root: hex!("c4c24fe96afde6e2bbe39ffb82b1385a1eb0805424197bc049251b8c5af6abb6").into(),
        },
        execution_header: ExecutionPayloadHeaderCapella{
            parent_hash: hex!("5d8301c97b80be6f94e4c3000124158c2cbe92ca8db38c6c6a09d81fb38f5715").into(),
            fee_recipient: hex!("0000000000000000000000000000000000000000").into(),
            state_root: hex!("04b361e1bb8763058a934ec9ef62224a63d8e5fef915926261026aa2ced4bf88").into(),
            receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").into(),
            prev_randao: hex!("5c15907f0f9ba5c08bdda0ee009e8f6515400767bb79e85286262b3d532f26a1").into(),
            block_number: 70,
            gas_limit: 74711588,
            gas_used: 0,
            timestamp: 1684385716,
            extra_data: hex!("d983010b02846765746888676f312e31392e358664617277696e").into(),
            base_fee_per_gas: U256::from(91685 as u64),
            block_hash: hex!("cc3f686dcb2bdc8cafb810d936b0c7a68058430100baf0d8064d4446b4793b8d").into(),
            transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
            withdrawals_root: hex!("28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30").into(),
        },
        execution_branch: vec![
            hex!("211255a4f25e60c027f834a7b5c1f330d5fe8ceae2e70c0de2708ea1021439d8").into(),
            hex!("336488033fe5f3ef4ccc12af07b9370b92e553e35ecb4a337a1b1c0e4afe1e0e").into(),
            hex!("db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71").into(),
            hex!("9c9b69df6b8cbbf1b220de8fbb767d0ce8bdd0d95db0ddd48b0533fa1c448e00").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("8ca20631749cf52fd3a5ee37761f6d877b98966f412f2dcc64815e4683e6c957e2297b9bbb0868a42973c910f1caf16e0d30570e02bfd409ec6f7f51d781bbeb1ae269654736b0ceae1d069c4ae883cd90e2b481ee6e4ef0d1095b29cca12205").into(),
        },
        signature_slot: 71,
        block_root_branch: vec![
            hex!("5629f3117ab668ceaa7a73850a0963737a9ade6b4dc933f33f89cca29d4afe18").into(),
            hex!("d0acaef3827b61abf9f7e84c6c00ef967da85acc535ba73e5fea4bf08e9b77ac").into(),
            hex!("e63fa5d13b626269ea6d1712d92ae6366e0db878c8166f5b20c803e48b0bc262").into(),
            hex!("aaddd43ec51ceae099faac448be992e2ec918f429d7f988b41c416d1ff95ecbd").into(),
            hex!("bb8ecb6570c51c9c0e085cb49f091876ec25a5bd502e0d39e5f6d3e20256cb3f").into(),
            hex!("fd0d41d7851435aed0bcf546897b846d0f87c9b430730ef00399a4d614092268").into(),
        ],
        block_root_branch_header_root: hex!("40f276b469988631d66dad68840582af223ef5311fed38b82e6877ff4ea3ef11").into(),
    };
}
