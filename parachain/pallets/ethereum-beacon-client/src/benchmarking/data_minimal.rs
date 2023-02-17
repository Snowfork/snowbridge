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
            slot: 16,
            proposer_index: 5,
            parent_root: hex!("4160d4f2db3e573919c458f5bf7d29a82f18e78d9d98ebd71bc7b170c111428c").into(),
            state_root: hex!("85f571cbe5f3c204e3a33d758ab958aefca3215bef6e0e6ebd3492e3ffc6045c").into(),
            body_root: hex!("64ea641794a0dd3a7a12ad6a8cfd9f1bbd558953b337a7c40975ad80ac49e2ae").into(),
        },
        current_sync_committee: SyncCommittee{
            pubkeys: vec![
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        current_sync_committee_branch: vec![
            hex!("92df9cdb8a742500dbf7afd3a7cce35805f818a3acbee8a26b7d6beff7d2c554").into(),
            hex!("058baa5628d6156e55ab99da54244be4a071978528f2eb3b19a4f4d7ab36f870").into(),
            hex!("5f89984c1068b616e99589e161d2bb73b92c68b3422ef309ace434894b4503ae").into(),
            hex!("d33a17a3903ceac967c0afc2be32962dd69f5836e7674b4c30b2c68116720b2c").into(),
            hex!("0d0607530d6ffd3dfffafee157c34db1430cd7a1f29dea854769cf5c45aed99d").into()
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
            slot: 80,
            proposer_index: 7,
            parent_root: hex!("2a937036a7baee76abe846851de22ff66e7bf3028803554b13d51bbf71bb77df").into(),
            state_root: hex!("8e7b9cef08be18d33eee71731196ee0ddc66f9651e92b1571bfcefb206591292").into(),
            body_root: hex!("c5c548c7b4101f5179820490beb7b22283791dd24384d32328eede04ea67b08f").into(),
        },
        next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
                PublicKey(hex!("88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e").into()),
                PublicKey(hex!("9977f1c8b731a8d5558146bfb86caea26434f3c5878b589bf280a42c9159e700e9df0e4086296c20b011d2e78c27d373").into()),
                PublicKey(hex!("81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e").into()),
                PublicKey(hex!("ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34").into()),
                PublicKey(hex!("b89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b").into()),
                PublicKey(hex!("a8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac").into()),
                PublicKey(hex!("a99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c").into()),
                PublicKey(hex!("a3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b").into()),
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8fe11476a05750c52618deb79918e2e674f56dfbf12dbce55ae4386d108e8a1e83c6326f5957e2ef19137582ce270dc6").into())
        },
        next_sync_committee_branch: vec![
            hex!("92df9cdb8a742500dbf7afd3a7cce35805f818a3acbee8a26b7d6beff7d2c554").into(),
            hex!("766fe587be8a7f4fad53f2fbab80a05ac860b972116de2cd5ae81731dc14b786").into(),
            hex!("6f64400ffec870f8755dc54059f53d7dadff72133e3086ced29033185a8a0a27").into(),
            hex!("4c6527ba7e971739a1843d570d267ea3086db63ea0d1d3f8bc4f09940c7993c2").into(),
            hex!("c80d5ea71deea67299e517919da0b5ddfc39372bbbc6eb9e517fa6058b667184").into()
        ].try_into().expect("too many branch proof items"),
        finalized_header: BeaconHeader{
            slot: 64,
            proposer_index: 3,
            parent_root:  hex!("598d94215d008522e6440e51340935560508aa30290812002575aafdadc9beba").into(),
            state_root:  hex!("2df4ad597b224937ca4991279fb11a87f8a9aaa12e8979902f2edbd1897f4e40").into(),
            body_root:  hex!("9642f7ad16ef365f89ef5d0a9e11f9a2c3502cd9eab914e02aa72508ed6352b8").into()
        },
        finality_branch: vec![
            hex!("0800000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("d3dcb1f293e906fc339a96cada5c25cb26d692e9f2df3cbdf20f3790a4ab9067").into(),
            hex!("6f64400ffec870f8755dc54059f53d7dadff72133e3086ced29033185a8a0a27").into(),
            hex!("4c6527ba7e971739a1843d570d267ea3086db63ea0d1d3f8bc4f09940c7993c2").into(),
            hex!("c80d5ea71deea67299e517919da0b5ddfc39372bbbc6eb9e517fa6058b667184").into()
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("a0a21827bd977489d79153a5df37362d583bef778a244209f931f975930255a66f5eb11accc444dc9703e2e8da10033d13ea41549bd7559cc2c726447e4ecb276fbc2564b0b6a1a95e0c8e4aaadf52f284cf7f5fb50a9c4601e6224979c3871c").to_vec().try_into().expect("signature too long"),
        },
        sync_committee_period: 2,
        signature_slot: 81,
    };
}

pub fn finalized_header_update<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
>() -> FinalizedHeaderUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
	return FinalizedHeaderUpdate{
        attested_header: BeaconHeader{
            slot: 104,
            proposer_index: 7,
            parent_root: hex!("8ed318fbbad1e9c82405ced0caad53f957a9e85e6d992d38029d6456796b6616").into(),
            state_root: hex!("056f3e56d4de4a81ac360a8e2e26c6d4bb3a7e1aa3a9e3134e143e2ed47bf140").into(),
            body_root: hex!("cdd2e91b57ad652b1a92bba4fb2b7169b18940c74a29c3ed6ffda5625c4c1679").into(),
        },
        finalized_header: BeaconHeader{
            slot: 88,
            proposer_index: 3,
            parent_root: hex!("03fe2f7ea47b2f99de47640391423be25a32c0e1a9747fd28b0278bfd855f0cc").into(),
            state_root: hex!("7050b0bc9e8b7c44f2d9a4778c33f590242245db7078437d43068bf72ea75d55").into(),
            body_root: hex!("6d816397710a3a8c23f79d475c1085811a2398ddc906bbacf6bc82ca09eeee78").into(),
        },
        finality_branch: vec![
            hex!("0b00000000000000000000000000000000000000000000000000000000000000").into(),
            hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
            hex!("d3dcb1f293e906fc339a96cada5c25cb26d692e9f2df3cbdf20f3790a4ab9067").into(),
            hex!("566cdf50bcbdb35d5043a315598baad7597d765331ff2d92bcc1f17aa45d48a6").into(),
            hex!("3ff7eccb38997f778c6ef44254937763bbc56afbafe517a292efa9990a063330").into(),
            hex!("281bece9b2c38d77b38f92c6c30a95936387252e658eb34eb49ec39b83bd6235").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8651ddd6e0da54ce90c4fa1d6e43d510a0958b5aaf752cd567b68cf23181dd253a8e7c79e371f16c120a723fabc5f6fc0b82d0da6c88a9a041407f405b8bae023262a0e392a64bcba170f254b07c335b2f380e6c487022b11eb809513e8a8cef").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 105,
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
            slot: 87,
            proposer_index: 0,
            parent_root: hex!("4df88448681695a205ca42bc7bb5d9872647e55a8480700b4ac9d5554cffef93").into(),
            state_root: hex!("be8964ff956cce27f8e64219cdedaffc2a53d8f79dbe0be813070f91cc2f0756").into(),
            body: Body{
                randao_reveal: hex!("b2c1bbb3903c8de9576eeda7c6d5fa7a1ea866a99da63d570942ebc2aaf83f590bba57769feb88297dcc0450bd28320d0ba5bfdfaa5cfac87b858a3b63dd710399bb33a0203986068a7fba45e628b5cb024585b47d1440c112d8944414b4ad40").to_vec().try_into().expect("randao reveal too long"),
                eth1_data: Eth1Data{
                    deposit_root: hex!("6a0f9d6cb0868daa22c365563bb113b05f7568ef9ee65fdfeb49a319eaf708cf").into(),
                    deposit_count: 8,
                    block_hash: hex!("7ba0fb9a0503ffae09ce8873ff147ea2e36ecc04776d2094be3bf4da32dcbea5").into(),
                },
                graffiti: hex!("4c6f6465737461722d76312e302e302f636c6172612f736e6f2d3331342d6265").into(),
                proposer_slashings: vec![
                ].try_into().expect("too many proposer slashings"),
                attester_slashings: vec![
                ].try_into().expect("too many attester slashings"),
                attestations: vec![
                    Attestation{
                        aggregation_bits: hex!("03").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 86,
                            index: 0,
                            beacon_block_root: hex!("4df88448681695a205ca42bc7bb5d9872647e55a8480700b4ac9d5554cffef93").into(),
                            source: Checkpoint{
                                epoch: 9,
                                root: hex!("a7558983d21b9c44e136723eee2424fbe39e062951e94c71a2f96e3907161959").into()
                            },
                            target: Checkpoint{
                                epoch: 10,
                                root: hex!("7cbe122a6b7798c35cec67ae464ab1641370da0f42d116f2653a9414686760e3").into()
                            },
                        },
                        signature: hex!("b5fa75f5e653181ef942dc34aa9f5ad68c786806cc31eda1c7dcf9dc87d255ec352b9921612866b33ef1994d897e92d00dec5cf88e3997002b3e8882876d8e5991e53d0fda990b7006c4f427a8bf224ee87c61a1024b205d262a7861aea1f2bf").to_vec().try_into().expect("signature too long"),
                    },
                ].try_into().expect("too many attestations"),
                deposits: vec![
                ].try_into().expect("too many deposits"),
                voluntary_exits:vec![
                ].try_into().expect("too many voluntary exits"),
                sync_aggregate: SyncAggregate{
                    sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("committee bits too long"),
                    sync_committee_signature: hex!("8aa8cd44d5c94c0409d5a46bfdbeba600085f39beb68fe48f5e544319a5a9777878fb16a661b8abbaa0f1511293b584d0f723dd5692a263d6b1564ddab2a3e4b9494dd18c1070e9ce7dd96149758cdbfae81a04f1b362d36b0de002ad59d61e6").to_vec().try_into().expect("signature too long"),
                },
                execution_payload: ExecutionPayload{
                    parent_hash: hex!("85cbdd145046d6dc1e10b4e9eebe352b62f2f7230e7356ce4e48bf9a6ba07085").into(),
                    fee_recipient: hex!("0000000000000000000000000000000000000000").to_vec().try_into().expect("fee recipient too long"),
                    state_root: hex!("421ebad655a8d351e683ee85a94a2ce2201fa49d19d8a4219249ee9b7af5744c").into(),
                    receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
                    logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec().try_into().expect("logs bloom too long"),
                    prev_randao: hex!("0d3e8ce240bbd46065d5e75d92eedd9ce2b9db5a9bec422352d745f7739c42ee").into(),
                    block_number: 87,
                    gas_limit: 73480927,
                    gas_used: 0,
                    timestamp: 1663828524,
                    extra_data: hex!("").to_vec().try_into().expect("extra data too long"),
                    base_fee_per_gas: U256::from(10149 as u32),
                    block_hash: hex!("db5bddf99fdec754707103f47568ad7c3544a7d36473a76e76819fed4e7fa970").into(),
                    transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
                }
            }
        },
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("ffffffff").to_vec().to_vec().try_into().expect("too many bits"),
            sync_committee_signature: hex!("adc869227de9fb08b67333c8bd012dc73fe4ad4ed5f3ff3db981f2b9595191ecde20ea47c7495d3ce1ac6510bc97de9a0095b0086fb17698210497935e6a8fa2e17cd70e6bb3daf6c8e06674e124c5447d6d841a17d46316b8ca7ffb0d731f27").to_vec().try_into().expect("signature too long"),
        },
        signature_slot: 88,
    };
}
