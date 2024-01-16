// Generated, do not edit!
// See README.md for instructions to generate
use snowbridge_pallet_ethereum_client::types::{CheckpointUpdate, ExecutionHeaderUpdate, Update};
use hex_literal::hex;
use snowbridge_beacon_primitives::{
    updates::AncestryProof, BeaconHeader, ExecutionPayloadHeader, NextSyncCommitteeUpdate,
    SyncAggregate, SyncCommittee,
};
use sp_core::U256;
use sp_std::{boxed::Box, vec};

pub fn make_checkpoint_for_inbound() -> Box<CheckpointUpdate> {
    Box::new(CheckpointUpdate {
        header: BeaconHeader {
            slot: 16,
            proposer_index: 2,
            parent_root: hex!("7f379bc8ce5dd96fb4f517bef8518f832414bff2b9fda689502efb386aa8de85").into(),
            state_root: hex!("5542b5b78c555bf8ec345a8f26424d26c1a4bc7c314f4a0e16cc0f6978e61060").into(),
            body_root: hex!("8887bcecf4e2b752db5dd10ac75e8a018facf5c8c44ce2d4ad855db514ff320c").into(),
        },
        current_sync_committee: SyncCommittee {
            pubkeys: [
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
            ],
            aggregate_pubkey: hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into(),
        },
        current_sync_committee_branch: vec![
            hex!("f4559c0f3e0f259bae33f93ca3463d6737b4f695f3ed5f57b3bb6d41335bc668").into(),
            hex!("058baa5628d6156e55ab99da54244be4a071978528f2eb3b19a4f4d7ab36f870").into(),
            hex!("5f89984c1068b616e99589e161d2bb73b92c68b3422ef309ace434894b4503ae").into(),
            hex!("283b5cfb1aede413904c812e6533d9572279a9f9bea9606dccbd6ff5ac0a6709").into(),
            hex!("f7b34b2d97b34e895e080c7e72174b6a05a92c76c63f25abba558592180359ba").into(),
        ],
        validators_root: hex!("270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69").into(),
        block_roots_root: hex!("a6515efff3a8b1ab8c54dcbf9c6d4955453f4d21a621261318d2ab8bf0d08143").into(),
        block_roots_branch: vec![
            hex!("7fffa67f9b4bbc37a145c57863a32be3155490f6c2db10d502efbb1a04392d61").into(),
            hex!("f669784a9bfe440c8941c072add104a6c3ac9ceb202794d2d139d2e00990a523").into(),
            hex!("5a3f9029e4d404925fda8c3d71e029746b716909107e610d2f14a6d23696fc0c").into(),
            hex!("0f8ec98fe8b66b1c7f5f427b4993589afaff2219d3ba46a94f2b9fdf6290bc85").into(),
            hex!("62958f2f1cfe46d70d9a7fefcd39d82f22b8c67f4faabbb181376a9c56cb84cc").into(),
        ],
    })
}

pub fn make_sync_committee_update_for_inbound() -> Box<Update> {
    Box::new(Update {
        attested_header: BeaconHeader {
            slot: 32,
            proposer_index: 7,
            parent_root: hex!("92d964613164610e9ad8ad7daa322abe7f3cd0125c9338e242c7811366c69f94").into(),
            state_root: hex!("811010acaf7cc727041e13dfff32588a48bc7f4d9f9af7c6b9cf7662994137e7").into(),
            body_root: hex!("9ab71a90ee42dd0be4b7ec5ee317723486faccd7ddf882e85a8e43c74d13b1c6").into(),
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("8da2c2a734433a5af10c059945149e9db06758d6ed2e1b8db8a880eacab69f29c5f4af14d351dc08e13b66894b6dbffd0dcf83f46fa3fcd8acd7b39d73fe9d47ae0f57407588210e1a7d54d19d151441557b03476cdb1317af944f75dca789ec").into(),
        },
        signature_slot: 33,
        next_sync_committee_update: Some(NextSyncCommitteeUpdate {
            next_sync_committee: SyncCommittee {
                pubkeys: [
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                ],
                aggregate_pubkey: hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into(),
            },
            next_sync_committee_branch: vec![
                hex!("f4559c0f3e0f259bae33f93ca3463d6737b4f695f3ed5f57b3bb6d41335bc668").into(),
                hex!("cb9ca7562cfddb1ca079c98e1cf6fcc73ea66ed0e46dd45310ef63512081dd78").into(),
                hex!("a48ea6c88dd0d4378129bac34cedb9b34f1723638861024f0992e2d729f57fd1").into(),
                hex!("842f91784a5da88a1e6434d3f5271f7d6a5f4499d4bde940f34a7ab7ddc5beea").into(),
                hex!("943c482e8f6ffd8547fe27de12a66ebf0940915602164ce074908452e7a6eeeb").into(),
            ],
        }),
        finalized_header: BeaconHeader{
            slot: 16,
            proposer_index: 2,
            parent_root: hex!("7f379bc8ce5dd96fb4f517bef8518f832414bff2b9fda689502efb386aa8de85").into(),
            state_root: hex!("5542b5b78c555bf8ec345a8f26424d26c1a4bc7c314f4a0e16cc0f6978e61060").into(),
            body_root: hex!("8887bcecf4e2b752db5dd10ac75e8a018facf5c8c44ce2d4ad855db514ff320c").into(),
        },
        finality_branch: vec![
            hex!("0200000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("347ab339e982ca079957b11da8f694e45f430b515c0ded851e38cc44ca0c352e").into(),
            hex!("a48ea6c88dd0d4378129bac34cedb9b34f1723638861024f0992e2d729f57fd1").into(),
            hex!("842f91784a5da88a1e6434d3f5271f7d6a5f4499d4bde940f34a7ab7ddc5beea").into(),
            hex!("943c482e8f6ffd8547fe27de12a66ebf0940915602164ce074908452e7a6eeeb").into(),
        ],
        block_roots_root: hex!("a6515efff3a8b1ab8c54dcbf9c6d4955453f4d21a621261318d2ab8bf0d08143").into(),
        block_roots_branch: vec![
            hex!("7fffa67f9b4bbc37a145c57863a32be3155490f6c2db10d502efbb1a04392d61").into(),
            hex!("f669784a9bfe440c8941c072add104a6c3ac9ceb202794d2d139d2e00990a523").into(),
            hex!("5a3f9029e4d404925fda8c3d71e029746b716909107e610d2f14a6d23696fc0c").into(),
            hex!("0f8ec98fe8b66b1c7f5f427b4993589afaff2219d3ba46a94f2b9fdf6290bc85").into(),
            hex!("62958f2f1cfe46d70d9a7fefcd39d82f22b8c67f4faabbb181376a9c56cb84cc").into(),
        ],
    })
}

pub fn make_sync_committee_update_for_inbound_next_period() -> Box<Update> {
    Box::new(Update {
        attested_header: BeaconHeader {
            slot: 80,
            proposer_index: 0,
            parent_root: hex!("36bb663ce560c573aad997a5046ac9f2476249705353e20be39fdf907e3326aa").into(),
            state_root: hex!("dfae1d7fcc57a6a627f2960f6bd2a983161ff0a7bbc96a70f75fdddaff2985fd").into(),
            body_root: hex!("f4411ad1f792f4f7f88965b0d8aa61828d8a37947bccab6ec5135d84a06347cb").into(),
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("8195ba69d3dbc695b34a5cfdfaa0d76388214bcae1f2dcb8bf75cd898e4133b4273c98c544a47d7b1d3129e5daa6c12201751c6b4e2db44e0934463c038eeafbf81af4f7dcbb3baffa71cb4d7804c39243129a6579be7f31bfd8cf796115cd1f").into(),
        },
        signature_slot: 81,
        next_sync_committee_update: Some(NextSyncCommitteeUpdate {
            next_sync_committee: SyncCommittee {
                pubkeys: [
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                    hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into(),
                    hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into(),
                    hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into(),
                    hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into(),
                    hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into(),
                    hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into(),
                    hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into(),
                    hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into(),
                ],
                aggregate_pubkey: hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into(),
            },
            next_sync_committee_branch: vec![
                hex!("f4559c0f3e0f259bae33f93ca3463d6737b4f695f3ed5f57b3bb6d41335bc668").into(),
                hex!("2f5c3f1ffe90901e4233249e2b96114376538eca8826c1e89455c8358affc568").into(),
                hex!("443bb731c28a655e9403263d605c36662184d743f3a0040e1e67bad2e3433018").into(),
                hex!("5b068184f755f0224916b9f50f757f4c90d33b066efd1216c1519179fddee0ed").into(),
                hex!("6a222825e2a0db30bdd8a4a9bbd6a97ba346d84193a312c57fd1e77d50067c3e").into(),
            ],
        }),
        finalized_header: BeaconHeader{
            slot: 64,
            proposer_index: 1,
            parent_root: hex!("f5836885a3658653018057131bb2804971b2c31cf9a8e4deb3f3b593073fb5e8").into(),
            state_root: hex!("b9798b26fccf8e4b55e18694608ef040d20cf127a5fe80cc49dcf580b67f1154").into(),
            body_root: hex!("b7141601a8b32c576812f75f30ae4ec143454d36f526d1160a89a80c4cca2f6e").into(),
        },
        finality_branch: vec![
            hex!("0800000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("93c1d7a5898af1bd11b58d29a14048e5917063fc5ce201e855a9cb4802823740").into(),
            hex!("443bb731c28a655e9403263d605c36662184d743f3a0040e1e67bad2e3433018").into(),
            hex!("5b068184f755f0224916b9f50f757f4c90d33b066efd1216c1519179fddee0ed").into(),
            hex!("6a222825e2a0db30bdd8a4a9bbd6a97ba346d84193a312c57fd1e77d50067c3e").into(),
        ],
        block_roots_root: hex!("9eaf5c27212f3f9e26943955d12189856a5ff6c9bbeb70010ba5af66c41fa1aa").into(),
        block_roots_branch: vec![
            hex!("d93b457d7b75d15bdcae37849ef0c41deadf905fed32745e4302e565d7322239").into(),
            hex!("e2d69345651ecd08b93d2edaa952b177309d7a2886aee0542886b113014d6c4a").into(),
            hex!("8e813d69e7c1989314a9ff2db97e23d1347455a155c64f8ade8666304bef44c5").into(),
            hex!("91aa5098af40fb8cf04cd75838d5cc5ba6619ab6fcc83b6499e28f07993ea148").into(),
            hex!("224c1fe6db9d898000b082879aa96a8d9c22b22266b4fed86c5140170f308f71").into(),
        ],
    })
}

pub fn make_finalized_header_update_for_inbound() -> Box<Update> {
    Box::new(Update {
        attested_header: BeaconHeader {
            slot: 184,
            proposer_index: 5,
            parent_root: hex!("13548d0b72ef2b352ba73a53274ad02e9310d8417d4782bf9cdad877da549595").into(),
            state_root: hex!("cbd5e841afef5103a613bb9a89199d39c40751af1af7cabadfa8e1fd49c4be09").into(),
            body_root: hex!("c9b5074ac3043ae2ad6da067b0787d61b6f1611b560ef80860ce62564534a53a").into(),
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff"),
            sync_committee_signature: hex!("80628825caef48ef13b4ffda96fcee74743609fe3537b7cd0ca8bc25c8a540b96dfbe23bdd0a60a8db70b2c48f090c1d03a25923207a193724d98e140db90ab692cd00ec2b4d3c0c4cbff13b4ab343e0c896e15cba08f9d82c392b1c982a8115").into(),
        },
        signature_slot: 185,
        next_sync_committee_update: None,
        finalized_header: BeaconHeader {
            slot: 168,
            proposer_index: 1,
            parent_root: hex!("bb8245e89f5d7d9191c559425b8522487d98b8f2c9b814a158656eaf06caaa06").into(),
            state_root: hex!("98b4d1141cfc324ce7095417d81b28995587e9a50ebb4872254187155e6b160c").into(),
            body_root: hex!("c1a089291dbc744be622356dcbdd65fe053dcb908056511e5148f73a3d5c8a7e").into(),
        },
        finality_branch: vec![
            hex!("1500000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("c2374959a44ad09eb825ada636c430d530e656d7f06dccbcf2ca8b6dc834b188").into(),
            hex!("97a3662f859b89f96d7c6ce03496c353df5f6c225455f3f2c5edde329f5a94d1").into(),
            hex!("4c4720ad9a38628c6774dbd1180d026aceb0be3cb0085d531b1e09faf014328a").into(),
            hex!("c3d59497774f8f1c13fda9f9c6b4f773efabbac8a24d914dc7cf02874d5f5658").into(),
        ],
        block_roots_root: hex!("dff54b382531f4af2cb6e5b27ea905cc8e19b48f3ae8e02955f859e6bfd37e42").into(),
        block_roots_branch: vec![
            hex!("8f957e090dec42d80c118d24c0e841681e82d3b330707729cb939d538c208fb7").into(),
            hex!("4d33691095103fbf0df53ae0ea14378d721356b54592019050fc532bfef42d0c").into(),
            hex!("bc9b31cd5d18358bff3038fab52101cfd5c56c75539449d988c5a789200fb264").into(),
            hex!("7d57e424243eeb39169edccf9dab962ba8d80a9575179799bbd509c95316d8df").into(),
            hex!("c07eeb9da14bcedb7dd225a68b92f578ef0b86187724879e5171d5de8a00be3a").into(),
        ]
    })
}

pub fn make_execution_header_update_for_inbound() -> Box<ExecutionHeaderUpdate> {
    Box::new(ExecutionHeaderUpdate {
        header: BeaconHeader {
            slot: 46,
            proposer_index: 2,
            parent_root: hex!("bed3746d93e09573fc00565f3b0ed19afacb3d474f37168e2eee9c8cb115330e").into(),
            state_root: hex!("ee67bd3bdbf21b52f3a43b70634f6abbf16986b7c51f744fbb660b8b30afa7a6").into(),
            body_root: hex!("984cf4f6833c77a989dd803a61bea605a5d35b2ac66aa4dc85a99189536acc35").into(),
        },
        ancestry_proof: Some(AncestryProof {
            header_branch: vec![
                hex!("9c7f24db6968bf17b1eb0586eb8ead0c1dc76d9bf0e7edfeecca221a5fd28750").into(),
                hex!("5ea3cfc6ca89e22528ec38288e8974863148f5ed4494edbea0cecfe2739801b2").into(),
                hex!("77be546b9ec243d669b0c4fdb06f5a7c13bb0109046df7d2991f9019d414ab9a").into(),
                hex!("948648ddfd941f2852c82964a8cbed5d1c6b6a4783c811f7df16d28ef1053bc6").into(),
                hex!("dd590d345ca5f68003333ac68a80aa767dd659cb4d7368ff169f51e3f9544e7e").into(),
                hex!("928ab023edb57be5341d09fa0cbfd8f44d5d0a348baabf8ea9692505adbfca1d").into(),
            ],
            finalized_block_root: hex!("6102ee45368efa4ce28743d8d520fa2edcb1f3a3e2f1caee7f8966fcb9c81a76").into(),
        }),
        execution_header: ExecutionPayloadHeader {
            parent_hash: hex!("3bd02217d4d30e3c1937e8622d3d8e56023dd802baf7fdefaaead4ed7bf0ca5e").into(),
            fee_recipient: hex!("0000000000000000000000000000000000000000").into(),
            state_root: hex!("f24ad1582d213b6f4609e98c5a7d1359c54fbec3dc565e75298709239245f86f").into(),
            receipts_root: hex!("7b1f61b9714c080ef0be014e01657a15f45f0304b477beebc7ca5596c8033095").into(),
            logs_bloom: hex!("0000000000000000000000000000000100000000000000000200000000000010").into(),
            prev_randao: hex!("e7b186195ecac8dd3612c8d34e78d38ffc6496f881104a673e97fb390cf050f8").into(),
            block_number: 46,
            gas_limit: 76484160,
            gas_used: 119301,
            timestamp: 1705413696,
            extra_data: hex!("000000000000d983010c00846765746888676f312e32302e358664617277696e").into(),
            base_fee_per_gas: U256::from(2240573_u64),
            block_hash: hex!("90ffdbbb67ffadb16f4b9473f699d8fcb69e7fe1399c50b0134b7cd8f017fcb1").into(),
            transactions_root: hex!("15a29155114530d5e632c145a97cec54ff21b24e4b5fc3744ea883fe7b929b2b").into(),
            withdrawals_root: hex!("28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30").into(),
        },
        execution_branch: vec![
            hex!("e49c3af33ab80981116b235e9fea0a47280b912024720f759e283ac354a607a8").into(),
            hex!("336488033fe5f3ef4ccc12af07b9370b92e553e35ecb4a337a1b1c0e4afe1e0e").into(),
            hex!("db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71").into(),
            hex!("bec6857af8cdc6656627ff7e5b26c7f7f82614f3be2eab2484494f8480153d08").into(),
        ],
    })
}

/*
pub fn make_create_message() -> InboundQueueTest {
    InboundQueueTest{
        execution_header: CompactExecutionHeader{
            parent_hash: hex!("b5608f0af7c3b6fe5c593772fc25436b8d6549eb236adb0855c6ad33e0004e04").into(),
            block_number: 115,
            state_root: hex!("47ed174789836c622499d9659a4ac32c3b91a7b15642d39b0a11b82ff23995c1").into(),
            receipts_root: hex!("42c08b5303fcdf9e49c833fe5f1182cdbc8206bf8aec581125fc34aba11e1f1a").into(),
        },
        message: Message {
            event_log: 	Log {
                address: hex!("eda338e4dc46038493b885327842fd3e301cab39").into(),
                topics: vec![
                    hex!("7153f9357c8ea496bba60bf82e67143e27b64462b49041f8e689e1b05728f84f").into(),
                    hex!("c173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539").into(),
                    hex!("5f7060e971b0dc81e63f0aa41831091847d97c1a4693ac450cc128c7214e65e0").into(),
                ],
                data: hex!("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002e00a736aa00000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d00e40b54020000000000000000000000000000000000000000000000000000000000").into(),
            },
            proof: Proof {
                block_hash: hex!("add15f439c8a57fe375d0a679870b1359921d70cb0e3e44f0dd3e272849f4097").into(),
                tx_index: 0,
                data: (vec![
                    hex!("42c08b5303fcdf9e49c833fe5f1182cdbc8206bf8aec581125fc34aba11e1f1a").to_vec(),
                ], vec![
                    hex!("f9028e822080b9028802f90284018301ed20b9010000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000080000000000000000000000000000004000000000080000000000000000000000000000000000010100000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000040004000000000000002000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000200000000000010f90179f85894eda338e4dc46038493b885327842fd3e301cab39e1a0f78bb28d4b1d7da699e5c0bc2be29c2b04b5aab6aacf6298fe5304f9db9c6d7ea000000000000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7df9011c94eda338e4dc46038493b885327842fd3e301cab39f863a07153f9357c8ea496bba60bf82e67143e27b64462b49041f8e689e1b05728f84fa0c173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539a05f7060e971b0dc81e63f0aa41831091847d97c1a4693ac450cc128c7214e65e0b8a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002e00a736aa00000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d00e40b54020000000000000000000000000000000000000000000000000000000000").to_vec(),
                ]),
            },
        },
    }
}*/
