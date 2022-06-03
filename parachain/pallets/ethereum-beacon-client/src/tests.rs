use crate::{mock::*, SyncCommittees, Error, BeaconHeader, FinalizedBeaconHeaders, PublicKey, merkleization, ValidatorsRoot, LatestFinalizedHeaderSlot, ExecutionHeaders};
use frame_support::{assert_ok, assert_err};
use hex_literal::hex;
use sp_core::H256;

#[test]
fn it_syncs_from_an_initial_checkpoint() {
	let initial_sync = get_initial_sync();

	new_tester().execute_with(|| {
		assert_ok!(EthereumBeaconClient::initial_sync(
			Origin::signed(1),
			initial_sync.clone(),
		));

		let block_root: H256 = merkleization::hash_tree_root_beacon_header(initial_sync.header.clone()).unwrap().into();

		assert!(<FinalizedBeaconHeaders<Test>>::contains_key(block_root));
	});
}

#[test]
fn it_updates_a_committee_period_sync_update() {
	let update = get_committee_sync_period_update();

	let current_sync_committee = get_current_sync_committee_for_current_committee_update();

	let current_period = EthereumBeaconClient::compute_current_sync_period(update.attested_header.slot);

	new_tester().execute_with(|| {
		SyncCommittees::<Test>::insert(current_period, current_sync_committee);
		ValidatorsRoot::<Test>::set(hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into());

		assert_ok!(EthereumBeaconClient::sync_committee_period_update(
			Origin::signed(1),
			update.clone(),
		));

		let block_root: H256 = merkleization::hash_tree_root_beacon_header(update.finalized_header.clone()).unwrap().into();

		assert!(<FinalizedBeaconHeaders<Test>>::contains_key(block_root));
	});
}

#[test]
fn it_processes_a_finalized_header_update() {
	let update = get_finalized_header_update();

	let current_sync_committee = get_current_sync_committee_for_finalized_header_update();

	let current_period = EthereumBeaconClient::compute_current_sync_period(update.attested_header.slot);

	new_tester().execute_with(|| {
		SyncCommittees::<Test>::insert(current_period, current_sync_committee);
		ValidatorsRoot::<Test>::set(hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into());

		assert_ok!(EthereumBeaconClient::import_finalized_header(Origin::signed(1), update.clone()));

		let block_root: H256 = merkleization::hash_tree_root_beacon_header(update.finalized_header.clone()).unwrap().into();

		assert!(<FinalizedBeaconHeaders<Test>>::contains_key(block_root));
	});
}

#[test]
fn it_processes_a_header_update() {
	let update = get_header_update();

	let current_sync_committee = get_current_sync_committee_for_header_update();

	let current_period = EthereumBeaconClient::compute_current_sync_period(update.block.slot);

	new_tester().execute_with(|| {
		SyncCommittees::<Test>::insert(current_period, current_sync_committee);
		ValidatorsRoot::<Test>::set(hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into());
		LatestFinalizedHeaderSlot::<Test>::set(update.block.slot);

		assert_ok!(EthereumBeaconClient::import_execution_header(Origin::signed(1), update.clone()));

		let execution_block_root: H256 = update.block.body.execution_payload.block_hash.clone().into();

		assert!(<ExecutionHeaders<Test>>::contains_key(execution_block_root));
	});
}

#[test]
fn it_errors_when_importing_a_header_with_no_sync_commitee_for_period() {
	let update = get_finalized_header_update();

	new_tester().execute_with(|| {
		ValidatorsRoot::<Test>::set(hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into());

		assert_err!(EthereumBeaconClient::import_finalized_header(Origin::signed(1), update), Error::<Test>::SyncCommitteeMissing);
	});
}

#[test]
pub fn test_get_sync_committee_sum() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconClient::get_sync_committee_sum(vec![0, 1, 0, 1, 1, 0, 1, 0, 1]),
			5
		);
	});
}

#[test]
pub fn test_compute_domain() {
	new_tester().execute_with(|| {
		let domain = EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("00000001").into(),
			hex!("5dec7ae03261fde20d5b024dfabce8bac3276c9a4908e23d50ba8c9b50b0adff").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("0700000046324489ceb6ada6d118eacdbe94f49b1fcb49d5481a685979670c7c").into()
		);
	});
}

#[test]
pub fn test_compute_domain_kiln() {
	new_tester().execute_with(|| {
		let domain = EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("70000071").into(),
			hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("07000000e7acb21061790987fa1c1e745cccfb358370b33e8af2b2c18938e6c2").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_bls() {
	new_tester().execute_with(|| {
		let signing_root = EthereumBeaconClient::compute_signing_root(
			BeaconHeader {
				slot: 3529537,
				proposer_index: 192549,
				parent_root: hex!(
					"1f8dc05ea427f78e84e2e2666e13c3befb7106fd1d40ef8a3f67cf615f3f2a4c"
				)
				.into(),
				state_root: hex!(
					"0dfb492a83da711996d2d76b64604f9bca9dc08b6c13cf63b3be91742afe724b"
				)
				.into(),
				body_root: hex!("66fba38f7c8c2526f7ddfe09c1a54dd12ff93bdd4d0df6a0950e88e802228bfa")
					.into(),
			},
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("3ff6e9807da70b2f65cdd58ea1b25ed441a1d589025d2c4091182026d7af08fb").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_kiln() {
	new_tester().execute_with(|| {
		let signing_root = EthereumBeaconClient::compute_signing_root(
			BeaconHeader {
				slot: 221316,
				proposer_index: 79088,
				parent_root: hex!(
					"b4c15cd79da1a4e645b0104fa66d226cb6dce0fae3522789cc4d0b3ae41d96f7"
				)
				.into(),
				state_root: hex!(
					"6f711ef2e36decbc8f7037e73bbdace42c11f2896a43e44ab8a78dcb2ba66122"
				)
				.into(),
				body_root: hex!("963eaa01341c16dc8f288da47eedad0792978fdaab9f1f97ae0a1103494d1a10")
					.into(),
			},
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("4ce7b4192c0292a2bbf4107766ddc0f613261bb8e6968ccd0e6b71b30fad6d7c").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_kiln_head_update() {
	new_tester().execute_with(|| {
		let signing_root = EthereumBeaconClient::compute_signing_root(
			BeaconHeader {
				slot: 222472,
				proposer_index: 10726,
				parent_root: hex!(
					"5d481a9721f0ecce9610eab51d400d223683d599b7fcebca7e4c4d10cdef6ebb"
				)
				.into(),
				state_root: hex!(
					"14eb4575895f996a84528b789ff2e4d5148242e2983f03068353b2c37015507a"
				)
				.into(),
				body_root: hex!("7bb669c75b12e0781d6fa85d7fc2f32d64eafba89f39678815b084c156e46cac")
					.into(),
			},
			hex!("07000000e7acb21061790987fa1c1e745cccfb358370b33e8af2b2c18938e6c2").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("da12b6a6d3516bc891e8a49f82fc1925cec40b9327e06457f695035303f55cd8").into()
		);
	});
}


#[test]
pub fn test_compute_domain_bls() {
	new_tester().execute_with(|| {
		let domain = EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("01000000").into(),
			hex!("4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into()
		);
	});
}

#[test]
pub fn test_is_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconClient::is_valid_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				vec![
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
					hex!("002c1fe5bc0bd62db6f299a582f2a80a6d5748ccc82e7ed843eaf0ae0739f74a").into(),
					hex!("d2dc4ba9fd4edff6716984136831e70a6b2e74fca27b8097a820cbbaa5a6e3c3").into(),
					hex!("91f77a19d8afa4a08e81164bb2e570ecd10477b3b65c305566a6d2be88510584").into(),
				],
				6,
				41,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			),
			true
		);
	});
}

#[test]
pub fn test_merkle_proof_fails_if_depth_and_branch_dont_match() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconClient::is_valid_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				vec![
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
				],
				6,
				41,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			),
			false
		);
	});
}


#[test]
pub fn test_bls_fast_aggregate_verify_minimal() {
	new_tester().execute_with(|| {
		assert_ok!(EthereumBeaconClient::bls_fast_aggregate_verify(
			vec![
				PublicKey(hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into()),
				PublicKey(hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into()),
				PublicKey(hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into()),
				PublicKey(hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into()),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		));
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_point() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconClient::bls_fast_aggregate_verify(
			vec![
				PublicKey(hex!("973eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into()),
				PublicKey(hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into()),
				PublicKey(hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into()),
				PublicKey(hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into()),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::InvalidSignaturePoint);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_message() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconClient::bls_fast_aggregate_verify(
			vec![
				PublicKey(hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into()),
				PublicKey(hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into()),
				PublicKey(hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into()),
				PublicKey(hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into()),
			],
			hex!("99241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::SignatureVerificationFailed);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_signature() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconClient::bls_fast_aggregate_verify(
			vec![
				PublicKey(hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into()),
				PublicKey(hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into()),
				PublicKey(hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into()),
				PublicKey(hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into()),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("c204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::InvalidSignature);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_kiln_head_update() {
	new_tester().execute_with(|| {
		let sync_committee_bits =  merkleization::get_sync_committee_bits((hex!("bffffffff7f1ffdfcfeffeffbfdffffbfffffdffffefefffdffff7f7ffff77fffdf7bff77ffdf7fffafffffff77fefffeff7effffffff5f7fedfffdfb6ddff7b")).to_vec());

		assert_ok!(&sync_committee_bits);

		assert_ok!(EthereumBeaconClient::verify_signed_header(
			sync_committee_bits.unwrap(),
			hex!("a8a5ed4270ed6ab5a1341c12c26a7f6ecb2a1174956874b1daa038bfd5d3c61b0d4a9577579e6088a2834fba2c5666ef1870fb2c31cdfe6ac6f596680055ac69c72a5a164622b716a059b4119236524b130bd1f7510f55843b6114d8bc14d61f").into(),
			vec![
				PublicKey(hex!("a7b96861916795c6a4a8fa4e1faae26eebf4567485aefb562c545559cc1cfd4aa293839ecf87267ab4a9942083ba8d3e").into()),
				PublicKey(hex!("a499702e95273884cb939e54132c611c1971a1ac92366e50a4fe478b6be928bb86747d3551b778fc17322bab3cf35d1a").into()),
				PublicKey(hex!("b8679e8df90e7dbe42f05d5f79aff7bf1dd24133f3f64419c7f6adb9bfa7f149be6d955c1ea56298ff4683e523e69da6").into()),
				PublicKey(hex!("acc4ed9a2585b24b61874e34955571e5a0195e1ea4b564166fa185f773751d4e35194e6cc3855cf07fa24be4587a07f1").into()),
				PublicKey(hex!("99b0eb6403d9653bf0729a54ed499d714c8122853a2d5f2b27f161d1ca184c5375f2581130fbc209d568aa689a85c1b7").into()),
				PublicKey(hex!("8d6f3d7d8770d38844abc2f27797e8ae37a08f4ea86658a414ca0fc68e11cde785c85f0f0affd8ee0147b6415f00c169").into()),
				PublicKey(hex!("931e5efa17397fb4fd59502ab3195e1b3d1b76264f5e9b00c372295162e6e376ee76b853ab15168b2da2aad975d60b10").into()),
				PublicKey(hex!("a04d959a920db7863e6baccbc422d8022ce29e824c60e2ceb2c06a012aba0e14cc8dd8770e2b71e25d758fa283b8d84f").into()),
				PublicKey(hex!("b520ad20ce7028ee0689c4567c00e74239a7db155fa5bac54fc38415745da24e7cf673896f88db227bd8e0bdecd7911e").into()),
				PublicKey(hex!("91bee13412d44ba2ad9308bc4952f04e7fb5e17b375c3e1bf0ca0aafb8b05aabddc6496286a532ebabcbe3619fdb7f1b").into()),
				PublicKey(hex!("b2d9699f36278f0e5d9de921513df57ee8b976ca7d97a6c27c25340261418c7ff3cc4b3579b175c48d7bc0091665c8c6").into()),
				PublicKey(hex!("b45219ee95cf6bf3a91e1240daf8c822686695223adea30fd6ad7e3c5d79e6928c77ed056b5e0339a3d86a4df25fe44a").into()),
				PublicKey(hex!("acb1a43488dcce5711eb9e9d20f5dadb565178bd5ac300130026d2ee569afe90767b801239c57070c9e8fd16bb098ca8").into()),
				PublicKey(hex!("93b2ad22da9d68a70e3cc9f282b5b807312e8bfbfd75b5c1d6fdfa42410e75c4d7b6651adf445eb9381bd9c6a40f7386").into()),
				PublicKey(hex!("adb66c67377bdbd18aa39dadf13f95d3a7285b6896d2644f2d3e1fa0d35002a13c52817d95be11a6eec73f8fd33f8dfc").into()),
				PublicKey(hex!("8702d555c8b0681d00104ea822fe6ba8a298cd26625f23b1f33baa1056249ad36db0af751b90374641673208078bd800").into()),
				PublicKey(hex!("b31656a5012414649674de5b427a0cc63553bbc268c036473f73dfe7dda4083264a239b3b5e83fdac35fd18465f084af").into()),
				PublicKey(hex!("85aaa262953aba8a180c7e541942cdc2b448df3c7565648392d2a1c54b9b8d4ac8ba2cbfa912c79377ab98ddaa137627").into()),
				PublicKey(hex!("8707fe05f8c9d41a8def523460bcc08c00ca669803ba12ee2e959a7158d2b3a10ada1a70f6588419a3ac5f0e3a2a4fa5").into()),
				PublicKey(hex!("a1c1ad596c54083da36cb566779cfa8000faa740bd1f2882f26680d4b62fda2f68b3bd66cc8f8935d392a06df3faa20f").into()),
				PublicKey(hex!("a46107089c6931aab303e27058263c4abe0e523056f03cee0266fb837366b2f86e2a2f22d4b547b8f886f0d84561d45b").into()),
				PublicKey(hex!("a68cf065dceee51d1c51dbd7b0f165e2cbdac6a7adc593604a8589be7f425e7a87b27eeb88042d315fbd5b5b0ed40635").into()),
				PublicKey(hex!("aaa9f2cdf6c9ef536e077ba4266ada0ff64be24e40421bd6297532f4726652de7fab7eb1e0246ab611eb209569f826f2").into()),
				PublicKey(hex!("93d96345644445f85d8053fbed8b57e2f3ce5eade31ae8076b3d0eed956b2c58b016cfad288c5833f043b21cecbe474e").into()),
				PublicKey(hex!("b7dd04fa52434184bd2d8a45eccd14f55922ffae0edf3a66a29bbf0ef0fc2602dd04db8d2c55bceacc89ff5eb738cdeb").into()),
				PublicKey(hex!("96ba778c1a3196f1db9a5ff7698db78cf47167acb2641ecf7e9fae16f55f6827c574993d4062e104a09897011ebed69d").into()),
				PublicKey(hex!("b4fa25402333ff36c62e9ec97bc29969d98919cbf23f0f6b5f1f0eeb9025f46f149de392e1a60ccf66834bed472373df").into()),
				PublicKey(hex!("a6141cf3dd2430d75528eca0c8af44906970db63ec2fde7a76eae73d24d336c23d4e7a1ceef915e5bde77c8237a920c0").into()),
				PublicKey(hex!("8f8bf872f1e03b5c3a0d4c271bf58791bd7a2c16ee4a0dfa0923a96f08a1fd4920ea35a1cdc0ea3a352012fda9debb18").into()),
				PublicKey(hex!("8c21b63ecf5c0fce2b746bbd35424f59d70d4e29c2c19e34ea2146433e5daf4cce32367dc8fcb7627a5cb351805f842f").into()),
				PublicKey(hex!("af207fb14f0fc74da01aca8d598f6edbeb10be03ecb1f98bcb074863cb71a657175180c7529e86096dbed7cc3edc6d1e").into()),
				PublicKey(hex!("81310950b6e78cbab87c83e3383a7ceee23372bf8b9154b35e3efc0fe24d4c4e9f7659174df7124c87c179e33bf8e59c").into()),
				PublicKey(hex!("b0114f0e3fbad78510e3b0d806a8258bdf9a1999dcaae34e9c42f8e2d39a70ec5ee703bf98f17964f1dcab16b76de471").into()),
				PublicKey(hex!("86533f0ec1f1557777617f0311928143a34194facb5edcbe3965d8b1a26dffa0942ceec68f2088460d86b4a8d01a00df").into()),
				PublicKey(hex!("a362db1d7c414c1fcb6aac97de2f5b0d196f3a18f0b7aa30ed38354ed77105b774f839d0172579ffe5eccff3e9a08d6e").into()),
				PublicKey(hex!("b66c29dba821e8b26b512bbac39c83fdb8ee6342b1fffff8a93edbf25f39004ae9710773428993af4fb1fa836ced5882").into()),
				PublicKey(hex!("ab43ec94d60d03af473757fbe9b616ae76c16f8a63b769b444eca1aeaaa8234eac47dd18e8a5e659218f0e05bd0ac206").into()),
				PublicKey(hex!("b9a41bb066d4310abaa66be03ae8d55f96c09901c6925265026b0498045200dc37fab88231ce0491765ddd768afad78a").into()),
				PublicKey(hex!("ac548bb7a9c345a569076721a8fa9271c0b46fe9d369e6f52a4e5c60d12977d4b0c47528ddd8f7ca3ba29a712d8b5548").into()),
				PublicKey(hex!("84e000fe26a15989bfd3dc65a9795da51b5fb866f234b911d55a676dbb123f553a3341fc478c2a83a89ff892630a05e6").into()),
				PublicKey(hex!("b138d0aacfb165b444e8d4be9e34d7b5513f221cf418057b6969f1ae1d2966a556caf8136eb1a41f565b75a6288e1569").into()),
				PublicKey(hex!("98866552b197f456d82bbd074c2392526eee4d1caf3131e538182fce87758f18c115ba97b63054ed71faaf21ea6510a7").into()),
				PublicKey(hex!("90c244983dab39b8624e62c13996d9a3d19cecc7fce149214a5abaed5436daf365a3607700c807cb343197da681067b9").into()),
				PublicKey(hex!("b5f5c47e12907ae336967cbb5aed738f6e0669da00d36ad41d80dee9b51e612cd12719ac9992ad2be28443d1e9031706").into()),
				PublicKey(hex!("8ff468388dd4d134da7590711165ea4e4b563ea6f01783e80fcc0fb607a9e78f8bf04313e9319d470fb7050d917fb661").into()),
				PublicKey(hex!("aebd470292b011d01250873f907daadf2b9d9aa7580d97de3c2e0ab3cd3424f98bc3838374a4adbf48b4ea14d1905e0c").into()),
				PublicKey(hex!("8f47398a8e69a3e902f49d9e9138d94a765dfe75ee12af759bf72045be77aec72f0aef31e2bdbc606677bf6f4cb93b0b").into()),
				PublicKey(hex!("aa881c362373e0282518cec079b00477479a8c4cddb65d20110e18b70119d434ce971795c7e351630b6bdcdc2489b597").into()),
				PublicKey(hex!("ae6bbd1748f23756895bda4013f7da88654974ce8194cdbc50623d8b99cd32cc28bbb7be34d2bc5c78befab032828711").into()),
				PublicKey(hex!("a8db45dde7b78e6e3e9553701b0427de6501bad83a9a3b764dcc1ee53e0637fef3887959ca9806353b79980127ed3b13").into()),
				PublicKey(hex!("8c5aee15a907fc5f84822cb54e3a2321ea38ac5046f052efd1f91a56a4c3352cf6682e3b76a48491cd76f2ecfd49c105").into()),
				PublicKey(hex!("a6dac3fd32be159188d871b33166ee33811e8e63e49f9e3c77abc16bbfa5bde4dde4ac26835fb0ac3bea9e5e06dcb183").into()),
				PublicKey(hex!("8700aad2de2ad8cdf5958b733efc684bddd233cb845eb29c00847aa9b03278eaf6c9ac7c0aa0b5d80bfefeac69edf226").into()),
				PublicKey(hex!("8c9823c583984a0a73559816929a445a3a97705ec486fe3b4c9c93e17ae8adde8c9d53be7fca6481fe48bc27d16434d8").into()),
				PublicKey(hex!("b9c9a3110834ff7f2172f8bab4fc00546efdfa03cbb2621f27dc60de06cb2b4c206078c0a8c4716a3b797d29ed4ff388").into()),
				PublicKey(hex!("b77d597ff8b0d409354da7669a7c16df72ee32a2ecb0b9d8ac2d4f918b69eb62d4ce8ad49c35698e0be061e2f51fa1a7").into()),
				PublicKey(hex!("a941915ce628f06ccd25033d1c4c0f3c8d5421e31ddae711d3148a3f5027b8bd09cce5fbdc60c1e64a37313d9904c418").into()),
				PublicKey(hex!("8271cd73eb2cd6a242d4c22aab7e7a69a6fefec77b17b8f1a35b5c317bb9cdd9d93281b520a37801d60f4615afd61691").into()),
				PublicKey(hex!("afe45b4e07173359cfb87332ac0cbeb582bda773a14ed5a4834f2271bfbce51b2c095ebbdf41d5fa60085acc03f62c6d").into()),
				PublicKey(hex!("811885a6ee0e0c198d6d02e7bff5b9886f70e8a81139f24a7665baad350a5910b6600f6c7aa65d11ed4441254ef434a1").into()),
				PublicKey(hex!("a99acfd28f0f723293c5859c4be121eaeba7e0dfea976a828dfa796ef06a1d6d746139b270fdf4bbfb74262e4de55c4a").into()),
				PublicKey(hex!("83bb01575679e01866e2c125fc7a421079435db25af2fcf6d2b9a2116b0746a27a640b60297761fa521a09c3c921443b").into()),
				PublicKey(hex!("b8fee0e3ccb83b32fbe4743579d156ece5fb301d52e8da23db7348dbc0d808d456bedb44cb3d789c8f2f6265bea7eef2").into()),
				PublicKey(hex!("8eb18d0dc8ac500d0565581ef2eb6f894fada452c35a3e42d16eb0c8696127ca7d54c83563874087515e8d99addecce0").into()),
				PublicKey(hex!("88f9505a78ab9cf6bf7552296746e3d2b1814012a8b3fc5e397361af26696bd011b49b067a52f64071010fe4285b25cf").into()),
				PublicKey(hex!("8cf08a7726c7f8999f3d1eaf23ddad13fbb78bc28c63c966f59437176aa03c4121cee83d87fbf3acafb190ef9a9bebaf").into()),
				PublicKey(hex!("81d0137115939532723b1860d9d9c1d322941b19ea1fdcdede6257ed6c5126ba970f31cf46f95fec42fb8d4288aaf254").into()),
				PublicKey(hex!("9390e7b274a90337a4fd13195d0523574207504d07642f91372b059a16035c4f247c1f71c8ed0e36ebb6ce550ee28f9d").into()),
				PublicKey(hex!("b79f8b9ec56882bf17e908ba5c4fdfc8771f2ddf6224f7b79e9dcd189ea70ead7b1d2ffd2c148ec3000bfeea3a8b51dd").into()),
				PublicKey(hex!("ad2c54401dad688c3f3205b1572113357f03889594d996d37b98070c6124db0362d9914d6fc63ea52cddd22d4ac8575b").into()),
				PublicKey(hex!("b101038aef357b38ed0c83a8862fee2ea30094437a034c277ed89e207606fb5c44ab9e5493401b147b2050b6046e86e6").into()),
				PublicKey(hex!("8206453b14fa433c4e0713719b6bf895741eaadcfaec31df75e823859cbd4f1150deb08ceb064d5749d762ee54197636").into()),
				PublicKey(hex!("b92ac74cab944011878d30d95b429ad736dc24ac0cc52a46a6cdb8942c27f29a09414caf25a823b05d91231afd004810").into()),
				PublicKey(hex!("ac21fc10065567484232dd30fd8c5b76f7ef6be13bdc764d1e8847e95528b7bc0df40ce03559aa260cf36ea2292d3a36").into()),
				PublicKey(hex!("97146ba7f41348c2ee31f67b2a7326ae8c72715f3c16f462212fcca3bd204221969e44cadf8e0616960ec6e943e927f1").into()),
				PublicKey(hex!("8704c4910cd8472a24893f953415fc601ebff7c718b379b57212a56bf5c0c07e89db21476fce32bb2dcf7b49ee7aca9c").into()),
				PublicKey(hex!("b231334d1a8154b556600346744bcd9584b9f2e7a35879f556bc04954ff0f2a72643360ea90073f5ca0ee8470c255e42").into()),
				PublicKey(hex!("85aac73931accab21136a9daa2fc47b2c1d90783795a9e079fae7cd34f4f5590d33b130d7e75b8c1d0cf63e403bf5836").into()),
				PublicKey(hex!("a1680e0fb4badd2cf018145419a10d080c32e96db776cf62c743298ac80aaf2ecca63d09e4cd1498ee744333c9b13e9a").into()),
				PublicKey(hex!("a1e7831bdc720c316a41326104002d24c4535abfe7211abc759cf94e369158739bd4083d1db5ad2f5a5298e55bae39dd").into()),
				PublicKey(hex!("80f2e329b895f4074dc80441d3c1fd3dcb776d5426b0dfcbaf13d031daeec98484758fa247fc423757759cd6e5f57067").into()),
				PublicKey(hex!("b4ba319cecf424433d185d7426e53bbef2fa84b7c961adeab9235ab895526bd6ffb53208a24699abfe2283eb85b67c80").into()),
				PublicKey(hex!("86a17cadeab3f5ce861a7f44c7810b0859c7b09d3e09662fd49dce59bc50ae2a16d18ba1fb5d613ab0c87920e051fabc").into()),
				PublicKey(hex!("8ab642bd69a33ce36f780a9142c1f34d0376f7df2d291a94c2d1186d2ac1d119c4d9382dcc3a024dcbb2b22966cb2869").into()),
				PublicKey(hex!("b5f37861763f448633da06f3a750af981a522bbc39604ec6d450b822e1471207d8c808ef2fa488e8c495f09f6476605a").into()),
				PublicKey(hex!("a034c1a8f1fe3b99d7cf7c27e01265ee2d41cadd855defab428ce54c9345884b665df499e514fd68795ba9b94d5848a9").into()),
				PublicKey(hex!("93519d4bf5c3978cda90bf45acb5146d2c206f534d713886cffa05b29a041364ef4b666e58fef8b5f292e674b52d16b7").into()),
				PublicKey(hex!("9518b4947c068d9e68f9a27aed77f3975ac92bca160a4a5f5a078e8ffaa49b43f5250b8c0a13d2300ba96fc70a57ee74").into()),
				PublicKey(hex!("aab25fb313921fd770850582f7d436ac21f5c878eb90f21e6427cb3b9aa048a2e34745aa96d89f71a0cf626c8e4387d8").into()),
				PublicKey(hex!("830d3894c7e6057ca2e408bf03c54e3e8790df3612a10de7ca321552bdbdb93379b8df3ed54245cecc068ec64b120ebb").into()),
				PublicKey(hex!("a19e9f21a811767c05d1254dd3969c93685010f1dfa83d323879d407879105b602867fce7033f5c811221bfd685278d2").into()),
				PublicKey(hex!("b633d69633a55e32e5c6de9eafc0a0dc41deebee7bf0ad202ce434c8af6c9d4b4d3a3400805eff51c5d296de9d3a8a06").into()),
				PublicKey(hex!("83d928717f8f782f3b26a0ece0e742560c35bd1f7997d0887eea03286cecee11eed82746ce0f8e5b526414bcebff6443").into()),
				PublicKey(hex!("8bd7f8085db3a8f5ad6f47d26b0259c92904d5eb8ec0de1faf8c927ddbf1177a9818452608240de8dd397e399ca90615").into()),
				PublicKey(hex!("8bbd7eb94cb99a6541669dce9e2cb9b80788fb21909c2d7431572305a9ffa2f2ab232c8d3008b9f5fbbaa0bff9afd028").into()),
				PublicKey(hex!("98aaafa53d912c555be571e2e818f1da2e23a5f6c320bf071ff570e2d0f7244b8b36a56b342fdcc6cc6721481c2e4a5b").into()),
				PublicKey(hex!("971b7b4f358f71f9d5d8f63eeffdf51225f20f26767b66e9a107e243f68eb0a51abb35613c523186bf869fed4c59ec68").into()),
				PublicKey(hex!("b817f46b133c1df71388fd3b1706b02e27a2c3276cd0ccf3b8e7946f9f3cf72ce5f4ea0922e318f32de416b2ffc18b17").into()),
				PublicKey(hex!("ac8d2860f7741d97862e4794d5cbacea8501e1796b10d33cb6a4788c7695581e43b0da1440bb40636585f96bf4dc1e3c").into()),
				PublicKey(hex!("95569c6489850dfc792827ca7004f4c1ff94aa96a6a80952ed7e49926e099a1490a74fc5d79a147e9045b7045104d656").into()),
				PublicKey(hex!("b4d4037879dcf997da195bb4fa2dcd4f04c04abacec87a6cd4ec1b006b36f69557da67c430fa6ccc52f4153c7d6c9ab8").into()),
				PublicKey(hex!("b0930c5637241463e45cca3679ae7dd497fcbd90d4c505c907139558d57be9d78a2ae7ebd4bce16ce974460fd27690c0").into()),
				PublicKey(hex!("81e6c33ed36f1c34007510f70eae7a19b6fda87094c4296a70b983bac23b159b345019e3d512bf37f53f6dc0fa2140b0").into()),
				PublicKey(hex!("9604e0cd1455045fe07de05a939bcfc0af762f9c7040c24081178687d37e903572a348ef09a2c126ebdbb0dead2fdce5").into()),
				PublicKey(hex!("b23e38113bb9e9320a5ee3eef862da752fc847d126bf2d0a37793b6ca25b32401605a0c31f013079143c359f3b48b0c0").into()),
				PublicKey(hex!("b34f1897f6e83685a4436e9352c02eff1b26ee5d1ffd32dfdf6663c41d8267b3272525783fd24d991f29e73941d63c97").into()),
				PublicKey(hex!("9954e00ab2572f0bcf75e46cd8d58f4a7f66045800026c4df10998c54502ed140feb70bd25423fe5313a453d126f4a74").into()),
				PublicKey(hex!("aa9ec96d7de39501e6c06e995ae11e343bd48f11e8fc864811431e0ea12d5ef5a3e1056cea4c60c7f56328aedb3dc970").into()),
				PublicKey(hex!("9762fb2ba80a293c181657bb5c421b4c92232ecbed3f40b8a28840a1f985e4226234d9588489fb6f2a0d7b81642acb0e").into()),
				PublicKey(hex!("94d3014efc65a2a93b96f53f5a9849457750b9e1956c0f0bb27ff1770035e457c2bbb8143816c9c9f6a754a5131ca495").into()),
				PublicKey(hex!("8c9782f1ffe16a2b273551e4ef7151b466725949cdff667ddaef51c979bc4756b7259eaccf51268dcd76b202d83f9c3d").into()),
				PublicKey(hex!("b596ee86f544e13c9d9e1aabf804b43f8b0f2446fbb5987c05ac84ed0b3e88fd581a5befdd3c328917e1ed2b21e77761").into()),
				PublicKey(hex!("80f87c030fe3adca7fea23fb13b0f6e4f8cb20fa69c6f40ee838eed56a2c04273ba23cafdad251efb642992e66c1d260").into()),
				PublicKey(hex!("818a258f107ad50468104219504822721aa20b3edb54379507b7436ee06a51793e2a4d2e472698df25ef9099b9a2c4a3").into()),
				PublicKey(hex!("956b9d13cc9c4b6a1da382f250d0d33d26cd6faca3746b8bcd5fd31cea0a2bfae02459cd88f28edcd6aad41e162c6270").into()),
				PublicKey(hex!("8842bd9a327c3e19003dc81b9f4e1c289187d9a71663c54ea8b56f8683e51a0ac89cb52af127bf09bf57a083307891e9").into()),
				PublicKey(hex!("99b7b4dade5f9cf28805a604cf7757e1776d0d9879bec5faabda63caaf53d7a4fba456c07ad46a0b72697c8e6160219a").into()),
				PublicKey(hex!("a7ec5280401e10612ab02a2e0fdbd3b29545731946cdecc4296b5143ee898e6a890afd6c7f6c892c8f5a14bb4a67fa3c").into()),
				PublicKey(hex!("83be0a9069cbb37c89a063eefe80f7480a0c80188c2316986e303fec942e2c8b003727819f7c74842cc6b68fefde3c2c").into()),
				PublicKey(hex!("83ee4d2296ecfd3f850c6a66b40eb1b538a65a37ea3a74437044f86e5ea62e46fa9774134e101f959eded44cdd5b2900").into()),
				PublicKey(hex!("af8f436f2f09affcd7476a4eb35367c46ca28ac739d8cd9e6d74fa18c5bac2e85f2c20eeda91ab95ee3d58480a439af2").into()),
				PublicKey(hex!("85773ed3e0a5b7f388fcc3bee6c89c5b1028dadcf7b445475c7bcda101692f38323d2333ad522b010b8ad12062644f1e").into()),
				PublicKey(hex!("849df9562a4928b1ea92627f93d82d8e4ecfca66b84b62fda384f12086e535000e04309be7dad55742e3ee7a2f7d4a76").into()),
				PublicKey(hex!("9388589e22e278c33a6a7bac8b31093226ee85512d1ca8b40aea6da00f263ef8c58957d4c588fba2e42224013d8c1479").into()),
				PublicKey(hex!("8021d1df26c603ace5a0d3362abfcd35c3bf154642273c4898f7f86b1291c1a7be04f624c6d2f25d9a69983405412361").into()),
				PublicKey(hex!("8a1039e0083cb1e17df1a08634ab14ebe32fd7b2b0e923e410e937ec2bf9bb768ac41d0811920aa985a033387d0f7b53").into()),
				PublicKey(hex!("8f65af75680ebb88a670fb42d82a02a1642df31801c95d47e1ffca3afccfd5db6080fea1daa2c7bb44e3c0e672261368").into()),
				PublicKey(hex!("b8ab63c6ed1251250476a97f67945e5c6c21882862b1c598c957168d250ca84a28a8a5a93d244349ef4590a0d7789d1f").into()),
				PublicKey(hex!("97ab28d6609a7432ada0c04285cc73386248b2207057d4d0193fdb90c0e57449bda37f57a000051bc97013b19c437896").into()),
				PublicKey(hex!("8b80d59eb30d272ce0756988dc14730a11f4d511e71b457e452ccda7c0a1645e48326699217f2feb08807b2d504f10a9").into()),
				PublicKey(hex!("af8ce2f4d85025ea655fe863c0571aa3c296d5dadec689a4b108c951638ee1e785cf883409469b4ca4b0ae59b2ac86cf").into()),
				PublicKey(hex!("950d3733157558c2112af653a13efc45957a5c1afedf921ea87d835337518a39c4a407d65679055da18aacb8aa4fbca3").into()),
				PublicKey(hex!("81c77f3150ea545e758669e7b5361310a8de06f6130d22e60bb53319c039c6a5de25f0690d57bf85dbec101c693e7d10").into()),
				PublicKey(hex!("b2d3a293ced76d899929837635a7727f5fe0737a909b430b255d6c649be3395c093018d6b6233791fc6a0d18df3fe2b6").into()),
				PublicKey(hex!("b44fe640f1a2193e1362753f44d62bf6180d83fd44fe38a85667f131c66fe0b63ce1e6f3a2fe82f5dc609c0b504450cf").into()),
				PublicKey(hex!("b8647ca1b1991d39f6b404aa94049af32f39c44eafaddfb59f9683d01b32e71671bf91613b31936e322e538f7ad09c0e").into()),
				PublicKey(hex!("af9a3ee9e934083d47470e82cfa9fb98da0380be709efb91fd6fb08b8b4072a315af5e00827478aacf06dc1ff2622322").into()),
				PublicKey(hex!("ae50281f792faff1f00fe2b5c3f1b6a33758be36414ebb5ff76ef3526a9985d56ff49ee3fc68a8c82a3a20ede712101c").into()),
				PublicKey(hex!("b3e4a91dd8bdc06efd624adbc5bd93217af30bf3bf48a3773f1dc4ff083480e41e6e946f2c3c51c0dd9fdd55ec4534ec").into()),
				PublicKey(hex!("b0a0dd84170f6c139a179a5a176fbcff8a0131d27b0228cedb89ff344baaf8a8af28e76eb94969fd9b468d17159e537c").into()),
				PublicKey(hex!("9667fa91f64922eb7cfbdb8e887660c4e949229703ffad54b744b7cc55cb128e79ea09aeb0ce05b2173499618e6f0273").into()),
				PublicKey(hex!("a02c7d9dc18bdb5e928fb039d678e068ac9cda7dd694564ac3aef0b389dcc2dd12ad608318e9d04b81891e65b3704760").into()),
				PublicKey(hex!("a738f76425c03712fd72d4c518a2160101ff1526639d0e8878b1f09de10253e0092363ebfbd1524c1291912c61f84990").into()),
				PublicKey(hex!("a50b544c97f4a4ef250b0ea7729ec1aec85758b4eec18e77413e14bce0d8a8948cead4fd9a72218796e68fcf4d527d02").into()),
				PublicKey(hex!("aa77f825d530a8b894dc69500d9744f19560ba98687213cfc442f9e2867bc18e5d8efa37d48a13fec6ff79d04be35f44").into()),
				PublicKey(hex!("adf78fe7e507ad8c679b23290348f77c7746bfa6c335cd3291ff35a1ed3b6e5c467815977265d0df5a37927b06261cd2").into()),
				PublicKey(hex!("b14585be28ee8182875152177b00be82a4abfd243b171d3ce60586900f336c0be4eeb3e38a9e2b9747b2f1ab062a2b18").into()),
				PublicKey(hex!("b16aea4b1b9e5270b92a957424eca382f2186b4065307ac4c2c1b616cf2af8fd8a85aebc3e11b27f5f2e17bd1d801f39").into()),
				PublicKey(hex!("ae4e02620bdc62a65453fd4d2d199207011facad3fdb97f5e62cdb8a60b0f419f4d6421f35a90eae557c8877fbfe5e29").into()),
				PublicKey(hex!("a956257ad65ce2caeeba765f6e46530f4d39297eca6a76f997f057e9225892e4dd65d706fce494e95beff5db29842430").into()),
				PublicKey(hex!("a1fcf17c5cbf3b4e35d9db212a304012621d79f272f2677ca40eee4f7b60030f8f2979165c57ef982345ceb91ecbf00c").into()),
				PublicKey(hex!("93e047be9c6e99f57bce7f492a30fa652784179e5fc7dd7f238047c3e1389b4556fbb166983cd390d4270389664b72b3").into()),
				PublicKey(hex!("b359ad64fdcf0d046d2e67edf6381b7809c3da5a16ee01aa70d0355656908b8b8110e1e5f9eeafd6b6a504ad5ed4bda2").into()),
				PublicKey(hex!("a6c4e04ef75cbdca87cd01368dc12261a444eed6b54d0f451b54658a217c257572e3bec4df8a7ab1bd501fff2df37c85").into()),
				PublicKey(hex!("824e49e2e38a18ac93cc268e3e7a05d81c12bb68216b96fa3a349126150456879da499177a53e8d588fb22c3b44616d3").into()),
				PublicKey(hex!("ae9f0bf65735b925bf42b42c57c7be479c8c343dd0d127897f4d6e30f4f1d625cbecdb8422f4d0e721ed083d768f1795").into()),
				PublicKey(hex!("b2d833a2a5855ff318e8b31368f2eba8d157520f8ad84e2ae7b5e3114aa2985cb8a9ab3b5e6d1ba2077fbf0ff4dc4257").into()),
				PublicKey(hex!("80e088a9cdffcaef75a5b9e28136330bd317312aebae743f935973d90f1b5232214918abe6fb5d16a7f4dc53651b4235").into()),
				PublicKey(hex!("8f12b5b3f5faf41a1547b3649b7d1c0d8d80d310f77409de6599e40ffb00f5db13f74d4ec318ad0d69b6813c672c2571").into()),
				PublicKey(hex!("a0a8ac776fe3cb0a82e5945dd0238932f7e51ed003ddd7a18758b30bcdcec2c8eae664f09458fff63b1834ce15a9b8b3").into()),
				PublicKey(hex!("b935f843a7180bb6855acaee097bce159fe632715c23cad3ca3da4af0ec60aa764357f5810bf611d03ea6b5d3f58aea5").into()),
				PublicKey(hex!("838674a46c83224701168729620fbf28227fe9af19e60f082415ef258a1abadeb3b85cfb0aeb808678cb7404bed35d2e").into()),
				PublicKey(hex!("875d00e3acd2127328827b2edf84303e851c6799eb54737577a8939b850c03c33fb89f6cdff7b276bfbb6d5369580e57").into()),
				PublicKey(hex!("b0c4bd878a7e1235e26de74321a4719c355fa387a99c04cd9a615c46341734c60cf5e3ffd4b2460958f8609a87db2d97").into()),
				PublicKey(hex!("a6f09d254a28f54d4fd010f7861b4eb85cd3e694f2bea2a229af63677f31388b7f60c919e40c1c00684c8d09e3c9b0e4").into()),
				PublicKey(hex!("b65078455f112c11d9636382ded841830571dd4d8f124b220a4c38396b2870f175773eba64b8c095d326d95eb26dcc84").into()),
				PublicKey(hex!("8f5a7bd1f2fafa24e45531352a25c35160981b471eeba47c8a2a660a3fcecf80dd20f4393fb22fb1ab7de42670048a2d").into()),
				PublicKey(hex!("8a0e59496e3eaad7393fa5bea6a81f0586a335cad9286d3b21c4c406e4673305ac6933680e083c2acae4940daf57e0f5").into()),
				PublicKey(hex!("94e3cd1d5a4681e9035552abc9737fa856620fb3a387ada78d583122266f82a2d380ee25df6d28f4f577e30514d6b444").into()),
				PublicKey(hex!("81be12ca367c6469c38b62dc5878c173a2c5765636e4e18a92a518669be04f379612d0a7088cad1a468d9296a909f5a7").into()),
				PublicKey(hex!("89a8cc019f5d8c315dbffd284f484b0270fc49674facbd19fd9d5a758044c35c452fe514d73b2f76a0842e239838dc35").into()),
				PublicKey(hex!("a1ada639e546903afbf55e5a762cacd5933401486321c872eebcdbc6f08f3364377846215e22372c03fe858b641edbd1").into()),
				PublicKey(hex!("8221086706072db120e09fc22f7e1968e63c7a80e91125b702f8bd16aad29241c5f3b97895cb9b19506442973bc702c0").into()),
				PublicKey(hex!("a6c5c3a521adbdf24366bef94c3d41a893a5994b4015042617144213790f544bd2faf6834de16e890abe808b90a9ef93").into()),
				PublicKey(hex!("a60d8f4fe74e63f57c1c8ca544e65dd900f0963a4d537d6f8effd57424e2e43c01b1197468e94faa74834b59efde136d").into()),
				PublicKey(hex!("a1084cd3ca324757cc79faf890d4540fcb0dba0a049b2a58c2c05231267bed1dcdedd8828c957092d4c1ada59ebcd600").into()),
				PublicKey(hex!("89a9742225b25f05004eecb21751df654a66837e4d8310a34d5980df31c27641f740c287ee6616092ecfa287ee14f9d9").into()),
				PublicKey(hex!("a0343878d3be972137496c7c8676304238d7c3f9fcb76e2bb70b6b13664c041bb8ca59a9ce87c226e12111e84e0a2e99").into()),
				PublicKey(hex!("a81323c130337e9a4b640d1a67ccc9a5d44b1c0e4a1843cdd4a835a0a86e25025226cf0c27856ab3d7a698f19af532fd").into()),
				PublicKey(hex!("94e59f3ed1d682f6787d79db4fa38d87aed3293a04ac0d0b5d4d18db19703dd3424de76bbeb19f720df712f3ee9af495").into()),
				PublicKey(hex!("b2ca0ddaf7070f61d3e14c90f0c3c0a0a929ef36e15b1b4932093d6a4032136fa82beed014b542357d4c0e7aaa01ad72").into()),
				PublicKey(hex!("8abd9076a041d7346cfd549943627ef74a55175331caf977587fdd90e9492325f9f4480988ddb4d97e301616dfa67204").into()),
				PublicKey(hex!("96be10a3fd7a236841ab0eff3d30a3f91d9ad043ba305a8cadf25652f0f7a45bbc61f19c0edf93c30b0c3672163c159e").into()),
				PublicKey(hex!("918f08beacd02cdf0ed279ec6909302fd93b2b778d6c7f13442a1a6d730968ffb9738d28c313a8dded4f4c8f603a74a6").into()),
				PublicKey(hex!("a4ef5c700bc94c92294c4efd7925395de4923b29c48c4210c47acb6935b877f50af8f15e6ba650185f1055b67b523716").into()),
				PublicKey(hex!("a8a174a2ed797b2d633e38b9325419a105566a9ecc88d1e9c30649a280be2d4759cc1ebd2f2148d87f32eb5d962df9a4").into()),
				PublicKey(hex!("a01a2f500f0fa1e90af942186dce0ef8ee6a0233041c66b3f91f46e645d4ac5e17dac435d0e667fb761be0d6dbb5d6e0").into()),
				PublicKey(hex!("9335fd76139abfa7ed038d92781d5adce0bafa3dd57ead0cae7894f2ef63f846f3c208c7e1458dec694653c8d37ec6c5").into()),
				PublicKey(hex!("a3074e72316f8f21fbfad68733a80a596749decec8f0d5127d6b81576e21fc8a1a337dbd2b8886781e7967cf7d90d9e4").into()),
				PublicKey(hex!("b4e40ecf2606dec5e596e8cad3f4a210837aad603c6805d8c55a253a98272c8d7fe5bb0af8a25d1fac0c08671d270b93").into()),
				PublicKey(hex!("8fbe2cf3d9e411fc7c56c2212c4b649c446dc1366f214942d4e82234e288489c1d500e05086591fb626a873ed34a1aa8").into()),
				PublicKey(hex!("ad490dd5f8902e8abae9c8a2e9566433cc9323aee1262db6b62ea8e50e2049e23860323d2651cb047565148f815d49b5").into()),
				PublicKey(hex!("a307dd9b31b478fd735b6a243ac6092a237d342d8ea15a6336a65ed07855caa16d358a0a640c5f7a6c7ece6de9626d3b").into()),
				PublicKey(hex!("8ceeb9b050f33c6e0137c5db6a006156f741881f91cd29cbdc1bb3d48fb0cdaf21f06cb4cf5d25a50ec834f1f9fa04a6").into()),
				PublicKey(hex!("ae6d03f6061863f8dacae74cb6754e3336ce854f9e1e7658a1be1b9d6f3ab7a5eaa1a6cab2a586a01ffe9d864243566d").into()),
				PublicKey(hex!("92e0f98ddddb7404749e98c0587b611698534f125689031466ddbd6ba316de185e098e94a21b398e07d6e913b0b68f63").into()),
				PublicKey(hex!("99c28a40a4b33064bcda608aadaf5d02cd5cd51e51bf03c178b790f163aca4472057da0e0062f908144ae5ca8c43a14a").into()),
				PublicKey(hex!("95604927235cdca490dec7882f762fae6af74507270c579cf732c6b3c9fc59019dc1a9e4a71b75d541f94097d88ada74").into()),
				PublicKey(hex!("a9ad2557bc0e25fceb9f4121172fb7dafde3af500f17811bc2aab057b6fa2e4a7e1f62165cdf600231f39cfb71acb587").into()),
				PublicKey(hex!("a86d0ffbe72b5f357366db24952c9a03f97ca0ee61a00c94302f6c2b3b7dc23dea0e7d2db0d5eaa624608b1e43bdbe8b").into()),
				PublicKey(hex!("ac9ecc568273a8c5ab721ee8e8d8f9650d976f83950233569a2441e6214a96978a1319191262bcfb5784020bcb3fd3e0").into()),
				PublicKey(hex!("ac7bca572a86f7e21e594cf8603a0ff02879fd8999da8d09aa7a68225aa80d80740c2c6bb26015ace95b1266da3d62e8").into()),
				PublicKey(hex!("b5a451a1cde344192519a15801fd9f0b7226ed454d796f0565b7588c6800d0234004476931fb487147f9b3e19aab328c").into()),
				PublicKey(hex!("87b2dd1d9fa2e16e5f44d99d21b4328a1c15a7e16754d565fa64291eebcdceecbedc790a63152d0a8780a694d083167f").into()),
				PublicKey(hex!("b15bdf686fdff8d520ae661904ed248b7098580ae0fa59a81e5d3596cee6c30b97cf3671bf5082f9652ba0bd6d0e445e").into()),
				PublicKey(hex!("b1273645647b881c6c2b1d04065aad0bdf274dc5e22884d1a5bb5f10ee102259cb94357483f8bd229ab52a9e1a8032a1").into()),
				PublicKey(hex!("9179cd69c0ca9c2651b4e30343c826a683eba5474a4a3192a84f739eb49a3027951fa8dd8fa34f3be5159a665240e7ac").into()),
				PublicKey(hex!("b08398f05c31c50d40ed7cb87a282be2c8aa86fcda01a8752d824916631d902a23445a179db56fd9a24822f7273323e2").into()),
				PublicKey(hex!("80aeddcaf90d61b94a990c69544d5337d016960be393c2aec665c4299a4c6ad7889fdb5dbd44b3707b83dd4e50452b53").into()),
				PublicKey(hex!("9422a79d2dbfb03ccc6e221505d3e09578126b98758a064562973dde74281ae88e7e0dd5f40a1c8f743f24de7dbfc276").into()),
				PublicKey(hex!("a73d36f7b7113a6f40996910710e73f2962c5c2770664d5702bb0223378cbf9ebc842458327c3616f8ca4b4d9ff9e8d4").into()),
				PublicKey(hex!("92fa0199d670e1871d55b5bd5b814e540adccea132cac5631097ff3623833f9c16d7a95cc1b9870ee8919b8c047330d6").into()),
				PublicKey(hex!("b8d859b6626fced3676e30ce82c7938de2df09e13b38d79403b5d00006708290209438c50400a8950e552edf46cf7191").into()),
				PublicKey(hex!("b9ad2b2bb7e3c11c1123c1d7c5caaa168b083d00fe496a496a0ca1e5dd6431e8c7c2163b245deb4b0d0f6063f07f9b84").into()),
				PublicKey(hex!("b080b7a308f1bdb10bcc33900386f37da74b0f6ba1f9cae0440701de24ee005cfbbe192dab40b77a1860647df49b5e7d").into()),
				PublicKey(hex!("88e4f1ffe63efb1c4e26bfe1079ede503d4bcc208c496e45abcbd57c4d9b58876aa793823910b2315ee68a6937aeeb9c").into()),
				PublicKey(hex!("a81a3648f6d58cdf46ae72b4f69c569f593eab126abe8274669b27bc15142388c619bcc8c2be13dc66c4b6e4eabc050c").into()),
				PublicKey(hex!("af4ca9d6029f03d3ba2a325a827780f708c878b33db19d34263698355681dcf5a2278a682c253af1b6d2d5bfd06b9627").into()),
				PublicKey(hex!("a572e2b3309d2d56a47692f35cc78f7697b8f789caa8f82a82f2b831799ab634f3c6aa582e764f7e117cd737b07a9f92").into()),
				PublicKey(hex!("a2790f4cd051891bb36dfc3c0f735093289a17dae1085dab5ebca6fe417070fee764e76ad58976dd8168ae643a3a1e93").into()),
				PublicKey(hex!("8a99329161ed7a130be9df9aacd9afafaae37f73540b70d71d1d147f2824894aef3e32eb5bc345d9f6976e6836eab4f1").into()),
				PublicKey(hex!("b9ffe3cc0530e79608319753b6e69ddf0e5e7746fd105e6f2f37f441d3a5ff4cf00c75d68b1d6319c29218eaffe6280b").into()),
				PublicKey(hex!("ae576d3a05097a5e01017ac0da0ea78d447979ff024d2e7f3f3d74036864901ce22a1bdc8a08659891b0e4067ef4a773").into()),
				PublicKey(hex!("b5e9b70fad901a4f569518710851026da17779622c92d62d82cde3cf6ca4a7ea119a08df7fefd76b4aad59378dc8f047").into()),
				PublicKey(hex!("a91ccd3bc98c10e9432da5b2dee9e64104346f090fa123d38b707a83e2c1c5879b197c293789080f4dbe8ef8e02061b3").into()),
				PublicKey(hex!("81743824a44668918e9d5f136baedd2773c816c5319e5f1d675b722318957179cb41f16819edc488f06356fbb9223fe7").into()),
				PublicKey(hex!("b79bb5d72181b5ff8f9c6c8ed245a12e1a4c0b499a0127a84fc17af40c0c6a4de595c39dd84143b4af8574aeaeafe7d5").into()),
				PublicKey(hex!("a2df5bde21764ea80d3ad9cdb7664aa327368f1313ba8ceea2d2a5a0eddbf042a7c8bb4668101f643af3405421667544").into()),
				PublicKey(hex!("979c24fca3e1d265361900230d030d7e631bd78f64bc7294024ee59f35ec322781b2f938130d33b734cddc12df1bd99f").into()),
				PublicKey(hex!("ab7e22ca8e3cad04acb715e48eeb661c74f52e541076569d013cc38e5708c6f162d0b87f87b8629a3cccda1acbebfcf8").into()),
				PublicKey(hex!("9400122d701faa6e48b254e8cd7fa0aaeda1f82ef29b648ce7185a7af4d7d836f8a09639e2f8175dcd25c5942bd030d2").into()),
				PublicKey(hex!("b3b8a96c37d0b29816eccc3cad34ecf88d980d36adcc522ed22b4ed780e44ca0f01c258dc6f8cf8fd5b96a4f28ea7810").into()),
				PublicKey(hex!("b8cca17ebb5c95c9b45414716ca39e0a306964b7bc57a6621b31e7527d233d68fdec7a5ea5e351111b206e0e5e22ed2f").into()),
				PublicKey(hex!("824c5c8be7e667d8294aa9ddadba4936f9e6591070e0a54ffa1bcec5c2a5d1677c96c54252875af3e51adf1573284af8").into()),
				PublicKey(hex!("ac8f5a5032039b217d8e90e56bdd097eed109554c5a319694107d24ec6a35d534dcd89917305cb562fc4d8bf99b6315f").into()),
				PublicKey(hex!("b1fab45bcf8ee44835f36bec661fe022e20b9f6d81b5a63b3e9c1b4e6c80e36dd9b223da625ad2cb067d28502a06ec41").into()),
				PublicKey(hex!("86daf9846e72e971f42386bcf1fd43c30daf35dc2e8947a4153ff971ebe91a464d126ce75b5010264345901fa1473152").into()),
				PublicKey(hex!("acb43f40146a8d2c7d913b3394cc508feddaefc97e7623d13a3170511c90faa616966f3042bc191cd9c3e35f36748e0b").into()),
				PublicKey(hex!("8035a44c0012d1bdb2848869bf3f5a023f46b0ca2d0379fcae6da6d30964b768618922bce470a07f5e12f1b21aeae9fa").into()),
				PublicKey(hex!("ad7492d364a3f04baddba49d937cd7c89f2d9100010e56d20eed366e785fc791ab06eeaf48fef28b62c0e9445bd389a7").into()),
				PublicKey(hex!("92c841b66a24a5e11063d4401a2f739cde540e71cb9fb89f7efb38156871352b36d13bf69ad15dddb6e195f6058c8e9d").into()),
				PublicKey(hex!("8f34fd5872371bba5b4328621897e4c583d4df15540fa46adff808eb043a2236ae0b79dd95a6b01f5d4db98dd607727e").into()),
				PublicKey(hex!("b57b05f60266ff6471843655d33d248378bd9602f83919d1aa0a33c62d95a5277841d685c12133b7d4e0f0eb218fa150").into()),
				PublicKey(hex!("b9c696c55a2173d12df67df7a2518e4d9f9a963c6bb5aab37aba1ec1d675e241673c2a383824804d03d2dfba11019bb5").into()),
				PublicKey(hex!("93ca510567991e2c135dd70e3d7f6e38207ef4ddf3d87ec545bf99fde27a22452c432e2733cd00dad0ce25b1a7fa2dea").into()),
				PublicKey(hex!("8805cac63aed93fc8b3397266107414364ea583d42c058349027c4d3989aaa07ae54100b01259bf892d2cd1b65f6ed56").into()),
				PublicKey(hex!("978e5986e54325f907490d03c3582e10734db73e347bb4b5d6c78890c020cab7a12c9c5583254ee292cfb04fafd34c83").into()),
				PublicKey(hex!("b5ceff871dc1cf53ec4e9afc1018947c7eb5a6d7a39d7edecb9c309d691a8016be49b7050be5bb7bdccb76fb9f495d00").into()),
				PublicKey(hex!("a39a7f9fc8965df1a1ccdb904e29738cd3d62cd910603008637762a84298e9067e481c5d5dfb56edbf78584ef9ee9dc2").into()),
				PublicKey(hex!("ad2373df7f5a225ba7f11c6ec594d79c15b999ade9f2c0df5c3edc64884df98f19e9372c2253208abcd4b4e18e98aa6e").into()),
				PublicKey(hex!("930b845522b1da7d8ac54bbe35aa3d086b6aef52310fb4dbbe990c525a5540735b9871c298aaa817e449bedc88ac35ba").into()),
				PublicKey(hex!("8aba847de70fbdc90710735a8d536d8c9ad581adabb6022cf21414a98072deca804ed21152a2477f7d0861b73eef2542").into()),
				PublicKey(hex!("924f34c4728f756d504399731b68ebca0dc9fce0a185fa0c5931915902064f4daab006d39091b160e330007ee4f4a6dc").into()),
				PublicKey(hex!("80551f6010e7fc1bca35654babee2d5cd42763084b2af0f67a07faaf2e43f42d4b9ae9402b26f9c7667a7eb1ec39c145").into()),
				PublicKey(hex!("970ff1d894e807347c8ba6b06fd10a2f6dd6ec3a81e01eed3f3775e3362b83f7a9d46358e9293151338f38c6ad7c4d9d").into()),
				PublicKey(hex!("a9b071768e13184a5578f000ea2e8ab42fab7ef2804c819118e1510ac6fcf91d46f2859b97d711b174e389a48af82874").into()),
				PublicKey(hex!("8d37762806884f2b0e7065c4f8ad24ae5c5c51b61ab59fe0acba898fd7636b6c7a7edad9694242f228bcfa332844b743").into()),
				PublicKey(hex!("96d2bd2d01627daefa5e1bdd087bfccaca3911f628e86eb700b3fcb6defedbb665f88ce525935508476b81cd52dcd49a").into()),
				PublicKey(hex!("b8d554414406bc424dfc39d5a180c9f1dcf5ae6502d637c130a2de87136b0ad39b6db6eb388586b1527866ce26f531b4").into()),
				PublicKey(hex!("92b8d502e1b8727d80e64abc557c04c8b48318e42eb527f70c3f71c38439cce240bcd73dff7e32c4b10246e7331dd841").into()),
				PublicKey(hex!("88d2a60e26c485d5692785c6d30de30a3e070cad15fdf3a18827519ede011f858df77127dcd0f9ace9dbc734d0559171").into()),
				PublicKey(hex!("81fa93ae005a1686ca5874398c30cd853696fa7d0fb93851488e49260f27edea40c8efa4423f1236ef7a8e39abe8f9a0").into()),
				PublicKey(hex!("af053a711f38e13378266e5fd5eab0558328644bc2b5b33c85da8a014fab42f7b814dcc39bc0a41ae43c5300cc4b06cd").into()),
				PublicKey(hex!("b03140575f948b209658d5ee82016918fc019c1e378a2671836df7328eabab1f0f5f3b1a960c39100162c0d1e4cb183e").into()),
				PublicKey(hex!("b4e07a5a41247cda27a7c843d44bf22803fe3ee3e9045d147cd167deba2c0e44165d4ed8fc49562c5a9cdd14f336deed").into()),
				PublicKey(hex!("890251a7f77d07c95a10b80ee5ad360e8637eb6eb7db1704d51bd34d4dfad4546fdbfeff56ca533c14eb699a9a084162").into()),
				PublicKey(hex!("92fe3f0095a6b32699c9fcc4e54640441503347654174fc5a000b1becb3f5bd5beb4825335b999b33949388d59d5d9bb").into()),
				PublicKey(hex!("83c7911db58a863d31bb64e4ee64c9bc60aa0e65470450774f171273842a75e8abb195f1837bb261f73daffac1081547").into()),
				PublicKey(hex!("a639f6f10383fb0b97a40bc9751dd81173e041e4946c53d6ce58180ca1d658c289332a3476f2eedda1ebc3dce0a7a45e").into()),
				PublicKey(hex!("8aedc960340081bee87cd4265378e240554ca8e6c22b908b8f219d0e9c53f7d85b248a2d65a414769b66ef3687eca1f5").into()),
				PublicKey(hex!("b35267fed32c3c9c3073516a8762d5134b9478cba55e36c040a2768d41da19cd8ec8789415f32c8e5d581ac7e1283652").into()),
				PublicKey(hex!("95712884c3a4a03fbc6bb6f0e3b16a6f8c031c80c515f3fa6380625044e3950cfae4410eeb42538beabf4e24de82f822").into()),
				PublicKey(hex!("a2e1c4787d540e6c7bd824f2fdcbb865c63f38c85432feeeec5437c42daf7e2ebc45846a318f51c1da25092f43bdf2dc").into()),
				PublicKey(hex!("ad896c092e1bd43c73d73eec34932127e0ad48135f4d5659048b24af6c005564fede14276808a1841ad46098c965055a").into()),
				PublicKey(hex!("801cfc9f25aa5bebeae1d70caffe29957c4a01086d2513ba9f15fbd6391d3bfe39a0b1ce22e1d14072173b04bd1c8952").into()),
				PublicKey(hex!("af5eb6a1eec858b7c3e4bbcc43f2469fe03eb699f7b6a1e4717f4afece74900a43363036083f527e29c87d6654f7120c").into()),
				PublicKey(hex!("b1fbf32ae265ac4a724f4b5bc867fe85271162a6795b8b2a81945db110d80c59eda35f6e41dff206eb437b45d4ba7209").into()),
				PublicKey(hex!("9194c98c0a0a23a19b811aeeaf12bf8cc9b57d370144bc1308c7e2418be6ec1ad45e543387b5b1811964cf17407f5fa7").into()),
				PublicKey(hex!("b8b6f9d8448a9adf295b2d9758a5ee2446c0ac5659f19b653bc86d0a64ed3f740f2f9435e7beecff5ffcc2eed13e6213").into()),
				PublicKey(hex!("a5735af85edbe9d41fa48c6c461f1c6ffbdfe51356432a0b83faabddf10cdb0d795fc2d2b528304ae3ddd7a43020024e").into()),
				PublicKey(hex!("ac4ad9b8d12a047df4a623c609b5545dbe1f1354b52dc1dada38cbcaf594f335e859be79399fecf66cae7e5cfd1d1890").into()),
				PublicKey(hex!("92ab700cc9be6755886bff63335d4f834e2b7adfec891eba8b0b6dee4df3caf46dd27f7819cd2136b5304640e4aab9ad").into()),
				PublicKey(hex!("997f8a4b46c0b93bf85b4546c15dd50c151ffaa9ecec58ad74f7efdf6873343e3ee0483cf672741c2da0d6951dec6355").into()),
				PublicKey(hex!("b28c5fe4f6dc77c5f83ee6d0e9d9c987fa130595dc1faa68af76c7a906de4101caf67aee6e43453833f0bb27b7a7059f").into()),
				PublicKey(hex!("ad740e26c3cfa3f6c9b3e22847aa07ae047e26477019ea56cf0fa96c9812d67c15231ef8608cb6968f25d01b0c95acc7").into()),
				PublicKey(hex!("81bcda5f4c803fb71ed45aef2aae1b446a5217b65895c0e597423f652578ea42a6a76b4c0cc7fe8b95d590f169f2bac0").into()),
				PublicKey(hex!("97d384e1d63b2187b6ae78a8fc90a7f79f3fcbb868a23df1602a1d247c14dc6d552ad6c0b7a944fc8ddce2e16add28a7").into()),
				PublicKey(hex!("98cf24a5d5b1ffeaef43a49199cc2ab5bae48941eddcf315e1f9a7b44c997f23377527ecf2583dfab0a063a26e310beb").into()),
				PublicKey(hex!("8638cc29ef7b8b2702774a2bb5c32ac387c3a10781308928b67a5ab96ff70ab1abaac83441642f664790e0df2b714a0a").into()),
				PublicKey(hex!("91c4f08cf602f3f7f3702a535c71bb67d1919e1cc89dd0b2b604cddd561fa6acfc8d990485668981367232882e656037").into()),
				PublicKey(hex!("acf76a5cbf05ad5b86efcd072640a85843d0eeb1b70338ae6634fa0610189267b2b55e95762ceee100a7816612d0e2e7").into()),
				PublicKey(hex!("87ee3debe1bfee4161785709d01a662bf8216bad6bb58fcf0e56c146e33ad3799a4b83e7c6e73788cdfbca8a7377cc2a").into()),
				PublicKey(hex!("a4a8d5ea38b9cd73ed624f669fc22f267d2aaed951f910c4a998a5e7fe2faaea3098efdaa20d7c19cb622802d0685d5a").into()),
				PublicKey(hex!("b2aad09f444175f81c4e536ab8af96d20866b5ee731f02b84feb1d7191c77b63e67695fb00e3b6eba24d146981201e52").into()),
				PublicKey(hex!("b4d47b58062cf9ebd4a3cba8e69b6bdc268e56ff4cef1ebb0343e2feb522cdae921ead18cb294df4e64f454c32922d83").into()),
				PublicKey(hex!("85fba3ed11547e1f8051887a7ecfddae1297cefde3464cb426af0a39262aebacea61b8003563c3a38054ef069bf078fb").into()),
				PublicKey(hex!("965dc753f1a1abb84d9d6577a5aac2017b13cc30b602c9e54581bfbe453d8d9e53e116ee9815c5f4b2b652e9ffff3968").into()),
				PublicKey(hex!("86c93be9db8ee8a2b9b68e1281a3af99661bf33219fad3b52b14b0edad33ea8ca72d3005065c33c51516317e710379c0").into()),
				PublicKey(hex!("83d81711ecabcbc10672b2fb8f33bb94b56f85cf6e246575b49c4a76509c3d2e59823b0ce8145c8ae7b0dcd549902736").into()),
				PublicKey(hex!("97af787a3f9b5e9ad423545bb5cfa6e19e8f776c3412390a8cee04828a9b413719bc656a70217d334e5d5e820aed4842").into()),
				PublicKey(hex!("a383360ce6aa787240ac417123740518715f1db819f5acbe40daffa0ecccfc22007c017dd260767b62f25192ea55a2ea").into()),
				PublicKey(hex!("b86c9887ef0d0845353fb1345e92d2b8ae8aa59cca4dfdf2ae1b4b46d4fa09b19cc2798309541b1ac43860cd1f70ee7a").into()),
				PublicKey(hex!("8f77e48c21853c4f66c276b627f53e9322176bfbf265a1ce8d9ec4129d30c62da1bee570090ddab9e48aa981315d7cb8").into()),
				PublicKey(hex!("b17e19ac722ebced2085c6c6477b5908075b2a21b5d4fac61f1a7146677da54068bf6e425d6ad5c0ae8518cfcb3ada63").into()),
				PublicKey(hex!("affe69e332fea2b018dd830d1ae438b60ab1fce83cfa58159de5e8039cc4ca0cfc9ace82e3dd21853988a21a8b606c95").into()),
				PublicKey(hex!("a29024bbbcf7f98177c7a632ece1c020709f95f5059b0db3c7810f4070c4b156511b9f1869bbb14c1a2620eee58c0192").into()),
				PublicKey(hex!("b564ff12195a9d8ec17c02dd345533040e29901b36b1078b24439c5036c9e114fa6ff05b8e73c18d861e1591d763103f").into()),
				PublicKey(hex!("85af631aadbd22100864c9ddb804d8253bd8817e3e112ec54bbf213ef0b7f0b507852968dcc68fc276981176e2f16f52").into()),
				PublicKey(hex!("aa1743bc124a6b23736fe9c5c1bebfea16610e9188fccaf6f0e46ce46434c223531327aa49f0bcc1cbd794c606fab5f8").into()),
				PublicKey(hex!("85a1e949352c701e37b481b9057fcae7bf7a782975a9fa4274c260bc4af7c0c589303d72f8fad3bbe5a2fae343deda0d").into()),
				PublicKey(hex!("b9da95fc32336864067033e6e35269d68682591bd147d8305873215770d45af97b215ee22cbc7220c38715211f483987").into()),
				PublicKey(hex!("a07d18e5b3087ddebf6c54028c57e57e800c9825454492b2351a6705af9cdda7d8354e705df9c68e3640bc50f5be583d").into()),
				PublicKey(hex!("94fdf607aa3b7f4e9aa3d24fea1ef944ab54cc58d6f52db3ccf612d498b411a86b59e88eaac0b6900a93d2b2058707c9").into()),
				PublicKey(hex!("b312406ca8ef0a391c2770e7d8db4307dbddcd18dd7486832e452b51fafe7abbe578ede583986f66e18a14054435d564").into()),
				PublicKey(hex!("8f6de7dd19fddcd0f0ccbdafe2f0815f0142fc82ee034f88f3263b9ff4c9e128a2b1eb11f22a583ae04b293f4a4052ac").into()),
				PublicKey(hex!("97183b2a3f3ae66c69e601e3217f7176522549cf1197ef513893bb84a255dc2e839cc7c44a139f70fcd957b3900e48b9").into()),
				PublicKey(hex!("b5db71ffce39fa6658a88c36b1eaa699e6f1c66cbd8349775c081fd92bb28dcf2552fef7069d3e6dff14bf6f36c1fc7a").into()),
				PublicKey(hex!("8fc55a8b4fa51f30e9cf8c15d94b633b9002386c7f2f9d7d6353894115e2ffc020a76d47bc52e4c7cc66b65e1f0846a5").into()),
				PublicKey(hex!("b911df35828359ef26bc2b1429bd7e4b1a934f18aa4f09dd45047122b458986ba233749e781bdc0276ebfa4659bb6a68").into()),
				PublicKey(hex!("862970a04d5f3e7f05b1cc5e14d7e1af86dec3a0980eb6e1e1003630fc5e48b3eaee5bff4ae087a186a31a23a6a511c1").into()),
				PublicKey(hex!("aa0cd6886f137fa90aad119511355e8be0b3221f3b759e082b7c16c3903e162d304bc939ebb46b5dde03d30137a6ff7f").into()),
				PublicKey(hex!("aa01e87dffcb7419664b3ad7ec8c6369f7e7b227fef857b77a221931f848f7fc14226080eb7075b1bdf96dabda91dfbf").into()),
				PublicKey(hex!("8409ded36265ea544aa9c583215ea19f587b892c4e1d39c70f01f8d01baa6bcac82e7c426547c4e0dcbfb7ff79b1d8d1").into()),
				PublicKey(hex!("824e41fc906486e98613e04e5a2f414d90f27039055bc44eea64751ee34b226ca06fa672c152a548f984c351763482f4").into()),
				PublicKey(hex!("ace6eac07f7b8772dd34e653b293231347903d5ec72e5ca3e7a8ca030ce7de59b47ed3a59502e509969027a4678e676e").into()),
				PublicKey(hex!("a0199cb33bfb9cd17e1cf3c1f66057f1059955dcfeeb680df54df60cfb4aa1aef7025b26ee527a2019255e640f2c8e27").into()),
				PublicKey(hex!("b5c9cee2ce233b69ca1de025c14a30042147aa3cf5910bae34322ae87c7ea3fb095f8d7bf72c1c2fcf0ac4c03a80b082").into()),
				PublicKey(hex!("ade27f13d48927d4cdad8550e6c1b79fdc3366881c66d9438d7281cdc4ab7807f4d15db072bcbfdc5b00839ca7eef6d2").into()),
				PublicKey(hex!("a00b8ecf49fb3a93ebed2af0b8929aae42c0bbd2481d32ba96c5d9fe220119101d722735fe893c5a8cba73cfbce9612a").into()),
				PublicKey(hex!("afc4fe4d8b50bca998595544e534239f96be37afe6f07ec37bfecffc97405e6e0990986dc6b0eed2f219f40debe7a42e").into()),
				PublicKey(hex!("91ff75fa001f5970c6f491fc056cc977ce62cccffb693ae518af5a5e3165fc52432ad13bb0b8c8f627fd4ab5f226e5b6").into()),
				PublicKey(hex!("b8317c2258dfa560841b267c4bb2fbe67ce675a819200ea2c6915e10aee84bbb8a09367d8df3e9bd3bef73a1eb7f5872").into()),
				PublicKey(hex!("b140da48b8b2be89d2b2b6285e91c9485cfbec0fe282adf766240326aeb1979dfe5e6fa1fbd41b8d84c7c36ae3c1e6e5").into()),
				PublicKey(hex!("937bb6ee7897358cfff20d142cfe92d1b45ccccfeeabc4ce47b9bfdcc7d1d19c059f244e5e27a4aa10543b35484e2e1d").into()),
				PublicKey(hex!("8746cabced2f40c7ba13eb26f681e7e7a131084baef5c02760b53ebcc1bff307c960d1059e590c64241e4b8113301ca5").into()),
				PublicKey(hex!("95c09548cd4a1fa226487d1a44d9bc1f235382daf9021561286be5ebe05b59b655302ca3c3475288fcca23b7b8e0cbd7").into()),
				PublicKey(hex!("b5653c6ff1d8f950d58b6f2844e5e896dd8880e0baf428570a5e49ee225b8090b28a977ca6773d28cb1f7bcc920e6bb9").into()),
				PublicKey(hex!("8424b479ec1880950fbd23c6fa726fe33531dfc7ecdeb300b6132b17dbf553e777eaf1f51463c4a92c00c440d4f4caba").into()),
				PublicKey(hex!("92d867c7664e24420aba86b19564461c0dc334d0646d72993e2652c2f5be076605edea6b9f24cf98ea9cc54529ed4f72").into()),
				PublicKey(hex!("814fbb5e541a76829eb83c66afa9685c9421941c1a10435ff03badbac3697ae65820a57d57ee9ecf930633557dd1d75a").into()),
				PublicKey(hex!("b54e552cc20f160d243e9346a68e604526496f5c6488c30253c05364ae118068953e00fe5a9d191c3d34b16af2375a38").into()),
				PublicKey(hex!("857e3c5af6e2d8cb25d8bdbfd56cfefc8a5bfe9e8c1bff559997ac2b420f899fb3db01d3878a48d96eee14900cc42f8e").into()),
				PublicKey(hex!("b6a222053fc56c4a087bc4b9deacda4eff97f008e4853afe3cdf937314b84cfaaa465b535c81f8e23ea045c66417b3ca").into()),
				PublicKey(hex!("a913fd71fd5824c095773eea581b7c63d7a63c27024da51e52e20980d67bfc3c77c5a3506ca829e9fae34cf1aa46ed11").into()),
				PublicKey(hex!("aa5705bf8799da98f3117a1b718e422d9f6e69adc3605facb3f4bf46719f7388b34bb0f080d18b6fb45571ecc2488d58").into()),
				PublicKey(hex!("a8e0ecc4f01f5950a400688d6f4de9e1f7f10681923d3ea956f2da4f2903f9533c60667e5748c42993b10d9781bd4f12").into()),
				PublicKey(hex!("9176169e98363a7d99c3b71fcbd4865891ab00daf3644b5102eae7526db45913a057d36eecf109df2d3c8e72cb63d7e5").into()),
				PublicKey(hex!("92cbd8987c83872d317d36718298054d6f804fed2a2d61d0da662b56edd5e56bf54c187f429dd809ad6b70682fda7262").into()),
				PublicKey(hex!("8c6605ffed2a2e8fe53b0a9ce62e26e8844c314b1a7134b02373f7983c4a323a8cb2c6c87f4566db1a7ca773070dfb53").into()),
				PublicKey(hex!("b28dbefa54f74a13283b9bd74c1b2d50e77977be440b65330fcd314c712af38c0e2d62335a5a08015ce5e13b91d17770").into()),
				PublicKey(hex!("aa0540151ceb5c471c16e3671488eed4b00ea59d3a647d4171752768a323c7d46eed8d9415174d2060cc7003394f82fb").into()),
				PublicKey(hex!("a9951b27646fe74b58afdc04e4bcf7c563b213b83cbbfe27d163f31a28362f093552ed7daff5585ad3a9e68a7ac0c91b").into()),
				PublicKey(hex!("a1f68c809e71fd702f43c2e30a65abc436c91483859e2407d4496ada2adb07d1eb41279feedf8e85ff36812ad2620391").into()),
				PublicKey(hex!("ae075f7b133180a4f990c26e0fa48deb56ed41a0f2998d1070c01f57712a970a3161d3ad8f193a108dc0e2c4a6237a01").into()),
				PublicKey(hex!("aa09188df5bc8dd1d0b0262007eca85181b4c8045181c7d6272861b34533637f0ab67f676cb1f76f6d56195de9d8d7e8").into()),
				PublicKey(hex!("806d70379f617f8c175a67fc9a794f34e2fc8abc6d16a25fefcd021578a84a9fff272865204e46228159f42daa138eb2").into()),
				PublicKey(hex!("b152d12eb066ca623fc871f25645da2b8ee97e571000fae5053893e07f0752669a899805cc6ca58ac064d381d7faa893").into()),
				PublicKey(hex!("a53ccb174489c972c3560b4610c2cc36224452708502e0f852cff8ba3165c00e586a72e66e5ded887116b88bd4d89422").into()),
				PublicKey(hex!("8bc8e0f2ecb8113cdbba43e5bca22251bc3ab708fb88778bf408e1b3349adfad5454dcc2cc9b7f77fc2840ef00ee6188").into()),
				PublicKey(hex!("92ba8da411bfeaef4ff124b7bc60e8faffdec17b6ec422295cea66bd630992f9e69f65ef85f6928308efc81c7578ebf4").into()),
				PublicKey(hex!("a2e9221af02085686e6dbcf20686129ef927f98e9a8e6f58445d0da359e2ecbf1037c2cbab1a5696afff29fa36d5ab63").into()),
				PublicKey(hex!("84644c30b927bbce768100238ed9097d33fe7c12025d6500334649842ad197273741416a7afab96beae5c71690b6e277").into()),
				PublicKey(hex!("8c54211542fef4034ef4c1d26ec16793bededdee87ef8911ebb16537fa0704b18a2486756c19091c643c9535247da343").into()),
				PublicKey(hex!("9904b73e6d120bc083b42ac49cae937d80d01bd43ac38bd78f602bf588b53aebb2df7bf9e2f7e210bfb307eed33a133d").into()),
				PublicKey(hex!("b886c4898cb2dd7ef5489f0149595a43678f2a94f8e5978f7b1b45511aeec1255f1450f38bcd7a4fa97e094f419d0cae").into()),
				PublicKey(hex!("a3347838f5bd6c042685d7524ad49fb38eb410499ff5b089e02949b98270f5236ea4bb03884f260c21bca9cd70533d3e").into()),
				PublicKey(hex!("a413da2c1c37e7d20752602462a3d7d64a1fe317d0a5741e6d700553e84f999b684aa80ba7eace640205ab511fe1d096").into()),
				PublicKey(hex!("a74c93931e7297811c314a3e45bac4ef29c608b9204bf1443f89d288ce9f736058df0254082fb6061e809f586b038d19").into()),
				PublicKey(hex!("b70f7e9435c52418162189d1bdf0350d9e9a5450e9edde62ea3b2ba57b733f4a1cb99113ab2240924f0bfe9cf3fc3e02").into()),
				PublicKey(hex!("81540ebff32178a4fe74d9c0dc5a389409d81ca550864677f76fc5b633f0b2a1abb0d625f0bbba7edb8834e384428c3e").into()),
				PublicKey(hex!("af52188f449b3b2df67e6f956e28de562a2a4d6fa8c5015520067413081104822d771626c7b768b80dabfea362842ea4").into()),
				PublicKey(hex!("99aad135dc8b3f72c01feac2d675e9a730489e8cd28093f1ddc21acd9a94aec6ad30a5c18c1ba6b386a9c3bed621242f").into()),
				PublicKey(hex!("855a5590de86b0ea8c815afdaef979c4e1abe510eeb0e78f7f88d4867d54feda4c06e1e25e289d4bad9c1a3bd1d1df83").into()),
				PublicKey(hex!("a839ff60144ec8c52a41c428338604be4c46243624ba647c7c7eb99af3d987ba7dd8a218f7f301a39577eeefa19751d0").into()),
				PublicKey(hex!("84720d03934b0c553facb0077e68f2d750fe46b8ffc5f30324db5855c682933e9a0ac0ba1a5000763597e94484f4a112").into()),
				PublicKey(hex!("a4ab0f494b1b1a72f61b1164ec68b2068420b980000ee572e7efe78aff8704d291e10b338c4e9c2d2c7ab735de880941").into()),
				PublicKey(hex!("8649912664136c03a913c012e29864f502e0b79d3ff47736636361bbc486519e01541ce0211d203c796a29f4cd9752c7").into()),
				PublicKey(hex!("b7447b4eeb4a03750e1b14001fceba7f7b1e5548e63f3c04852356888b7c42fea4e1f8b7887c151926b75e1862797f62").into()),
				PublicKey(hex!("a0de8b275b454fb7ef355741b3d57c9750604243645bb835a9e88eda1bbaa25c5a70fa9b68e440e3c8ba16a70f6ce866").into()),
				PublicKey(hex!("8b3aeeae764712292d58b88808ef08c3b843e0dc7b779acca0ae6ab727c883b7c6fb1cc32f27e2ae9893dee646d935eb").into()),
				PublicKey(hex!("ab4d6b0d2ac5c3a5705ac49e8e2cd3331e7fd6b0dd5d11a534e4d839b1b16fba694c20a1413be76fcc052d1f2f37f04f").into()),
				PublicKey(hex!("a88a8db85f0a8933ba67d7454ad4f636ad8292e4c03d97ae9b3cb2f56e1e4ad9cfcf34660508b9a52b543e5b0e5659c6").into()),
				PublicKey(hex!("b44d50671b0e0cf6b158a13698fd127ad020eefffeae68a158871ef77ac1c4d5d6d63ed67b1f21c2ab20715575f4360c").into()),
				PublicKey(hex!("a80fd1bb42da5d01cd0ab08267dcd323d585aceb48340890fa00676593a2f50423f1fc95815661f27bcd08b66dfc617b").into()),
				PublicKey(hex!("b77860c52ab1d91af5846025b2a7399788655cc6261955f0c403846b1cd7a732d7cde626e224f04d3ac423247c509327").into()),
				PublicKey(hex!("ac6c4bebd72ce2befd72476fb1455fbdc6d4d1c605639c3d5d7d14bac58fe0146944f1a1d843983302f59c633a0a8820").into()),
				PublicKey(hex!("b2bcddd90ca69cd45043a17f8571a7469c3e6a3fb3293abbacc9dc1984ee8dcb6273319ebecb27cca11af8baa2ff8bbd").into()),
				PublicKey(hex!("ae9ee07cf6cda1e959afc14257f5a1273946e4cd9d6b8d16cf95493348c4658f50b320350d576bd0a3762bfacdc60762").into()),
				PublicKey(hex!("aa944955ddecf0979fae93b38db8b206dcd167400c8bc6fbec11e088627751a279ae2a255625083b1c6480bfa8005bc2").into()),
				PublicKey(hex!("887dd1cf3c6af760f9a442029cc5bd552ac6eff90f4c1ba9f095fdd50e1222808b348eb00339b4ab6d126a3e3c0d4ffa").into()),
				PublicKey(hex!("b38833849e194db52f33952750fd25e7b8ca235df16663d03a4b9492e2d8096d4df487da093ad768ee3c61e72d77dd6f").into()),
				PublicKey(hex!("8942d4f2f7b106280b3859c86f360f96be9041bc6d8d5e4a103a481c18286993313c304e5d51409712a229e8d42b7e59").into()),
				PublicKey(hex!("956bf829389adb1a88dc058c3ce0632cfc97be0a5825a6eb273def70c1943b09360f6c7840b53aef47220eb36fd3a461").into()),
				PublicKey(hex!("afe6a32905e65970de3c559c54dfab15b1995977b29b3b1a6dec11c701a2f2e7b66c92c0ea422bc8189370c153adbd86").into()),
				PublicKey(hex!("b7444f6199d3dbe3103fc256249d61ff68a12d435f29e8221fc8cf87f5958fac34b5f6d1d2d145fc2d390467c7715845").into()),
				PublicKey(hex!("8e31196762f0a86d0986c45061fd8e1c895bc56a3cd5d26dd2b1986f29636409f372f9f2c7d52894e308a298e8e2da08").into()),
				PublicKey(hex!("aa5061070d225d13a493cee0397ec13ec5c9569ca0ca08d097cc796fe8abf56655355e9a6a2f7da489f312c8129fe617").into()),
				PublicKey(hex!("82efe3b51c5c4ad4b20725b907fae3a0270d6182d4bb695209031837685b6a906fdf59ead0cf11c77db67bf8fc317434").into()),
				PublicKey(hex!("826ca9eec7db799d4b445f4548b96b1a5953f879efff81132b2a17e7abc1397c189f239fcc60aaff5dd1ea9479dfdc5b").into()),
				PublicKey(hex!("8d946ad0fff13f0e1a803cc2899c468c6d7545586d700adf70794a18de0744800037b6a9cde399fbd0151c41e1edf688").into()),
				PublicKey(hex!("900dc3c6ce3ae52ce29d30148108695e0ee07bb5fbbd9a0fbbe6160b18585861c355ccbe834a9879a3d88b94bbf89430").into()),
				PublicKey(hex!("b5dcace998bbd9f12c91b8c4fdb6a3a58b477824ce501754eff56d1d745b3796a7ca88a7dc64c31d72d1503cfe6f67c1").into()),
				PublicKey(hex!("875ca29ba72b7c3e5f9ec636ad8178732c5cd9de3551365af5156c1f837beaa6e583b6ec9d49b62514daef09abae5723").into()),
				PublicKey(hex!("813bf8e50f88f8e4b30ee13dae797012ee9b34be2384a277fdafb0faa7acd7242dccd9364cb756a30f4460f8bde1c3be").into()),
				PublicKey(hex!("afaec3e542fd838e77fb75d6fb358a6ad785492191293a26166a3ed20afcfb9af3edaaa3d23a8c7bb6c8e74ae848091e").into()),
				PublicKey(hex!("b86fe008016b3930125ea529df108aeb24a57c1755331b6c1184cbb6fd0930601e0e6eba80e513c95f54b8a1c161a13a").into()),
				PublicKey(hex!("b09a4fe7ead3f95a5a30b8b2742ff5a0e0b390ff42e023919ef9da3571f3321c0705e2339a3f98f3f3728c8b0549306a").into()),
				PublicKey(hex!("858e6fed572e9bb4bc69c01e45762fd24fa9a62894833a6925c1f385ee239b7108c79827fb0f9bd7a4f27228fbc0fabf").into()),
				PublicKey(hex!("a9b3526b1507f0904ee8d8b6b1f5483f4c9aed881e5207a22299579f853554dbbb03f2d521ed40afe53dc9ee51bea106").into()),
				PublicKey(hex!("b290086e0556a6f441c3a6dbb14346fad66bbe27063d6ed3a0081a41a79e09aa731ec1ec1a9bd9b9f4f977158fad85d7").into()),
				PublicKey(hex!("9885e1fc012e6ae2770eb8444a15a3abbcfb9b10bab2c714097417c8f37fc03f9dcff3c78dbabb581226722343af2a5d").into()),
				PublicKey(hex!("b91daf5b0be5914f8a5a03e450151973905ac657c296661698301d0df204fad5890849c3cabc7297866ae618e677cc6a").into()),
				PublicKey(hex!("8b1caa4afe985e26533f58150523a546c5e4111cfbddcc631f12b111b565ba5e53281df4368af021e0d399a5d5558fb3").into()),
				PublicKey(hex!("882b8601af62e36b1861489bde51dcb44255e4c33d42047a641dfc7b413fd65a355314a72d64f95bc63c4da37719a662").into()),
				PublicKey(hex!("8952635bb2a9ea6bb9e7424f83d42a708ac4a3f3236d1abc6797dbac83846f7e9f2d9dc0f1f7ce10d797d900cb34bbd3").into()),
				PublicKey(hex!("835960a740dbad91596985fe1446179a38dd7831c87c039caebe522e2f43801032124992500b59010ff5be6fdc9c0c8a").into()),
				PublicKey(hex!("95257187a61dfb8a2f5bf16a84d5ea3c08b581c42c8a727042ab3b3a4efbdfba34ac7c2f6b923b39e2187d1d25af412a").into()),
				PublicKey(hex!("a6ae0224cb745f4877fb36d57f48526eb835c9694b4dc12fa0507e343e2f1adf892ec695e31cfaf128f1a6afd321205d").into()),
				PublicKey(hex!("b94694ba53a902bca317c04856ab623105294afe4452e55ed36f58b5bb17443d9119d1128a14f8799ae89fae75ae80c4").into()),
				PublicKey(hex!("a893cea8cefc48b2f8ad435bd20a0051ee6e912339dd1e0273b39c81e07f249b531c62c3f727c5ef8a9162fa9b2979d8").into()),
				PublicKey(hex!("997344c24b61731b655773948f65c640bef603aaf4ea96fd7250a52484e60459ede18710f21229701eca04cd0a1ee917").into()),
				PublicKey(hex!("963e0d1145f04dc68b308d2e820df8b2e03933325b0f1137cbb3023d56b46457a492b370624876a0ef9d39b3e28ace5d").into()),
				PublicKey(hex!("b9b0f5a81c14c110d75053f7359476104204a6b407fb18adc5efc89b6a0298199d9a25bdecae26d00e9c41de2b7a9a06").into()),
				PublicKey(hex!("8f25582bc92975737a76f2772e5741ad37ab1f867cf2cbc0be2924c0da1f977e47e2296bf49c178656d9f266e20af8e4").into()),
				PublicKey(hex!("a8a73ef7f27a16894662efcbd576315f980d48e059eb459428239a64b5e1559a8a5dc518d6b1c62138e6d3ffb5441aea").into()),
				PublicKey(hex!("8b94dc5710bf532970b97eb9357aacbc761de61ba89421cce66282ffe721385d843112a546664ed8e7b691bc91e9ab3b").into()),
				PublicKey(hex!("a5d7801afaba0fe7a554659e8f4644ae8182063a892bf4dd55d5629b37aa270fe037ae63782e009c03b7bd588454e140").into()),
				PublicKey(hex!("a92419025e8102bf71fbde1929430f4a23c2ae239ac784ebf2abbcfead60bf040208f7d4dc03a067f50a4ad836b5af4f").into()),
				PublicKey(hex!("86e8961aa7298e4480b1f06467c8bc987e633c0d40e33944dd58d47b948aa21c2e9e9cb62890b9352ef1c320628d4988").into()),
				PublicKey(hex!("82c87de6c5b1a32bf2c6361f079353b4c7b9931ca62009acdfd7f75ec9b91dabeac1715c9ecdbe1ab09069ce4cf82b16").into()),
				PublicKey(hex!("8ba6a140e330fe80369c0dc84dbbd03d21969f5c7048bb181fe534e1d07ba525d95e7ca941079f04966f948e15b50404").into()),
				PublicKey(hex!("a7cea3a44a414a7dec25fac9c520b0d61af5575639ac8a4b2a4ef08431cbe5e61b6de256954258e22b43433980e1c287").into()),
				PublicKey(hex!("82a6d8e617bd78dd912e8a265a7536638d956e2f3bbdd7da46684508e8c29ba3521f83b1ac156534475992ca4f312280").into()),
				PublicKey(hex!("93147b4ef897d045b36c600c4ef0f7ab3093a4aba1932a7f3b4552286dff436b0ac479d8db3ce9afea9574baedc0e7fc").into()),
				PublicKey(hex!("ab62992f69faa64417f5e11ab0835f221225b370e9154e74f036a5e1647f65fea99fb44289bd27897b171fa90ec438ed").into()),
				PublicKey(hex!("a2cc286d4f9843cf1fb9083e7c0c6666679d21eab475ca08b084802efe142451bdecdc19bb48c65ca020219e9fcd4bbb").into()),
				PublicKey(hex!("8a83ffdab76387847fbebff596341bb7de19aa8386ee73729dc452fdc55ad8e764b210bf515cf678ab5b113e144e8fa9").into()),
				PublicKey(hex!("8e199d91ecbd102f92f60152614c08121e2c5bf6c2812a1d9f49b16632057023688a4c21e507a6ad90d9dc6ed28aafbb").into()),
				PublicKey(hex!("aae7bddd65f4b7ecb7f0d9787717461da3fb2308b630cb85a187786a7e6190d7c9ddc105d0170b58acdbf322f76de37b").into()),
				PublicKey(hex!("a8edf1ea7bdb3cc1d38050dc2ec0d2726dd47dc08c877ce22a033ff5032225e562b54bc1702a94054b734a68605529e6").into()),
				PublicKey(hex!("8e107ad8d9061041e29047f82dae25a5cd714df62c0b542ba203697a77ac9d2792869a0eca21b46980d8b4bb68b23fbd").into()),
				PublicKey(hex!("9527d94a2f3bb13b7b7a076b519320a0e0fa428ed8520aab03781960362c932eb0a883b8adaf70ad29e434604d4702e5").into()),
				PublicKey(hex!("92029078273c3c410fe4b7802ffa68df551b1b9ac9add2f8e95216068554be616370066a816142d8ede2f4fb56be5110").into()),
				PublicKey(hex!("8bc08c94b80b8aa10d29f095cafa2eb0eae9267f2305117dc9ffc0bafedfb9de718a2dfac280263d7529072f9e0194c9").into()),
				PublicKey(hex!("8ec2487cf14a0fd92ffaaac07272c7076ee5bce8e2f5527951be98c3ab853009856c9f5818b05f68f0cf82d4ca608825").into()),
				PublicKey(hex!("a0d999db5f8443c9ecc1ac936e89ee488058a133a2e1151e60d195fc19bdfed8022b1816158b4252d53dfb78a23a6c13").into()),
				PublicKey(hex!("8e599c1c08761dfe6a706870bebe7af39375827aced17059dff5cb15228542909ffd1490e1ae44f37afce121e97b65a6").into()),
				PublicKey(hex!("a59fd26e412f601be8e35148dda71c299fda948eb62af696b592923de19ce75ac4b37f6a088fa5b403677b121bc8bb8e").into()),
				PublicKey(hex!("82969647026640706590b54b2b00deafb3d0dc42ce3e1164128ecb915815dda704d2b3d56ea9aaecaec8bd4313f014a7").into()),
				PublicKey(hex!("80b3caebb390ec858ae36b7a77a8542345e7f3cb2603039b644c71aade9670f3ff6b41b5fa731b7c1a2be62a2f3d6c32").into()),
				PublicKey(hex!("a33fc743ad1052c0eba1a8ca4fbd955998cfdd3524e543cb1cf62c283ea485b88f34f071ee4731b69c173fabb2ae8d05").into()),
				PublicKey(hex!("b3bdd468d39107fd01a4f0d4719e1514e33eeeedb311ecf813152437e274a35656f095e509eb9756fe3d45fc4a052194").into()),
				PublicKey(hex!("a9dda867344bf6fb0fd7207e3f5f0b5394d05d515df38a7fa49e143d09192999cf8903b415e50e8eccf95b51e551321d").into()),
				PublicKey(hex!("80169f7d4afaa820cd7a276ced1d6da3b8fd4cee8abe027e68c33ac6e3198619151598a5caa0c7745903fe228d007690").into()),
				PublicKey(hex!("81a7ecc249e05a42c7c5b1a728995e17bd4b8216c005a1cab43fc902e2bf659e1175a03267a5b3d1ef81c88c68266d0e").into()),
				PublicKey(hex!("882195a23d7ac9bc32ef4d7d46f963143d93c691fc1fe3a2215a8cf5b6aea4743d1d975e3d3d70760046c7cdf159f60e").into()),
				PublicKey(hex!("90c7aa9446dc354741d749fb655529e59af478ff9b043540c5787bb3df9e10c2da984ce13e5ec87e2b9378cba744d12d").into()),
				PublicKey(hex!("b5afaf03a9e0aa62da23131b507d9f6427ac42b0fe1eaceb0c1119f2eaa5e8d2d7b53107511d71c0afebe26669a559ad").into()),
				PublicKey(hex!("8de3211c025f8cb5714445e154ed7a151076a1460d670ae9af10f02af19479ddfc5d914b985bf7367da49372e9065e4c").into()),
				PublicKey(hex!("96e73bb0d30b2044bc689562e157ffdd0e2453aad1d004f4bf0471b25d190dedc7101d9fe5f79bc5f98a8692b107f9db").into()),
				PublicKey(hex!("861e85a58ddfc6b3a5cd7f8dfa6ae4b8dbbec31b36e93e52619514705221677bde391e104a55607e43fd3394c7bac331").into()),
				PublicKey(hex!("9776ace3224f69a0486fc72ec1e6165b9cdc0364c98dfc34fabd0669e8db4ed0374d2649868edacf30d56d9e3bc8a618").into()),
				PublicKey(hex!("89002741b4a72609c747ad5386d9e33e0924b08af47f6e76ac3136f26d81f991c1076984e75f5e903077dae8d3954b5d").into()),
				PublicKey(hex!("ab96ba924aebcad127ab36fb38afaec8a20c60cfa4c5ebfdb4b7b2ece909a7e6c42700a19bccb5f081d288d8a2b064ce").into()),
				PublicKey(hex!("b23ba262876f78a1db6fac132c88c882c2cf71c83d6689520596523dd259a604d3aa14d6d19efd378e43da7c1ec79008").into()),
				PublicKey(hex!("a98a8f26a7798b17692796bb51f502d39698b6695e4666e4f572d493151b9ff05437f42581f643861f73cff1eb75ad84").into()),
				PublicKey(hex!("a006180c0d641e102b57f512f3b4658e9f426653cdcb17f1a258391dc6b2b0169ef73d7f2cadc28f07b56413b5b4cade").into()),
				PublicKey(hex!("8e682f527099a9b153172eaa5883bc7ba97edce4fb3f38c90b7a34d65b2b36e984b36663132d7c2423b73887424d3eda").into()),
				PublicKey(hex!("8f3d0ac23a9cebdf17c93010974a432cb2c4f0d045fb9ab78b64ab03a7e5b4b3e22d256077159693eb298aa864637153").into()),
				PublicKey(hex!("a7e0ff0c7973b9af74fbbd0e418577d368b66e06befac10a647d059842f1e003274dbc70ddbb4080e61f27e30acf1664").into()),
				PublicKey(hex!("873f665e5409a2d94f07f37507abaa75c1c96b22199c067307c92d8cd0c8467ad5cc382530df42f23040d75ded66a826").into()),
				PublicKey(hex!("8d6ac6034720889d1015f8ba55b626a8b539c83ab12fc6ea90dc807c7425c0c3c515a86d848767f6e817728222eb3c22").into()),
				PublicKey(hex!("820fce07a2875c28070db9c40f07cb86a4fbc52ece2b2e3bb41ab3144748b4ec7293cfb6223f1380d454349520a93eea").into()),
				PublicKey(hex!("8e4f7e92fa053327f3b3c6126c8dfeb33b791339484493c76c8331f8d43509ea42b7c5bd9bae73c59d28e6a50e922761").into()),
				PublicKey(hex!("8f8dea5a94c7b5499b4de5fcb1eb5c651a656f98250af3c40da48736dd6b1f607d51a3cbd37ccff3f2fa51641246b6a8").into()),
				PublicKey(hex!("adb41ad4a6f186cac42939a19babaf7f1f4138ef522490d5a99a70d3238c8afbf696810af6a1f846a30f787bdbbc4098").into()),
				PublicKey(hex!("968ede3473d835391872a124b49fb43a81e8d8ebc651e307def51b69f5ca15062181ec9a082e4374efa34d0c83b45e5a").into()),
				PublicKey(hex!("ac0a7e7bb603cc2728f3360ee62e0b76c29b44c9fe0f26fb739082e2cb1db300e95930418834a1c28cbe819dc691fb7b").into()),
				PublicKey(hex!("ab0a7ae08eeb5acf177673bcc51b7eb1758f3664399b671f9ee83c1244a5493c0515a3dfec0aadae4c5d927d6aab25a7").into()),
				PublicKey(hex!("b76deebb745f3254eb4eac5b898b24cc487c4ed9bc050caccff6cc84e9947075b637a59c30a6387a630ee31415ce28b2").into()),
				PublicKey(hex!("8db191081b0a38f3b19e7ff7c2739c201a31ab36035994a4c5e13221f91e4b424a88dfef74d5e64a66c354600129d186").into()),
				PublicKey(hex!("902aec708bde02495807b7f00f992fc4890cabc353c9a6b0cfa689d76ae995726e29ddbee4c7363813a064291cc8581e").into()),
				PublicKey(hex!("ab01ce979699ce036b8ad23fdf168110080c727f43597e2a4348694fd286111a1776380db1f70556f04a527ad1cb54bd").into()),
				PublicKey(hex!("82f83e7386bc693af203eb6caa7af4768dc6f95eaca6d0b465872d6348430c65f152ff7ccdb01d18bf93d241b224b8b9").into()),
				PublicKey(hex!("860cdebccd274a35f8f733d7ea4286d27b6532cbb3f9b177e2b248235e9889e40e2aaa3620e3655fdb40dc382cd84d0b").into()),
				PublicKey(hex!("aca6db7d74d01ef56da06330d131bd8fa1056d2bd38116a7579fea86f35ba6e611ff2383abb9b893dbcf0115b844419d").into()),
				PublicKey(hex!("b6a5ce68c2a01769be75b1fcac6ae3cbad22b349e1809fb296a36d72776903f59a1452bafe2cb65ecc71e343bbe0ed26").into()),
				PublicKey(hex!("8e58987a9f9d19367b0c9c233c3624b8b1123031f0ffa6356addee8c674017bf82aa6bef06629dea455677c40df34781").into()),
				PublicKey(hex!("b18ff5f017a829a63ea79b3b55c39517f05f180235bb0584efe1000ed5eddeecc079110bb25dab4aa82bcdd9b3186ff5").into()),
				PublicKey(hex!("aef0ee0d730bfbdb52233c00f2f29528ebf44ba28a06a7ba1326484c3910a141bde3ce99e906835b77cac23d243be988").into()),
				PublicKey(hex!("b040896a9635643fd8312ddfe41445a1494d083cd7e5c5906eb836c68c22bfa6d3e59f0c667a7adf99d27f84258c2896").into()),
				PublicKey(hex!("90e523a6ce96f96935b2669b8027ac9fdbf596c66a268ec1bc968c1ef8f531c450579c48709ce4906ce235c57d3f4dc2").into()),
				PublicKey(hex!("902eaf5465d7b09001594b12a3537a27774c4f4d479399eaf7ef39d5cd2c0686516b6301bc12f964d293da07ae8d5bd1").into()),
				PublicKey(hex!("8eb036118f4f39c43cdf841c1b577eddf7d26c218271f866c6dc700081c475ee0040320388bb1b4df25399fa363629b2").into()),
				PublicKey(hex!("974d71a0038c3cbe5f1ff23d931a30aa059923e2b79b8013027ab56b11083eb00011d25e436b1b0ac628660561fcc84c").into()),
				PublicKey(hex!("94b09d9fd31286ee6d753e322c81c7e96587d890d6bd3dbac1707939b8122c6ea9439a41b3c621e8390d3ece6db5b78d").into()),
				PublicKey(hex!("a17dbb107743a9f9b1e91c3f41968e067f99e19f56138d72513d9f6f0e9c54a312f4efd3573dbe9aeac36ce0b41228bc").into()),
				PublicKey(hex!("af2d72994d6aa4ff8a736f9b214ed59ae149d25e3ae9c4873568491ef2be5b661b209e1abbc80e1a441968635385b383").into()),
				PublicKey(hex!("b880653c96b7b0bd61418653d224aa02c2670297a186ed767acc665f776d648d2f6ae5fcdb0b14d2101283b62e968a24").into()),
				PublicKey(hex!("8a89cbbc51cfa851c72634dcba30f948fc4300c967d058bebf8419061dde562ec49a6bbd52e0ed1939ce4f8d6d9c0d77").into()),
				PublicKey(hex!("94740bccf18e00a5aefc59caeeee9f191bfcfba9d7dd4dbc9146eb67849a48c9d1a525e5b4616a9edff0b94c5b59208c").into()),
				PublicKey(hex!("94ed1a52135fb327ff008ed8a2db9615d7fcbfa47b5fce32ab358e16eaa524d8bcda8a8db8973ace0c5a24b0fd096ffe").into()),
				PublicKey(hex!("a9651f259fc551195eb5e018ee33d60a55b220d7840e5024a864cd22020e45e9c3f48e9a32ca9039bc969f3d90a48d4f").into()),
				PublicKey(hex!("ad41f14b55227673726946b6ea99b741addbdf090370ceeaf86c864ee45c048c0ab247f176925e647bd81c1219b88f55").into()),
				PublicKey(hex!("8faf267ef9441a897301a5e011f85a9a069b0695d5f3e378e9a4d1f024cf09cc32fa02f1386d40f20a0e21593bdae6d3").into()),
				PublicKey(hex!("ad2d6d7f899ef1d4139a1471aaf10d083ea7de2157e079038407aead2401c5a75792e25e0a58bc6371d1dca986b2c9ca").into()),
				PublicKey(hex!("873c8c40365d3e0e9e92809a2b36e62853eb38ee12bf0953eb5a48c8941f7a23f2b319714e258989c0e4c52170d568f3").into()),
				PublicKey(hex!("b5af760fbdc6c08433e8eea10a6c495a4bc6ed78d028f7eb95fadd66e4a724ef2cb4293c292bdca54a8fd07172d04108").into()),
				PublicKey(hex!("b2a200799ae8eed5908307ca3044f0a3dff7fdfbb901dc7d570dafc85739a557ac8819bbc30a42db0708ea5f96d6ffdd").into()),
				PublicKey(hex!("83e1155a802d0195a5d25f6cb45cfa98ad1e0498670ece00e7d67ba80eadab6604918bff2a8583f39ba2619e9e228f90").into()),
				PublicKey(hex!("aaf161f1f7c194076befae241055962218a775de657d641ce3552e6e1f317f3f2f2f48706bc80ca46657b839f987d3e4").into()),
				PublicKey(hex!("97d933c677ab31f4e900543e781e67d357b3535442a35a3fa7f6b3d7c0e42593b75157c7d8c99efbdf1ff0da2bb8f74f").into()),
			],
			hex!("70000071").into(),
			BeaconHeader{
				slot: 222472,
				proposer_index: 10726,
				parent_root: hex!("5d481a9721f0ecce9610eab51d400d223683d599b7fcebca7e4c4d10cdef6ebb").into(),
				state_root: hex!("14eb4575895f996a84528b789ff2e4d5148242e2983f03068353b2c37015507a").into(),
				body_root: hex!("7bb669c75b12e0781d6fa85d7fc2f32d64eafba89f39678815b084c156e46cac").into(),
			},
			hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into()
		));
	});
}

#[test]
pub fn test_sync_committee_participation_is_supermajority() {
	new_tester().execute_with(|| {
		let sync_committee_bits =  merkleization::get_sync_committee_bits(hex!("bffffffff7f1ffdfcfeffeffbfdffffbfffffdffffefefffdffff7f7ffff77fffdf7bff77ffdf7fffafffffff77fefffeff7effffffff5f7fedfffdfb6ddff7b").to_vec());

		assert_ok!(&sync_committee_bits);

		assert_ok!(EthereumBeaconClient::sync_committee_participation_is_supermajority(sync_committee_bits.unwrap()));
	});
}

#[test]
pub fn test_sync_committee_participation_is_supermajority_errors_when_not_supermajority() {
	new_tester().execute_with(|| {
		let sync_committee_bits = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];

		assert_err!(EthereumBeaconClient::sync_committee_participation_is_supermajority(sync_committee_bits), Error::<Test>::SyncCommitteeParticipantsNotSupermajority);
	});
}
