// Generated, do not edit!
// See README.md for instructions to generate
use frame_support::traits::Get;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	Attestation, AttestationData, BeaconHeader, BlockUpdate, Body, Checkpoint, Eth1Data,
	ExecutionPayload, FinalizedHeaderUpdate, HeaderUpdate, InitialSync, PublicKey, SyncAggregate,
	SyncCommittee, SyncCommitteePeriodUpdate,
};
use sp_core::U256;
use sp_std::vec;

pub fn initial_sync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>>(
) -> InitialSync<SyncCommitteeSize, ProofSize> {
	let time_now = 1675679352; //2023.2.6

	return InitialSync{
        header: BeaconHeader{
            slot: 40,
            proposer_index: 3,
            parent_root: hex!("72e2a9c90739db3ede455f945cafac2c7684ce03d93b1d5966098e6aeb58f1c4").into(),
            state_root: hex!("5a433de7a180b5f63b7cd4f2a39b87f25bc27749697377c27547d492ec7b51c6").into(),
            body_root: hex!("9c1d53e655431ac4a0c7d2a62e5d682af72c09f3df2188b591a043fca7567831").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: vec![
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        current_sync_committee_branch: vec![
                hex!("36b2b39c3982e2a358329a3165723f7ecc7f78f33ae8253dbd8aceb16af9d69e").into(),
                hex!("1197af99b4532f0eed9fe7e2d69f09d43180911a16d706f8c8285803757166c0").into(),
                hex!("fa45b0100f39cb4a9e4164d57b49a6eb79e4b785ba2365cf398f1447f84baef5").into(),
                hex!("8d3206950c1d72b72d103fc40e18b1cecf341bf76fb44f826e98cbb8f4a1f4a9").into(),
                hex!("a217974cb00420183019c0197337159982a1cf30dd79acdf37423c82b4472278").into(),
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
            slot: 32,
            proposer_index: 5,
            parent_root: hex!("af94ebb8d096af4413fa2079406d99c61ae1c98f9907b01a3a2be43ae7c126d8").into(),
            state_root: hex!("694ba46d69182f2b41c89b446f2ad2d4b96d22c95ef8db6b563883116d3f58b6").into(),
            body_root: hex!("bcc890501e9caa20cab1dec961391360f33f72da65469fe6ae5064203a43c393").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        next_sync_committee_branch: vec![
            hex!("36b2b39c3982e2a358329a3165723f7ecc7f78f33ae8253dbd8aceb16af9d69e").into(),
            hex!("3ace0ebd0031877db4bf5c16370a6b663489b4c5f5d324c8d928e763f15f4085").into(),
            hex!("7deae0ba24c4446ac0b55202a317a0c47edf6f7b0c368feda858a593d84e954a").into(),
            hex!("a5ab09a8a4a4657b9cbd3bb61161ff3896098276bffc4b10d99595bdcd761b4e").into(),
            hex!("c1599ecfbf73b21688b95dd9b66f40e5fee14989f9a9042a382f6da9416f9e00").into(),
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 16,
            proposer_index: 4,
            parent_root: hex!("a846e144ef34dd70c35f90d3f735c3ed26d882841827b496d46288f0e07bbedb").into(),
            state_root: hex!("05df9201c9dabee97ee52a3918c6602e069b9b0ec11e3f866a92d8ad79a5741c").into(),
            body_root: hex!("b757e3556011faa339be84a48780828e527d7eecf0feac47f4e18c20acc7cbae").into(),
        },
        finality_branch: vec![
            hex!("0200000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("9bb0fffbaa49f831ec27bda5d521c831634abe486fad0d9f2de3cbbac38d2231").into(),
            hex!("7deae0ba24c4446ac0b55202a317a0c47edf6f7b0c368feda858a593d84e954a").into(),
            hex!("a5ab09a8a4a4657b9cbd3bb61161ff3896098276bffc4b10d99595bdcd761b4e").into(),
            hex!("c1599ecfbf73b21688b95dd9b66f40e5fee14989f9a9042a382f6da9416f9e00").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a893c54f22cbadbae9259265486044cbd92af458a9350eb4181370112d23d6c70d3308526e22c472238e5b4b63cb7947075c4588b2e1a5da9cc85078746719656d7c8b8d1a924cadc97a85daae9c58fff3c350a05fae9dc31eaf1f3ee79672d8").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 0,
        signature_slot: 33,
        block_roots_hash: hex!("503fa1beb081cb6b5fdf06422e306d27f262dfd26c7aaf5dc452a79c521b8fca").into(),
        block_roots_proof: vec![
            hex!("c9a71a92244319e3e7e9353d3c77e2e6f0e5300269f2fa67f9bc8a6b1b2d07d8").into(),
            hex!("2e89b833be2f746715f4c3a062a52693fa818c587f9a00d3a6dbe5a92d28b492").into(),
            hex!("336deefd747179354c9f165864c3c94a0a6e5086c382bd75bf4a04e9860bc0c9").into(),
            hex!("7f21760710c190d77e5ed966cea1138bad8eed01c952f33c69c754b329b853d6").into(),
            hex!("fe645fcc57e50a0eb0ec69938217d59720be219ed9597b40fbd1d2c0fe1e2691").into(),
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
            slot: 56,
            proposer_index: 2,
            parent_root: hex!("f971f2d94f7273d38dda7bb86d9668513f4e1f60d373d7d773cfea3012001d7f").into(),
            state_root: hex!("96cf09226f6768b183162a26e578ca90fbbcdb7724256c78cc2143fb90854cc7").into(),
            body_root: hex!("217d8335842d1d52905af1bb201116d60e0380611736b208c016bd7a92156245").into(),
        },
        finalized_header: BeaconHeader{
            slot: 40,
            proposer_index: 3,
            parent_root: hex!("72e2a9c90739db3ede455f945cafac2c7684ce03d93b1d5966098e6aeb58f1c4").into(),
            state_root: hex!("5a433de7a180b5f63b7cd4f2a39b87f25bc27749697377c27547d492ec7b51c6").into(),
            body_root: hex!("9c1d53e655431ac4a0c7d2a62e5d682af72c09f3df2188b591a043fca7567831").into(),
        },
        finality_branch: vec![
            hex!("0500000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("9bb0fffbaa49f831ec27bda5d521c831634abe486fad0d9f2de3cbbac38d2231").into(),
            hex!("86d5863fb4c7f20268ebc9e7ffed7bc21cbb353b48b11ee33a167b34fe643c90").into(),
            hex!("0c0eaa5e54e678405f89ab9952c5847e3c3e9d699b5f90230896cd14fca2cae8").into(),
            hex!("5f133b1abac1949dab604f1c6d41663286f75f8b7f24507fe8abfa9441811660").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8a370474baaf17d44e661d01bbc623f775c30f02a2acc7340580f36052e1958dabcb1eb656d056bd6f2f450c983dafaa11dd33610f2a916ece4d9c4af546e005b6a0d3b7dab095b2f885b21c3c64207b608aa299a6986fe962d514f9de00a976").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 57,
        block_roots_hash: hex!("9df79d09541deed9d2c42ad1e5d3d86dbb44575a6c92e52262e1b9c48c335bfa").into(),
        block_roots_proof: vec![
            hex!("045b63a65d49b7834f30126d489453ac2affb159ba0887a19ef4a179c311cf98").into(),
            hex!("48a0b8f8241e6a63e211d89792d4f6b222216e9a89ecc0ca5c88495e8ffc530c").into(),
            hex!("187ed094fcd3742ecca7a6d709ddc3b5f2de2fe772e9047667d9e621961e530f").into(),
            hex!("9ff09f9a3f6a5a64ed4221287e6225ddc7f3b917b9186818f3b8d8d8177b7c3c").into(),
            hex!("df84f74541da859b7ba9fdb2cfcad37e86d4cddc8ee64879cf09088b9d1e48d9").into(),
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
        beacon_header: BeaconBlock{
            slot: 38,
            proposer_index: 4,
            parent_root: hex!("e036bfb3484d808701e2ff0e1174d278b54375356c0867e2f0f8a2bbdad5f32a").into(),
            state_root: hex!("cea97d689427d594590a879b38da45fcc944b372fb89a0612106d85d84693d04").into(),
            body_root: hex!("6fa9eac768299b00bd824cdd844b307793db9932fb7bbd2057d4116d79209d82").into(),
        },
        execution_header: ExecutionPayload{
            parent_hash: hex!("418965091125b94c820531a4b9aa89dd0c25413d2468dde3a4fe70f77b3e00e9").into(),
            fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
            state_root: hex!("04b629f4748b0f30b6dfd1e21d37cb334eb669d0b16bebf42c2ef7a343497393").into(),
            receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec().try_into().expect("logs bloom too long"),
            prev_randao: hex!("efc62a03d46fefc376bbe52d586bdae5b7de251bc8854271ab4cef83a145860f").into(),
            block_number: 38,
            gas_limit: 77084315,
            gas_used: 0,
            timestamp: 1678351469,
            extra_data: hex!("").to_vec().try_into().expect("extra data too long"),
            base_fee_per_gas: U256::from(6256137 as u64),
            block_hash: hex!("76adba90904a9e354f5654240d444fbd8c2628b4366417a2c3f8ba1b773ba05b").into(),
            transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
        },
        execution_branch: vec![
            hex!("9f794f1b180b18279833d0be8d77153ca175f1447ccce68a0d39523efb24a815").into(),
            hex!("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b").into(),
            hex!("db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71").into(),
            hex!("99b60da94401a15d49d0e7eb891f5ce94d78c98a89a8f0ed972177e488f22edc").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8ca7805a8ab0e70b7532c28b2b7b79bf1a8bd5dd49b1b624d9afa75a76e22af2097243ae17790e97477184906d678acb11f0ef614aa21fec5a2a4c077a96e463801d8c4363d24796f3e43ace85616407d89a0af1e458fa305f9583741cb38b93").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 39,
        block_root_branch: vec![
            hex!("72e2a9c90739db3ede455f945cafac2c7684ce03d93b1d5966098e6aeb58f1c4").into(),
            hex!("edcdff90ddedc4aeae0f421dc6446a9682846ca0f16032e51a9c7fc1920f5053").into(),
            hex!("d6bc55191c46e5368d1ee516aa67f59648dd95d06f1a33679397a84b2a8cc316").into(),
            hex!("c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c").into(),
            hex!("536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c").into(),
            hex!("363c9b34b49b5be3f992185ccc920bda3ad7e8765c7e39546c5b86b88ec1198b").into(),
        ].try_into().expect("too many branch proof items"),
        block_root_branch_header_root: hex!("247a3b0a106642dd71bbe583b53045636ced59371199da0da8d89400b6a80551").into(),
    };
}
