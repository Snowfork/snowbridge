use hex_literal::hex;
use frame_support::traits::Get;
use snowbridge_beacon_primitives::{SyncCommitteePeriodUpdate, BeaconHeader, SyncCommittee, SyncAggregate, PublicKey, InitialSync, FinalizedHeaderUpdate, AttestationData, Checkpoint, ExecutionPayload, BlockUpdate, BeaconBlock,
Body, Eth1Data, Attestation};
use sp_std::vec;
use crate::config;
use sp_core::U256;

pub fn initial_sync<SyncCommitteeSize: Get<u32> , ProofSize: Get<u32>>() -> InitialSync<SyncCommitteeSize, ProofSize> {
    if config::IS_MINIMAL {
        return InitialSync{
            header: BeaconHeader{
                slot: 48,
                proposer_index: 3,
                parent_root: hex!("d393d8f104239a867d99a044afb51cb868cf4d80fd4feb879cd819bd8e3195f8").into(),
                state_root: hex!("b48d4dbbb174fbfaaa5b8d5df1d54cb96f514c9058a73324c9f661b794552fbe").into(),
                body_root: hex!("5b7f8ec8c256dab49e816d265e39df6ab4e0a9bc79e679e0dd5aa3d58b534da3").into(),
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
                hex!("3386e0343ff03c4a148563dd4cbb5a7ad6947087ea6f6e7969a23ff52c535a11").into(),
                hex!("dc064afa8fe4539c0b6bfbbf8090bad226fa7c884c2ec433298a4171bee5fc4d").into(),
                hex!("3d5d83ed07ab8f6623824c1f1db0ac955636b96d07c24fce50a4c456b2264f55").into(),
                hex!("791843f852bba1c691eb0f060da8c9700dd6b40675f870a0d305b79af79ea86d").into()
            ].try_into().expect("too many branch proof items"),
            validators_root: hex!("270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69").into()
        };
    }
    return InitialSync{
        header: BeaconHeader{
            slot: 3979904,
            proposer_index: 205890,
            parent_root: hex!("129f2cfed1924a35c783b61d21b7b4a146eb8f35a8b9da88cca249dc38167cd7").into(),
            state_root: hex!("24768d4ff1fb8008c4495de4bf53df99ca8deed5bb3a761512f9fadfede3d3ce").into(),
            body_root: hex!("38954c4a68796c2849915cc0862181948f876d93ddb868861d1e8e7b6f084c3e").into(),
        },
        current_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("aaa5f12e4c6503fa7b582037012ea84ee4d100bbb6ae177fe79a787639a06a997fb060dcc633c2516d7ce5e00ae7ac5f").into()),
                PublicKey(hex!("8e33e9de5bdacf5498f4d08ad5a5d66e5973f9a2b26f9531cafbce19dc70f7f6da38a50851b214a3f94575aa08211492").into()),
                PublicKey(hex!("8dedff1d920bea13aa136706a60b3fc19d768e96ab5bfc3f6e97156baa4a31145486491c351b17b8f1849f36470233bd").into()),
                PublicKey(hex!("a3c68dba2d79a02d35eaa4b76a22d5f8d8496d9a718c984ffb1ad3bb40dcb9ef3bfe5fe9d5e054d78287185a84d0de85").into()),
                PublicKey(hex!("825ad4f3414228b3202d7c5f9d7c0b2de2f30684ba23075e0c49ff6521cbb9a51f331ce53ed266e2e025ec8b81249ed1").into()),
                PublicKey(hex!("8abd92e1e5724b9fe0c6b22a894dcfdffecd548b0e3fc8b1a027eaee96ad5381fffaf6256bb87f7722afddb076218205").into()),
                PublicKey(hex!("9357c07122135e26dcf1aec2238b8463125dc7dd8daa1eec48213656f7fcc5ae05dda3113897f2cb2ad09351889eaadf").into()),
                PublicKey(hex!("90986d3cead49506d79fd607bcd5143a93a72c2e143775e24324ab2ffb060e7144c6d3f3a5833a8d2ca237d438e655f9").into()),
                PublicKey(hex!("a3ba6c820b35cce17c571b99e9791aafff72ec39368b652174f8ca7903b0d7a9814bb9386bff76262dff17c8d5261b58").into()),
                PublicKey(hex!("928fafa5d75e0df2c5f2809c2515ced91a0fc1ac81935bb51bb3f931c13a0dd5cba806ebe09a90c66b794dd6f742415f").into()),
                PublicKey(hex!("a3f2a43c57660dc45f08c54f2c29512427b88f419c313247c8d7d87ea9692784dc15d92a7ce5bbdff7755b9bf431683c").into()),
                PublicKey(hex!("8e5eac574024216fc4c130f035e6b10c2d21052bd4c2f0d09ceb4580f7cec4e6fd04baf0d6c7dfce0aed249afc152623").into()),
                PublicKey(hex!("873e738ebcd8be037f168e1b65dfabd36e121cc9cd3cc5d219ce877cfea660e9bcdb70d48f59093e0aee169348b9b1db").into()),
                PublicKey(hex!("98c6c3ee33a14015489cfa476a80a002a97212fa48a747d9e77cb247ca71e61ca8b7635409d683f8fa4e4dab7b6fb486").into()),
                PublicKey(hex!("84a93cbb2a6a48d9c816de34584cda2ed3d1f59507c0c8095cc76b632fe16228c5e12fee539c68772ff3a25088cc6e0c").into()),
                PublicKey(hex!("a1974fa680f2a2b61e62a622736d9599749cd5993cdbe581ab23ed3fe64590d6eb18d05d03fa2de5b9bf1294ff4713ec").into()),
                PublicKey(hex!("ad54aaca2711de451037a95b5faf4765360acb15fabcea043169cadbf884d5e2de1e145288ca667f5bd2552a7c455fb7").into()),
                PublicKey(hex!("a9ce01b3d65e10ddbed67c6e85bed9ff71cc23168ce8a0548fc6e8aac791ff8f387646ce8bb96d28e5c3b006514bb585").into()),
                PublicKey(hex!("88ce9cd24e87221a34de1d61c97b243bc15fec1143700051f9a2f5292dd343386e60afc3e33755dd743ebedc2e5136a6").into()),
                PublicKey(hex!("b5427567c70b3a528decb1b14cfdd53dbcd44686eccff3d6d4daed9cfc8a32ba57d5ae4c582c57ee5ca561c28ed25215").into()),
                PublicKey(hex!("83413739ec3d9f53e80f1a79786ad140788af16be267a1441bde6ff3a8caf84c3775028d6fed6e542577787b33e71799").into()),
                PublicKey(hex!("b4461ba87fc016f6149755bfbc8b20489bdc424c14c002f3a8f8b1baab5925acb2d4533bbc686217fba0793d52b26afe").into()),
                PublicKey(hex!("80c32d4595263eb19f4382e19b2e09768e5d407ebe59a344e0d080846d632814e72c2185cde9c0ebe435645ea12fb984").into()),
                PublicKey(hex!("8e91d9acf41c0e3589d7985d7e2b8247060286a07817e574e199f423c1aa49b49028e9e521d7322530619d3d39d50f2a").into()),
                PublicKey(hex!("8dd31615467249ef3a909063b0be7880640e7fc7b5c1cfe7212aed44a876ecd19596dd1d8d201ec3e0c669fe7cf50a5a").into()),
                PublicKey(hex!("a095a76898ca2c70937493f9835658fa89540488d2cf47358780ea9b6e6fbbd83408d97248a65df99007c544fc8bfa22").into()),
                PublicKey(hex!("a4063852635ec1615c95de10622d6eaf597426a3e946c4116940d7e5d0ce82bd36f4fbc89be53ec86d79b75c79b3bae3").into()),
                PublicKey(hex!("9478fb43b735c6d2a3c398417e90dc00bcd45595f4564c22284415fdaf720522eb0ba649b77bbec769e023be806e16fa").into()),
                PublicKey(hex!("b7c3508e533ffd4837bfde3aac049b310f6f92132c4c50836144de5bc7ba10fafab8d9fce5fff28a0ea097fc6770a507").into()),
                PublicKey(hex!("870cebe53c6a17841f17800c636c26dfda7ff365e55dcd6ce292d24f7fbd97750dbf42d3f21bb859a0e08c7d89f67745").into()),
                PublicKey(hex!("a1d22e9c7cb92c5deb205942148bbaa07286558baf45a56ae4ac3709f682eab7f19b4cc2d2a5f26a72db448a9bded3fd").into()),
                PublicKey(hex!("81fcc8a25a41859161bdd5ac14d007b0ad0528ebfbfe7fc464bf6247bdac960f1f7d0b980ae3ff15c5909316652cb02b").into()),
                PublicKey(hex!("a73bd35fd39e110c38494d82f30667e64ea4922bd6b52086f9e61ea5ac6585c206a9ebab6a3184b2a781c7fd51205897").into()),
                PublicKey(hex!("b8f745359b33a99667033a6e2b0af6201029d5ef030f02ce0fea910cb36c58025cee7ba55a5c0071d5d8e7d9d2bb7ecc").into()),
                PublicKey(hex!("94e14e03de977732b7c7faa60ec8180e77233a43d513a37c443be4fa0bac64308d6a1929de075b5d51efaa9bbd6855f7").into()),
                PublicKey(hex!("af912613cadcffe4b1c740a724cda44ce2808d35f9c0d6c166561f4016498679e8547f5a83ef4c8c8593298b949c713c").into()),
                PublicKey(hex!("ab10243155069ab62d5f7a443dc97531835d5ca11dda30ee3e745a3b28d6061c0c7b94410e5d736b322f96eb084b86b4").into()),
                PublicKey(hex!("993bd676c1e011c88313ed4ac1d7ca636815b41d55b551d503672528cbb58c7ff39a06cbc1680242901637f8845d6dab").into()),
                PublicKey(hex!("8e32af2b1fc89ee39254f4d047ccc099f8a3c59f49857f64d5e9623d3e0289564f29a843cf907ce44116dbcf1be7b38a").into()),
                PublicKey(hex!("96bf26b58995021cdb44abc2d4b8d30f3fdae6df1881ddb05587d964d53593d1823d294eba8182b4cd7f6e33bb6d9ace").into()),
                PublicKey(hex!("b835b11341cb78061f3b4a398e63d078a2e46291c467bfd50e3e21997f86d5ac8a18f65fc55b1605868ee39a5eb73580").into()),
                PublicKey(hex!("8433234df1a79872accad25ad663328c125227c9fcd26ec8c072c1fea692d8709b5b3a5debbb51b64f9e5fcc2a999611").into()),
                PublicKey(hex!("928c080cc97eff5415af4d9d562a7b2eceb4e9f3e3f5fb3064b2ba8165a05197b88a09a636db71d82692edcc997119cc").into()),
                PublicKey(hex!("a7f5d7e9412c56d81a75b3338cb2c474405a815132dcb6a2f3975a358a0aaa335e2bc75d62e903941e6ad4d79bce8246").into()),
                PublicKey(hex!("b0f024a6e1346f6723284264eadb4d2fa5b812e6d6443fa64018de41e4f8827f17dec28e4595b320aa0855f1c0e443a9").into()),
                PublicKey(hex!("b719a3f7cf01899c702559a1a8af30172a05216d94d8c75185902a517620333048de27249725603e1a476755709aee7a").into()),
                PublicKey(hex!("a7e9f67759b4023138a1d4b1fc71c7313795b9326d8d0a7820259ebb39867c6d66b4e9eb0726668d43ba37b71cc923cb").into()),
                PublicKey(hex!("b4679d182ea7fcc7f132d665570a22a8345594928c234b26e6cdaef9666f39e58563dea4f482bd6bc878fcec009ea4fc").into()),
                PublicKey(hex!("8e49f125a6bcc56dc8221c79cebd8d10828200d2fd9f6a9a24dc0221b9c14f817cd201c3a9ff0b4b0a975c2b6cf67861").into()),
                PublicKey(hex!("8c5a4acec55c4092b29bf02041e0f918693fc9552148ddb38c09b4efc7a30ac9eb4b78d0fc94c9e524a318f08248e24e").into()),
                PublicKey(hex!("a1cd4cd3ad88991a1e171c7f3ecd8bfb39f1c317108ba8758c8db364a4040fdbf307b9a306bfc5428fb3ed443cd29424").into()),
                PublicKey(hex!("b69b287fcbe3b66fa08aebce44e6449c52efd10dfd61f7b3cd02d0db0a6af07c57ab13a01b246de14303cc13b2dab282").into()),
                PublicKey(hex!("b8b23a681476b712bb882824366bd34c6afc365d63424063c053d7ddde5bc2bddcf440453c8d0f31b369383000f21471").into()),
                PublicKey(hex!("8e09a756ef56a203275913f3d1aa4c3d8824a748961f209560942ee5774595829ee17817b38742b6b8a52316b5d087c9").into()),
                PublicKey(hex!("88f8d2f832ffb0b3f5944dac728fbc5841edc531a047fa4692abeb658fb3e187cdd830f3d4427ec3f74b3201d17fdbc2").into()),
                PublicKey(hex!("aaf1ee9eb703023a96e0f64abad42ec314a5d9be9326656bd4ac382bf268c8a511e0539725f77798331e7836eb41aadc").into()),
                PublicKey(hex!("b0c15ec28af33480cd1232ef461cae9d73029141d56592c9c35a5fa5294816cb8fa4b1030fbc6e0585a89c17eb1f6f46").into()),
                PublicKey(hex!("b56e8f38b9266a1f8d42f9fc85ab23e5c7717b749d22ec4471bcdb9c10ec5c4e88e7aaec6da4e0fd7d0a4bc05fab8650").into()),
                PublicKey(hex!("82eec051edd198d6d12070bd842a0b284c49b191dcb0a13278efb97f6e3f6c280f81768219b5ca8867b696b402ccc0c8").into()),
                PublicKey(hex!("8c677d9e9a4617552309164967ab4221db8d3a90eeaa16fed5cca81aafecc88bbdd37ee04979b7e1824cf6e5af47013c").into()),
                PublicKey(hex!("8dc602d2ec3c34bb76d002ba3b96dac91a2eb811b5b0f5df9f53ebac3c527aa89292f14ad022e01b887c3c6da776672f").into()),
                PublicKey(hex!("81d205e9624bb0420aea6dfb0727863a91b1f480b21bb81b1f3c06020bbfacbc3d42cc8b74fdb31472c934b0a10ff169").into()),
                PublicKey(hex!("b78ff764b79898d2aac79fa64d27721eb6c5ad1a448a734ef54a47fdc1541d68c7cac57899296ec5c5b159a0167a36aa").into()),
                PublicKey(hex!("b7c2ea48b17be095f9c0836384c23e0ce7d2eb3d88c8dc9ca6d8139d1a9ace8aad6f937c4b10c80280c13c34485e65d9").into()),
                PublicKey(hex!("b635dbdbf344da994eb2be6fae0e70e4b3892462c4713af681a8c3039ada595743a93b19b40674fc54741eec49b19600").into()),
                PublicKey(hex!("92e21887bb0b3289abf8ff8b7e3bf02b88e47c7ff9fd9a6baf486c4a3a1ac3cd9855f4d5a284f7c9f582ab3517925258").into()),
                PublicKey(hex!("a01bfddef75942269279a260d68a827fe5012836e9b5c1a80888d2fcee7ba0dfac38b4f2073871850140aa3673ab8c62").into()),
                PublicKey(hex!("9206e579a61df2b2f9dfc19bd3a7e542e8ab9fe2422c160d1baac5ffafc19b779bac5c4055ad2e436482fdcac46f9d9c").into()),
                PublicKey(hex!("975dfc6f4d8fee2c7b65a76a8e18b928b4aeb645ac31189bacd3294ef5d6b1c4e6be2b8ff05ead394beb74be442360e5").into()),
                PublicKey(hex!("928fdde439b9600652ed9c796f81b1907fa0a700d0cfc71164f93aabc51928e3bccfb0384cf3f3e44c150329c5c16067").into()),
                PublicKey(hex!("8e47d7fcc499e9b107883c03b2bf41d20bf4d50a3568b3cff10858e8029c4e7930693355034817c73c460ea998b6ae62").into()),
                PublicKey(hex!("8ae85f5afba076671bbe06518fee67a0f3111fd13e6168101b3e4431231e538dbbd515f0f5f90b4e7a0348c07624d41f").into()),
                PublicKey(hex!("966aa666264e6474db4221b1a725b08b096943a8d1df77ed9fcb44a5810ef7c888fe717c68f681f1b2754fc44a6cbc4d").into()),
                PublicKey(hex!("937680b3daf4035d545e661899a0e5b0f2e646f1bb731b9fca990b49897bc6b9853f4d8029ae7f5529b164be317bca51").into()),
                PublicKey(hex!("81cc4df78b27cda2165018744c5c8df824f5439c7b97ea1248ae48dff727514fe373ec61139ff80578a478087636436e").into()),
                PublicKey(hex!("b33b11ebae624f6bbb59314794a26dfbc914cb0ea058e0cff28bc79c2f237967fe3b4a071e582e188fa3124b338f4ae8").into()),
                PublicKey(hex!("a5a46e05f2adf0a08ca5a5a685d5b680c584a0532a412d2ca9239c6307f687c937be7cb05298c3a23d3d70fab4b625dd").into()),
                PublicKey(hex!("b96b98af5ba1ee4d521f66cc5d995f41f02db7a8c760824007fde820731d64151de4e1811eb54fd91856b1e1e7c7e2fc").into()),
                PublicKey(hex!("8319f853f1431a60cbbeb27e627f74de90043819e82e65680446dc1d54e8b714372d90605b7da573de36a7c1208b8701").into()),
                PublicKey(hex!("b783e8a323afa3582017438fc875c16155c98870f271e3d78e7d164371e446fd4a9eb7ba601ce6ff7e6c06977d27ed14").into()),
                PublicKey(hex!("a20f9d9771bda9eacc3950299b6f690c169c75f8126208aa6967fdccd15b816da221d64cb8cf9682200767d073c9e894").into()),
                PublicKey(hex!("943147c65a7d08c5e72dee29563c2513c6c08d9e94ecd1d890480bf84beaa57f060de8f2b7ef6477251b01ee78ceb0ae").into()),
                PublicKey(hex!("8bb7232ba1b636d3e1107673d4f65b7dd1bda5e25bea41c3590e063e74beb4cffa18a62d14532063988d95ccf991953c").into()),
                PublicKey(hex!("a5da5814b8090940b51342b6b67384dd11cec7afe5b9d273cf65f7a4113dc98e4b500178a887ad703fff6e1ce0a834f3").into()),
                PublicKey(hex!("8757e71c2737327617c1d489b62dbcbaa428a2ba9ef8355cb9645723097edc3061671910e030a21e24b2bd1cd85c50ea").into()),
                PublicKey(hex!("93cb18ba048a3dd1cc7c79948033ad61fcc7e85749dc1d67db038a174697c7198d02bd084a029e84dadafc1f36bda298").into()),
                PublicKey(hex!("a2ee6a4062e0c5c3d2452af1e973bb0a47f559cf134510a0da3d10c307a90f5e64ffec367e929ed04783d114adb8b821").into()),
                PublicKey(hex!("aef0ea18dbe7d6754f41c65f5305054ae2908706a2d0d0796f834cbbfba7203aee2c274d07e75708fbae118f5e2e0ffb").into()),
                PublicKey(hex!("a9248ce1ee41d8e3604c5d5d7e43665e526d5730fd72892d430f42c979c4e588a33386126777d72826a6943d42529c1c").into()),
                PublicKey(hex!("809200985f166faac275ffe627d4262a3740d090ea07866e748ca50e0637082c73ab4607b7599c064f14f1487a21f9e9").into()),
                PublicKey(hex!("8686562860ae6d9b7fcd04318dcd627aad8c08e0e44f7af9bb2812c178242b1cf17de5412ac15d0f1c76fe189d388e40").into()),
                PublicKey(hex!("ab81329e506775c015415e248520ba94abc29cd68f0bc2816671d521440e94c8fd333975d4f42b297b48aa1ad511da1d").into()),
                PublicKey(hex!("aef5caa9905ecee2a6348cbba811f7da590a381818595828cffbedd7303e58b8354e7ef74badd227e579d53fd98a0b0f").into()),
                PublicKey(hex!("92434a07277b242b486e6bd6601273d044a1a70adab4e8aa0b74678a7da2decc59987adf60f3a24822e9f4694914cd10").into()),
                PublicKey(hex!("b720b224853f5682a6558e8347e55c3e9681c75065324bcee444227124183ff07bdcbc2757c8691ad13856fb63a395f0").into()),
                PublicKey(hex!("b692325a485010af65670f237caf5384123cff64079a8c21fcacc9dd4f243e21c3f9cb91573d2756fbb3313d9835bfb2").into()),
                PublicKey(hex!("a94980d2b591bcf741801b5f0a9b9f6309f5d278f9aa8bbd6d02116ed8d4297cbd0b7f48dc196a900174bdff16d34b3d").into()),
                PublicKey(hex!("b6aec4dbf5d4901d3bd8d4fabd0814637aeb671a164d1b44770a4b6adcea93ea5ebf680cc08de86d5ce448c1d367447e").into()),
                PublicKey(hex!("b0e5ced48b2073468dc2853daf873bacdd0b6f48c0ec368d28667a3122d6e32ecbc3b279f8d44f832e5ddaacd49f53eb").into()),
                PublicKey(hex!("9552f75fbf587d81ece4fda4880165711036d460e31e552db0f9a22fcecc47c37425eca150ea3dede55286a94dc56197").into()),
                PublicKey(hex!("9275951888556bd48ac14c49b80e45b4077f0fc25fd3577ad4fd9f9ac183026c6f50e01734ceda04f6e54b1078ba4574").into()),
                PublicKey(hex!("9840ce72c950f581f1a7a280c342acab55ff8db9bd2f4aad9175c9831bdb81befeb307d6355dbed898ed0254bbfb7406").into()),
                PublicKey(hex!("8defd58055b484a61f4e43c4103bff0ac1148d231f9a197899c158196c6609817a77ab19a68809aae4fe6729f9974ead").into()),
                PublicKey(hex!("8de5f0d88c22e44c35bafcf664368877fa0c1e5bfacd196965408d014080b29857048bc6550ec0429dee689e55f3f3fd").into()),
                PublicKey(hex!("b85d78362565f3b55f498fe475d0d35aaa8b789b3f26a144fe29e4d8232089afd88242aabf2e36344e4da25a436c7cc5").into()),
                PublicKey(hex!("90f7310928e21d49c96fad33f5732fbd23aeac7499d44ec91c4a43a02778f8c3a3dc3e0daad324df411389ed7fe2743b").into()),
                PublicKey(hex!("a382721fa7034a2497cb8097f7a69f0e3446669582fef733c9c359048541d22ea8426842e4e001a38f15ccbe662b01a6").into()),
                PublicKey(hex!("8bc9896a4d54eb1ab5226eb436b9fefba67cb10a326bce1a5a171926f526dda989cf71ea59e0bb4b9c572eed71cebe57").into()),
                PublicKey(hex!("b37385d05682b3c7d0223561bef0dfce6dd1d73138d0a76535ddc55f913ff4a40f0f3669647727271303d647003d321c").into()),
                PublicKey(hex!("8b63fc30191028e00e4cf74593ea209c1e19e9d08e1119ffff272564f384f0c652deaa00ae7c6a33a59425ce0f27782b").into()),
                PublicKey(hex!("b62745fe6705cf529f0e11516b8206cc8856dc2213531cb8aa0da69fa379afa9832cc2b6b5070f12161bc6733404c572").into()),
                PublicKey(hex!("b5d07440e851e0487846d3461f87af3116a1ac7ad2719c5e159f4b4bfd0ff52efdec3259b04949eba2f354c492bf527e").into()),
                PublicKey(hex!("b7abfe20878f585e497afad88e8edbf9eb561cafebaa70c187d6e7558a231436a8a5d59946dc722656c0497ff9a1c317").into()),
                PublicKey(hex!("85ebb7501169996ba17cc98bed91f6e225f56edfc665db3a508b321b3804fbeecf6cea587ce8481df6f18dd51ce084ac").into()),
                PublicKey(hex!("b1020e48b4ae30a011c7889a2bfb3ecbd37af7fe4ddddd905d483059e42dc71070a67f1241a94de375f4941ad30ed65f").into()),
                PublicKey(hex!("b159dd9c7f7f7274d3259aaa6ed902150aa0d546c454ca0a330f4b8d3a72388a4a3df3f9c80bf8d9ca22895ef4252c79").into()),
                PublicKey(hex!("b2f4ad72938590b2f6395433469aef8d372aa3db5f59623abd57c746ba4727b9e00ef2307c2fa315f76b2ea3f66b2566").into()),
                PublicKey(hex!("8c5a40d20e95717328c57a41e49f9991278bd65c41e235f5ca6dd626b8900878c7668b2609adee556825344b05d80d3a").into()),
                PublicKey(hex!("92918e8ece57bc454d8421ec81490348560d38bf437f4f830b9c9764913b9b4236011db26503edbbffd4fe118109e220").into()),
                PublicKey(hex!("a3037156f5399b3e819abe438df7c2cf74e48317ab3c824c03da9808b8648db3f7bf4151135db6d7c2c7469b6e9ff459").into()),
                PublicKey(hex!("9380cfe72b521393a54d3994b0767b761a97f30545033f6b8fe4a045ad08fb2935e45d9666d686efc0b099f4ebf69baf").into()),
                PublicKey(hex!("89a7151a2c04281f9ccfa3cbc3cde68c182e6de5728f683eac49d8a89977894fcccab92877a2801c02973cb069737e47").into()),
                PublicKey(hex!("890ae41b205c04abcd18fcbda2087a017dad9b940c33a8315dfd09fcae8747fe5f54ac0058753297441012154f6c2673").into()),
                PublicKey(hex!("90ac34c24cc88a8f80541ddc1a2b4d35bd9d13f648aee23bd044d2b3e87e29a2351056f0fa49e15b0e2ac124baf5a05f").into()),
                PublicKey(hex!("81d0f4602b2cc1a1c7f2e7b6e20de8aee2b9d2cb9fedc866378197e180d01f17690d444c749914ecb08729d181845d77").into()),
                PublicKey(hex!("a9abeaf1c00f99ea38e92de9caa4b079fb71ca0efb39e4dc01b53aafd7b8850b6bf2669da354f30f5fecc3a9f6b8ecd9").into()),
                PublicKey(hex!("8ff6aaf3e52da22f6e1b7f4278315ffdca632f07083b22974578a731037f5e10cb02d05d16a645bfc900474f09f8ebbb").into()),
                PublicKey(hex!("b4397aadb9b67edbb518193dad2e8119d8430170e6009bf3cfd77059650310c6d6e7331ce263ac74e180e5a044486b2e").into()),
                PublicKey(hex!("b83f8f1a05909f42b7505be96f44f8a238a7588bbf59385edf6b1e1e9ea711958427c9a037f8e5a5cd6cec4658ff5633").into()),
                PublicKey(hex!("a71a4f69741c89b39e52fe54a9942b4ab231fb278e29a5ca049435f475c9e6d847dc299e8af644ea38d930c856e1e60d").into()),
                PublicKey(hex!("a122155d6adc863b7333cf679258da8f7475397affbe7ab9460ba7f15fee3a2a298e03e7f9ed8f420f37193eb9abe419").into()),
                PublicKey(hex!("849c159a06c2c9dd30583e077d434f6b4bb5f148630f45afa1ec620f9e827648af659b7df596c8132f476ae114bb5e49").into()),
                PublicKey(hex!("81fafe5bfc5f7dfcb46698ec79fec1ef83937fe74b3e6594196a90752a4a8653da222da8db4534632e491dd188aca2aa").into()),
                PublicKey(hex!("983a8b916c0287ff1d1e0c2a2b95d2189f19fd313aceba9b0b382e686c24734a15d80c7d52826bf396e9a4f471026957").into()),
                PublicKey(hex!("89daca52da9cf28a043eadc861f0d3454c98e9964fdf069293bbf646a17cdade7c9400dc270192811591ef6a159367f3").into()),
                PublicKey(hex!("81df7e53c95e51e70d64327477571e8e7734a6a06aa44e308afa95a934b65546020c51e5b8ad6ffedb0a195817e2fa0c").into()),
                PublicKey(hex!("84c0fd1a4e650c4a54bffd93f7c226a65919de74a1580a60daff014013a27c39a325b117b46fa342e3e603921b0057f4").into()),
                PublicKey(hex!("a57b11b840cd4a65618afada6b7817e10e438ef4588fcc15e4c6031919eefe0eec9be5797734ebfc2b52e5be63cd7aec").into()),
                PublicKey(hex!("aa5254c277535ee33a3a711118bda9462d0106e6b45100351639329cb6d9b11e47c99a0f9bf565699cd43a21ba6dcbfa").into()),
                PublicKey(hex!("97f5b40f12ef51c476e5123e8958421d5461129330bb0bf31133593f8a492fb6dfc25fe1e04e071b4ea8674856c0f86a").into()),
                PublicKey(hex!("b74d68b63d1ff7ea27765a2741071cc515e48c39f896d7dd84279e41a580b5cc01c32afd251f9052381585dd673178f8").into()),
                PublicKey(hex!("8f4802ad0660896642b878139242b5d58095ac30b24d43d4b7e05f001d86149d5b4c74e68a7390c326e10b5769ea39b0").into()),
                PublicKey(hex!("90503de4c181e2aa1a0be481d866dc2087c276cac8a89c442195534f541a1d8a9c49280458b5e54eb6a9aa2c53f3ae00").into()),
                PublicKey(hex!("95e35567addd1329046796fdeed26e114553790c1824218b7618f117795c6da60a549af85ebae39358058d08d500e3f0").into()),
                PublicKey(hex!("8d7126e89a455352be94e3e29e4a1f567770e38f77abf1a48c2e7419887782c7b2ee4f66cd7652fcb7667ef03865c058").into()),
                PublicKey(hex!("b0bc540b2e09e41472f8dfa537f85a5b2ea60ad88d8a9bf3ad183cdb4792179a1dbd7b65070aade0cca6bbc9dbc848b7").into()),
                PublicKey(hex!("b1fdfa2257925d4d4df6911c373c2288e90aa82945258c17a5ddaa5d1b8c37464ab61c8982dd48bb44e6e595c2afa7cf").into()),
                PublicKey(hex!("920acf761419f86a16976470801937340f37355f197f9ea5b282355462e7fcf97d4699c330d587024c58b5efe63c2932").into()),
                PublicKey(hex!("97172941d154cb1c1fb4ee3a428d7ef254c1111a8f633aac99ebec2ad1263d186ca925fe6996251f46908d0207023cec").into()),
                PublicKey(hex!("8affa36f9bc732c4c68212d44afc71bf3b849a7617e6897d4064206901aa4b9dcfa97c1e3cc7db07dd9cec617cf6726f").into()),
                PublicKey(hex!("8885f947200ad4ec96930da2abbb161c62b372c7e9cffc41b579295f93ad2fe6195540ed1533a299049df71d33462371").into()),
                PublicKey(hex!("b7833caa065c2353a2df11328e53380e799f3ceb286377fdc78aeed78aafc38a4b8a374bf90275ab2d9afea87d9d3660").into()),
                PublicKey(hex!("8383fdffec866ddf88172d7007b5fdd789850f16b32a7420775327068c654922a72e05087618a874d75732f516b40f4c").into()),
                PublicKey(hex!("975f66b3d1a3342f43de5527a46b580ec4454894e205d44d7ea5aa4d0a3730b56450e0feded82eadfbaad96c6783bac2").into()),
                PublicKey(hex!("8926d6275a17a52d1368a67cf5776948a61e221a304e06e12a857886c4f93c990e1b995e4af595ad0b6a67f842926706").into()),
                PublicKey(hex!("80ca858b0e0c0c022b6044d190d3431fc77f2e4f1f59a4d7ae54c8f4993d74a78632b66a3eb7d1d03321ff7012ae2b71").into()),
                PublicKey(hex!("ae693c33cd72257eecfbb3cafdf0319ae66cf474147f0da1d1f90faeb907467d4b8977a439065cd87a9036861389927c").into()),
                PublicKey(hex!("8e4f8a0db8ac2d739393de91df5fe90c1ae4a8654dce498cce627f27264068e3b6405a9fb94af8873a2a6d76102d0e76").into()),
                PublicKey(hex!("a00b516bc3112879acb8b2111f602120faa9bd55b479c6bc1fcf1526698d50ba015e28eb759ffe1d3174c9c5d965e692").into()),
                PublicKey(hex!("99ebcedb5848a05a90b0284f487bd1833f025db7c6d751d57a18664902c6457397cb439f2f07cc4c70dd25816b9e3e9c").into()),
                PublicKey(hex!("b4e3278f98510126c54d3d6dedde95da7eda022e29877073c82fe29918242649a942f3079b85dd8dbeb58766c1002180").into()),
                PublicKey(hex!("b314f525fa131f06013f339d8336ae29c51f16c1290ac665158041175689eb508a94dec0ed36f51ba425b3e46559bc59").into()),
                PublicKey(hex!("9752e8ab279900d252495985d6a44d0e0ad9b42ea583ecf9e5edfd89a2db866aa03883db1a19c845d74419e43504ce15").into()),
                PublicKey(hex!("a6ba71ed5551ab00a0dd6775d4da60a5e7a9b963810dfc9cba46a39e1c90c0760299dcfd48fb9a1ca7745298c858007d").into()),
                PublicKey(hex!("b0d663f67fe4e45f7ffb0bc95abb494573a1133279dd25a59c017296f220d39ebabafb17f8e759cf6a60e6437745bddd").into()),
                PublicKey(hex!("a328bab1a96f817bac209d7d350ef0727ee837e6712d01769a566f61be37e3e97bf79c5bbebe18b7626a9e63ed931190").into()),
                PublicKey(hex!("919a79f5561ff9cecd8126c7084df5c359c316c22afc64f222c4d89142f50ce76b4f6485981fa7f092840105c5ec5b85").into()),
                PublicKey(hex!("ad1a0d91f6539572224e076c44af7438146cd1c94579bf4b7da09574ed72cda85b944cd45c39a80049ff71010b11c250").into()),
                PublicKey(hex!("95d29627e2ecb5acac95c39aafe2d5d8e806f26d08150c6933ec8431967a9a1065578a959ccd1d46a50b38815af77f9e").into()),
                PublicKey(hex!("88a53b6acd5e1a466e72b49d6f16d5c4621cd17ad249eb23aecb904a0d13748f23faa07d6f75d3231ed924b526211568").into()),
                PublicKey(hex!("8b61713d337ef0119c94c19a77c7ea564a9dd032cfd0e3bcfc34c1f8db93f039d39fbdde3f3aebb3247b6963e6341d61").into()),
                PublicKey(hex!("9546cc469a16e5b48a634fb033a213fc24e46015e8e5175982b30706d4e96b6afe04640811d39d2e7d5a9f1b1de04f1f").into()),
                PublicKey(hex!("a95da12b299d823e6c72507b74c6b70788ad47297da2282a1934e48c7d052c07ca00712b4640b49cbaf945b86d000991").into()),
                PublicKey(hex!("b1d2852ed007eaf12568beb05cefa80f6297282fe673a1cc05f7270ab0864ce60df8cca37a9272eb29fed23c85599ac9").into()),
                PublicKey(hex!("88a57c79a5bee6b131f2558c4239a0da3f634cf9a5bf505a05f0e6f3818ebadb14a7b8f0122704b3ba4d55da0d34c555").into()),
                PublicKey(hex!("b096cc5c4de707becd58d4cdfda60ff9aaf0d8e3d7cfc150e34b2d41f588008b904a9b80a2215ae64224712ea8b9c74e").into()),
                PublicKey(hex!("82dca076b0d3b56afd51ae6d6daafc708f9cb151c45a427b352001206acc03e228b8c4bf30498af7eb7c16f6cf172bad").into()),
                PublicKey(hex!("89579943b23e87a860b79b2be3a215ac04014c7057bece95fc240ea7cadc919b00cc7952451965c263dca826cefe8465").into()),
                PublicKey(hex!("a4582a26c2132bcfa1ab68b7698ac34935fbf18eeef96daea991c8faae71d75abdccf4f85a7ed65065deea26baf65472").into()),
                PublicKey(hex!("b34ccefaf95aa7ace9ce37c1bba78d048cd0ed4c7444a510128ba87d72b0e634427f1dab8cf201d7574171a8bf1c0f98").into()),
                PublicKey(hex!("842c2e680962315608c5b2008b9bf280effa5d26fc105ecf8c0fafb6c01221164f08dd8a43b577e1e5c87823ad53e71d").into()),
                PublicKey(hex!("a26176ddf57be65b2148fe4d434fa6f419dcd5889bccee99a1726c567684a8c72cc9d5a7054cef9df2dc7eb06df80334").into()),
                PublicKey(hex!("8232fc494c01ca6503da57a2dd8cda3c3218598f549b70cd818844153721165db09d3cd9153e93148df83807e42b6c4e").into()),
                PublicKey(hex!("b9e8c4bc3604044479490c665045cf1717f1ba1fe1f9841f7b718c4dc3ec482aa21feb05dce52f44c693e759b090d8ec").into()),
                PublicKey(hex!("a036ec43d34465faa5db19988b260f0e1b26afef2b437aa02c54af3d91f61efdb87a4ba28b318f00acc9f13c0bf1f390").into()),
                PublicKey(hex!("84c6441a3255f1289837bca60595696590bba534fe6ee5078dc3bc3a6eb7c964f5881fc6a96fcee90aee595ea9a27cf3").into()),
                PublicKey(hex!("b1830bce55af7e1226f9e9cdb5ea083d865de1297354063ff49a51f2ad80e6ff8b23d60a54c8aaab305c29e68a5d1708").into()),
                PublicKey(hex!("a6973c69fe5733094b645777121c17796c74c82c1ce7a088eaf64d6fd5248b5b0a0c0086d52a976f15ea1ec4e5ec8c96").into()),
                PublicKey(hex!("a19cea7dc9b852f67725e746cb6865918efbd2b1d490e538a2512c609f1119d16951b614207ed075286b6943566c06c7").into()),
                PublicKey(hex!("9638c19b9aa1563b9bf98a12e27f427b0e61be47bfbb50c6f70951b223b42eb0e82fda9ea333fb086cf07aef4c91fe91").into()),
                PublicKey(hex!("a3bd61c1536a678549b5c1701553eb21140c2b6ec0c46b9f8e6c76d77bcaa67a78e93a813b03a095e409d785f9eb7958").into()),
                PublicKey(hex!("a8542be9c242ef84016c2acb52f29b453b8c16bd9be0012659ed98f2008d3bf83ba65667bd46118503442e85c66022b4").into()),
                PublicKey(hex!("8035d37b8d06e5fe22daff584a46528b4789090c8b4e7987a12f3d967bbb0afff567b21bda6805c5a6d2ebbfaa57c413").into()),
                PublicKey(hex!("b43c6c25f61e5bfa5e6c56db2d4666880c8c3d9d2ef276d9995fd5b2bbcb5f0a94efdcef25edbe1f69e07fbff4268a6c").into()),
                PublicKey(hex!("908c75f25ad4850f92510c886901e68adacbeea57c3cc3f1a948bf0e8495439cbe9594b9fcc67313478b9cf7024af7d9").into()),
                PublicKey(hex!("a5d717a5b6c4dbd12b0ceed933cf22d0962db598d831ab9887eade8a299519897f1b0240ca057b221a0818e32c8cd5a3").into()),
                PublicKey(hex!("b10aa71a8e88c1715109cf42db90ad4d2881f69e039a8c04226f347ce265dac52f23d319a69bf4c2763e1c758f7a0f47").into()),
                PublicKey(hex!("81e80c0678bbbfcc98aab5a80fb4fb6e00843982c5aa31f181f925e26b09ca23d43ed49488100dd75298a78538ef9ea5").into()),
                PublicKey(hex!("839ef6a9c77e94079243c28adec2b91709d21e6e5c34516b958296d205c17b5a4622ab1a1f34960b8e8b9d05e9b37bbb").into()),
                PublicKey(hex!("a95f31d71642e2b9f6c9f3a02cf884b04a34d1326cd2ffb90fc7544ffb67af3e65de1a70b6bf7154225feb42540eb959").into()),
                PublicKey(hex!("909e0cba2f68c4d8663b80b90b8002280b3b2fb91d845a750a3e58341e58439b8859adaa6dc2f55dcb358a2152b7d588").into()),
                PublicKey(hex!("914d416a86643e0a1329c8ed167512eacfd8bc3826fc4f8145bb870053a9cebfdb16ede805d173e79f4994b41b847411").into()),
                PublicKey(hex!("992e271d262cb497feac2ce47d983fe4e546113a23007f27cddce8e5296eada23edb8d1fb2a58436f68c0e212ec28e3a").into()),
                PublicKey(hex!("a0b1c26dffa10dbeb486f3b64ec227a175e51b1302d34e3b6b9e2d497e1ea63acd1a54019c272cdab29b2b5481416fdc").into()),
                PublicKey(hex!("81011da631c889d67e7ab437a7cf0e3530d9e66c5261ab2d2f2bcfa29d621c7cb8b5097b01d71d395002ad2fd1a9a6d0").into()),
                PublicKey(hex!("b93fbb65e2d85cf38142bf96ee6ab33f7b76a00bc23aad78e5053225ad3b8bc104f92f6c9fcf7c30d9294b7a720f80cb").into()),
                PublicKey(hex!("8b23c516426d07eb1b39b08fc4892a4a3170ced4e5c585b91254996a3f256a810f4ba77c4e1ec92e8155381e6f6ff6a2").into()),
                PublicKey(hex!("83767aae8714c65ba08ac9ae63875fa037a18590dfa788a5fddfd6eaa4ae93dbab01e41af6224844cf0598c2580a119b").into()),
                PublicKey(hex!("a45b08c8437848ae6b7ab9d8b276191bb0bb079bc74866176e024fcf7421a7def19246dc659fad01736c61dde39056e3").into()),
                PublicKey(hex!("91aacda98c93d3ffe2f2742104eb33763da6ef7c5c44d3bdf6826ad57b1bebc59d3ae11918ddea6fa2043230f136ac6b").into()),
                PublicKey(hex!("a2eee701bf2bd28d2db7e9baa4b467f4847f26dd678baf54b7fc675a19361d0d78a922f5374e2637476c0b8ffa8f55dc").into()),
                PublicKey(hex!("83191437db841561b07e5d41b847d8d646bef4570b11cb2759669aab3263957dbfe45f225b5a27e12e8cc02f819d6c3b").into()),
                PublicKey(hex!("a6b943824cf61ba89e6103859f3a6497df4b3416c9db5b587bb5688ebca1a2c9a1024fb36ff1abd54002bc705042c952").into()),
                PublicKey(hex!("89dd716eb0d750404ea8b3863fa1527c5ea1792cdafc08ee2be998d61c749e79bfe3e5dc9ff449cc23509c71eb1057c5").into()),
                PublicKey(hex!("84273a100f4b2ce613620279a3c25e9a5674676d8a6f28d77d91eebf1349af472874a7582ee9d051706feef64e1844da").into()),
                PublicKey(hex!("82d24f634801981682e1e929a3db2d6f4fcec63fa74227bf29400af24fb7deecda234efbe40b3c4471806c3af94a7051").into()),
                PublicKey(hex!("8a3bc87afb4bf91ec613501766bbec1478d13d69744ef0a14f3d7b26b52a5dba59f76b14497dc0f3addcf201224b7b21").into()),
                PublicKey(hex!("b31a9743b4f7367aef83a20fdc19014eda12785d9f7a49959ca17edf2fae071ee8452abb187a0da4f5e78f5a90281388").into()),
                PublicKey(hex!("835d9f0570c18b531952688c1849e296f677513a765f0d781fd1291dc7bb96cc82eca8fb86ae56916f0baae2e2bb5843").into()),
                PublicKey(hex!("afaabb83f7927e1b9975cea26330ce96c007e30088e3fc482809a9041eed1060494aa971dbb9e876292891e4d008302e").into()),
                PublicKey(hex!("b79333a7d4a427710e3507a4fbf22471ad7963432c8309695f7569d5a41dd7224ddba442e7d804c14cc30d81a530d9e9").into()),
                PublicKey(hex!("a611ab6c6da0e0f50974924d30357e505eeefbb81b1ca37b67fc79d5ed259c074401e479b0cf89d4076dc83f2a70cfbb").into()),
                PublicKey(hex!("8b5f22b400e4c6e0e7b7125a3ceafc653fd8b038a705aed35d96ecfc6ee21678021756c490a5d43ec6bd6bbca0c22389").into()),
                PublicKey(hex!("a83cf99015a7d03167dc60765fdf6591a49d503e8587548b9fc09497a536f4044509a3e84d440571544cbabe47c2bf28").into()),
                PublicKey(hex!("910b2e9ae1fda8ae0aca996aae60ed0dd25992ff54b000dfe32f53e992cdbef2ae2845b2fc497bb0bfc37af95c44cfa5").into()),
                PublicKey(hex!("82bc81839c12f113b3d7b785c1d0f5bcb17a0ca7c5513d4df892c5d652f5549d7946a988a5f8ec9aa198ada2f2a901a1").into()),
                PublicKey(hex!("94e370d49ab31f193eb5f40f422287909b688e38f99011c1be422e183fee80deb2ff8d0841dbdae381e24e308700c9a7").into()),
                PublicKey(hex!("8bc2e53bae218d362519f718a64a8af15a7653fe9991852d256ca56b7dc813777fee516331ba2eb09b8309e520d73870").into()),
                PublicKey(hex!("a49618bb3a1b78eeb96834c1bc227a4025f0351efa90f62979007094b6d75acfbe5a8cda6070e33463b46125f863bc46").into()),
                PublicKey(hex!("a54cfc5b0630ae8cc9f5af609156eff9952b2181b817c6c92d58dd96edbbb5b29194aaa8cf9dcfe772a60f579624c3d2").into()),
                PublicKey(hex!("b48af6f4c745b4a619c1f9f1352f392f2848add140a79686db6225a0bc4772eb4833bc28fa0e7ab592099649dcded2a2").into()),
                PublicKey(hex!("87ea59ac67475c163bc9f33b6843eed179b384c3060d5c027b62e838a7eef568a901f15a178eb094cac7b0f8c21252bf").into()),
                PublicKey(hex!("964f97b381bf9a79c23583146b2ebf07afd7e47211269e007ea741a211a9b4bd051f6ba5fdccc234354f78392bec42ef").into()),
                PublicKey(hex!("b25e960916f8240f91349ed531f12def47cb2e1fbcbe36bc61ded00f0da1e1820250723bfbe76674f5b7cdd2bc5a0ec1").into()),
                PublicKey(hex!("a39b5ca7f53197190e12fe5be5c8b332807dca6d7dc46a0a771f1877847bed3c565a693e8ce1d2604b41436f12c96ca3").into()),
                PublicKey(hex!("8082f6b70ea00e5f7773f6825f68c67aff0ff24a6dfe0375aa5c66f7133b69367e7bc3eaedf40726f44b31d2c4d8a3f5").into()),
                PublicKey(hex!("80c031562db3702496af1f62694a1ecc89b0bdb062c541f9cbda8e6d724e39eb8a5ae5cd60540d8397d4b24f630b3b98").into()),
                PublicKey(hex!("8585ddbd8ac00d4bb75d69dff79ebecc59b28772b542f4e95ceeab2c6da8a756f8f0366f7a23ff52346a9166ead150ed").into()),
                PublicKey(hex!("806c369093012dc6e5015c90168458c4f957c3c87d86193c3880aa610c4f821818d43280a1dbd6ed5ee08acc2ca7c3b1").into()),
                PublicKey(hex!("b69ade5bc14ed97b8a16a58ba89d68e9bdaed589ee86680b6cddbafd828ad046597d233c47d851c2a911df2fb1a0e353").into()),
                PublicKey(hex!("8dff0bc9491353dbc2a51e407f4f59fcff8eb27fb5cf4d1868702aff8acd1a3b4b3decfa43db9625355a0a976d98b770").into()),
                PublicKey(hex!("90e056125e802d4f2f25d40bdc1563af7f275d844edbd900843038f495c7289028b05bb79ce9c44a1c57542d8dc224dd").into()),
                PublicKey(hex!("a962caf9d690511722039493ea580b7e2ac0da35b63135ca505eed70197976ece6b56ba1449649dbca69a467227f3a00").into()),
                PublicKey(hex!("a1f3c30f1f50a2f5c256783382c645aac0df25fa7b4721d1d687d8c432a7c1fa177d882bb5f2ea189fb67f568d9340b1").into()),
                PublicKey(hex!("acb5190cea8aee54e7f691e10eeec26d7f4ae477eb5063599062714e08ba054c1005c76a257f813a6cd25fc04785d147").into()),
                PublicKey(hex!("865d0082c6d809e47f46dc5aa8bf982ce3ef39cd070b84bde5c78ab8804e65de706f5b3b7a674ba286499c56fb40e9e6").into()),
                PublicKey(hex!("b5f358df760662916946d4138d412c85dff314197ee716369939d287ea44c0cc9f1b991607ae143cf555c9be5aa1cc97").into()),
                PublicKey(hex!("b52b7ee81f469e1ec33d0a73119a6adcd55a9a3d9e3785e0dfd64c9159a5a9100b6599d1fa39eb2a6343d9f8592391f1").into()),
                PublicKey(hex!("b4d5e029a0d230b4fa666bcd4da41d7601edfcaf4596601c1e1d8f634bbad662f19e7f585d9eb552950f871248e93c34").into()),
                PublicKey(hex!("83656ec053d658e189aef12123d1083b2d08a74829dd54162082da2bb5f87c8c7504ba09ac027625f974ecd4f6d12b5a").into()),
                PublicKey(hex!("a90efedab6ed956edc40426bae1ac6758d5b6c21e4f6136d5e982680dbcd6262c864f33f369153a483f07d09cb26e582").into()),
                PublicKey(hex!("8d4de0d63d9ba35c8e9cd28f27a5d0fbfb6d2ef4f423020045dfda21d641bd08b3cb85a69541f2a0ef6cc770106a355d").into()),
                PublicKey(hex!("862a831527f5a87b631fe71ede3a5788bfed0eef491a43ed0128376634db72263b2771c54723231417a297975e801af9").into()),
                PublicKey(hex!("8e99e096a9f9c9fd7811abccc3be8ffc4a165acb34847ef7ebc69f39ea8ed6d904c24243129ec7b746b7db94adc0db4e").into()),
                PublicKey(hex!("a327d2cc1f76aaaf31e43abfe5b143233d7c030ec246d823af79870d9775c0eabc05153a8e3226a8bdb38f4f100ae28b").into()),
                PublicKey(hex!("97c41ab22ee3a1630a27c6b3cfc05a8b207a73cfa03f4acc5d480f9ec04f5fd5ab0e473b5c8cb199d71a32381f8608a7").into()),
                PublicKey(hex!("a1713a70ddd9d405dea03e6b681ebc26e61759948e77548314f40d61a36db9f7f469f4c164287a5f450049423fa8e906").into()),
                PublicKey(hex!("a06ca3975dbd93c95e3debe03f668f6a934c227be6b00f47d5705312926b0edf0d3dd5a0bbd910c0989265c5b0d6cb24").into()),
                PublicKey(hex!("ac82cee668355813b33b5a2fe2b77920a38edebd72fd30b9ac448694855c929aa62a2ae3028140bfe2d07de781b2ad28").into()),
                PublicKey(hex!("8f4d5984cb127b0c316f00ab0001e41bfc66df9b722dd0f4b73d7e8661240d1af313139c79f0ad62005e1f7bb217ee1c").into()),
                PublicKey(hex!("b74c99b38473ea4e12324db61ab52780841fd61f4dd93aace6b38edfb24aad72488db28fc03ab35c2cc2cd3ce06656ad").into()),
                PublicKey(hex!("b88200a20366d7c496f34179b52e2e9509f8f16de33756be23dc102df5291e620ba53ba2012dd9f6f0fb22622861ea0a").into()),
                PublicKey(hex!("a9d4cb70f7a24793e7023e424e9b5c54529d460a76085b12de343447d453fe10359a9e15e73d0251886cb0ca45fc5ed5").into()),
                PublicKey(hex!("91184c560d214b44a15c968b3c99aea6d5a22e75d19ae9749afee4f39498747dd9516498ad599aeed9c1ec599b058dd6").into()),
                PublicKey(hex!("b6bbf6082283216a9bd9460ad4dca9e66f2178a2067014f10b36d12b9594065a35bb287830bde72e6f678fbb4c8aa929").into()),
                PublicKey(hex!("b7008d5622a5d1702d7c86bfb2fca03ba46499858e7b0f287ced4b0fb66f7e7cea316f2c711f08535d9cd76085fb0b8f").into()),
                PublicKey(hex!("873ae2f1ac52875ee6af29c40dc5cc6c6910c35a2ae305c04f5d97d919735c24f75876c64b5350a0ca528dcc90e720ed").into()),
                PublicKey(hex!("999ccc99fe518b35509337ed86803957f6297fb2871c702cde49c0c23531ddf0b4ed4dac28014ae15d223b41d0f8aba7").into()),
                PublicKey(hex!("829a8a475161bd1841eddc2b60abd538fbbd1f9fb0f855f8af032b99375cce9ac016d7c5ba44348d0e454515da18bb2e").into()),
                PublicKey(hex!("92e32a9d3fc3c89a66a9bcfa1f90c708f2144b5c7a70751334e7870c5f23e246615d59b8538e3de32f0836bc473d4dae").into()),
                PublicKey(hex!("8d7e279f7caa137948527d72427f1b798f5831f864b12b482a1b32616723112ab72e4ce7a0c210f2574afb0de25643e3").into()),
                PublicKey(hex!("8aba93af9225126cb74c31bb04fc72ea4f5503e30deee5d168f67399dcb46c657a3123407a38216ccee8154b554bdf47").into()),
                PublicKey(hex!("955c948dc3c85a2865cfbffadfd2f921170969631a186bb9579f22c43005c2a17b5874f792e20498978f0c3811b2ddfe").into()),
                PublicKey(hex!("80bf37e75b9d12ed7573c9ff5d69b8ff8dedf57c5409fa9c2386458f925f362916c90d507a82be5844cc8e935f2b5490").into()),
                PublicKey(hex!("8acc12c95f3bf08c7d8c5fc5338e198e52a09725e79e9619ee47fee7804e9c1ea645f12b56004ff642757ef10c66ac6b").into()),
                PublicKey(hex!("a89c2a0ea32094484930f2dc5b0f98acdabbf709d340d35297ba4b67e05a6a8486f3dd3731f4500a50d4eb2ee9b548ad").into()),
                PublicKey(hex!("96c90eac900c15d5e6d8a91dbf3aff8e98af39c5a9f3b0d08bbec41ad02f95b5855682aa537bf654f6e1fba40b32080e").into()),
                PublicKey(hex!("8b44b568a1fdba8841999158ea349dc747cccc59601fbf8e7233ce833b9a9468af6cb10ab1cf039a84f59b732844f07d").into()),
                PublicKey(hex!("a77cd1c02c49753567530f2217d6c451fe0e5704c40ec1783f273777aff0ca05d1f603b72f242bc50b893d90c82b3d20").into()),
                PublicKey(hex!("850da661f3aabac974d76fcad683456a85ae387b2c9b3b7e9ab057608d66dc6c71073cfaa59b3703c726c1f3b938d8a5").into()),
                PublicKey(hex!("b151077d6fcf1e305ca9e98d2d6144dde95074a7a05198ef48c2fed005761e86bc2c28ea18924b52f684d8a0d0cd9ce6").into()),
                PublicKey(hex!("affd96ef10e0f37577876beab7ade978663ad87b7a2e8a8ce771d960f373c62c1a69792760e7173a94f8ea1c7d8e0c3e").into()),
                PublicKey(hex!("83b16acbfb47e50769167d52a19895d1e64fe7a98135f2b9121fc36ada19f504ba8518e50b74eb5e24cd248759cb6d23").into()),
                PublicKey(hex!("b3d096f3ff4552799977ed00e248825ac60ddc76d109c1331d0d8e9feb3c95357b0155f71a962157b37eeab74b56784a").into()),
                PublicKey(hex!("b90b4083c56bdd6d355b5ff689bd26ce5431ec473c293483f8d98c987904eba65d4a3801a2a17a896937b2a2777839df").into()),
                PublicKey(hex!("a94686067598d8a2cc536e169fb8a8b3d3fbe346e680e0b8536f6412e30f67a7fa50867b2fb5566c8015142767462eaf").into()),
                PublicKey(hex!("b28e7db41c71ddd7de04fca3d4717f09abb59ed08afa1bc8b3724497dd0a655ed3b23939952b9baf48bdde3ade0c18c5").into()),
                PublicKey(hex!("b5b2ab8c237ba8f61a23d906a979da9ba20fcc5a01c525aedcdd90e28d2de49b60f807f6380040e8f739f8cca55eb8bf").into()),
                PublicKey(hex!("b1ea6adfc3bf086a6eb3f95a363cb1143920538296f15ae20934656920071660473dc420fc5200922a446a6340f6f7e8").into()),
                PublicKey(hex!("83d79b7fc9a07cea0a6222ad569acd99d40212fd8fcf931f0135f04cf28fa96292d0c2464f93996c5babfef06fa885fe").into()),
                PublicKey(hex!("b8cc21695f2b432fdea0c3d19a5da0b7b69ed6295c4800331eb27cfb873736ae68466872fe3a5a331c2417d197027e26").into()),
                PublicKey(hex!("8767a27ae578bebb3aca55f16fa176c94463d302e6d186c6582d6b91960d5ea6e014b95cbad7950e1196806d6fc15636").into()),
                PublicKey(hex!("abb8dc9507e45381cd8d0ee8a65c0f4446836fbef50074d03e42262ff7368974a7f9f50f6d63bb5be970d868a19dc4ea").into()),
                PublicKey(hex!("9584d6ba4144e292c76757fd0e7418f4c3348ef42e634ce8b111c4ea633da715e386f9fc3bd8bbf8c9624c890730ed4a").into()),
                PublicKey(hex!("b8d19b9c63c933c940b29223890848db2838577026b5eca7ba07d865d1884b9df4ea39671076fa0043e979ae38dbb87f").into()),
                PublicKey(hex!("ad04bc5ea7b829ac688455e127af45d809e152f943fdf950b571d4c3a178e5ed6d4bea6c1ab7fc60b13c689a3d2b5663").into()),
                PublicKey(hex!("893368e4fa97cbf70385412c7e04c5b56e680945dd209fdeccf36d7b21b11d468d251010c833df3c677c64ce1d2f3039").into()),
                PublicKey(hex!("94d5ee5054c63161371355592c61f2034136f7535e5297356cc0e200ee9ea4f43196a3f9999622d972b2d05028f6f5b3").into()),
                PublicKey(hex!("b7ae7947df3061bfdf4bba3ad536207cc3cb94cbbbf19872c0af370abb127f99c15fa54b0db5ecd866c312b1b3ec8b53").into()),
                PublicKey(hex!("ac355e8e65197f656b0e7c4ae6a9ae8160a046f7336bf40af04ec234088d9757bc676cb5c0789ec3944d0369202b9d77").into()),
                PublicKey(hex!("99a05a0fef17f6922c6dbd77f5d553d2cec30f6dcb8bd8b9bf70194f9d03ee48063bf03a6e91cd5d72b2f78c31587b72").into()),
                PublicKey(hex!("b2cf71270292ce714e853821cb914844cfb4d61103dd03135c9c539bb9386cbe832cd096d7bb5563b3f2178af7eb2f1e").into()),
                PublicKey(hex!("870cf13d88c2d9721e343e1d087648857668bb0db3f039c3cdc20d799eccf26b8a0ed368209830fca4631fb6059feb0f").into()),
                PublicKey(hex!("a16a5677371aadbb70904ab0de04f7f766bc92f4986dec691e2fe38cc780a4a8e66db91f17c4dcf7a51134f0f65b2934").into()),
                PublicKey(hex!("910c40f9873aa79c07839c269c687537e18d2d6b610386255264e08e001fea51cfd81ebc3d5162ee478889eef08bb587").into()),
                PublicKey(hex!("a9fd4cfd465068036546d90c0e251e03a45e17efa3029981929ffcc1b3033752a7107390f9a2dc280ba2d66c6fc1c256").into()),
                PublicKey(hex!("8888848bd3f51107d84c83a6052c2a2ac2fb055f33b6c3776b2be323295979c4166fbd28a7b5a6e08144577a246e0108").into()),
                PublicKey(hex!("80b4f4e3f9d3a3b8c4de3d380aaf8411e1c6a35bd8bbeeb186c71b2a265eadf6733042c350b4a4712e1ff40850741091").into()),
                PublicKey(hex!("a7bb02de98ea09c9bae1e3c3e493fd86356ad14bcf8166a41b8ec6c1cbc7e159889ba0b775a43ff6e29e10e0fb3618c6").into()),
                PublicKey(hex!("a7e231fa45e6967e751847d53602fcf92be12b689dc1358b2697b4415752546d108092099e8db0969ae43bcc8d8844ea").into()),
                PublicKey(hex!("87cb53fcbce10a76bad2d1dd757da56814b963fe5a9cd6a29eb3769cc98dbd5f565e7279e7760f32fadebd5a1659abb7").into()),
                PublicKey(hex!("a71c4ac6b501a4eab2c75e5b577628bb1d0d66f652bc6735034a083ac8e0ef78180eb2ce032d93dda9327dd410131c89").into()),
                PublicKey(hex!("82a3b29f7beb64494137caaabe358bad98e9d58b1c96ca22a8692ba854e3495516632cdf003ede07026be7c354fd2cac").into()),
                PublicKey(hex!("b9892e3a43d152c56796a6a6f6a9dfb5e53c87e4dd817d2b94bf4bfbcab69c0eb13a6b7be5bb51b072f022c563d61502").into()),
                PublicKey(hex!("a160b196c9259f3ca50ca2eb869a90967d61f552f3cce4e6e0aeeed5bdcb2111e3855a362c7c686b17c03409cd8c8366").into()),
                PublicKey(hex!("8b6d121a149ad47eb2cfcaaeffa3ceef0e311a4c3d59cb851b3131ad53903472bf1a1e9a374c2e9dcafb9361925f0b61").into()),
                PublicKey(hex!("b56d5634a85d61f1eaee00addad696bbdc4316013e1d4564fd6777a496459e7b188f44a9e29b7028f2dec28c574aadf1").into()),
                PublicKey(hex!("8c7f8109ad291cade1f475ca6edf636566b36ba86d9bf6e805a00378108c0e1069c82cc05fa92bf69fbda4cf2837da40").into()),
                PublicKey(hex!("92b7f26ec11678d1ed525afb46cc75e5430861cb6a52a9e2b101f4b3173d07b53052df69f2b2d18e9a05f47698898e1a").into()),
                PublicKey(hex!("a34328ce6f4d8079cd3bb3185dc9400c39af1daee699abfc5ebeb1cd02fc43659a310e149f76488b666a8580ac128341").into()),
                PublicKey(hex!("90829483256d6ff934cbb6e8accda4f771080ab92e35f20ace8b8cd5a1f1efcac11ef04027fe89b76a908724a3c2cc38").into()),
                PublicKey(hex!("a0b5d2e727a94f59c9eef743d27e0822d11117b7e8a7b75c7755517c514bf7a1875e355d7416c67f73cd6e73f2deafb0").into()),
                PublicKey(hex!("9228ea35c6488368e5d7a47f813eb28e6575f598c757e375e90b4dfefb0eb3c9685674521dec8639c2635c2d17ddb1cb").into()),
                PublicKey(hex!("a01fe29585073a9c22cccc86892996d97860fddc653f1e377e6ea7ad732589095a123e16166b649958f827e08bed3f12").into()),
                PublicKey(hex!("915ff2d9bb04b454dae0c2ff8c49a1a0e0afdf718204b052bcb8a5db487b3dc7448f78df3395fdb4a67cc14d8cadf3ee").into()),
                PublicKey(hex!("b0597958b75e64fe5c6e56ef803284b3b7420fd537e5625c75af3aef814a87a5ff01951261c2c7b27e374466658711d8").into()),
                PublicKey(hex!("8fd59286fded89077050c5e826188da705ab7106637e15280ad7d253652bb527a4f775d464e9c3941a4463ec3054b505").into()),
                PublicKey(hex!("a472e376e381bda3df20d8479464cbf9670641a1f5fea54990917cdd1e3567d24bc5abd4839bc582f87d1b1a0b4b1456").into()),
                PublicKey(hex!("a7f2983314a2364cabcc27d1ea78952e178163ea37d3b24b0eec0d7d3aa8d71cb003c9a2cfaba3b911ef7a1cd24f4dd2").into()),
                PublicKey(hex!("a2471ea25638772362139f3b54474cd4d2413d3b4b3c6eaaefc9af09b24870595f309e2761437084a48f06b8e3dbe021").into()),
                PublicKey(hex!("a790f7c7251fd25e5363c0537a7cd585e9b14e1a210053739f2d6d81f2cd3b3fa86ed7c990aa48aee5eadc66775f6950").into()),
                PublicKey(hex!("a983b0a549a6aab16c16d04fb56eb3f77fc339b0fa92d1ac8ff1b2981b30cbfe6d8cdc0d8d847d1b2f850ac7bc877d68").into()),
                PublicKey(hex!("a0b26ee952c8bc29291562146cdfaadd1b6650836d25f536bae21e057b12c2c0c0fb650d423808cbe8c4685e6f0e3eaa").into()),
                PublicKey(hex!("a2d50b22452b948857818ef769cea5c25fc41aae379e66bffe5ef52259afbce4e6e6bccbd88acbb4ccde1821807f2be6").into()),
                PublicKey(hex!("ac65a3fb2bee877ac6c94d47864e37d7f32ce3a4a9b16be440f54e861e9e1366a503393e4d49f135b1b4df509275b214").into()),
                PublicKey(hex!("b0534fb964933ad6fafe3cc1e2f9179920d225a91873f9cc5a26ebee2fda6aeb7a1c7a22b880c7a77e1b78483f10c66d").into()),
                PublicKey(hex!("b97724ddb8dcec0342e43f0a7fbca6e4fbfdc758acffff4f91c8f499026c478f2f643bb74c6e3c9f425de250633ae072").into()),
                PublicKey(hex!("838fa61142f29cd9069e16c7ed6f1d04156ffb52d5c9fa6a0dd71699793a1bd5ec43c7482614a788b9934e297c356fc1").into()),
                PublicKey(hex!("b313a08ea0fcbbb72246bce52577a529e538f909fca9228b35906a1a1b605b721a69e2f71f0ce0a41e7374b72208555e").into()),
                PublicKey(hex!("a8ccf859132c003d69efa8b14184015541e876f74b600d76bd53f516fab4eae59ddf98b2c855fffa00172d34a92d699a").into()),
                PublicKey(hex!("8133babebd96644301942a1fd36f11ae98271a9c0473c46c686da81c94c78ff11a1fe0f9292b575f61570632ed6f3879").into()),
                PublicKey(hex!("b9a01a41d2c020ec2eaebe5d4ab18ceba89398760b39fa9348386682f136e9b15f55b7fd07aaad6dd59d147a33a84ef5").into()),
                PublicKey(hex!("8381558d0a45ffae95ea63a0ceeb4465e1081bcdc30f38122dd44a2aafec1caa54392d5645b5bf25035141eec37d4ee8").into()),
                PublicKey(hex!("ae3f80448e73b7d6a8afdbf040116667a8f6e5865b741773308e0f1dbcb5d4b2120621b2026d895c078176113a87164d").into()),
                PublicKey(hex!("a238d72e94f6921fb08979fa4b5b56101bee26ce7e2c7bdb03c05fe6923dddca1111d5b79e309436dbb55a00d05e6c92").into()),
                PublicKey(hex!("93e501c4e37e87f4085481c1fa54742a9b2e74565371d3b1d594dc3f20abfa527a9f65e7293162f1e3fcc73e9dd4e9c8").into()),
                PublicKey(hex!("a23dc78b106e77df216ca46aaad70eae1281cd467f936a92bcf84b09cc161bfefb05ea29374c276c8f9c9d1ccd200280").into()),
                PublicKey(hex!("8ccb3cccfb8a05239de9180f305c48ffd61556c571f5de05aa4438a89b802aa5696d1537fb35c80f9849e7b018acb4be").into()),
                PublicKey(hex!("8878d7794d651a41aa14b4720cb4bcd48cdc02156b038c87710bd59a87ab1a33562e5538e13ceaf89cba2e873e6a7ad8").into()),
                PublicKey(hex!("8420c7872e14fe1c248366624433d0d682b6d8e8eac8ccf2ff2eb0df9f59f8bff0dd97e10801a09c1171ba7dc542d266").into()),
                PublicKey(hex!("84da601354334b052350c6cec4ab6746436b51280a213cee3a775857eaf1b0eb97c6a5d2a32458aa79a552cc5e3031d3").into()),
                PublicKey(hex!("b5f6539846e4de5d27d73b4c80a2acd61cd5817d85b5211c37bb45693ebf4bf1bb53d5f3ec84cb65d685c97078595239").into()),
                PublicKey(hex!("b4f8a952e2331acbaa99c08fd304c4cd155d51befb1249883c4818b6dc36c1ef3f177c537f8b231248c9cef22c43f788").into()),
                PublicKey(hex!("8dcb7c9dbfb642ff98f8ec14480c90d3d857932cef788d799f01e04704f8e2816b48650aa756d5652c8940d831c10712").into()),
                PublicKey(hex!("84d912fec494400074f15640a7ee8bfbc9d53a1cdf15abf91c803d580fa31c987190365d3e84532c79f06e7311933f84").into()),
                PublicKey(hex!("91f1a7c0b366c18e861dc1f5826a21b9d300d60a69dc14af0bc2d1c281b24c1f1ec5010a29e12e75214196dbf5f6e18b").into()),
                PublicKey(hex!("97681e48a7bdbc014779e1ef4bb9833149b42323167837f72d503b0e376bbaca86717a82c43d07720cdcf103d296014d").into()),
                PublicKey(hex!("b05e495043b14830193cee7f602f189f1b545096eb03df32881b7b56d907c1296335b95a387850eb6677cd5e167dd216").into()),
                PublicKey(hex!("a2e3ddb8b8a54051c11a314d0571c9723b951356066f5109a46b016ce20df5a0caa1dc28ad5a7880f6bcfe4375020e23").into()),
                PublicKey(hex!("86d41359379306eaf237dac768967a65fc6d3efda0181dc508aa119cb544e88553d088e862940067df73ac1ff90bb982").into()),
                PublicKey(hex!("af62a112b952c7e61ed20923632bbaa9640bd182d6342bdf6b42f4a9826fd5d814521dbc2c5f242e36fac2236902dedc").into()),
                PublicKey(hex!("8812cd45233e6384bb545f9925abb499bafe50bb3798162a94b5052d9bd57dd22f56137d0317d27228740e330be5eeb6").into()),
                PublicKey(hex!("a47c090ce35da0f09ba31d989fb3f0c41c5e4da0a2b359cfe0d09ef37877c8e858b65d93fdec5496ac4318b86e100bed").into()),
                PublicKey(hex!("919fedc10dca71c6671e11946311837e2ec2507878ee7acf99d4de2652de1d55cace543ac6cce87544548ebd5ef246ad").into()),
                PublicKey(hex!("812d9962e6d61518057bb62041a42373a55f7ea4805a53a0aab8c82fc79cc457eea68865c51cbf278da3f5b9180e5720").into()),
                PublicKey(hex!("b237bf206ac18b16a49233c70b503920e0d18cfb02bcbc85111d5d4479d0e324225bbcd2903918bdc83ce8dfa531e542").into()),
                PublicKey(hex!("8a0098d1864c2d0da124e6a86ccae80df27861d169c271797469bc1103b2b2c8aa2ecb6ab0f754eb33dc1b6feee9389d").into()),
                PublicKey(hex!("968ec2842fd64cc9a8e245a42a63b083fc9f473d86f1c18c286006c65dc29b11283af8c4ae890b85eddd3b2a23d46553").into()),
                PublicKey(hex!("a971ca74117948ca7b20cd1099e74f6d99913e9a91624cfb338278c517c767e1146704f28baf59dbeec336def30637b7").into()),
                PublicKey(hex!("a5101523f93ed2b0d4313558b5b55fdb491f119232597512cb72ce817d9ee0b1db71f3eed2c9d03e72b4d360eaf7f251").into()),
                PublicKey(hex!("89e08e4674efc19913d5a5ee104a2eee304912a4852594b44485ae466c35003a583783add4f7e1f4c7388b731183556b").into()),
                PublicKey(hex!("b604a4a9ffcf59a7b77b1cf9afdd7255c7ed33b6266c508d07ae55486d819d195f9b3c2d628decfeafa20ee56d466a6b").into()),
                PublicKey(hex!("9608dd764c357488a0a37b04701a4035e82e375d030ad66e7c6c31820fc983a8e4bb7157fdd780d384590ecfd8cc4f10").into()),
                PublicKey(hex!("a0362c461470e07b139d771f1cdc8263b89f49878b8b94108d5d145146b49e89e1984ab7e2cdc262919cfca5c45f1459").into()),
                PublicKey(hex!("b1831c76756c8c9aaf05c61261467e76bc9373a95df1a662caa94ddacad182b563fddcb7baa443bcf0e0f7eee2d6c2d2").into()),
                PublicKey(hex!("a92a7e6a2676e2a76d4fe074cda356c0ec87929752f6115bcaf29efcdcc117972d8362afcfbaad801d97144ea18cc862").into()),
                PublicKey(hex!("abac9de4b757f1b6d7a281ec1442d0e760dbca630fced73c581f6845cf0d4f23d711be740a06b092d37aa6203433f75a").into()),
                PublicKey(hex!("b4fb4d248970489d509d791c3b0d2677d37330ddbff193303fe60361912cf3684f1ee08aad3eea9f9df0f98d405c6fe1").into()),
                PublicKey(hex!("b51f068221020979e0495d847e74b6e0a157bad237fc5ddc065d8d1b796e09a6384fb0c8737c85c7645dadc34d6f4c07").into()),
                PublicKey(hex!("964ced99f4b76f8f956984f0748d8bb5ce6f3bdc87808fa5afbcd7d99e807d5df67e6f4bf9687849aa0ea2f0920c95ea").into()),
                PublicKey(hex!("8d9c8d05acb7feed2beb6ee951545482259ca1e1b667d2426b60b9f044f135fd52b46f89ead179f78353901b8955f971").into()),
                PublicKey(hex!("b8ae7db026b3bb83125a495c10563006c0e5be3ec0b31c72aad197fecd2b5f5d913459fa3b4f4289b865116674e862c1").into()),
                PublicKey(hex!("86af13c9a22f4801ad7fe7181fb40c5f9c0550997d2ecdc297bb1c2948b1cce684f40f56cad3db766a77c08ac549575e").into()),
                PublicKey(hex!("802001dbe194f0d844bab116ce31b77d6b1ea69ff15225125cd392e41a1ff11b185c73ef2e2317470a7b3b1eced26210").into()),
                PublicKey(hex!("9116764779c0746ecfb2f1a1334de85238ecf555e1080b91838326d813b94c56e4e3298a2552b2a20cb8c8b3a4e0a5e6").into()),
                PublicKey(hex!("996ba13d229e9577dc07c7a31b9f5123b590abb9a9c9a58ec2e7df7223f6df70f77bf90adae3f37a78b608e1eaddbd2a").into()),
                PublicKey(hex!("b71144428b562efcf71f61c94f026dd0ddc24d5344fc1a013d916af24686291d8eb86513a2b22b5dfc022b7353c19826").into()),
                PublicKey(hex!("886f243b32f2ea2cba0124212503d916055e091a37919f34c655b781f7b35d9bca64a43103223038dd464913143af492").into()),
                PublicKey(hex!("abdc5be5091953301a81e19d80931a9a02de0b81a6f5612c39f96ba2b8f8ca2bb41e9669185677c52b0f72b766c22747").into()),
                PublicKey(hex!("a92834f565eb130dd5d10cc3b0c6952712536426fcc8985a1990da14e4ce7a5cff5c784d7a468f2c3aca3be9d249a069").into()),
                PublicKey(hex!("aa49bc4fe3f9b9b0a63accc454464ca7fcce72088c3ec7d0592106a6f4c1539c06d501897252c8acc2e2f39beffdd309").into()),
                PublicKey(hex!("a116eba07edd7701e767e20febe1e91b5131a676558efd98ffb3e16251351126c93cfe3996d3c6b0aa92e8db4385095e").into()),
                PublicKey(hex!("925c9356c2340c7f26b3189a5d460ab99e7aee91213b114818083989e2eb4c4f172b5280745e19d193ffe0e4489cd80f").into()),
                PublicKey(hex!("ab2654e823f46c2eb660fea84596c6938ffd6303604330e27325ee604a50602b5581c4ddb056cdabc0ed17a151d9e3e3").into()),
                PublicKey(hex!("aa81f10fa1e15703981917bc5571d99d4cf260ae5eadba6564853a201ddeee439df02623245ae1f91bb20cd1e2dfae0d").into()),
                PublicKey(hex!("aa7395e247c48fa16198f776df01b6eaa58afb079a15c3710a87b58d51c0b517e01e620bd14fbd4c6c1766a9738a7f45").into()),
                PublicKey(hex!("b401f60bdc451d82e03b370b7e527937c93fdeb555a8eb102672529b71f695db9712570b35c9a1959fc037cb0c8112b9").into()),
                PublicKey(hex!("a1428032528cf0059994a15d81339d7d99a01df6e732dc950a6d3c7392a60a4aa0cde11494b344b566b851b72c40e1c7").into()),
                PublicKey(hex!("9229f36fba6343af8a4ee218214932461dfc3c565936cafc00ee9506d9af187dd04177ded81ea2633198a5fd3c1f7db2").into()),
                PublicKey(hex!("b7ddfaf66ddd1c26ca3789d840bdbb7ae31e656245e25407ebaff5b1c8f2bdfb8a0c5cbdad48b1d4cb6fbe8d2f791735").into()),
                PublicKey(hex!("8b69f93109b886413c23b4e9877c94da025c375bed11fcb4aecbaeb1ebd6eae631c97c75d5a0dc7f6ea2112eb48b3c25").into()),
                PublicKey(hex!("a58a0663d6e7c1af992d820bf912f0c89b317c8ba9e8d60debb93e70a4672ce4c3abbd882960c899c391a555a64e81b8").into()),
                PublicKey(hex!("8375902abcb0ba00b31161128f0c052f42867a86ae6f7adfa30293ea8e4beea4a97f7183b593ac956c4289b9c5e510c3").into()),
                PublicKey(hex!("aa89961d3e97eeb0da5a004455629f85cf24ea851ee453f7414df4674207d8a2f10125fc9aed1c4f26ef03f165d0908d").into()),
                PublicKey(hex!("a8c157a18f72045cd3fd582ba4b528377124f7653e2b86ad7f5315822f34523cbde6363562c40c0156a3692d39da6a53").into()),
                PublicKey(hex!("a6fe30d4b31caaf2f7cc1d3ab9919926f96bc161b368d2ab61b15f485cc9d45d01d08c0810998f68f56e64572339a6c0").into()),
                PublicKey(hex!("878b4e4c9635af50791a6503cf126e0e277d59cc128e76c3db6ab6969f661ec5800906c40c339565dbecac3f1b6b8c3f").into()),
                PublicKey(hex!("a7618e50520a23c4427f1a3fdeece259f84df65760add61446ed33d25af342da7bc1b951aaa88f6680a3bc96c6353015").into()),
                PublicKey(hex!("9236327b2b540df60fdf3003dcf20154de1c679832b4b62e9e83e4447635d8e5990be4f5b71c3fc929085a9f03ce4ac6").into()),
                PublicKey(hex!("ab3d7f667a3ffd354914c2f336daf3b5ef24ab01a93b9fa29f2df1317ca19c14eb6d26893882a4ac5ece2a5d09d05073").into()),
                PublicKey(hex!("b1b9b56e2ba24198018a31cb505fde5f92f07126c5aa3b19107f0fd5e7fefa130b390a9d24855bf0008ed952947c9b18").into()),
                PublicKey(hex!("97e1f81f311463c8b0aabf18b46dc4abcc0de2798d4fb0aab80c1e9e9691cedcb6407133b79c9033cdc497a691071357").into()),
                PublicKey(hex!("990d9051957aaac91f3d1492303469177d769d26a25e13043e1c9a5b30648506fe5ca9487f07791d08de20b2bf93fd07").into()),
                PublicKey(hex!("93f9c3b1b46094c1a5895ac2e2a9729dded641d7b8a771b5598f9b52837d39a3c47a23879ac9ea49296c49d0ea55c259").into()),
                PublicKey(hex!("80e4995da18a2198b4d67b4aa1a9d6723a0a6c7d88459dfc3e846c1a2a348b2d7edb37304e218ca6b604dec34c5bbc63").into()),
                PublicKey(hex!("a903b1aa33a048b13c97255b44a566d2f3a2437896f62051c23c6993b1846f41b82d30431be29ea664adf727fb8ce18d").into()),
                PublicKey(hex!("b589fbf90986429964fa47c1c537fc8cc00e972412eb8951618fda7be45f1cc7ace9dd1692d56f00748b364dadf67663").into()),
                PublicKey(hex!("b305588889c1c8e5adb04eb58fd15c8de6a8f81d822943e3167e7df44c6af44a9db3c4c09125c79240626b309eee1929").into()),
                PublicKey(hex!("83f53ab926be9cea3bbba19cb08eb4e39129832d75b1c00dc1c4bb003ac81a9be6e78c6e94ef35fd1561ce2a6efe578d").into()),
                PublicKey(hex!("9913e7b63e8f058e5c0ffa823c2c3502f4950786e41fa95bc0fabd8a1ec4593defc59be5295977a0c12003dc10b2326d").into()),
                PublicKey(hex!("8827ee24438f029a51276115b97c4017f4bfab673d7809641730a3ebeab08e85a5969f10443bff722db1f8c1ab940195").into()),
                PublicKey(hex!("a205ddd4bc61bda5fbf6208bd26516dcad0060e52033945ff18894c1f53faadbcdca280440d4e974fabbde5e5da38b0a").into()),
                PublicKey(hex!("8d0080fa5da6dbbd9306dd37b68a8c04fd1816a73732a3b8386740afec02c50f79391f3c0414a9851429c50f73627148").into()),
                PublicKey(hex!("8b86547de8cab2df2100213a5a7fc1fdad5a2275bb7d47953a8ce4d125a6a7d655a549b600be0fb6a6b0b65ab0605512").into()),
                PublicKey(hex!("b7013b089dffc8610fa7d3b65f6b054668ab1d96ae1db6fe997a20f7b65b95c8d17bfe8c04c272613015f20c3f2587b7").into()),
                PublicKey(hex!("8322e62f9c36fae3f70a28890e426af711278b1a0d28d50924bf1aeffc7292376ebdfd31f2ec9468d2dae9be7895066e").into()),
                PublicKey(hex!("ac6fb47aa232ca54dca4b7d26df3fff4ed6126e46461a69e93cf3d48db2af4ae9ea4c6814d6880d0c5349b1e1229a2f5").into()),
                PublicKey(hex!("b7bc27b2d03d172395d8fce23678357aac199e381ec58cb3fca94d1163d4aa4c90bf056945cf9a10961679afde824b45").into()),
                PublicKey(hex!("a91c342a4dc933f5c5fa1da33584a85409fe7d871af8d5280b1cbc6d2cd666fe579e2acc5ab0922bdd2ce6a5cf111684").into()),
                PublicKey(hex!("8a99040f6a5739496a28bfcbad317047be2cd6e4b9a62156aaad421ef019edda3d09c75723367ebfd413f4540a4a0637").into()),
                PublicKey(hex!("b2061892907028a83dcca07e414ab0009117616d09d771eb268576e2fb3c17f2b550c1a514b4a151c4e8e50b7a509e40").into()),
                PublicKey(hex!("91c3b565e4d031fda445fc878a11993eb44fc878c3d897986665e79921d2c26ddb9236799e77a72485e327fd3d0d5dba").into()),
                PublicKey(hex!("b5a9d8a212d54f001da07b2dd3c89258cfa991d5b839d0fae980c36172e265587c2c48766e050813e46e8740a9475de6").into()),
                PublicKey(hex!("b5d96caccecf2961448cfea9aa59a5f7a33c385837c8566700dd175516a4a598e48e6fba5124f964b47650bd9c58425f").into()),
                PublicKey(hex!("9049ab7279cd64f91fd04388c7470e15982c2cd3e666658c4b9ec67ddd40fd4da88492b4e18fbbbec39f81e724cdce38").into()),
                PublicKey(hex!("a7dc051ae68e576a52007c3d65006d438d7f7799630a8a65882d01893e527f4c4152a64d7286eb09e7bb9b2dde98808f").into()),
                PublicKey(hex!("a2fc567b7687bf15b49e901e6dc1e4a4657aa24a5f7715fd753d04ecd23f46a818bcc32a105eaebc11834f9786725ef4").into()),
                PublicKey(hex!("b1593dce38c785084d1691fbe7f8f85fe2b4e407a647a21c44b65456e2e396c76ecd04393008a4532b9588f241853658").into()),
                PublicKey(hex!("b90bb6cff7a0cc93910a296dbc7e5b67165e743b1a8c02e112202838417c91f4c21243d253b4762dd65cea213ace2f8b").into()),
                PublicKey(hex!("a668653374ab8f1dc50d63559b83e590df9d49573ea55420ce25b771653089db9f7fa80e3c8b4c987eed962cfde6c841").into()),
                PublicKey(hex!("b815afced2255cf59bbee0bc86836cfd88ae1e2c9ecec7a87b41c579dba169d53067c020106abe29666e4ea379d0ae29").into()),
                PublicKey(hex!("890a7afcd9985276e388333fcd770df1e9a87841b30ca2624a3e7b9c5b47da21cf527361918d125bbd59ce7415565f99").into()),
                PublicKey(hex!("96cc12bd343d4d60b2a26aae55d266c4022fc3767d469404e9566362f3d89fcb530091604061e97f605b38f3c1549968").into()),
                PublicKey(hex!("b0c6ea5be7ba5f459c1aebd3562773a2697b5c3f7d2c7735de0ec2f3868f2ea3017d4fb819b93317ed35f9555a4cb59c").into()),
                PublicKey(hex!("a55ce60b84f92fcb1f41d8ebd71e81c0075657d1d5ba563094c68b93a52ec5ec59402c61b5fc4068fc56a73138e57cd2").into()),
                PublicKey(hex!("94f03d0b35b455b3693b183eea7ec8505850b4524668722cbde1401c0a3806c15fd9cc423978590e7112af484525879c").into()),
                PublicKey(hex!("b5e429bf688d715309ffdb8f5bdf7c0ccca0a0016996c0f5e898765b36da6cddf89c504fc05934538172ff77768928c0").into()),
                PublicKey(hex!("b51e8cf960ab6326c9cd9093d97d7deab170aced75ed93a2b2274a7571363703cb53d5777e68f97580980e60bac04838").into()),
                PublicKey(hex!("b6dd2b707b4b2978a62598edaf7fef30fad7859c3ba9ba000b8e587ad8db38796d0507bd566b8885d28f70a66059758e").into()),
                PublicKey(hex!("b431968ae4a014d70f8204654b972c10f172367d51f822e16c1d12b730b92000b289cf89bf4f085274afe51c76477893").into()),
                PublicKey(hex!("b8da58f3e8e04beb5006042b82ede48a56fd177c11ec0e088de92f6b25f77bcbae7f2e7283e6982c679a278fb17ef328").into()),
                PublicKey(hex!("844e5fa0e7ab1e46b67cc7d8eb86f90f0e01b0a475a8bac1e827355fd18dc8905c2005d3fc7db6b44a785fe1140b99b6").into()),
                PublicKey(hex!("80377ba7d302051aa72eae79031d85607ff666dc50ca52aa6adaafdcf71e2f1efd7d53b0a024ed38618a872c7dc48194").into()),
                PublicKey(hex!("a34e9a28fc2de5bc90e6fca35672da719c2eb8e74b031d57eced932dae04620ba240b67233d53451081fb4c01248a0f0").into()),
                PublicKey(hex!("99d0e1e7eb878e3eac30aa899ff0bb5747d2e11d41e139256ea61b21d960c7fba82b0b942d9926209a405f71d92d75e6").into()),
                PublicKey(hex!("8b7c19d83631c9dfa9e7bce8d1bd21b000a6cd6e7d3668422ff7428591531776d6c95c908832731cb0d7f1785700d482").into()),
                PublicKey(hex!("a4db6436e5628ee24bcf1ddb8ee52b8e68bbc0c757e6e28443aeb02da1162aca42ba5672b389d9cc563fdd4dbd273c04").into()),
                PublicKey(hex!("b8bb956294647ec60ad4206f7cce1f2593f047d5ac47578dea5c323da47675a3387f18a3b320c79f2995cf7c7ad0744e").into()),
                PublicKey(hex!("88593dfa25f543984791750556cf4b76b55b860939a6a4121ad647dab27352bd41b079d6abca06d0ecd7fdbc42e1fd6c").into()),
                PublicKey(hex!("b4b0e9de56bf7d0d10a3b83b5607da9d713ff1bcce9b40f165ccf2765aa124038cfbad0c2f54d9838835ae80f2ab82e2").into()),
                PublicKey(hex!("91a6b7b609b586d49bdeb45447b3723f99e104332fc976d0c2c35160c57bcd50bf049f3d2f3b8f240278a444595e70c9").into()),
                PublicKey(hex!("ad8c90567482f3e7d6100905e08a3bc6ad3e35b3d8720e03c34baa0a7081b75cc0b6d675b08e3bcd02ebf92d2aa4f7c2").into()),
                PublicKey(hex!("87d03a608e09ba28ad0352258b2d5b31a1a967c4819e1a79f8433df534169e36d9a6c624b4ea1acc6e09e5bae76b0c08").into()),
                PublicKey(hex!("8623afa0a77dfa58068b32e99ccb24fd5dd73f14a668123afce368e7d145b5da0805d8d6f070345019f83e8aa5a8cec0").into()),
                PublicKey(hex!("b45673441dd82d28d89f526536e9cea0fc9485a9fce3fe5f404cec2fea3c27e360c06a2ccc695904ef3e0f6e088a54b2").into()),
                PublicKey(hex!("af8121ce9489e8229d507592a47dee0cc211f5e77ff6341ea92be67996b18e5fe73f725401d097f8bb5beb004163f821").into()),
                PublicKey(hex!("96cfe7a263d4361ce6ef7ef55f92c2489ef25ad6d0b6d440fae6d62ca46b2cdeb62190f14425db36729879c7e5ed7332").into()),
                PublicKey(hex!("b5f20644fc9f3a39ff1b22d3fd4cb132e1c0f2c70672e94cf0b0265f38177f78bbef029672e596b2d324febfe497c92f").into()),
                PublicKey(hex!("a740d15f382021e21a8416fe8a0016efb3c8340ad73c821b6848a86a1716da9160c97684c5527a00dafed0b38661f20f").into()),
                PublicKey(hex!("93961ca4ec6fdc2c8fe903ec8f03d6e87b0bc748e021e16baca59bb4094c1e294ca059b2a83773e9bc00ff939802c3d4").into()),
                PublicKey(hex!("a7b98026fc05408e6c0e3f87a750a928f97b570351ce380e2b30039c9c970689ab277ccd2884510f8724f97937f5c8d9").into()),
                PublicKey(hex!("9141b945647b69f6a385e62986ca7e7c082d613d13752b1cf87ee1d9577073edec4c1a3e70b1997b0265a12b042d4d12").into()),
                PublicKey(hex!("93f60708d9eddfd9aa03231d7732c5b3107cb283d6e84f3aceb8889f7bfd549023b7e01c618f3c8470bf4d5b6e0a3993").into()),
                PublicKey(hex!("a858eff118ab39387a55dc6a4ae0eb4d2c52a4d66335253dd29da3de8a328588507af5d8c47fdc0ab0a780cbdfdc1e09").into()),
                PublicKey(hex!("93ec328b13ab3246f1488df0b417ef6c58b58767f53f290b9dcbb21bb6b916787e24f3eca2a40c2641a5861cfc669d98").into()),
                PublicKey(hex!("a453a88fffb4e234fe32f9116ea907d44f49594bf94b1ac684eb72f90e9626235e3ee13edb102207b9cfb61b1b38de24").into()),
                PublicKey(hex!("930ce5ac41ba9ec3b4bf6043eb0f1dddfcb746e152d3f82d73f967d85359458096e714f02cdb9555ea086d9174a7e545").into()),
                PublicKey(hex!("b219ce9b0ac3a4f4e13db25728e1a95bb9979f3d93d825177aa67d169534d044cd885bf9ac363a9aa0b16c65a859699e").into()),
                PublicKey(hex!("8f3a4c32fc1f44b877ff3560fabbe9f2308f83d9ef11be987dd9e1a534e9a358af87fb6a05a34e339eab904e4ae9c1f0").into()),
                PublicKey(hex!("86253cd3b620c408ab0420dc97f589e131af4d304adb28695fd4e464078e44d03cf3ee575d339d9f8f63d67489bd2a43").into()),
                PublicKey(hex!("a1f925fbfe34e61a103237567bd36bbc39bef4190bae1cdbaff128d5cf483de5a9b799699b5fa14ae60315599e12908e").into()),
                PublicKey(hex!("93b255f1632c0a5dbdb93dd6523c14e4e40cafb2ec11380711fb33f0dbd078759e47aa1ab78c1773df491fd781dc2817").into()),
                PublicKey(hex!("868100d9752c19daa256123c1def5136608ab9761a0f7c4ccfb0a5df6add41fe72f65e2cb75aeccf517bcfc1061293e1").into()),
                PublicKey(hex!("ac5e3365c71371129a0698ea2eaf5cafb50a168b5e6bacc21720a25113f55b9b1440ba9eded5f3009ac03effe8a8613c").into()),
                PublicKey(hex!("a3cab345c4ad1b37b52bd89d5fa5a25256668805261882de654905005660777c718c69299bf68cbdfaf6803cf751e52a").into()),
                PublicKey(hex!("80d5b1cbe5449485227b0cba2be9011d0e6f3ca8f5f4a95889e858eb5aa5bc82af28415ac8f2a1580615f8586b372e30").into()),
                PublicKey(hex!("a4bd33ec8b9180e772ac71286640413bff36530a62e871178ba1b370bbebe55a330c18edda9f80a7bd9c727365f9c383").into()),
                PublicKey(hex!("a87ad27aed259b28356f39c7216dd1be913ece2cb0e3757ad4d718da51c3d9a9bcc80e61ebd7efced5b90c2194808c64").into()),
                PublicKey(hex!("853d0f27481547927f6125c67b4bb7136a703c599f20255b51bbd6b5a87b62dac87dc891dc2484204c1fca8b01bf71ad").into()),
                PublicKey(hex!("994db09ae88566c20018c133304de60576a0819d408e79cd756ec578556c443044b806586235c770ffa6df0a43b0584a").into()),
                PublicKey(hex!("82e811aedd5ac016dc4623dcb1ecb5040fd1d7ec05d24dde32ef135b58bb7fbc2d96fd005bc021e840e63404ec549ede").into()),
                PublicKey(hex!("a3c10884c5be715ea4d18bc1082680cf7b2eb2cc570e11d1388ca7cdf415e430cf35f1bc942c5adce21b9c52277e5f3f").into()),
                PublicKey(hex!("a00afc15a0a4bb80bcfacfab5ff42d0142945f3fb075bfa3d68d40129a145f7321ccacbd4d99974f2a0723e0fce82c78").into()),
                PublicKey(hex!("8e3bd02bc77a29de907a70d3ed73d58eac78331cb8664d732f4f67fd5fb490105d139263d4145b5f653c40d06b34cd40").into()),
                PublicKey(hex!("ac4e2b6162d4e6238d6c1a24d156bfb6a6da582786af5ac8391b3fb425ae02b569b3a4d6dec32d84eceef677b37f7580").into()),
                PublicKey(hex!("abf44ed457d12153f8943bc373dbdff354d464b7596e4292e2874a0e4ab415d0fd46133bf20944730267e567e7c1da91").into()),
                PublicKey(hex!("aee9fb51c770d3a221d231f6baba41b2ca1c1c1f7cf0bab8fc11e641207a56fabe85cb14692f528fcfe5eadb8075e4dd").into()),
                PublicKey(hex!("89a88cbb080550c8b4fc484975a5a9f1c22b52b7772eab1e3b4d62c9b0a614fbed1577bde0728700163f7452f3fd6171").into()),
                PublicKey(hex!("ac8efac6d83f793f2d5971af4d5b37213cbeab410c2e43f5ec78d81e3c53886c2bca13e7e6bf6a18dc4a58fcec957eb8").into()),
                PublicKey(hex!("a17190e47a29877144661dcbe63be4eb34f0ce91f4a3d50f1a182b861dafcc510a6c0476d0586d015ee5130f94ab1e34").into()),
                PublicKey(hex!("98b87131dba29e84fd6b96beeae00335d3216489d18fef3c1f63e1f7ccc1617869104ea0684bb453126847f21369d5cb").into()),
                PublicKey(hex!("b6f622c70ef7d2c08e4157c39e87f4d86307873104459fe48814403442f369f794071a609aca78b9685626e78f65108e").into()),
                PublicKey(hex!("84323f6465cf18061ab043df9b68f5e590f1a0756352c2945c656487273278c496261604ded93186aab8040e04bd8322").into()),
                PublicKey(hex!("b57b6a83e0e4ec13158f17be24d16ff5602b918a77535ef0bcde21aa31c8041a205e43897f5623b16d402dba9cd4f3cd").into()),
                PublicKey(hex!("83987eebb870869ef8d67ae19cf39349ea655d2d6b6b37b6ef884593788a986ef28621814315051a0e99c35b198ba439").into()),
                PublicKey(hex!("8e7d3138834f79e0b245fb396a04f40da2f0fafa0c58509d66866e45e9de683bfc87689276ac76542a1aeb7f3ec55651").into()),
                PublicKey(hex!("8b03d72e78befc9d75183a9e98ff7c76882aa3de65216e0dc96123365cce89d2c77a2555c693b9161c39df1412ce00a0").into()),
                PublicKey(hex!("8910e88f979fba40ca692f15749d41a668cdb4b2f8129209d5fc4a2cbef412bb7c0350edf601aeb25e786ef8d3848267").into()),
                PublicKey(hex!("a9351bd27bb2660632b128163a95f3a5b2a34394ae953df826c4a9f5455e0c6a754a22bcf4a019ffbc1516c7452be53d").into()),
                PublicKey(hex!("905889f9b5d913e4df560e48b524f045958146f4b524329e6730dd9bae7e9d2d8d579766f7dbd1e4757727b5081c25a6").into()),
                PublicKey(hex!("a0a8575a23192bcd8d11a7892e78569facce7f0054961ce0de28b9734fb3a6363830672eaeb3e680afc2b211aa810d9e").into()),
                PublicKey(hex!("994530c2f676e1ce775be93489ed67ea3f5524d9673ded1fd998841708ca645d6af43ff51d3f3b3885c3421c88e64dbb").into())
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("8d7fb64510ca766ae8dd78ee226a89b8a777ce71acd5ba5abfa06649e7d097404725d1837f55c8415b74b4198af6d55b").into())
        },
        current_sync_committee_branch: vec![
            hex!("c1bcfd9c44c8b9fec443530f7cf06f281c6b5d2d1ede77a486eea591fe79b0b5").into(),
            hex!("af71ab8e71d20aef2a5cb3283230b064d3f77efed74f361e389ea1fe3a8cc9a0").into(),
            hex!("01aec45072d2445cd953121060456c88d8750497c9f4f4136157ce8af0ba5102").into(),
            hex!("b2af9144c7758e1ceceb35a4f0f86d5afd112c3284820dc823727d6288598ca7").into(),
            hex!("48d6a2e1255c5fb4bf1ba7693a6caabb740ce4a8c1fd5cb3b69d34300b2fd725").into()
        ].try_into().expect("too many branch proof items"),
        validators_root: hex!("043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb").into()   
    }
}

pub fn sync_committee_update<SignatureSize: Get<u32>, ProofSize: Get<u32>, SyncCommitteeSize: Get<u32>>() -> SyncCommitteePeriodUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
    if config::IS_MINIMAL {
        return SyncCommitteePeriodUpdate {
            attested_header: BeaconHeader {
                slot: 32,
                proposer_index: 1,
                parent_root: hex!("3f2ab239c82804669e6b174e77b894b39a12d15ab7d605c61ec53352392111b2").into(),
                state_root: hex!("8a9e9f924d91ed6054b7e9eb9458fedfa5f0cddd8bf817e4138ca0223b4e4ade").into(),
                body_root: hex!("5eb79001e630f0bc4a755f74617e2b3279a71ea19f4d91cfcf824e863f8cb167").into(),
            },
            next_sync_committee: SyncCommittee {
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
            next_sync_committee_branch: vec![
                hex!("92df9cdb8a742500dbf7afd3a7cce35805f818a3acbee8a26b7d6beff7d2c554").into(),
                hex!("058baa5628d6156e55ab99da54244be4a071978528f2eb3b19a4f4d7ab36f870").into(),
                hex!("5f89984c1068b616e99589e161d2bb73b92c68b3422ef309ace434894b4503ae").into(),
                hex!("e77bee7f098b9357db8f1a5b69471f5f09f983061f2814a1b13dfb40ebb25ec2").into(),
                hex!("89b217c6ecc5b7f169f8d25f05fece35d528e4bd4e4c03f2fe57fad66aa5589e").into()
            ].try_into().expect("too many branch proof items"),
            finalized_header: BeaconHeader{
                slot: 16,
                proposer_index: 5,
                parent_root:  hex!("17b99b53e56ff417de2f1a3b42343b01299e1b5a9c12b11843bf9060fc9d762f").into(),
                state_root:  hex!("2ac8df667e2560d1ecc4c5355c9f28fa34ceb048b15dc9b0473f4cc60946c488").into(),
                body_root:  hex!("8cae61b66925fa7f0f52f3d64710fd30110d454839cee6ba5e8616b1cb6f673e").into()
            },
            finality_branch: vec![
                hex!("0200000000000000000000000000000000000000000000000000000000000000").into(),
                hex!("10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7").into(),
                hex!("2a44b3c5730970355d5ea71ed4cc8b032572975961e4d8cc83630bde8b2e7b37").into(),
                hex!("23ef328a65df815911a8e9cb5d41caddb5bd066987bd8fcc0006ca0273647884").into(),
                hex!("f2bf3c1c758c4c2b02639179a8d22bf7c13498ca69e34829dcb89ca88754fc05").into(),
                hex!("92240d7c89a4015823144fd2fed1ed8ce2a3aac9fba829e52f6a6204d45bd449").into()
            ].try_into().expect("too many branch proof items"),
            sync_aggregate: SyncAggregate{
                sync_committee_bits: hex!("ffffffff").to_vec().try_into().expect("too many sync committee bits"),
                sync_committee_signature: hex!("8f01feb0138edcaf0d50bf40f056c02d11454b340d51c2ccf77d55b56f76e1cf6197c899622db22c8a239c3481a188a609d6190f9fcda6c74345b22be2966fe2c5a6f0d647efcd2eaa5154cd2cec5433362f7ccf611855849a8b3e6d1178967d").to_vec().try_into().expect("signature too long"),
            },
            fork_version: hex!("02000001").into(),
            sync_committee_period: 0,
        };
    }
	SyncCommitteePeriodUpdate {
		attested_header: BeaconHeader {
			slot: 3976636,
			proposer_index: 109415,
			parent_root: hex!("0e3ff2db264240bd75f821848bd0a077544c5a4b5c3c231dbd98a52a82d857c8").into(),
			state_root: hex!("76366916e3f9538a53d22f21f2edc9457ec45f3af4974a405614c01d943d1dd9").into(),
			body_root: hex!("d76ab975b30c516c656069da5fc14352b44501d5e161ca265f9f35e111200d3a").into(),
		},
	    next_sync_committee: SyncCommittee {
            pubkeys: vec![
                PublicKey(hex!("995ce8d6f933957a002c5e054d73cfe6b21f753bb20fdf8d5b373972836290dda106b090fe7b91e2dbe6bf1a3fa47b70").into()),
                PublicKey(hex!("ae1a9caac911d19e94007ba35b2a0dcaaef0fc06290e756814e9642de72ab2092f1527c921ec954f963c982c5dfd3b15").into()),
                PublicKey(hex!("8067662d705d781323b26c1444481b60c8d3d2c8d247006587f586b4bc589d457d57a76c98256ee4e85fc3c0f804562d").into()),
                PublicKey(hex!("abcc3d3f8ab1e58e4c8fd01f9d6293a1c886deb522f86a2d7b312b7ddad3acdb4a2fa6bf3754334776dc1ec3619dfcdd").into()),
                PublicKey(hex!("88a6981de3699e799dd43563c4c1beeb2b5d1b465cf7202d9c838eedbc619f401d038fb1b414c8e3e5afcb76d626999b").into()),
                PublicKey(hex!("964d280068ce7301c6097649e0696dbc446b54a1b8d956c5ffd65ce3ea334a70136e980b040755e5c6fad9178e16c0f2").into()),
                PublicKey(hex!("85e86e3510ef0f7d948a713a0baad9a691c9636aa5c3d8857e9666a3381df35e56a9606689ea95adfb3ded8f6815cfc7").into()),
                PublicKey(hex!("a053861e6fe6a2cd12e0f4cc5b88924f3cd327e1a3c1ab7ee7c5aa9237850cb0d68ed321d940ed943bf1cba3d01876c3").into()),
                PublicKey(hex!("a4f54e2bf67aec1fff66200c4522f868415f82932e6677aba8a429cc244bad30517585ac862dfaaaad5b4d907c830510").into()),
                PublicKey(hex!("8d10bc7427465703f1361fdd85b026acc006ca99a0a6a2a45c72dc4e9a9661ee2ed9dd3238f3b59b157975d63cc1fb8a").into()),
                PublicKey(hex!("b99e7d75a25f6b4502a9181e4e4be3ed4268cc98a489e0d7a09bd315b7c846615f04e74cef7c671ffd03be564dd12488").into()),
                PublicKey(hex!("a95c5a38593a212cf55770dd0ef2cae6aaea76d7d0e26346fe3d74f342fdd24219de50de76f144b3a32585ca75e0607e").into()),
                PublicKey(hex!("981a47bad585f4700045a694002cdff749f949651e398fc4c4fd3517a488b719762bf08ab1bc80c5238d4aeae142e601").into()),
                PublicKey(hex!("80694943299449427a2411e0187448e86c4efd8ec9a91ce87daea5fa68603e6af22cc6c8f55c6ca574a94a4bc63c629e").into()),
                PublicKey(hex!("a963290f937abb67cc687bc5ec89ce37e341b9b2278193bd8937f1f7cf8e4c3656826ea7cbd3b4c43bd00ed0c3fdbdd1").into()),
                PublicKey(hex!("b10c39c4eb1dfd31459c40d4ec62ed70b0fb2414d09fd65436d0636cafec9c64710e8a8ee20acf9e58b7d2d8255263e3").into()),
                PublicKey(hex!("a4a4fe6eaf36ea3aaeb81aed0845c9c5e973872b1c409a08aff4f3b9755d6d3d118976413789c1c40d13599a987887ba").into()),
                PublicKey(hex!("8834e71882aaddc32029e402c48c6498264646a0a9a6ed43df17cb29d64e3dd1b35e6d6cbd7085bb84cffb09e61a18d4").into()),
                PublicKey(hex!("a1795b3a427056930617dbaf41778e76d095495d5c18a481e6b469f6b1fbfb5de701c93bd5ba7dc8a2bde9a09a912af1").into()),
                PublicKey(hex!("829ba502386ce46a93612908c5f74e16e14519807d58bbcd4bb5ef3e56c4bf547e5544795e2b3a48f86e6f0b8cb99e65").into()),
                PublicKey(hex!("b11fa6a943d304e80daf9cc1b6c01f26aaf9664c369cf9b1f1c5e8a368ebc98063b37b899f11792977ed315fa560a343").into()),
                PublicKey(hex!("b40ba269492d68d53cf3a76c337afe2ead9a7b396d99a2277f34d6c68098c185c4e51200caf8f20379a73e72cb594a04").into()),
                PublicKey(hex!("8b4afc61c6a4c0227295f0567b8cfdc6d6cfe2ff7d4f88e553d39bdedfb51c901598d0ac7ae4587df968c7b8d4bf1de7").into()),
                PublicKey(hex!("8f350c579c35203dac1801027f1e70d3a4ea0a6b07074038c357665ee5d7879c8874fb7b607a94da50c2bdda3e3a304e").into()),
                PublicKey(hex!("897adce74d322cb6a1ae0a7784806c0c2e4286935ba1b9b7c10e121c1bb1c8bd88276834b5e9307ffef77ee583c9a140").into()),
                PublicKey(hex!("85fee007340a64c67a0387562d097be226550669daf1829eb9396b3971a75bc4427b6fad76bb3f6d52f48a0f4740f992").into()),
                PublicKey(hex!("a8ac3f4630e55dbea6273411b3645fe718d99275fa2077a35f701b76313e5bd9045157df8a22b84193bbe4807c55dd83").into()),
                PublicKey(hex!("91370d0172337781b6e84ffb8b53e0692bd164b0fc8e64e6da1cf9b9490478abe05a191e44716021fe9def6b12dd45a7").into()),
                PublicKey(hex!("892d0edc12308bf782df66556d56fbcdec361a478b45ad9ffc396bb266851f5aa54720c8ff172306d3f0507b5db0f353").into()),
                PublicKey(hex!("86ca8a73f373fea3ebf120b4fb8e5bc3560a5cf5075e41e183e28f335113ef0fb0e782355341302ba7f2d20d4a60aced").into()),
                PublicKey(hex!("8eac4a744da97e1dcecdca614273f52d6c5fb86a99ef239f18ca85fa0b0846505208772c63fc1a70cf4de2e1d05e08c2").into()),
                PublicKey(hex!("b3f4746fb7e18e9ff3253cf3e42439bef2260b3a3e9c208abb35aba54f7b1e1d9a1d81b2709e57cc3bdeaffa1b4e1eb9").into()),
                PublicKey(hex!("a0fa213f5571634049ec4ab39e3bb69c0cb85089539375d857b29f94991039ede24112f5d1256f5a6b3c85d09a6025c8").into()),
                PublicKey(hex!("b7ac30bbe4530ee608d7dc32105460feed2b6b6f0de2601a68f8c9c797d85dd016322ede99b11d6df15822bc0788b8f4").into()),
                PublicKey(hex!("a3ad197e05090cc9a98e4462ca10e5887e9a130a6a2bce917258b49fdfbe4dd1400f4e12a7bc8b89b12606b9e9e469d6").into()),
                PublicKey(hex!("94562a45948689c5a70431c18b9e704376974951b2cb942509d626285377eb5fe9da32212385f012e6a1e0547788237d").into()),
                PublicKey(hex!("989e61faf54e90f06078760959ffb436d6acb71e4a04c44057b725b5ca9de03e77c6feb88f57af6a9d545dc4f723c3f3").into()),
                PublicKey(hex!("ab810bba3a0781affdbf70ce093bb4f0bdc6cbca517e32bcb83daa0cb4614b27da935be8864137c48ed512fbf3981c63").into()),
                PublicKey(hex!("93046deeab22b76ac5a97519b545d056f09ec1d823452da13662f4d5650af58ff7de647912adef20adc5684ad1edd583").into()),
                PublicKey(hex!("af17e208e3a65bcfe6b3d5ae8bec20ba265886bb233ad449edec3a4ed1075b6ee78ec8d6201a9ce88cae58b539129aa3").into()),
                PublicKey(hex!("95db41be7fcff67fcb5c0d298c8ed58bac4432cc3ec3fc2bb0feb4608822bc6b118abb38b9e22c1a108eda2f589c45ea").into()),
                PublicKey(hex!("a99fdeb598d2a05ac3eb4e20bb33609ab244dd0d360ac23f6364ae434d2d6b407a8ec43be131a4f36281dfae48f2a69d").into()),
                PublicKey(hex!("8a58d92fea63bd97a65c6c59cc7e253a91f7ebc7fea553c2ee5ff7e9a3454e69176e3d5093a65a4e6accd3d7bc84acee").into()),
                PublicKey(hex!("a84fae60e9159fcaa053f46fca4589da65d462ddf34f6f1607c6b098df181135ea777854dd6a105f36658e6898943645").into()),
                PublicKey(hex!("a95b95be5be7bfebf47d95633319bd2e841458cf50ea4a2a5ca8a2a1d56f0c815f730655f35a6c4187f568a1463d3cc3").into()),
                PublicKey(hex!("a0c89a5acd55d3b6b51e6baf1509caf9d412f6ecf36a2d53927940ccf96aa2d6f787ad1fbc02bbe10d743d1862371dbb").into()),
                PublicKey(hex!("87fac2652748091dcd19365c4f1dd9d5347fb2925da10e33cf1e204fca39de7f5181c739dd4a38c2395e0b19edbcffd0").into()),
                PublicKey(hex!("b09fd0a307071300384a4eea55785d06bd25d5998b691b06c5e1b9a65fa5b00d94c0db1a37d7203788e1e6b67e3d655b").into()),
                PublicKey(hex!("acb0b203f4a70942615c14f4c7c0fb4749188fc6c6df202fef7cb308ec6e286480738a42f2c917b7f9f6d8da0f70476d").into()),
                PublicKey(hex!("a1fcff4f11fe7cf3b92c1541e35269f3828e2af27019a3961b995c3b4e7a2d1036071c518b0b587672d186cb528d38a8").into()),
                PublicKey(hex!("843957d897cc9aeff83428f1d07c66ba8273ee2a0296ba0fcd6d7d0b8c0b7f85dca9146865528f5ae2895569727491c2").into()),
                PublicKey(hex!("b7c306e6b4ab090e95ce8f1459ab2020ad896410bd643023646ac8dcf9c43a4dbad40a2839eaef1de4d840c2c148ade3").into()),
                PublicKey(hex!("84814e3e0b3175a31f846e260725e28465d5a1a5f50db51ca103e4f7936e00c52320fa72bcfc233f72a92761c351192c").into()),
                PublicKey(hex!("b6aeec7a042ab8758b7cbd50416aa2d2e701ddd798edac7afb7ab167f97238fd62d676e8cb75beada94a5560b480b1c7").into()),
                PublicKey(hex!("99107796009d783c9550408527b216f1d35d273ac7e7dcb8b8026cf1fba64eec52eb812797d703fb5283ba08eaa40fc0").into()),
                PublicKey(hex!("90794eee35531e5ddc0d3860f6e164b2d74910cbfb57d8feeda2410004ca0385252cbf9b8f1e71ebf5c2f011d6402081").into()),
                PublicKey(hex!("b8e3418ee9c420a18e3cd5679b5be333e68b1a4864ae9788b888c32c54487ba7aba3828264a19203805f99aeeab5989b").into()),
                PublicKey(hex!("b93120b376c8cd2b6a2cf2c359122037fc3d050bd3f66d801dfc0574a6aca2ef0616bdf043256944391f729c27bca33e").into()),
                PublicKey(hex!("a55ee772be464f34d9d78005d55a220e3d9a0ae2e9cc33315497dc03ce804c26cc91c7e992427b92407db56d5f983161").into()),
                PublicKey(hex!("82bd84e6a295170c5e5544dbb25fc1044692ae14c28126f10064e12b219ae55fc00997ddde57d2f8abb64b9e9b701a40").into()),
                PublicKey(hex!("a089238499f46cf34caf625841f36902da1cba8ea70fe64685f2d2168ff89f916d59bb61926bada5abbf3304c40dc834").into()),
                PublicKey(hex!("938817b6daf5051ac2f5cadd5e0b4c2edda3f603fb647717248690281fd6c181700952cd403efa6b1e49387c606b7986").into()),
                PublicKey(hex!("a8816f6a19275cb5a82422059cc4335111b51009aa1c95f0dcc85c2d5affeaf03f8f4116a742bc7989dd8ebaf1dc77f1").into()),
                PublicKey(hex!("a82ae3c3b2b838f3d6b435f9333399ae1c51ff41ccab108b8c9a139209b79d6b8f1cecd706edae25389d24584ebde009").into()),
                PublicKey(hex!("acf1fe80fd3430d5d80817b4df762f055a3d67b6e485d6829fd3c56da39863aca7bf2fe24c37348772edf36371f0408f").into()),
                PublicKey(hex!("a2034ed38ea36966abfa51bb38a777ea5c1d2bd55accee36ef2861c738645db96dc77ae926609a3ee0236aa840fc7526").into()),
                PublicKey(hex!("8bcb43b4ca3d308a73fa703e27aa903f8c3a750ab8166cc2065b3383e296bad15083dd1d4eba237439245f4ce9c01dd8").into()),
                PublicKey(hex!("82b2117f700dc0256320712705ad0334a7d04cb66d314e8d60597207d5ffe04eaffd07ebc8eecff7caf4447bb46209a6").into()),
                PublicKey(hex!("90bdacd2581f3d6bc3e94894572b842e6dd07b1464a0f8e8de7a3761a4e0a2fda415c95a7c839be8d3a2c43e0cc2a72f").into()),
                PublicKey(hex!("913e6ac4f24a3a251c5b57fd3e5329b9a9d78c854ffbe9c9489982bb338a14b87019f5c79e2f9918ed6e0bb6255bb51d").into()),
                PublicKey(hex!("b1fc0b3a1da9141dcb85df0de6cd3ed71d81ba0c57188c11758f86a7eb3ed5c99a97d287e0c6c4185c57a83c845b60b2").into()),
                PublicKey(hex!("8016dc421e3acc719ac5cd8be3cef8832c5c1fe3e25daa5f8cf3c0090a2c418f0b5ceac124a3632eb659caa00ce22c45").into()),
                PublicKey(hex!("8d5c76e4ac25f7e0b494cc52ad50b0a472b839a1a3a82b901d90bb728e44a38b075aec2aa6f4c76e615409aa9f460b1c").into()),
                PublicKey(hex!("b32a120990068e534762be5061524786a27c908a0ccbea644e8650646bfe92f5f564b77c2cc4bcb980ae41a38726c51c").into()),
                PublicKey(hex!("a78b1a8c94869f7231907d8b63e4dd3afa34d4602dd6e2da3e09b68ec473dbb0beea2e694cbecfeff4e80bf3001a8ec1").into()),
                PublicKey(hex!("b5accfecb89292b64c429f1814051eb51321b57b57021e5418e646cbbff701f364712c411712f90e58f2e03e536f865c").into()),
                PublicKey(hex!("9372359c67329a7d96166a42185b11d4fa90c2aec4f9640bc1590ab2876b3232016c293f43e87f3eadb6fb3ffa131610").into()),
                PublicKey(hex!("a6c04891df324878a5a7a516bc56c0f3d7c91313bd441b01e16bbb4625f30af002a666ca899d356c2d393ac696db9449").into()),
                PublicKey(hex!("af3eb01517dc8e3a4ff6d10d1e0b061e8572ef231ef455fa12ade246da34b29d24e6b759a5bd1aefe8a603f83ae29c7a").into()),
                PublicKey(hex!("b7a718777b6cf0f99dbc98331bd8ba791682d92616846dc4b728c30ece448c4abed2e4a51a9f420341a29a4300a1ed9d").into()),
                PublicKey(hex!("98726387f70a8ba63154189f0291968c8c46b6d134c8c981b2fc668d3f84de7f27b866880d58cba168dbbfb91466e17e").into()),
                PublicKey(hex!("ac44727979e098f1bb69335a7750473f7f33f1582e187c9770d37348e910fe7fca4bf4ec7eef83d7663f6b3c63dd38cd").into()),
                PublicKey(hex!("ab800e59937e4719782aaa7f70128233560adf8ff16b98c5ba84b8f65adff837d82d13abcc5b5fc93b1ae5ba21e9deb1").into()),
                PublicKey(hex!("a118559a9faa4de92fb3300f847dc84dbbd650d35fe90b00d25c5737291d01872e37f15ec78a0d2568f93702fa9ba63d").into()),
                PublicKey(hex!("a0dba6714338125e8bfd1cd9e60715cb1237a8a3c9f9ca8a4b9ab158694de218b49606a603396dd957d5d53e5e4ad51b").into()),
                PublicKey(hex!("b6f7c0eb7b298d91c0fd79abe7e6e80e96d2e6c2618bbe92b8b5cdcdbfac75242572b30461245ccaf401849d09d9db62").into()),
                PublicKey(hex!("8c84c091f328b9cd8e9596af82929e3eef548781ca4dfbb44b9d929255ad213d5e5897b1b3fff56a76496788b77f6344").into()),
                PublicKey(hex!("84f984ee09f98824b194c8635598a02167f7aff54a267731557c3d90907db37eb9355721fb6193813eb2794027b6680c").into()),
                PublicKey(hex!("a361e0cf46795cf832b9cab1843aecd28b2a8de630581c21d84c517d4e52b81d57ecbbd00d7180724333ca8d692f5472").into()),
                PublicKey(hex!("8b91745a95b36883afb5fa62920ab4ad8c03ff4f8c442dc0061ba6a388cdd668d08687c11ecd45784e422db79710e41e").into()),
                PublicKey(hex!("8a62c760cb88c05fba1e31a055b9db8ce74998d6982d0133753df7eeff189b9678f6d0aed7556d8de741a8cebd6574f6").into()),
                PublicKey(hex!("a2557ab41c78517fef0308ff0b11302f52c2941e8f32bae133fdf581399a659ebd2b84cb3c8f215fc86f3273ca0fe1b1").into()),
                PublicKey(hex!("ab6118f604c2d9228d7cf3b1f7d8a2587ee579b9ea634de10a96e3da5ec78a1fc7f92a2d57b369171491e0c0a7d9ef33").into()),
                PublicKey(hex!("91b70ec3492a926c23612b3ecdabdbc0e6d86022cb89146c943f085ca55771c954dc35a7931ac26feae4864124f0a234").into()),
                PublicKey(hex!("b2d147b3f0cb612be796fff988716ad313a23ee3b3c83dfcb795e2286712f81bf02cb4636b3ac9939320601f716e2666").into()),
                PublicKey(hex!("b7d37a5ecc086b026c4f045445022c88e3075b1553d906a6e2e0e1435b2c2e65f30c734da216ddbd0130923b2b36c1f1").into()),
                PublicKey(hex!("90e5e75c8f635746bc6a840331a501f96497b3f2194b181a2a1d93d36e881b195c5108cf2bb3c7b268afb034a9896beb").into()),
                PublicKey(hex!("99c378b85e84e7e98cc6273c2e7257baac2215c4bf0d86e22f93885fce91a8b6ab5c463918411fc81c721778856fb809").into()),
                PublicKey(hex!("ae9b232bd4789d7c4ec03db8ca76c4a6e6bfac48303dac8a56ea5dd7b97ce02c8d883e50596482ae129185afb7b0439e").into()),
                PublicKey(hex!("a981394b9cabae0ab800c08ac5c02e5ef375da9f53de23451be2a22f1d337ce3a0974db86638f9c4809883542e880f25").into()),
                PublicKey(hex!("81587e8914d85ccfa997bd67ab2db97bd2c529d3494aed23af8c107413719c4c6d01d004787824957ecfa50057d0f23d").into()),
                PublicKey(hex!("b9cebc1e13b222ce8c86efd6c1486bca852a72d0b2582720bb5e54a09e359940664737328747b785ffb571a930728f7b").into()),
                PublicKey(hex!("ab0ef49b8ed1843c64a77b0af3bcbab4686416ce4e26a59a928c4c3e1de7f0d02a6c67cb14f53152de1b5a87faafd933").into()),
                PublicKey(hex!("b9c7bc7341def84925c31f5c77a848492af57d706971078c1948a7e10bbdd1d6d2875c53522d91679f6ecd8dbda30e80").into()),
                PublicKey(hex!("824be5aaf5db94920ac2883ce5ddf55392fd894bf8bc27040e74e110945418701297c86296de4b7c666b41975c3155af").into()),
                PublicKey(hex!("b9c099d11baf186fd6e2aeff692105ed93477d772cc02b83e277561b3072b29f8b36eb8c03f378527388d3d2fc513959").into()),
                PublicKey(hex!("aa4790a0835335aac6a42715e6749d85963c64e7f00572dfaea93da7809ce801e1a8280908aa24903f5c4cbe5f84d204").into()),
                PublicKey(hex!("93b8dadcdb02023f7605f86dbd16c727ef993dc9846c7bc5d7b24f381e04bbce3434b936046df88974352cfc9ed89bf3").into()),
                PublicKey(hex!("a7d2780fcc0a32f8c659b21fb1bc403f05d234a2277411e19caecf9eb8d21e046af39b0364b5f6491f8bcd61ff2ea469").into()),
                PublicKey(hex!("98b1377469898c3f5cb58ab878eb1b96c0d484bc7f86917015e0b4fe88299d56d8eb2cafad2cd8dca068b82aee263842").into()),
                PublicKey(hex!("8c5bc104464f62205bfaaec438bf835cf7420cd3824da187eee1a40c0ccb32a7dea18523180394c96da515ea14dfc620").into()),
                PublicKey(hex!("92cc1b768e5239803d8f411f0293c2b1bc7fa68bf82f79dda8e75a2a2525f5b7f805cd4c3cabeeb192965d1388469a05").into()),
                PublicKey(hex!("91a557d536dc690557a3cfbd27a21edb0ea13216b3dea203dd7feace06ef2bebbc94a06807d63fd91bdd4068a8d0bf6d").into()),
                PublicKey(hex!("a196c1fba1a58c193cd80dca76bad9c2ac038fc1dce7ef69e90421ab521cfe2d5bccc77d2972767e20bb4928e32ba39e").into()),
                PublicKey(hex!("b2a4074a3c8d7207aaa23002a6036376b7ff362cb8701048808bdd3a15fd43db868ccb0345ab33b8b4d96fcf0497fcd4").into()),
                PublicKey(hex!("b54dc15d6fcbfe9dccb5977e2500b79503ac9d20f2d769b83aa50aaff73733789f6a50e00bf7b63c23e5a358bc10781d").into()),
                PublicKey(hex!("b7f514bb3c29d98b92171049590410d20d8bd8d108f6c6c4bbf5220e099799013348d0e2a09a8d6300ab6bcfdde1c710").into()),
                PublicKey(hex!("8d6d754d4ada57a712a0832c7ca4189b7bf89cbf8bd687765f1dff915a1b00b21ab6e5a51dca344aec4efd491d45c425").into()),
                PublicKey(hex!("8908fe2b3e0d5a90e673cd2daecb8ae460f047d535400874b416ff153710ee48ca0dad6391c27ef9a5169d6c8b0e76b6").into()),
                PublicKey(hex!("97b37dfcd3c6fe3c84f12806ad368b173b2170f8b451db49138618224bff8891214e5e7de56e5cb415210555d5ec054e").into()),
                PublicKey(hex!("98763aee7af76d0f9c693020a205453387fca8a29e4aaf369a070e15f591a7765c81726dae7bd00c6f8e5e64e44f3d49").into()),
                PublicKey(hex!("9706d59d86031257efc2c2e3312312e6b4386bd4137e1cde9b316e07b3f5bdd625bf1b1124746f7520475c4db81a5fac").into()),
                PublicKey(hex!("a71822dc8191faeef23b83830c7e7e867870809058ce738920da2171ccb06baf11a3fd9caa9b643ab89082f21cbeb1f0").into()),
                PublicKey(hex!("b0fcdecd437e69b6c8e2d0c9f3aa8c754d35aa62c342de3d030f6d6080816261ea8c69b1f0f6123cd6feae78e2a7840f").into()),
                PublicKey(hex!("ac5da5e48851c9593d17fce5fe99b7fb321234cf24b71ac9ddcf147b64c586a16925835e2220132e768b05c071f959e5").into()),
                PublicKey(hex!("a4972c43d2c58c699fc6281e22b4448d7626ec8c330a0b4fa91be65fb349c051b8dcd4f485c1f5aa811bb44e2e7950dd").into()),
                PublicKey(hex!("87c3725ccbba0d08944731eacc38d768dd2e3357e4fb37a0956311bd8f772f691de0a4e6d7b2581e1cd863a184404b86").into()),
                PublicKey(hex!("8c51d8a737babd8c5c1e594264c398d76a55423060c71c53b4ba62c3532047df1ba9ec7492d0b7e39f515a03d6679a41").into()),
                PublicKey(hex!("b2173df25cb321b5abf0c8bcd9acd2c7fac2be1ff90f9c592060e473d684f2f900376566ca2fc5e814b8fe58d94ac5a5").into()),
                PublicKey(hex!("a46ab11e491de8294806ee617ac6c31598ade579910ae8c74a2cd1e604b4b4d0027887fb61c16513f0f7182dc3131335").into()),
                PublicKey(hex!("a739d8958ae3e2a18c723e5fbff04bb9956979d9ca31528297f3df008afe7f5e932aa2bc8cf526ce6361372ace2602ab").into()),
                PublicKey(hex!("b5e5ddc01575c3fa6d02845c07d77e395ac26c5b83e7e2756585903052a3f4a62da3dec598bdabfae93d96aee97ba992").into()),
                PublicKey(hex!("8e12a042e0c0106d8c6cda7342bbfdc829ef4e7d7f15185e1fb95a3891a2c91ef11a712d1c5154dace2af53e73dbe135").into()),
                PublicKey(hex!("b6cc63a5b677bcd64fb03d9a82d70380458c08d9a4bb01d024efcc8432abfd5b6ed1f8ccf4e59856c59bdc1da7913814").into()),
                PublicKey(hex!("a51a0d11ddce76382efee000c7df8d6ab2de3291acf71081eb1b0d87db2d0f510ddbe524460b03a5e0576681638511b6").into()),
                PublicKey(hex!("b14fc1bf0c3ee2be886d4897465d7ecbcf7f6aeac7bc6cea7bdc35763a68e1965a27753f1b0f006075b96e207d3b1525").into()),
                PublicKey(hex!("a0324ec32d42b757db2bf0df159733409bd5e2d92170dd87fde24eadc30ffb85dd0a4e235c39d753b769daa1f994526b").into()),
                PublicKey(hex!("825ea4d13ebd8369f47d060822be0017a75fc6343c3427b89df468daa7bc56256f6fe9bca06dd5489c83a579090006ee").into()),
                PublicKey(hex!("a6044cc57f428d62dbb9566476fcc3ac25dd50458401b3f772d53b65820c9c4a38c99bac8553ec4a2f6bf84fbc7a0dbd").into()),
                PublicKey(hex!("84481db7d16e30a5d35ef5abb81735e03f06bcf4236d87e7a431c6c50f3aafd352203af69df0fb62c8485fc06b786daa").into()),
                PublicKey(hex!("a0a6e060a29137ed45fc1f5f89a7fa13f8e0deedd10eafb97a30356a6198fc55236964148eb2a29ba640a0bb80c21bfc").into()),
                PublicKey(hex!("a800b7a9ef1317692bdf3b874bc74dfd30bc845ce5891115611ecee431a04763b47b2db825aead5c7df8d7245f6cc81f").into()),
                PublicKey(hex!("b9f684c2bfab6a088e1715ae0def1dc1c1bf2e0523965438209b7fd00d7820a2ed472222b97ec1d8b75dcaa43078a69e").into()),
                PublicKey(hex!("addc0ab11d4bf6103733bb6985a3012588150dd522a207e6f6f54405a1bcf3f906e481f9bd6caa69040f3b4d7b3321c8").into()),
                PublicKey(hex!("8e1b71a8e53fdf5c7eb2f2359cf51556bda3ac5909cbf8ab6d1e1cfe42390784c178eef9fc456b4b737ca822fd549bdf").into()),
                PublicKey(hex!("839dca2a0b5c57e75b4c0e10ee5fa79fdf5e760c63f9cd56c489fdd5a31884fe1d07fa4070e69b1af38799c74eb0df15").into()),
                PublicKey(hex!("a44a6df84e9d4d3ee95cc9c4f202a850ed13fc78f8702e47bf602a99a64f7a10bb76060f68e00084fd1ff2e391ac83db").into()),
                PublicKey(hex!("8f775bd2b1deeef18a5fb358d797741f0c6af395a0dcffbb514289cc57fe54fd9e420761a029d458b21f6eb9fd488a22").into()),
                PublicKey(hex!("86d967d23ff35ca8130ef625e91e65fbc9cfe3bf9742601fc954c2b946846adcf6c6d85e0f8b7eb45e2d528b9f82d62a").into()),
                PublicKey(hex!("a1e852384c6e3d217943e74d2176b3b7aaaf1b032ad4fdaff7bebc8173075bcb4027fa054dd4a2155e1104b3dd66a17f").into()),
                PublicKey(hex!("949886832e61a22af492dc8b709e956050240475e474ea790e38a58bf3ddecdc34ab729ee8be90b2d299df578cd66d83").into()),
                PublicKey(hex!("aa1738c23a2c0f162e150ae080daf1dc083462523e1a6ae972abea75f0907df414670ba520904a9860f33fbd9fc4f45d").into()),
                PublicKey(hex!("ae3122a10174f3e48b677748cd5ebaa4dd86a1fd7f8f9bb155cde95a8138264c9480c95bbcde4102a3b0ba9550081b40").into()),
                PublicKey(hex!("93756374448632aadd6a05bf63272a465892153c1369e3d7d3c2af49f57096ab5d982b191f061849b33ed899eae6a730").into()),
                PublicKey(hex!("adbb39b14d792a133e1b97a3da1ba6f8dad66d2c869f7d09858e0919f6b0bc616ca647511dbfc08d481fb5bf254cd855").into()),
                PublicKey(hex!("b014d35436d68b0b743710d6355d2987fcfb6c749fdf084779d41f4eaddea507fe52afedbba1ab5650462b7cf142eb8c").into()),
                PublicKey(hex!("98d728942f541cb20bc84ec61e8b703d7bf589187d70dcea2b0eac67bf28109ed9fd0013407ef9d18eaf2a2f23af715a").into()),
                PublicKey(hex!("ab72de30b3b205b47c2bf5a6801183b651a7dd418ea8229bea50639cd5cb7fddc4187cfdfb06c3dc72a032eaa05bbc98").into()),
                PublicKey(hex!("8933231354485de83350dbab30153dc37bbc704aeb5879cc8a3cfca7bdd6c68c7e538aeb9154ff1217f6b366fcd0018a").into()),
                PublicKey(hex!("81b1098e9a8c38e9c00c71cd5a9ab53bb53d1e758bcba938c19ba6cd4cf1059787b543b9b6f76b858e8b6a3e41a64855").into()),
                PublicKey(hex!("85507fa804a6cc5a96268ac3d5c176d831600441acb09579353870631183959643e35c958c24e8e14e60a497ad4cc0f7").into()),
                PublicKey(hex!("9820bfa98fb1b770b9b1346ea8382c2ae399b7db200e37285f5bde8e8127053a5553d54d45ff636703a2bba23536af80").into()),
                PublicKey(hex!("ae25ef59277aa8b49048d844a3f46e4b48e82f8651af39ab5087c64864b248d5179d45b289437967b2e5fb0b7811857e").into()),
                PublicKey(hex!("af1c3ddde88043d25c0a5f232659dd4e4312afb08f3db000b6efb4ef345203fe4e57cc9a868ff55d18348fb56489ab7d").into()),
                PublicKey(hex!("b784674ea025541b07a84faca5829c0123341ec38f30fa86e056e665625e799cb15cfc02a5695b1b960169cda6a0399c").into()),
                PublicKey(hex!("acbc6b4aea95316bfbb852c9a48a15202bdc8fea7978935344992f5624c1f7eec53ea977e1917d90c6f46302fcfbff83").into()),
                PublicKey(hex!("8887d32b36eed875c535c6b7f9da02e5c20cc9eceb9b9dc2252ed10bdc41b125aa416065e327ff904d35df719a3a9fe7").into()),
                PublicKey(hex!("97636763d7eb244971d78efd1b796745aa7872c1b26a3c6bd56f6a6f0eab5cad0b1f4ab06fde7b77f98495ac1917a35e").into()),
                PublicKey(hex!("90fe05fcb438d854179186b150b16c44154ee3ad17c9326f34becf5ca731d1fc51451830ec30070f8d89afb2b83c9f5f").into()),
                PublicKey(hex!("8b3e4af1bd6853c273ccd2462f90e784ac38a349d8d0f6bb4888a20b1a90de63995e61f46c34e085ccbb2fe4d3f2a06b").into()),
                PublicKey(hex!("a181872adf64a21a9e2a6d220952e49f2ce300523fcb8f7c34b5f8ba4731505d9cb69a8dcae896963acad1987c7f0bd3").into()),
                PublicKey(hex!("929a5403cc503a6e685c8066c07ef7f004823e9424eff951a1834e1494d085270dac1ecb9d455694a262d06864716853").into()),
                PublicKey(hex!("a6dee42ddc3d017e43c3fddfd39ff4f2b1278137f211380c62e05a1db98411c3af4329fedc6633fb40eaacb767d861e8").into()),
                PublicKey(hex!("aa7b6f4bc7b3d30a3d6ec5b9c8b371d6d9fe151fccb30a89068cfd1a27fa662f89c0991d28a59a53ad64f9f60e61f7c9").into()),
                PublicKey(hex!("aea1b26c6ad68d2e98c6978e84122245231994a436cc468e54594fc708bddc5d3776b737dfe3df53b3bd65c5ef753c6a").into()),
                PublicKey(hex!("8623d174402fbf7500cbfb28d9205e08882da1b1222a8e959d5a5619612fccbf25e3b646609d0d0c15f291abf7018b76").into()),
                PublicKey(hex!("b6998911ed834f7f410a9dec34531141a74181c4c6df4edb302b357fb4087d23abd5ab1572f966b611bf481f9a75f219").into()),
                PublicKey(hex!("95e9e0d40ae1bce6a9a5b241998c32c028e3d6440ec293fe36099a451593d45d656ede75dda18ac2760a617655e671f4").into()),
                PublicKey(hex!("87a90fd2b8f48d6a3abb7dd83d01ca2b674470cd07bdbe1ea7af989a5e6ddc0316912c6c7197e0ce1ffa4ec219d5c1e3").into()),
                PublicKey(hex!("a351d8141a754ae6d363ee0fddcd9a553119158eec61a3940a327724a0a169544a598281550d5eb9cb5075bf0b8a9862").into()),
                PublicKey(hex!("8c452dc232c35638aec5ac0d25b4293a99a5013cd317d79f6a30058df072fae193e27e12158325007cec659cc40efea1").into()),
                PublicKey(hex!("a081f2a9e386a8cb308fd2f141638961d973328f97c85558f24e6df5db19d040d16863414c0b2634b8ee2ab8a798bde4").into()),
                PublicKey(hex!("af0802e67efa84ed80fbf48ae3faac9a95d998050904f50ab0c9cdd5dd3161c62be076cbb4e8cad988990629e25760ca").into()),
                PublicKey(hex!("9363cc620a01b932fadd933dda79e80864a5d299f5819fd13287a023dc84fe66aeacb9d4b8efbd8e704ca35a6a2aec3e").into()),
                PublicKey(hex!("93c991157e757b2611d30c6ffc481b67424b07bb3fb151a55a4a63e0334c54a7d51457ecba6d2d85b9e3c7bae1fd4645").into()),
                PublicKey(hex!("8b91078930ab518b8433777df5d2f4558f778e40ab3dbbe8a4c96c08c4751bc0cd13eb564c661cbdce2aea624f8b4976").into()),
                PublicKey(hex!("8a50d06eb83fa56732aaf57e02a10d6794e4199e04e6c9d2124e1b6584610ac6e557ec42b69e651a531d4e070966956e").into()),
                PublicKey(hex!("8198ec5e80e94061f91e429fceecdab138368fd701245db203c6ec029ee5428d45b8355ea3da96d62019a6ce2d0d347a").into()),
                PublicKey(hex!("887cc2163fc7a801e1fb0b6b1b6c6be34f3ff5084e554b9dbb0b81de801ee95c533a273bd7157c69aada89756346a736").into()),
                PublicKey(hex!("8a18421246d505f7d78e58f076baed7772dcfb3dffb9d2b6be08b43b7d204ba7a99d19afbfd4a9436b70a93554ef836a").into()),
                PublicKey(hex!("95dc70d9fafd6ec9f5eaf6800380124828d234da1e3196f7310963734f5d26e88899208105ce143471887e825a2272a9").into()),
                PublicKey(hex!("8d82f4e72f07b7c25c99de78439361fc67719d7c098053c9e0cc27f81916cd96f3945e8511b56c736f3f09eeb0a49470").into()),
                PublicKey(hex!("b6b79c848fa33af28a563275b178a63733d6410fb421ebf8a40da844cbfbdd958d51b99832c6d50d7298a7a113424d0e").into()),
                PublicKey(hex!("930e7f5bb612e94136cf33872639fbe397beda5afd76bd075ac5d9651275328cc201ef1e8bbee13da21cf486aad49f69").into()),
                PublicKey(hex!("a75bf5cd6b379bd48f098f0aa2283fbda1323a5ce40add44c7ea334eb2a4a736dc0d1342919951d341dc3741b11a59eb").into()),
                PublicKey(hex!("b48eb1c3e49eb976c49286cf1b0225193e1e8a657547ffaa8f19d85640044230c0b1b47b6fab31cf46b58ab8dc58caf0").into()),
                PublicKey(hex!("866ab1c5be69ef591b6ebcc940e96ed773d77bc08964671f6cd3995e5681bba919a13814bf8f1e8799137c0e3a41b4e9").into()),
                PublicKey(hex!("9357878f8a721eed8f3e153e2b9d5306e581b955c74626a4197e1324071e0f6e44fd8272a2487eaeae89b8136f1a5d2a").into()),
                PublicKey(hex!("b4c9fa2892a8e88c94c04970ef7c0d398840652088ce31912cbca586bacff0cf5250674386fc6bb52ea807620545945e").into()),
                PublicKey(hex!("b5d9c6f433c461245c43d75029ee1ab0c8f5ba805538f346b111b68eb4229fb49e3ece69001b2cb14487264c6b12f085").into()),
                PublicKey(hex!("989e46cf4fcfc2c5f1cfe26b44def4ca0f6e542691ac26fddc2566069f8aa429e109249466d5441853b98abba9cfb275").into()),
                PublicKey(hex!("8501544afc6cabcd10030a5081a2737e4e806480580d2e4e7e478b1431719c337b4df1ba831deb3c7b9b59b4ff61549e").into()),
                PublicKey(hex!("a0935ca2e0872b8b2d48afbe9a88af7fd4d347d5c42fe4e850af7b91beb1f58dade7480117b5ddaf5d0e86aa34989118").into()),
                PublicKey(hex!("b49bbbcdb108a1d05fcc04b4f75e0b28368914ddfb43fc519652c3a79476ff7484d9e0aab58d3ed0ab8096d278117dbb").into()),
                PublicKey(hex!("ab413641aac054f3d783183826bf784e3a10a4be69b72d7fb20a35ca0af432c7bcef8aed6df7832b915fbd17fcf4632e").into()),
                PublicKey(hex!("8ccbda6aaede4fe80f9bc2f68dc5e186b0a58bfcd1852359da1f9dec4fca639ea19c619d4e1241a77d368a227cf5fadc").into()),
                PublicKey(hex!("982bfa9ef09d99f6dff5b3eebc394e0bfc4687efe17f636141456aecacce4c6ceb3e985314779eec005625663e9715ce").into()),
                PublicKey(hex!("8bfe4a7b45ef2c757a6b7de523744a803f1443c0f9113b6e44a40a2457bf2fbaa7c63a38bdd5597e2ff7074083da706e").into()),
                PublicKey(hex!("ab4a4020c1238ae1189af6134105a3f9c502b2cf729390b3e9ce2b3708e66ba99324ebe1ef9e5003bf250fb93cbb5d6a").into()),
                PublicKey(hex!("ab7362207e27a8b05297bbd080b814a7422c8742f364713bdc149aa6bf3876464feaf5004bf1de75be53accd4ce8591a").into()),
                PublicKey(hex!("a88d5db0ae2631a994cb9275cd124fe223ebea638d2510cf65c9bb369858c5ef68d980e2761d09f9124ebc3aabbc3746").into()),
                PublicKey(hex!("981e8af83a6b5100d15b3f067685008abe6dd4916481591aedc021a4963d0946d5efd4687e488ff274206a59eb45d8ab").into()),
                PublicKey(hex!("b3087accda242578592fb15bb3f851826f9b750c57e163bccad4f7e56df9b0838610bfe150bc42d316b9f97f26029620").into()),
                PublicKey(hex!("88be5e48ed5030b24b69f7d2342d7ce7fd29132abdf1fcfeb8e05575c5fd01b442f90d86690bfe5efe0388cc5cde7f8e").into()),
                PublicKey(hex!("abe4af5454af53d3c8c5857cdaaf502433fbfb53b383e267d409c6a6ec4fcfd62a2bc87fb78c7b4ad8af999031372396").into()),
                PublicKey(hex!("90d45fa3a1596289373f14b199fc90cdd935c830b29fa6868656f676e54d143afcebbb21e49a395275228909d1294529").into()),
                PublicKey(hex!("8a379cdc1b9c02864acfd15bf76e475fb91883b2af4565376e1d144448ecbb7a3b134a10d0a536c885f1bcbd0f370df7").into()),
                PublicKey(hex!("b9c5fd0775264ca20c872742c20f1525356966c49197fcfd1f28cdae85ce8bf10e668cd14aec7aaab7eef0cd5876897c").into()),
                PublicKey(hex!("8b4777177f3f16c6aacf2102d022aea2483e355e92f42708cde9111e11db988031ce051bfcf11e1c8edd1ef93234b231").into()),
                PublicKey(hex!("99e266a567248ed95f78816ed159dc648867bbb0e92b311f99d58de7bb65ebd952d26c116c8303dd872c85b620fa233b").into()),
                PublicKey(hex!("b53556a74ac75da4a5d6365a2f0ef02aabac1c9a4974b5dff6ebcff9917280ac2a39026ab6d5d566af58ee2bef2cb495").into()),
                PublicKey(hex!("8dd1e55c3d5d221675aeeef3e5293d9d9db11e27127566495626ecbd1b14f487d502a7f015764344ecebd44a1403b89a").into()),
                PublicKey(hex!("a71106f5a5e57784381f9c0f2cf900e766257ee32dda16ecefadb7667cd8043cad92784f28c18898cea1e5303eb0adb1").into()),
                PublicKey(hex!("b22416585448a76c17070df95306982a5e555932de161d22b41464ffba4e72af11d0a795f097e8154a36f7785f7be8c8").into()),
                PublicKey(hex!("8c32c38edfc9d07eb131672ffbfdae2ecc4a89972176d040fe02c0b8d1d1f99d852499f417a6ecd1cd7c29c804e02cf5").into()),
                PublicKey(hex!("896f02e86b0e1d57b9e1c70c4c1cc2da61c1e4b1ca34fe94fbe95fa9b290881dfacb32d98b238c1f49805a92d596191c").into()),
                PublicKey(hex!("938a20c63a85916602ccf4d621b8265a12e21bbaa351949e57588508040eaaa20fb4f2b651daf40b0a1b1a538e4e713e").into()),
                PublicKey(hex!("a51ac0f236100c61f7cb385f5ce708768041545a197aae45f940413f7ccbe07f8f666dcae17025803883be5f916929fe").into()),
                PublicKey(hex!("b69b088366a5b04a5d1b873c24ff8cb7f4fddb29d9b45ba7f59b8f66837858aa3b93edbdce14b13fda7f6daf4907af66").into()),
                PublicKey(hex!("b879dcfac3c8e172d1e7b9a6118cfbd60eaab03cfc1654c7b6aa2641020c1bd6e1b0f80f90a80213a0a9c1146cc5c976").into()),
                PublicKey(hex!("98736c8984af2167683d171e97d2a101900f831c2d4a6cc5f4f1dc8bbffdd29bec4d5cf54cd6c0eb39829118bc1f7558").into()),
                PublicKey(hex!("813dda2ffc13d7a9bee56c5b946ea174bffe752c5bd29315904fa174d420f979d7d6861f17a50d70b3cea6f4da38a41f").into()),
                PublicKey(hex!("9836260d94e5ec78ea8ccc19804ef4e4f965945888341fff6f866309e6a1e2a58426abaac9b5f9d8c7b3da270521db42").into()),
                PublicKey(hex!("81d738ae15f15500363d7643a27077ad790e6b2ea8a823d53b99bb2ed9df67088b00bd9dfd032384af3fcc3e5548ed15").into()),
                PublicKey(hex!("8da579f31045f275de7bfe467b17d49edd4a23b6f7450614e4bebde683c872c950847a1ca4752103dc655aadaf9ec66e").into()),
                PublicKey(hex!("b064e260de1ded194746bef807b8a7b88a1866783910c568a571d6b0fe6ccb7c19acf7f6663c43c12db269c09061854d").into()),
                PublicKey(hex!("b5348fb406ec3d0d9ad28f3efa1f0ae239e7abf44e96b5135bd12dba676c1156e3a3c8bbd237c74752fa05f2520e142c").into()),
                PublicKey(hex!("85de612ce9783fa98094d9059368703a2b02a115fbd01b4755d6fba664435ee281dd34d7d2892f28a349a0ac1982cf44").into()),
                PublicKey(hex!("a1195bc1b4bceb76cfcfb3c4e81b7c8557b7d6d5817f4fdbdc8b2de593e193f01c0205d1e6427c2213ed65cbc47c35a9").into()),
                PublicKey(hex!("899a05fa964dab5b63f2098c00118b6a78ea55fcfdf756475b411c4d481a1a25d6e0e72e9207e27d878904d564baf4e1").into()),
                PublicKey(hex!("80de081dc0b475d739971db6d1e0de682cb148c19f7efc214f20540a630ee20b4ee8c0a2f1eee12dcb5c7830f6134927").into()),
                PublicKey(hex!("8d2066b2e6947140c1bb745824479211344a5dbb4455356190a546d9920486b068721415f898b533fa38b4aae4f8f192").into()),
                PublicKey(hex!("87699a4ef135b735682df339f0e984a2cfc84a734c552a121807d8a0c7f43133693a3ac35dce05a45aee6bb6bc6cac6a").into()),
                PublicKey(hex!("89434e6206c69bf7bc195559e3139151b365a4cd2ccbf4ee15a77fe33d28ccdd2386dfe1557fa0615edd9e83e0cc4c84").into()),
                PublicKey(hex!("a68b9cd3969c4e830b23a2d15b794803dc3606ba058523847b412c267219ee9eb6a535affd11687d7c5dfdd4bb7cff7a").into()),
                PublicKey(hex!("a78966c6c6ddd6867d3bc31d1a872387de9b3fd0688f4e0b288c4cc2013b66d09242d8ff16380c071857e0229a8ca654").into()),
                PublicKey(hex!("a231ba8c3399c20c7c1c5ee02a0df8d8ef440884c4a5cb880ace22ebe22f288c5ff2a7f225652cd389a87934beb5b11c").into()),
                PublicKey(hex!("aca212e9adeb1c05ba42caffe412dd8d09dd9391704b8a8e2482c92d1b4a28fdbc69b8f0786f429d55bd934c3fbf89b8").into()),
                PublicKey(hex!("89d55d505a397a743a924e69da66a2c9a4ef3e2a40f4aa1522d220c76dbb96a1d4ebfd5ffac7185a62aeafa0c0b47a2a").into()),
                PublicKey(hex!("ac90b00cf35c673c8271e66d61814015cc46441686e7c23042aa46f1126e29f65d1a902bacfa6b0d8167161d849e8998").into()),
                PublicKey(hex!("8588d4935440f0ff07050beb8ab345908706f53aaff03454590851e6fcd77c9057c278b4cb3c7460f1634ce5d2448383").into()),
                PublicKey(hex!("b8fbfe1e619d6593e29155a7a7dc894cb13ea10650eb1ed72539e14b893833b9b88a6687e7e3848e44c2249f58bb870c").into()),
                PublicKey(hex!("a13fc335b6b03be8cd7fae1f8ca8154c64ff5efaf2a0c4ed410f1c685272853ac482e3f60cbcdca6a8a319fd766337e6").into()),
                PublicKey(hex!("8ca33502f1af24522426b9a2421a32118a1a277f4c040e544bd7db3a3dd306e78054e67f29cd3d19e01911c1ae1012de").into()),
                PublicKey(hex!("a5682819a3c3c61f8df8b49444dfa043134d647f42aea54a46930292820e71776c527950bff936126c9b2b113c44e443").into()),
                PublicKey(hex!("892938e4e91140486c0182a3a5307c64b16e9a215e8f6d137fba854dbb9ecc15b2f9c5024cf01b9ecc0d2e98c430fefb").into()),
                PublicKey(hex!("86e7cbcda20ecfda70f403471bf68a525c9f5d82714980f1994d9334088ada847ead966bba21e525426c8245534796e3").into()),
                PublicKey(hex!("8e6b197329670947098f0c6a5621549207f2811b2ed5c2d45fd09db212db4d8b1d8b0617d801e48e203df4450700e3f7").into()),
                PublicKey(hex!("83c2a4b758b9489aee59489060c99e66a3cd0a711cdbec3579f36267b7d42606ce7087f66a761553eaca30df1d4754ad").into()),
                PublicKey(hex!("a0436c77489293b1e4923d03f72096f922a349dcb207f872a7b9d1dadba74d369cc78192389e153cf347b93f5d688a7b").into()),
                PublicKey(hex!("b0694887c9d0a24df9e98ae93d48f39af69861a28d0195aace5b3fc6a06e2cfe8457cd83fcefa0e38416d1c56992bc41").into()),
                PublicKey(hex!("b0caaad6a140adcc52b9ef8bd1a880572b3d30c521be2e9f960f00a458c327e0a3e1d042eaa98456bed255d7a8f52823").into()),
                PublicKey(hex!("b11d9977e5f6bf0452a36c5d9c814fd1a31b652e4ed694eee5cedc89e85113f892d10f9d35eb60494588f4444c04c3ae").into()),
                PublicKey(hex!("8330604acf917dc7412b7a8b8651fba0fe4e98bd53b45b75fd150e84d656ca97820835785c5805bfaf9be625da3533f2").into()),
                PublicKey(hex!("a64828cabbb8b12b93ba857e0a4b3f947c6bb2b1222b70b58842992b491df6e7e20bb3a3a871a9410342b0e928534027").into()),
                PublicKey(hex!("a495a4e6e4a49570973a7431a205f0ee5b62844a304ee72ed2f40799a663a3b47a8df9fc7f28ff3271aa026212017c69").into()),
                PublicKey(hex!("97d64c302eb74472a6d772cbf095c9c8de06d1283c22353038e6ca9d42467168767eb141cab1d8fd0ac99363a98d8772").into()),
                PublicKey(hex!("930ce5655240ecfd27c6c3c3a3f5efd34de174153337beef0f7b92f8ea3fa5dd04597403e01e82103d4b793cf5508cdb").into()),
                PublicKey(hex!("acbf7b34bf313ff8a3e79e642ebc7bb0a54064adeef3bbae9287a93689af20f3b477ef4e1cf0cc6ab5ea076dd70ea0ce").into()),
                PublicKey(hex!("a78b0d2fe61fa29ee0ddcbef9daab0682f8749f09a3c720fe37b6bd67b96d537821d2ac9d20144a42749c324d40209c5").into()),
                PublicKey(hex!("8976a69b9f47e069354704972a404e1b52d046796bb23a7a6c47e75ccfa2b95e0132a435b2ec761ce1f513dd934cafed").into()),
                PublicKey(hex!("864cb5acf079a3242a3766cb044b84d4874e89f5b696a9238d9ce81e5a2962ea1fab676518ad4b82a79cc46155cfe98a").into()),
                PublicKey(hex!("a584bc3bc33fbe92162db7107837a16f399b4dbe309564350a4dc7d66d410230d4335d17e93377ae866b84c3558a2f4c").into()),
                PublicKey(hex!("8bc24ced527e6132f5dcce2625f2904bc7e61258a9f2bc89933ebcfe4f38f1ae2f9c48e15d746297510ab10edae29bdf").into()),
                PublicKey(hex!("aa74ab37358f8eb6bdde8661d11eba717d1ad3383726bd7c5678c837609936ac7b582df17d84c8e2f3d260a63290514b").into()),
                PublicKey(hex!("b75fd74b37f79939bd30c3199497c44132cb55bdba25829477213cf9688762091cf5e6a613e5f1b414d091cc69c6afe9").into()),
                PublicKey(hex!("a7c9904373de6ad8625f41417095dd56ce64d0602ca8da031ca18e559ff8384fd114d86db7c20030d6a25b861f710a32").into()),
                PublicKey(hex!("920d3cdcb2d7140b4ebffe2581652a1d2b9420e0fd7f1790cbfb16b45d487cf94dc1935c5aeaba424c4cf0eaf500359d").into()),
                PublicKey(hex!("afa5b467c0b4d503448dd4318d761d6876e73789b5b7ba7e8f8adae3f6ab76114a884cb291c1be1d5af65eae3dc70b4a").into()),
                PublicKey(hex!("8fe455e0420cb10fa007eac8344f7100e47f3d7350aaa8e427d90ae3066e61b4280d0e54e4d193a12b51256b802549d2").into()),
                PublicKey(hex!("aa5517f9bb3c2ddcbfefb4ef0f6df359ca15554f30016bd169d1dd9dba13b5db36aa236e5c2eab9c02fec4c2653f43bf").into()),
                PublicKey(hex!("b86393be9ff99f794dd70504071525b4113f0309d9ff89e406c2a3ddbccdbf559f1857a566fd772be6454227ac2cf25b").into()),
                PublicKey(hex!("a0e2621f6b29acb5f3129e18612abe0545a7dc0e1f4c27faf3b86c52e781f474cb4d7c8207e212af8a2b6c90ef59cdd3").into()),
                PublicKey(hex!("b50e41586d80dc5d8f469a05a8fe51c3bcea28243ccc46e245f7414881518034549807798556a81ffb8147d55d60809a").into()),
                PublicKey(hex!("97b72232c88325dd1105b749227d2ce88c79e74a238abf04cab710d6777a6cb58158cd70cd961049c2757595dd74c814").into()),
                PublicKey(hex!("b675f511a1ed137a2515d97220a8a641595aa6b8e21e42de39e2fa9474b5f2b7878cfb54d458a63f6fe3e6f8603ad764").into()),
                PublicKey(hex!("b6a322fa8e0f0a690e693c3efd4840df217d720bc07636882cc58b5e9831ad5b902cf47758616756040f836ccfb66b0b").into()),
                PublicKey(hex!("88c40604f54a19fc4b21f4419bf7bb3d590dbcde0d5e84ff7a93d273a305c5eeaa4c9a282d5d60a72718562212451ff5").into()),
                PublicKey(hex!("99d9df64469fddbf580f38d4790841afd77e4babb9a115cce736509c615ca1fb88b9ccbd216d90499b6c3d77ad5fdc67").into()),
                PublicKey(hex!("b3383819a2d4790ca93b92c247f0212153602ad8d0f78eccf0284b81077ceccbfea890e4c2316475c1dd5617a98cbe4f").into()),
                PublicKey(hex!("b28fb6dc61fb3cb6b32a9267beee8c5e9dbb8829ef9340a43085dce3f6a1117d309053e006cc9026a7c014c1318564b1").into()),
                PublicKey(hex!("ad8423103620ac942cc28597ce4d2afa0184bb649faed93dfaf2ee18716988debd56e381d38be75330aa773ab570b1be").into()),
                PublicKey(hex!("a39b7528ef1c0933980a10d1c795a4a7f5c4100f9f95c0d178603c3f5fdca63f01d92d73ef72d99978cbe6a919ac8cf7").into()),
                PublicKey(hex!("87a6676586f4502247937d52712f29f2dd20cf8983d5b89c1f4fb4a732535d043b94b3a4c413fe1fa9a4afeff008ca84").into()),
                PublicKey(hex!("ae6f2432d8ba1cc12376670d70ec4a9fd000c122465a2ad6d284da13600e22288f8f08556d0169d3f6b915fff3c78e2d").into()),
                PublicKey(hex!("8b23f660b24fade0b47e72c7d0e3c6b4c64917cb841ed89d2890e6903e3547492a21abec6570a8dad2838599f400eefd").into()),
                PublicKey(hex!("93e772dbd36839fcdfd318269ddd3c28c7f65b706876d3f1e37f8121438f455824a8d00eaa13ad81db6432a6b41d4aa1").into()),
                PublicKey(hex!("984e694b3dbee18328825c859a308a3187b51d9fb6b627f559cfd1cba94e2320ce3cc9e9a08f157dd312e3eb09d3c0bf").into()),
                PublicKey(hex!("81f973294d7b10d8d503f9f824061614dd5c02eca9cf6e22181c11196798aae9b72e6f93037a52ef52ef57fb226b0603").into()),
                PublicKey(hex!("8c96ef53d74a4e6505207f5a1aaa5cbc92c4d1dc432dbe5990681192beece698a18bfa7cf0e735abb63b178eb997dfbb").into()),
                PublicKey(hex!("b45c9fd18687bcd390941ddc9d9761fe60ee4f80e405503fa3b37e27710d6db2038d46329e0bcdcfeebffed9a3adfe28").into()),
                PublicKey(hex!("a208043ace29d9b2843e3483fe8f3431f0132cf13758929ddb5a4cb9494f905b307bcc6aa0f126643ba0f34624bda426").into()),
                PublicKey(hex!("afd86325db0cd7e5c5fed7fbd87a8aa6bd0f06c3973d4dbd06c7b5788842836fd107258070bcc4b288ecd8c1b5022c4d").into()),
                PublicKey(hex!("afdacc0016efb3ff0687c1b59de292e4fa8614c2d00560fa307b6b6055ac139449e292437c20e22b2d22c85fc2526805").into()),
                PublicKey(hex!("93587a583188f137a4a193294494922ff97eb62325a6c18ffc095a10b06c8f1a466dda63e05887a3e726718b335bf94a").into()),
                PublicKey(hex!("aab8992a55af205c257714f7cdf170e8f3feb82c0e3b15f2aba9a438d51788fa3300559d7a68ff05f5daad38a43f465d").into()),
                PublicKey(hex!("8658801142cc4c505881fa808a0463100a1728349f43a2b4a2e2c6c3ce0d9b383975951507e744fc8b2ebad331789d24").into()),
                PublicKey(hex!("b262624ebb729fad56002696f26a8726bb887f3af7104da3d40f8f30e093241ca1d5724c4afc3f96c1866dffe5fec93b").into()),
                PublicKey(hex!("894f4dfbb4a543850c9c0f9b4573153adf36619475a927c1a7757b39aa9709c74c05b765b5c47a65695bbb00d9eed433").into()),
                PublicKey(hex!("93db9eb37c20fecf44853dbf609fe67c8acd108a24f371ad3ad6f2e08f41ee1508d98f30095c99fd098addd294506107").into()),
                PublicKey(hex!("84c0139a111acf65d3fb040334fb2a2e407bce418fffea0294dd6c09194f42548783887d7f0dab005f1dc0b6b1d75aff").into()),
                PublicKey(hex!("b566366bfd76a55b5f13062770d300217f9b9a3eb52e1fd81b4993fcb6434ebe0540ecf0d27dc47f92e2733782ceab3d").into()),
                PublicKey(hex!("a2e60f111691caf31077b563648fc765539c556b65a143bdad6bb9e1e0d41d857606fad3dcadc9f05b547c80e77b5594").into()),
                PublicKey(hex!("915ee847a89aa89fc470082e5f753e60193868bf1d16b45796859fa7409b006995c49d1da886c229cdd9a57daa9c85fb").into()),
                PublicKey(hex!("af13ce1b6103b79cf4de1f5ac765ead26da5f016ba29ae5c1939f6969b1bab7481dad359aeca8a1f5dae7d86c9e890c1").into()),
                PublicKey(hex!("9650a45d489b66b26977f663672286bd6d98650b597588b150db3fa8daaf4d419bccb606d313d44fdc49feed166889b4").into()),
                PublicKey(hex!("8da4c06ae2076c03c9255bbbfd68f1825c48e49ed09f8f84b01aaea4c5b141137ce4b7919d6f64e35187efbe519e87e7").into()),
                PublicKey(hex!("8237e9edb4bf258f6a1951b8646cb2cc123c15fc7acb8b2c0e7c914c01098443337a22db35d59d4bee69acc0f97bd0e6").into()),
                PublicKey(hex!("807e3ac2a54eea5f1d07c7d65b7b52dbf587bf05da51a7e586f039623ac528d686b4657633bc6eeac3f162324ac77111").into()),
                PublicKey(hex!("b500f10b8d9e78987d2ea3c57f35c57924df38679727d5cc5b48b798c3a4cbda09312132f074e983853b85056a9e58af").into()),
                PublicKey(hex!("94dfa9c2eaf8729c866d9609fd889a878dc37e92b746ca44629aaf125805fc29e450ea4766013749d670cee151ff47c5").into()),
                PublicKey(hex!("af48f4576d15f8cba4d2b3c8735bfe43f25b26486bcfcbd4a3a314f7043a552b703bee0e5a71a51f3e92eb7c9f37a9d7").into()),
                PublicKey(hex!("a9533002f9cba18b67fe414705a27040d795de3d6b7fb822788247d08e31b18d1d48be768bb556821f8ea88dbe9824db").into()),
                PublicKey(hex!("994fef5e8a5517d8cd197df906d5a96011a81477572c51ba4ccd0a289a017def29f9666adeb0194c8411163b33460280").into()),
                PublicKey(hex!("80aa9b75f502c8c462fddd916dd633d130c3e7fb3fdd6ac29cd6a2c2f2d14a3bc28eca348f2065720a7548220f0ce576").into()),
                PublicKey(hex!("a24516dd216a19caf5d502f7638e2eb960ff7a6595df3ab650c18cfd33a2db63075773b71610767700245573ac87a31a").into()),
                PublicKey(hex!("b9196907835b7c1b919f9448be87bed577aa9fcae0ce5039a215c8ab1b4b2801377440060a62bfb5ecfea86e3b9079f0").into()),
                PublicKey(hex!("98b266e6ac478f5748c3710d50559a6f7a93fa3f940aec51a77c7331c96ea8d9405da83dc786c90c52fa9fb525d2a0ab").into()),
                PublicKey(hex!("8d1d61fc995ac3ca16a53cc5a92c3fd98ef9da84f051de3e3d56f08888629446a4be589757dfde6025532f94905e81d0").into()),
                PublicKey(hex!("b0c05a348f5d1e747e6f4efcfbd5f42e2557f195404e76cdba12b87455200f78e681b4b8bc32b3a7c25116c3cd2521bb").into()),
                PublicKey(hex!("8f4d5984cb127b0c316f00ab0001e41bfc66df9b722dd0f4b73d7e8661240d1af313139c79f0ad62005e1f7bb217ee1c").into()),
                PublicKey(hex!("a283fd54ffb5f6e79e46f1a02acadd34ea22f05407cf931e4568d4c217e821ecfc8dff9720db0ca55c9fead2193ed569").into()),
                PublicKey(hex!("a84b948288775f88d43af8ce78d84bc36e3bdf7d90932b35dccb5e6855b6ce9716e62d66f701fe22e97703da36d9ed27").into()),
                PublicKey(hex!("87acb63e250056c816c9650d404aaccbe08039d934544dc92f8117757a035a04e4daf1b17d879051f0177b4cb79a79a6").into()),
                PublicKey(hex!("856cd3d6ffa41c88ae1dd4e72a50aa9abdec62795222646951cbee5e9012ffa94de25807eb1794b128baf8da495ea4a4").into()),
                PublicKey(hex!("8bf5ce1dfc7d708f2db0be1b4ef79630211267349e1fcd8790db5905f5f4d6d03fdfc39d90fb7d56fe53bf6071c80bbf").into()),
                PublicKey(hex!("af90dffc98e8b6c0a8a7eb330ca5e15cd24087d74c600369e8e64662fbda182052eea7b3af667b1d08278ac676252038").into()),
                PublicKey(hex!("96251271439619421aa1d4f35e21522d20e24688e769b2a9585f1028f919c3aa5ebac94f682a886ee5efa6cac200aadc").into()),
                PublicKey(hex!("92519f48cd19443b5eedd98f31afd8490114a1303c33b67f7b68e51f32498137bab7d7be5e5c1c73f1c746b0856dd68b").into()),
                PublicKey(hex!("b55bce6c0b44973bbff8fa85d3e05a17c9831a1dfd4948ba207894243156bde86441fbcb0762edfdff714080b64637a2").into()),
                PublicKey(hex!("880c4ddc046902e63a0b24dbda5995b972b8d6ae830171ca9ee0281fcf75406c97af03e23938d38984050cd0e44ba88d").into()),
                PublicKey(hex!("88328b53e84e891a64b530d355efe93497e662d371c278e2cc6a416e8f1387b03e5f65a5e6f0f5d54eb4830003c193e1").into()),
                PublicKey(hex!("81be7417ed89b7fe1dbdcbfe289db53b5239af836e5b6c1bed6cd6be2f1588c06eabd352146f121b14086e4d5828c942").into()),
                PublicKey(hex!("a4a069ce0d08ba3c954b6d21a7c0a08799ae33888ba65b7735d6db6d5ee767bea338ee5c765bb39b90dc1e422b0f163f").into()),
                PublicKey(hex!("b014ded56333d58ee64e5ab82e35cbceeed1c7928c14d4e309ce88e966d166a5b63097271d8b531fce9d448c25f8db64").into()),
                PublicKey(hex!("8d42ddb5da2105b876c27c2d85136efd249078a0993ce0eed1a2e69255e63a1b72f8f6e8c9f1141d463c6e4e6e5d8ac9").into()),
                PublicKey(hex!("b8032bc20c0bd5c0ed89540c385282133ddc87e018a886f17e6f7641318226d53fa2867c2a0406899a8ba54566898112").into()),
                PublicKey(hex!("869e5b8e1450c62f783598b2ecf34248bc700ea0a3a6f376e45fe27e3a2ae35cf5b89a315220af88c5a8807d0784f3cf").into()),
                PublicKey(hex!("b3715fcc1b138d9150e3b7e10eb37af0f892b43f5e9f570adfcb787edfee211bbeae4ac47db1955840c2a938a3bc9d40").into()),
                PublicKey(hex!("9718a35f8b6ea5e38d1c925a8f28077ac6661cf93bf925790acdd659221c62dc1c893b7b9d05e23cd985046b3f25a4e9").into()),
                PublicKey(hex!("8dcfcc24a7128abf11547cda082da5c49f9b302421f47b79b398f5bb31ffb92b34bb16334248279081bc0cd63ac262da").into()),
                PublicKey(hex!("b2abfbad7107b9b23b323d1b3246506819e184ea491d0aa61318447cd2ad292224ed956c2134a910b5d8b70c8a3f609d").into()),
                PublicKey(hex!("ac00722c134af336051176f897a700170319aa6a19ecdf270edcd3307eec065fb92b3ab8f51407b41661f99456e30233").into()),
                PublicKey(hex!("82408a9cc9f24070bda4adf8fdeb7f54646e91437c558491d0d7e359fe563070cea3f2eb0ea37b08a95e048c01e0d44e").into()),
                PublicKey(hex!("abec6d70cd37d9049469635be50176932d08233b47cc57e6bfc48a1de60573bd7bb927a18ea3104e31b472bb9f7c14e8").into()),
                PublicKey(hex!("a8c09cf656fd70568fa65063611cbb97eec41b35ef765c7531e8a93f3af5efbc4abe264a5438eed57c0e005832ca3f56").into()),
                PublicKey(hex!("8eb8b2ec90a41b9cb9bc687395da9e448c9058fcfabac8dc3e790bbc38e6fd2e46b35c1afb029e9e66621ee822bd6c76").into()),
                PublicKey(hex!("a2606d65c730631bfeec395943d770e8e4030310d5c05e5abff200bb67ead13e9a6aa5933a52b206ae7a2bb478bfd02d").into()),
                PublicKey(hex!("a22d9864a5966653ab64e6d7929fcad9608cfa9315892abdc51da28c15c464fd0358cb96b2aef82b14ecf4b26049eb46").into()),
                PublicKey(hex!("87d1e1195e0a88518c745a044579447c4106f0278bae54cf61e50f66f00109c4a692c140448ecc7375265a04162f9073").into()),
                PublicKey(hex!("8801cea17e9d21bc93a3061807a055a07c1cdbe21b1c3b1575e31ed8873fa5565f3995ca725f60669c6212cce4937cfb").into()),
                PublicKey(hex!("b65df5bcc2cf89b8625cd222b22c801429e284a37affe57fb9dc89ae4fc3ec91de7fe8f8ac761a8142f82dd2da9f21bf").into()),
                PublicKey(hex!("a5aaae2f8d6f3e0c22b3de416f7f79631d4831265353129ef59df089326426b13468aa4c80f453e1730af06e618450dd").into()),
                PublicKey(hex!("a4b676d220aaaf1ce460d41456f30a09d7894e1090d83913655c5515bbbb2d04e0eeda2fa2c87f7c26f08a744b11cf57").into()),
                PublicKey(hex!("917e2411875f45390d21c79b96f83622fc97c3eca862728a1919d0185ac6d04b61ad847330b7706c94301357e30a216a").into()),
                PublicKey(hex!("971af2f904420fe41dcce2605ba84a1a11c2b418b1b3688b7c9ce7d0a8b26705a6e09bb9e8f55931be278a1a931315f0").into()),
                PublicKey(hex!("a4a0406f66fa65191b7f39a9a8b65eb4c5d7e69d38cb0eed133ffba8ad28f88b0ecbcac26ddc7d2f195a92a52dff7922").into()),
                PublicKey(hex!("90b8d4e158a196d6e241e32708f7566ab8768624568a4a6d8cbd72fff62f6e6f1b74328e3c750bb36619ddbea99fb387").into()),
                PublicKey(hex!("acf122225d465dab6950f8825af6f8dce33de7e32697647eb9cecccd8d21950f9f4d9b9135467934f72c7af73cd9f363").into()),
                PublicKey(hex!("b64bf852f5aa66a05b89462b32df1ccd1fbe73643ba16312188aca9e5abed4ff6d34dcaee321b055e7662f40179713f4").into()),
                PublicKey(hex!("a40c7034e9353fb37ca959d448d3bf8b9bfd3c6596162ab8177c3195d34956f49036c4d0da64733a0a2bd02d5007f2ce").into()),
                PublicKey(hex!("b0017bbb8a3a4a6cefbb0901813ee337f640f3e3f9b4854c26563c2a12b7615871b416dc2c3e7efca60a54e2fbc70235").into()),
                PublicKey(hex!("8d77221eb17420d0a0cb639bb42c825ac2a0193c35c5892aa57c2e7e2f38908c1124b92d663f03184c8f355754760457").into()),
                PublicKey(hex!("ad24c2083b7bdc9e9b710f3221a19c44b9c551aa567afa1ee5add309d9bfc49b92048d562cc7de4b738ac08905baea8d").into()),
                PublicKey(hex!("b210e55564ff38a22d5936ad44f4e59eb2d5d6c868b7b4bc96a3d9746dbaff893b76fb3ce1456846981fe9809832be41").into()),
                PublicKey(hex!("8510f3ed18be21221626589428fb0602f45d82dd84704ff477923d534a7c263156c246ba75d159af1b274c81001f0e94").into()),
                PublicKey(hex!("a12eccc044d694f734735cfdd641442b4e302f0ae329325693a31ca689c9a509bd6508709dc3fe43840eea0b59b46c3d").into()),
                PublicKey(hex!("847e936c65b9a7913738f4e242e1ee67eb70c32778bf0affa69963a43d6f44662ffbf7f45c519d9323fcc28dedabfc85").into()),
                PublicKey(hex!("b681bd4ee982e4e6de3cf61061a9410fdaf2ed32d55a9877e4b7cddb6c106549b11e6a4add0e32dc2f862e64a402bec5").into()),
                PublicKey(hex!("982fd790c06ad2df3881153d98175720b7f0a5ae14a56cc881accea729983aa17a003ddc5f8f0305650f881541a64fc3").into()),
                PublicKey(hex!("b6a95dda44c83f55a1bc6d04654df15e4bd7dc322724667a7a9901c40c73cc9979f769bd0f5d16928b3fdf5163447457").into()),
                PublicKey(hex!("80962c0d0fe92e8562e1a937168c08a4229dcd37727c3f65c2ebdf464c5724c420eb354e99858e6e9af6032f85132e86").into()),
                PublicKey(hex!("85f3547cd349e858f729f71cc5a7a3038c26803fe42f3d83735c84dc381edb3d265625c2eca13188fbfcf0babfa7a2b6").into()),
                PublicKey(hex!("a1df6fca43e0ea1ba7dd92eb4ccbf0709ea7e13018fc135dde919ed771730259964c5b1a04df001fac3e7a9b9fae7d10").into()),
                PublicKey(hex!("98cd4119a1bc386c1302d3ba5989a98c3b9121a87dcbd998c92ef831ebef15eee4135bd3e45e3552b3bf4930b1760318").into()),
                PublicKey(hex!("a59477b570509cfe84a426be18ca4b6c589ba69efed7c7d346b6387752e60981c041f5a6d0426e15728674cb3846fba4").into()),
                PublicKey(hex!("a65c17a8a2d03e407b25570b47ce1370950b1c2c01ad7f3ef0f05de45a7958722e36fef819765d58af1dac166a9ce381").into()),
                PublicKey(hex!("b4d7cca6b909e191c35ecfffadd121f54f724ff5964542fe615918f3c3aab9c26b02ef2b37a5925ad8926cd8557283c5").into()),
                PublicKey(hex!("87abd14825a5386e62d828b41e8ea17b9d077cc33ffe05d6d665ddad0b91da89ec5d38f86b4510c4d391f232832dfa9e").into()),
                PublicKey(hex!("a8d6489dfac2ae52cd9e4b3582a28f89ead033a46db5192bb30d9d73570cd561756949d9692e0540709b656866f40c35").into()),
                PublicKey(hex!("9011df07fc069c836c85546867d04ea59773a8b27979fafdd5b26982925b488592086b79d8882ac342ba6cd1a7748563").into()),
                PublicKey(hex!("81be7c12faf5428f9a0e3c3655362919acb793740c35d7826d7c1de3c60117886c2af3f304d8e2a1a65a9f911986feb8").into()),
                PublicKey(hex!("b1ebe689b68dc83186664b916169748eddbb37b65efb612e671b09ae095b98ac767f6d2b989c45028b53f513ad07349e").into()),
                PublicKey(hex!("a89315a4babb78b8b063985e32396f0e252ff328c033a0ba3e79df26a6670d35fe71f34239db08ef0f219ee05da0acb2").into()),
                PublicKey(hex!("a51cbcd816a9dd9f02f87b561d9efe3f85be59404da64e191ff463dd84167d38464a976d67806f3496a01f895a540fcc").into()),
                PublicKey(hex!("b9354603aeae3d91c0431841159398d6fdca9c538e089834d3d5180d046adc60a12c7760d983c1a2589461b8249b809f").into()),
                PublicKey(hex!("82f052a5256f1ccd87c53cb9c9e712b47852b29eaab3ccd709ec8dc8dd82e225d766eedcf33de2514c3813a6df07ec25").into()),
                PublicKey(hex!("85ae22ebe0230d08e55a2c1c7cdd9d5e98b766c358d1804464a62ee1109c4be9e9b97a2e877b5f005f56c4aa97106211").into()),
                PublicKey(hex!("8fa687942cb61b03b6d612037d498fbd1b7cfc5815343095c8c01d758c474f676202e3a1d2de1001762464bd08d38439").into()),
                PublicKey(hex!("a26af4cf69f65432324c3ee448491d53a9c7de06e9bc75ffb265a08d383e0371672a2ee6b508e0242258c9b344a79192").into()),
                PublicKey(hex!("955e3b6401c549ed0cea69e59c8b9cfb674f3c6c42220b2cf6fb038dbb7de61ed60eee615e7b926b862fcea8cff68ad3").into()),
                PublicKey(hex!("b03702deb865dd709a0b3a8b571658e5ccdb3e2c06625b367931e5e31e99190fc96916ad6d42f9b0f7a8e1970347f81a").into()),
                PublicKey(hex!("a84598eab73fc9696560ada53f3c7399f16fb2a2ac13ff972ae935a29f1b4722bb4eb808bb991e5fe2479427d9108a96").into()),
                PublicKey(hex!("a21d7baa547717c5aeee022060c99e778b36558cd07c13839c1026bf86adb1adeabf3fdfb6fce69397bc6a5a78bf50be").into()),
                PublicKey(hex!("a5348e45fff1a4795c5ae5e230d059516837c05cf50bd2a558a2cb2ae41f5a97578e302edf7c68e809eebae03db751aa").into()),
                PublicKey(hex!("830eb7ab3f1fa4453c9ecbace2f47dd16161699460f306b82fc4ee6ab9938e5f95bd2fa971a5a5b984e4785c935c0895").into()),
                PublicKey(hex!("97dc0ed12670be973ccff04c26fb80ab6065ddf5c105585012dde477caced2cdf39993d2717a81366574ccf590f0d2cb").into()),
                PublicKey(hex!("918472ee2ff1820c86a568ad6e1a8a87ad5646ad9a34a4ded4552a507d8d2e87a79b737fb9e1aeb50d05e74683ec41d6").into()),
                PublicKey(hex!("8c9489532a0677834a3c31f84923fe8ae9aa824abe6659712d23994dc34721713b1747e9d60e2ac34a99e9ae9fd177fa").into()),
                PublicKey(hex!("99c1213fbc9cd4582a598d968334ee41f8457ba234bae5cdaa73e84b7fa1b13d8e5bc6b7d11801efbeaae14e50fbce1d").into()),
                PublicKey(hex!("a83b4b42b897255803170f12803c970de0c00eef15b7df11c477217bf61f18790f692b29c9cf19d14ee008f8b8434ca2").into()),
                PublicKey(hex!("afe2644508739bf1fe66a01e32885c7d846857b038125efe181e52d9f6d7fe6675a3a044baa4d26ee6491022009b699f").into()),
                PublicKey(hex!("8a6e6dd5b227b28e2aef3248de9bf52b1683a7a41aacba6cdff516448beadcb537400b4b182da8af68ac168090eb501e").into()),
                PublicKey(hex!("98b2f17ac762f2178d442b9aaea445fe918c1fa1f238bc7ea6299a1acd2e1d2a42519aec6ff59b769201114f1ab38882").into()),
                PublicKey(hex!("b764d2ea37087a0a4b5b954d93d313490adc74ffbe1dc101f5e2d740df6556ada682b2976b06c34f9ada2163d0542181").into()),
                PublicKey(hex!("9032e6d9914ccf3a0e1446d98b775085eda7c5e25492cc25e6412669f6b920ebd708062695872bbb770237cd5067ea3f").into()),
                PublicKey(hex!("92fec011283943709d1ca33a2bc7710daf703bcaac88d1a00f3c6bfcee17d7226a371417888ea7a6a6de411d51bcb295").into()),
                PublicKey(hex!("9149af2d1ba7842219521ff93563cad1065246dc39216075db7f8a6e55b94b87a4267cd451ff0dce022f136494548862").into()),
                PublicKey(hex!("8ae2db2cb57eb3d22b5fb1f77cd7c7546fc1d18b38e39f14f982db34d7e38f471dc7ae71de200f51545c02b161113ad7").into()),
                PublicKey(hex!("91d2a168e21387378b4bb9f560ab89627e2f0919afc7af7497f9d668d26e6256566628d5010c4755c4218af9348588ce").into()),
                PublicKey(hex!("8be671e644cd465cb3aec3c7879e94ccc44dc2c9a8a387b2a412eb50c192572d6496b4e75195485f3518c81277265de2").into()),
                PublicKey(hex!("b7b38f22a9e0eeef6566fb4fd7d22610251d84553a2d309e09106e390d80bc626d90e1c309af483063ce6633a9354894").into()),
                PublicKey(hex!("a06a8c69a735a8f4267bc8fecd667cd9fe4b8f8d5397128b205bef461f3a15b365ed617e3ed369017fc1ca87d8fde865").into()),
                PublicKey(hex!("843a3cdc996f50e3ac9d55239334905da3bc57100d914965d1bed48b0661a286459b03bfdf057f106afd2142afd75e35").into()),
                PublicKey(hex!("8443ffb42256c6e785455e98fb2f87b19208a0b1f4a9ba47f45f613708ead3e3b0dcf095b9d402a90fe84d2123542552").into()),
                PublicKey(hex!("a2f4d6e2547bba9dd7d95af7449683ffcd8c3b7a092f3c985a36d71ec4ef87e07029c5feaa56fe4db8204b8dc90f175e").into()),
                PublicKey(hex!("b3180a0c9b3d6599e6eb5fc7e2ef5096e6c6ff6bd266f6b66f79d2d66449e66fca7239f15b64111be0c47109e38d884c").into()),
                PublicKey(hex!("887e42853748bacfabd0b613fb357f1dd01c30f19f3ef929b2be77073f1dfe42ab7517e73490072496afe53e503a844f").into()),
                PublicKey(hex!("b9b7a4b830f02ece15cbee2d14111e465d06e9795dc706dbabab7a35e0e331666fe1b8c0ed445eb2d560b4ebeda1bc52").into()),
                PublicKey(hex!("8bff2d075d9927ec8766f015e29b2bd1d90a8d51db71af699f566649ce4c58d3e679b1ac140c9501d69e91b4abbe48ee").into()),
                PublicKey(hex!("870e22070271b70cea7e955ff89dc4678affb3b735f4d78977e56b1dfc105afff690a73e468f1eb118141498a538207f").into()),
                PublicKey(hex!("8c5b4e77e6f7b24470273080f80b8c373e2840d980734acee77254981ebe42b3e07d970020ccda6543462658857c7804").into()),
                PublicKey(hex!("a639969c441a155fd02769aace85d30c25db9e9f4a9c5f7b55fe1564ab0e292a322755a0a3b3279bc8e0feb550781a74").into()),
                PublicKey(hex!("ab7179e1f8435b5d0359a43c29c7638b27ad6b010655b0b90f6ac15cf95d1c76de29197a98988e81d900e8fb665d2307").into()),
                PublicKey(hex!("b80b672258f696318c67eab76320eb28eb7e8e0f878546574b73eb7c54e2d413b57261ce0d243a6f8375ffafe65e5c4d").into()),
                PublicKey(hex!("919de8218c22459d6397e745a131c60dff76e4bd32037faf03b42a161a4fb78a42189d3048bc0ec22b343a2e73f2a756").into()),
                PublicKey(hex!("8712630d61c416e25dfba9d156f9a9645b096e1184aa441da9dcaff9b1d1365db29af32af29bbbfaf5bffb8efc0cca19").into()),
                PublicKey(hex!("b46839459663ea79798412724bb48c08b0fcb389acb516f5fabc936150e99b15a5a9b76f55e734a2ae907ec546403f1c").into()),
                PublicKey(hex!("84442f5211a80128212ba63c5205f792fbf577fa203802368b11ce5930ae522ada4bd4e06b90fc5ed016733cded4f007").into()),
                PublicKey(hex!("b0df2d4b97ca12fde583b775b9c83b21d519213d9e434c6be6a538a73e0b3acb5a8e8f5dcc73dbfb6f3778587d3059c1").into()),
                PublicKey(hex!("8264cbfdbfc68155dda8aad92551b40182a97443b3367d3f4ee221a6e7b09bc137bf48490e5e12c93cf28d2fb0ae0ab8").into()),
                PublicKey(hex!("84fcd146a49c435d3bba88566d3b7f22465da0c7ae0b6db68393c1dc8d55288f1840d7e73cf0805188655e56ac54b098").into()),
                PublicKey(hex!("879d45cba5a2a9ecb13ae169046d8a5f098bb1ac8f6ce7f47c86d55dfb7a23ebfe1c1ac3caf696d8132d94f08e937301").into()),
                PublicKey(hex!("83d5801dc5c7f2ea0a9f2b54fbe32226cbbaf29d81ef772035e7c4da31fb335e6bf362ccab5cf767e94e8023e346b05e").into()),
                PublicKey(hex!("8aff184d6819966173ee8c8e1eda470314964bdda15b7ce39c0e842b00d26c02720951572c4be4fa64f0c3f0a22a7c43").into()),
                PublicKey(hex!("921912c87af0372deaef407508f04c89abd5f3731b13ebae4ed67f1c658091034bee7c5bbd11b9f86fe4a1a9a25fd934").into()),
                PublicKey(hex!("b005ad72aa4e67c77a9f5bf2e28d8471d0e41057cb0c18d08a901fc0e3f354a12f3afa3407085cb057ffaf2da46ff62a").into()),
                PublicKey(hex!("8206c6c534d1a7521905c3a9ba27299d8db52fef630e073faf8c539f94ace13fc1dfdd4227f44e9f35e0fc001fdde531").into()),
                PublicKey(hex!("a0841230a16bedfb8c9a93e0bc12653073d88c9ff963e332d160ba01d5007ca4466ee331541b2cc22b0333b7e789269a").into()),
                PublicKey(hex!("b96cb118b5c23e722be511d8fb55e88dbb3aedc278877272a13b5d8944a85ae6c2588a4065a91de36774cffca7bdd33a").into()),
                PublicKey(hex!("a4ba9cca0afd1748da00b2c11443928c8bbdea10c1c03c37ca4d515569f874a5e77e061c4e739ab18284884eb74e5c29").into()),
                PublicKey(hex!("942050fe5373d04db61c45a7037fc85a43235903e4b6755a214320be762393decbbd6292e2e88dcd52ffdc644c23bf12").into()),
                PublicKey(hex!("82adbd305f1b184f690092586f21d38b5fa9ebed545e8caafd65d32113942790c992a17b0cfd28177fa97695984d9d6e").into()),
                PublicKey(hex!("b2df004152dd34a3d1f1f038b9f25921b2aafa6d8efd1e5d17aa6900e9350d678020e7d267d9a7bb64f01d0e2a4cefc0").into()),
                PublicKey(hex!("a9efadca6c05bd907f38cdffb1fb2df0526922413e8dd4a0331e5dd89912770086214773d752fbabeff13feecd1e93b1").into()),
                PublicKey(hex!("8bc3581b27c29348a3432539b0102dbfa1baadffdd16da42697e9c473350b446e267ea2a0514ea505b542842580e75f3").into()),
                PublicKey(hex!("b9d5fafe3fa50cffaac523aa8b003acca3d0cb06378ba94e115d74fb7ff85c250dced257f5087b7af26f07329bd7341e").into()),
                PublicKey(hex!("815a45e021e78ea67e3a0b28cfaa87bf5f2d8f3bb3217e1071af7845e13c40664e7817faefb35f60a32a145785f002ab").into()),
                PublicKey(hex!("b8b6a619775fbe6a93d4f4cf1e36560489ea2971f4b38205600bbadc4a249ce051e74dfef8ab59c70da2a055ccb35520").into()),
                PublicKey(hex!("b5cb7cb34f8bc544d148325d503f1a380ac5f630d9fda20dc56a07e08a9f17bd472e4f8efcb2b270ca2e900fe6fa32a1").into()),
                PublicKey(hex!("8e58d93b5506802b81c57a8ad76854f109bddbbacf622c86fd9beec3aa1b473e00b86419e2c23d2c3b95fbde11f8ecbb").into()),
                PublicKey(hex!("8389daeb9551245ad80a63cd21c0fbbc2e7489a85a1d51963eb7d5549cf7245661160a4f822403ffc93d862291f12b36").into()),
                PublicKey(hex!("8b90dd5b39074bb40d701e23fd436dadc968ff1cddea1c9953ab6eaee3279b3c1e7950bbc092e9076c308e4d856b2060").into()),
                PublicKey(hex!("99e513efe10d35c89ed08ce273a03f4caeb2905b5819066302506401f9abc9106179f40c2052a26d14bf0b1841fba954").into()),
                PublicKey(hex!("b8cb3d6225931a80b9bfed389bdcadbab7813d983734e6fcd4a5e6407a5c87a4ce9b9d51cad8c7bb2c4f830d9a750546").into()),
                PublicKey(hex!("a7b67a6d166c9118804ae46365f35a17651602696ee4bdd9d8176844af2f39601061214e22283e87649d66bf4afce75d").into()),
                PublicKey(hex!("b4b728b71bc24c1b2ad6e15ce74e31b417d378ccc9b823004c94d5c9639a49be42f829e82ce32bea25731d0d788b14ed").into()),
                PublicKey(hex!("86091e5783f18b0a90b78f4dcbb578682f4929204bb42bd03afab035ce4572a57002faf4a2c827f0ff6b6647fc6edc26").into()),
                PublicKey(hex!("88ad4ea440bc8c369cfecd781816e367c397446c512792908a0b8ad399fadd3230c921fdf034e93688f39e9274fc0cc2").into()),
                PublicKey(hex!("b23e34136c22ac73157c7c5cc8a9491b0b5bc968c95a9c104b402cad9de598e323ada4cc527555157cbecadf48faf87e").into()),
                PublicKey(hex!("b432fa57da121988456589b194942d6bb35d833b8ee3bdda162f2868c947d674fb9db9032720b155c316a05c46bc1e07").into()),
                PublicKey(hex!("a50a24c8a35638e0ab8b2df4413d1bf09bd2b4b88d3e434f40c0d5ef1d9f26f745f338b8c30bd0839b18b5b0fd0d8995").into()),
                PublicKey(hex!("b712daa6c256bcd0502fefc067e812bb7a3a33436296322783b2e55fc7cef83d79da8fec94538585308b3f45e9517971").into()),
                PublicKey(hex!("8a6b5472a2d35e1839f2b5d1a2e0a034cee7f1f4deb23a759edfa4cf6d70a7a084277ac6d63d5e13d27709515dd69383").into()),
                PublicKey(hex!("94557dd803d8715283924f060124c227803d4a8039ee2d4ae3ad2864d236ef7162ee207a59df81733d61a85c5a5796e9").into()),
                PublicKey(hex!("9153e072642acded7b348908018200c9bb510813d479983493eb92bf842d898ba4d282c8c01cfb9beec523b998a24554").into()),
                PublicKey(hex!("836ff22023a680bce766385c151f110ff07a8a8d3b608199853e90cc6e02853038a6f3228fe5d8e7794493b7692ac2b3").into()),
                PublicKey(hex!("9918e8d6d9a68734a8dce7b7569a0e0e128f3741198b7919ba356f090ab3246583bedd9bc5a5daeb7364258063e333b2").into()),
                PublicKey(hex!("8227131a6a953cbd69f418214f3b70655262e2c4dc15f17a0054c50a23894d2ba888e5b2a5151977349832827559b119").into()),
                PublicKey(hex!("851ed7dbb58389fed7889cbc2ea9cc296bddbff9cc20931ae53fd583c5070bee1adfb75be00f754f43fd19920088924e").into()),
                PublicKey(hex!("a193cf0c37069d2789aeec992f9727caba73172317db41038304fbd2103486f045102a2bf89594a0b27695cdfc0239c0").into()),
                PublicKey(hex!("96a18291783e21abca50af3368521901fa1dae91dc469938a8b950ebf73a906c86d626f62540edcf003770e55cee7f23").into()),
                PublicKey(hex!("8213d36a0d1b8e41e65f887ad10cb56bdaeae0e2e6f63d647f26ba22fa2d129e856e8c8ade797d0da36b6f494badb2b3").into()),
                PublicKey(hex!("872752cc8fd693a4efd432b511bbf1d4ec3917bd10b5ef13a5ec4743e31edf8302ef652e0bb365102655de8ede676429").into()),
                PublicKey(hex!("94e48175e8af2ef5409ae2a2dea844ac465d5f13539d313df9b9115cb10f5087b8686fbf341ba094c5d3205d8d23ed0f").into()),
                PublicKey(hex!("b0f56bb7221e4724a992da633527bb0d866159300d6ccc40833be5f3fa19d439203914820338d2393788fefeff7c2b9c").into()),
                PublicKey(hex!("a29b03a1daca7bf85a26567adcc358e71b2605556acfc8527d96dd38517553b026aee2dd3b36df6465ea0b2b11731f63").into()),
                PublicKey(hex!("98787249d66ee1ba6c119bd0809362c8add3e1b13586ab42618ee9edb9947b4bc19c9aed99da317be8cadd9c4b99c7e6").into()),
                PublicKey(hex!("9540fdb105fee54fe5ad42ecd44c70781e7013ab765512aa526558af02a64387564b9f62ef7e13350c116bf4f7749c30").into()),
                PublicKey(hex!("af5668865f948793fb7b8b3b29ac4663cfeaaceee225eb1e6b77f571827998528d2e00de7aee92a7afad5c9a8d879ed7").into()),
                PublicKey(hex!("b5091c0b3bbd449e29809041546f30cb5c07c0b4792022c27662cb062f5fc5d0d15f9810eeb2ecac6d67fa34c0b3effd").into()),
                PublicKey(hex!("aea3ac2ed67100372fce74dd48e167065cc971bf5f5433e1c5fe817e01245e3086f79b0b9eada7202e1510c618a3bcd2").into()),
                PublicKey(hex!("b5ab0f7cb58f514d38d174c7268275f9aa49463f448af7d72410e345ac7d54a8f063967f2a6b23aeae877a6888e7a8b3").into()),
                PublicKey(hex!("a398812957bd3a9ddb4641b581c716fd251023153eee977d2ff0b00b6428992cd603a3d5bd124b0e6bb1292efefbf9b6").into()),
                PublicKey(hex!("ace13b3b733220889f1524870bd782811329ca5d67a6ef18fdca6b183869c233cee572284fe2de9f5dbab358faff58d1").into()),
                PublicKey(hex!("82aaa82045b612c41a8b8b8a29daad9cbfc38295321cafa5199c337c049dd715a32971054f248e06bb058059cf974aec").into()),
                PublicKey(hex!("8990cc66ce015f5340207679d38710dab1b652fdfc395578ffd769af4062fd69daf07e0fd100a2049ce33a506aa11a20").into()),
                PublicKey(hex!("b118f061efac86a62458f89384601614edef72758afc7f3950889e0e2ad368c14f3d4b48cf82969e7115cf6412b1a9cc").into()),
                PublicKey(hex!("a94113300058402c7f88d4b9f1e7a5dbaa6da94097c7b4d7a004a5768f78a48369d2debaf6345462f5a6f2a62a46c39e").into()),
                PublicKey(hex!("82d83500656235dc6563282d4490052d27c4aa9d0bea1484a9d53c4bb458e114d130b32d48e2f3436b4a965526190192").into()),
                PublicKey(hex!("8558a8cefacda93828ca2818a33887e6a45e012b563460bb5911acbe76a2fe2da07e4a7aaf006af1ca8a1a54b4e853d8").into()),
                PublicKey(hex!("8dd0270cf8ecba8aabfc2e8382f14d55b4c924350c737fa3dec946ec26eb8575074e35e488ec57ffa6d3ce770d25d00e").into()),
                PublicKey(hex!("9619502f2daeca77708e1c72c808d8a555705652a5f5795597cf5693207484d93d7b9056b26182ada8fa5c9be0f72128").into()),
                PublicKey(hex!("a107fb2bbfbad18e0a72c3895ed91c2e1a23d3eccb70cff0a88e525a28b4ef1419d417af944ca0bc23ffb8502b2ccd38").into()),
                PublicKey(hex!("93d713cc0e2abdfa91419d9f5585495f75c0e30c1b1cf7c91f9e22187209fb2660e5bc0f54c10e4d51eee601fdb64736").into()),
                PublicKey(hex!("a3ea18c2f0a91b0d8db7d19d6c2ce7edebb3702d6fb8ff32d27589e28e827a351dbf9f441de799e70f1df38f2eaaf788").into()),
                PublicKey(hex!("8ac63de924f34b7fcfa347c67db6e0269f6ddd992b98d7b2c2621738fb8b2ba03bd10b725931ec2d2085726d30b64f7d").into()),
                PublicKey(hex!("987590376166ee47d232d1523cafa6500cec368beb87ac1170cb3596b878ea45f7ea04804d2a0626012cbbeedef17021").into()),
                PublicKey(hex!("8e541b1bf293006fef345767e28b04435942df8d49b4263b5d200d82aa5dd25afbcdd08a530ef17430e8935d33952ce9").into()),
                PublicKey(hex!("97b94b98b17eae56835fdd0aa4f63fb798199999eaf047258a50efe05cde995e08a249bd06f7993d7f46c6951cced70c").into()),
                PublicKey(hex!("863cf5feee5fee99af6ca3eab958dfac41606347cf50e7325cc89b6807d662a6e1fa423aa9d8005e298c7ededc10344b").into()),
                PublicKey(hex!("8028d0b741a7a4c4665396f95df6cb4dd3d8988e875274ecfe145bb8d2d5181f5d5ef0923ca2c2971834540061a66cb6").into())
            ].try_into().expect("too many pubkeys"),
            aggregate_pubkey: PublicKey(hex!("a7c80ef68fb8e49be7669d4b1b8923026de74dce88b306bceec9aee44e5bc1d9e5bd0308c63b4e0bfb5606a2d39a4417").into())
        },   
	    next_sync_committee_branch: vec![
            hex!("2572bb0751acd1f30fcef025a9fa561a9fa28bad8cd99f197ee260c6bc57a99a").into(),
            hex!("67cf535bdc97f271ee183d82698fe8b7b6f84e8746a35a6a65e0311bfe0aa8a8").into(),
            hex!("c9eb07afb0ae71ca7a0747dd6da6c22f84290e16a86eb2efc6753125171f167d").into(),
            hex!("3c4b67a809617b0f2f9d3640db723dc36a967f8a11ae99ed86241f4b79a84879").into(),
            hex!("f9c348f25fb4d9ffac9f84a31aa95d94556cba966db2afbe8b7ba478c554778c").into()
        ].try_into().expect("too many branch proof items"),
	    finalized_header: BeaconHeader{
            slot: 3976544,
            proposer_index: 198137,
            parent_root:  hex!("a753318963779bfe8bf25228087ba8e2d4a200ce2c3741e4204d0104806e1a8e").into(),
            state_root:  hex!("7e1521100cfd3d3593c1665a82e2c3e9950e629e15d765c23346f85ec34bc381").into(),
            body_root:  hex!("2ccfdd16e69cf5ac9bb8cbd85bbe0c91fcf666c448bbde3aacf14f54e36d7933").into()
        },
	    finality_branch: vec![
            hex!("6be5010000000000000000000000000000000000000000000000000000000000").into(),
            hex!("04743ed6b30f3ad14dfca1198c41e3ca1610625a8c677996213efa1591b33f67").into(),
            hex!("6d218eaefac861f1c843a8f04c790349b85a4fe2f3a059d669a264b253c6d962").into(),
            hex!("86f367f14f4679915f9400220e65ea3eabf1d93c5f61168a7a639fcd24fb48aa").into(),
            hex!("000811e772c5ed0b5509c90d008655f8559b002fc7cfd596f7dc37de90bbc007").into(),
            hex!("2d3559435fea4ba68f948eb27a79b6035127c18114085c145467cf50d0584e55").into()
        ].try_into().expect("too many branch proof items"),
	    sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("97efefdefff4f7f7edc64f7cbeff7bf37dbdd5effbb5effbffffbeeffdf3bfcf3d4efbffffbff7bfebdff7f7fffbfffe77dfd6deffffdff53dfdefff7fdbd7bb").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("903a134b0996e272e7287807d8753a743fa65087743677ab015d1ac5bc91b161ed38071a6db90266a1c4a3c6a0291a010896e139ea98d4b50e3aef479301b413dbe612fa14bcfee885cbd7f6d7aa2c38297f9a1de3777f185260c7a4363bc818").to_vec().try_into().expect("signature too long"),
        },
        fork_version: hex!("02001020").into(),
	    sync_committee_period: 485,

	}
}

pub fn finalized_header_update<SignatureSize: Get<u32> , ProofSize: Get<u32>, SyncCommitteeSize: Get<u32>>() -> FinalizedHeaderUpdate<SignatureSize, ProofSize, SyncCommitteeSize> {
    if config::IS_MINIMAL {
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
	        fork_version: hex!("02000001").into(),
        };
    }
    return FinalizedHeaderUpdate{
        attested_header: BeaconHeader{
            slot: 3979991,
            proposer_index: 204207,
            parent_root: hex!("a88eab05c0fbb43f56ec799df38e88dd71b27798e1d86aab6f7599f7b76245ef").into(),
            state_root: hex!("f70e5fd39953cc0b87b9074b1f0f8d34b64ca5d97faa7b7414a93d55df200e80").into(),
            body_root: hex!("07792efdf97d584fe34117c4a454ed59eb69da3768f5ab7ad3c217c51bac2ca4").into(),
        },
        finalized_header: BeaconHeader{
            slot: 3979904,
            proposer_index: 205890,
            parent_root: hex!("129f2cfed1924a35c783b61d21b7b4a146eb8f35a8b9da88cca249dc38167cd7").into(),
            state_root: hex!("24768d4ff1fb8008c4495de4bf53df99ca8deed5bb3a761512f9fadfede3d3ce").into(),
            body_root: hex!("38954c4a68796c2849915cc0862181948f876d93ddb868861d1e8e7b6f084c3e").into(),
        },
        finality_branch: vec![
            hex!("d4e5010000000000000000000000000000000000000000000000000000000000").into(),
            hex!("0a66ddd4ca99076bb50096fc5a09089f189e69d100344fccae5de5193ef4c24e").into(),
            hex!("6d218eaefac861f1c843a8f04c790349b85a4fe2f3a059d669a264b253c6d962").into(),
            hex!("636e4c75e078d58ee2296f1b7c16bed70d77c9040c9bf9f2b83b6d54daafc201").into(),
            hex!("81a72c6be5b372e8be7c39d92385b98883658779da241caa0e82d7983b82bc48").into(),
            hex!("17ad7968da278460c568906850417532ce09b85035546776ca613480aef812bd").into(),
        ].try_into().expect("too many branch proof items"),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("97efefd6fff4f7f7edc64f6cbeff7bf37db9d5effbb5effbffffbeeffdf3bfcf3d4efffdffbff7bfebdff7f5fffbfffe77dfd6deffffdff53dddefff7fdbd7bb").to_vec().try_into().expect("too many sync committee bits"),
            sync_committee_signature: hex!("8a9ab21878ae1f3728a41ec46976c3ee88d55c60f1a3f35dc553c6a9d61f20baafa3a1b6b9f08dabe311e5f37ba6af6900946d9e56806af0fe34d50f35abc9d407dba3e68b590c5e1c1ba09c41a2401742adb7ee877f64c2688f1f8d1dd73f54").to_vec().try_into().expect("signature too long"),
        },
        fork_version: hex!("02001020").into(),
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
	SyncCommitteeSize: Get<u32>>() -> BlockUpdate<FeeRecipientSize, 
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
	SyncCommitteeSize> {
    if config::IS_MINIMAL {
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
            block_body_root: hex!("332cbf177a081616822905703c4bf026dad64b6d726a59f5b46ecf1661f81808").into(),
            sync_aggregate: SyncAggregate{
                sync_committee_bits: hex!("ffffffff").to_vec().to_vec().try_into().expect("too many bits"),
                sync_committee_signature: hex!("adc869227de9fb08b67333c8bd012dc73fe4ad4ed5f3ff3db981f2b9595191ecde20ea47c7495d3ce1ac6510bc97de9a0095b0086fb17698210497935e6a8fa2e17cd70e6bb3daf6c8e06674e124c5447d6d841a17d46316b8ca7ffb0d731f27").to_vec().try_into().expect("signature too long"),
            },
            fork_version: hex!("02000001").into(),
        };
    }
    return BlockUpdate{
        block: BeaconBlock{
            slot: 3980063,
            proposer_index: 259057,
            parent_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
            state_root: hex!("a17da5a19ca5c9a6e93ad81c78028b18e8adfd12a0d92b53c9139f10e67fc4e7").into(),
            body: Body{
                randao_reveal: hex!("afd062030d581b05ddd13993159d7d61beef7d82baad38e26f20ff2ad7be924aa6a78ea0a44b65a2277a52e64f263adc11bec23a61bbb00504e17848f7ecc84962d4107332742be87ff1be4bd6ac31864b1a75ad6b041d9b93cf7ec50973b0ff").to_vec().try_into().expect("randao reveal too long"),
                eth1_data: Eth1Data{
                    deposit_root: hex!("b583f5b2d39299600ae9fbd396907e29729c3808dca6f92e3f467b8a3197c0d3").into(),
                    deposit_count: 182562,
                    block_hash: hex!("f308e3c8ce44bf1659a560bd8d2b9cd611718f2401df27a18f97c809ad3c3e48").into(),
                },
                graffiti: hex!("4c6f6465737461722d76312e312e302f32393630326261000000000000000000").into(),
                proposer_slashings: vec![
                ].try_into().expect("too many proposer slashings"),
                attester_slashings: vec![
                ].try_into().expect("too many attester slashings"),
                attestations: vec![
                    Attestation{
                        aggregation_bits: hex!("ffffffddfeffdff4fbf7cfb6f49d4ffbcefb7fefdbf7fc0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 40,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("958d59fb830ce74d96912ab4982f57dfe255a770d8bcb6effa2ecec847284c5a6b550bea4432e7b1886ac0ecccaa4b5b05bf6aeebdf112d0988be1a969d11a3ef19357d63265176f71de110a7c0e8bf9c15fa302965ab7fc963fe7a1950f5453").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5ef7ffeb7b1fefefd7dfff7bf6fdeffdfffc7cbbad6b3f0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 43,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9265261ac4c3fc38455a335d26da0fff841481dcc7ff6f45f7cdd56096d7e6ab3c95e3ac7262554517b76a676067b4ea14dd60af6acbd11414db7b5bffa63876a8517deeb1db8feee24c88e9dab9c27daa50100cfa5a4a98b3aecff068df7af8").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("e75efb7fcfe77adeeeddfdff3f5effff36ffdffff9ffcc07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 21,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8636aaeeb40f7798b86470f32b868f2296192da27fcf42889c3aa6e2f292956c4b03f7bcbf476471d273673fe9c2bf0900028509f659fa8a46f780e8998efb8eb97e8372eecde76be05179549504372b25d9a79493b570c561508298a8358f67").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("b7fdb2fff4fbdbfecffffbf6f6f7ffbdefeb7e7fcef5de07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 4,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8b7a011cbc59bf89ea1e66de191f17df6e8ff946231384290d0ff445a79f612e2b6d2bbc99d92cec8f73335c2ebab20504f5fe8ca1c05229f1578367f0058ff7125ff41718c7b6b198092f63eac714956ca98eae08b41f71895e3cbf9044555e").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bff4f7a5d2dba5ff5efb6dfeffbffb2d7fffb7d7cfffff0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 18,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ab5bc9f3e070209bc29883bd24381a27cc78810ff0c7dd13f9ba93b47d60aab3d6e649c71acf5031a807c6ed32cacb0a1038e5dd047327c84e808497dbe89a68931ffb5b3fa67e97a245c0190daa443c549a767327b5631015dc462609e4321a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("9fb0ffffdfedfb7ab7dafdbffafbafdcdf3ef7ff7fe65f0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 48,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("962de1a5795d73fe84dc45097deb390c2fe7c39dc42b7d2b6556280f01f30fa406e9b12c2c698b923cafe7bde7d1eff10c24328ce4daa9fb38d1378a2d7f80df3008e3bb6bb6d63b149a463130d912f6b65711b28b062a91de7c0a7a9ef80df5").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ff7fff7f5f1fcfcbf7f3ca57bfe7f3ffefd7bdf9eeff3f0a").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 17,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b6d8c90860aeda921b2cd7b3102477e6230dd2c2189f1428e754f7358b865d8fe40ee6707b2abc9ca1bbb2d10da89e080ef7f310e928eb2d06604029e8d2cc6cbf931f3163d1676f6179453fd4e1628584b4f503c604566ab42cd8c74323f320").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fef55dcf9f70ff9ad9ffeffdb37dd7efeffdf5efdfef7f0d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 8,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("85e9b4fcf299676762579b7abfc6369bfd9d4e31989a83574a248e766c5d106b1d4461ccdd3ec186a8dcedb127c74a8f138d8d3b226e18dd2959f8c6292ec0f1484e5d02818b9edfa30181e9595bfe92e02825226106bf5011346ed3eb9f6255").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bdeaff7f7ff6dffcffaffe7e36a7b797fcefd7f77dfbfe05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 44,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b3304954cb563f0f633b9015e50da2ceea78a48e909a3b9ce9f1dc1e498c05ec8cec0ed32137efc23290d60469d45d2919ab934633dc880e174b928687e8afe8b6fd4393a8df88f08eb1cefe5227a4f30740806f3589c3d0034c7b6ae1d72c66").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fdb1fff7feff5fbbffbf9c3feeefedbf8f7fbc9bff95f60d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 27,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a5bef69ed3dc91b3ce83499030c29ad690882bc2fc8a4ac0d74c81d9e3ad26f0d4a702f8a5c54cbdf0e35a04fbd8a1e514b0d1128d398c664581d4e7671c60cdc79cbf689195d8c88cf0eada7dfcdd062cd232ff6e291a9f2f29d1c863960c19").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("deff8ddbefadded7f7cfedffff56f69bfa9d7f7d4fffff0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 2,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("968c49c573cf67781bff76a862193ef2d6b0659316c5c012dfe858f3f6293962db349a854a1a714cb60e86f5602ef5cc11791a9a941642192b0b2b540d9d0819b396313ecf3311ac9821a2d19836f96c8afc2ac6142716124b063a9162220203").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("d9fdb15fcfe373fedbffffafeefff3fbfd9af72bffffbc07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 11,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9877822519fbc04e02efb0296b4976c1ebdbf3e82233e8e388bcda028a37941bf0c544696d7f7be557768d6c66387c0305707fd38823320d2360b1a8b4d22a9c5d41996501cd5d1f98acaf19777d12140ab7f4308bf616d80539615c3c21e798").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bffffbfd3fdfb7bf5baffff749b5f7fe9ef3f7f5576e7f06").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 47,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a6f10f61b1e23d26eb7ef431433485588e6ff9aaa7840c3a2257612000847cee74158e5058054154b52c4e61a7bac9d10c3c8f982c099ac38982b49bd45211d475ae8e355b03c5cb34f67a6b61b1f1b4f22bbaad10a30c41c9eb110d0c43b4cc").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("dff7f8debbff7ff6f7b2edf2bdecefbf7ffccfff9f5bed05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 14,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8a09752408b20fd35d5a8f7348a0e321c6b1125e832a4946c6d6809251817e7a1d8511929c22600e5b6b41a56e135749158198e86e0f5a3c70f9d4bbf8cbcc5d5862bc80f74ce18d1938fb386f06ec6f7a8861fff7c31e499a0d5ca5deea23a5").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5fbefff77d7eb77f1befff5dfffb0ffb7eefabd388ffdf07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 42,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a603ca73bebf302c4b8bc3955110087d09b45dd4470559eb564793ca2e67c6cd252f5654955aaea924558de62015248008b922a9377132de5d16b2df4255e00ef9972783de3140b768ad3a018e606c4192ce6920380bb86c2584057e823fc59f").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5dff3bfb65bfdfcdefa5efe7bde3b7f7e7ef62fbdfbfff0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 30,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("94c714f0331c7912e952094e45b45901c3487fd25ec776ff08b8ff81f53b8833fe1691c9741be5bfc77a1c2204ad807b02c84f755720c2311aa27e7250763187ac4adc9f1b18e53200edf82dd7abcf14a63afa305038b1b1370f0ab2d46c00c4").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("eeff9bcff7fbd5ed7cdfdedd4fffb5ff6daa9b7bfdfbbf0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 38,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8ccd22b583837c22b61f0eed10b8efd74e5f885f0707f47a0dd06f9c9ff020b9c9bbe5490dfb6afb42b9bedde1ba7238046b824d0f6a3fd2d0c6f6a37af91f99954d3b55e2dbb95f3e5a8d70eb64633958a0dd1cd281a19fe68dda5a875de081").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("aff87ec7fff7ff7ffbf6ff7c07fceffee3faffe2faf5170f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 41,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a9cd34ca4064aae38a45706b97b0970b9fa415dd67a998f8878b4707480bd4571a72b012de2ffc1086a03bdb58dcbe1c0cd163251025edea595e82f3997990fe14f6ebb79b1da71381d2b8775fb19e475b0f526c476cf8d201791df901484b9e").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ffbfffbdbefdeeb9fefdecefbf7ffcd3dd6d7bcbde29fa0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 28,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8fe5b86869b42cbdf437568ea40b1eb22fee5d7feba055f49895372c81ac76832c9555ea8000b70d05125c08b7775a7b0823b661232596ec465875e87b9184c1e73aa4701c5193cd9ebbab0cf204c6d179a3fafcb9ab50444827908561efd6a0").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("dd7f7e659efdf7bffd8bff776eeff7efe347b3ff9eeeff07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 29,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("80766b9ef0e1c25d67584833a51a61e7da4aec6b329aea42cbbb8b69e6060bf7d224965ec5a709b857cb4444b43024ac0e69ae9d0a7dccbf92556bc393f07a66562fa18ff01954d7ef879877f35675888c6cfa5768afc1d435657f2d33500b8c").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ff9dfef79ffec7fffb7afa7f3db96eaf3fffbcfcfcf3a30f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 50,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("813de01a619e97d51b09083e83d84ebb511dd75cfe81f3537395741328d3e8ead75409b2e140e1ab8e49cf4142e55ad81101cced362906ab73073369ac26a2aff2bb781209d1505d9c75c54ccd7b52576a2de67bb8528d8ba9fcb35eb9a7a3eb").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("f7a41ebf9fbb57fbf7bfd3ffcebf7fb1dbb7fbff5f7f5f07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 34,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("812e3002b7e1837748ce8cb8e6460ff109062ff509270d88dd8af5f5182e60df30beadcc87d4f4b84e021be35399e211198cbe0fa5e29e7a39f604bd152dfc8e4ea0773ee7fc7d53def86a6bd68a667b984d7bf00a178e3314430df6cde002bd").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("c953f7797fd5dbbffff67e6aebf9fffda9dffdbfefa7fd07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 1,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8f6465d48298c9e7630d89e140ebb2c0eaf2fd155e6ac9f5442f1b126a5a39e0e9b0a62bd9b52587251d0d211072ca1500638afbc26c8134ff1babb0dcc7c1fe1d368f9a24fdb21a5a3ce17b8bc1a5720d1feb59e1be9afa9388c75fd1fb53d5").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("b7decbcbffbefb7fd5fd7ffdbfffe7bb6defc77be147fc0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 53,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("92acce795e795aa236d29fe6a014a91a911609de3f90a8f0583f64a6315fd132321a784cc06e6620be5992f67e02b4df069a88d0e16714c80c71f61a476008c0a8e46c030842c46d475eef95947a8fa5f657baee3f814480b8d60d6f4eef4093").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("3fffd7ddfd2fae7d5fffdfde9effbfaef2fdbf73d536dd07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 59,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9077580b626793eaf36eb182a90532ea4e04e41842b6301d12afd45ccea3a750579fcbaadd7366e2d7921d7eecc1cd400d8fb6c40f41d1f2877a50a9010c7ec33f3a7cd31c88fd477585e31186ef2c81e4ffa833e17636c59b46718bddf93f1b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ffe54efb79d3bfdfbfedc9f2fad57f3df7fdfbbbee7fbd0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 63,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8a60177c409f6f780df7d155ed5ffd59a8f0a4abb2897bdae281da6bdfc30aedbb177cea30ce71d88a3a3030a3fcbf6301a04588f266828a4f23fdf63f36f6815e581bdd20d49eeb940fa45328b5770959d927e3088286d2aba96dcdcb3275c1").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("551fbdebf7e7feffdaee4bfeffbc5f7f7f9df377dadbdf0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 22,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8d74e4b52d71c59ae33dc021a963c61616c7390c230eb6dc441a7b9e4f4c35f67d3708af736fca449bcac6402f9df568081c8383905169e05c751f0a8e3701618b979344b4f1d557657ddb67197137156b663c5c8a57925c5a65661e1618cece").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ff563bafbf3fdfeef56e7fffc3ef72fdffdebfbdf0d65f0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 10,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b3f36bb06bbc4183523265022dc4dfdcf2478936abcbacc20622ee56a6ed0e75a09b9efd1f3864aa4f9acda1e0f22f0a063e2f2c1b0f5d71413cff6072e077ae3d88d8743bc284528fc5a51875837b79e8b9a6cd7eb7792808343805ac8bbde0").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fbcefefdfdd3e5beeffdbbffff2f4f4b87defb7ddf7e4f06").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 54,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a3e8f6f64ea0739e51af0efde89dfbf756ccf8377cc7f6c505618d8b54d5eb27485449a579cef4de131ecc61b767960b1755786e18a6c76f89b040face9aa3113413dea68df223d4deb792fdb219eb7a2b0d81bc3333041de315055c176e192f").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fffeef7feaf7fbbe46f7f7cb374eb7f3bdef7e6ff575d70d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 25,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8a0570e208082d8b93eff46c282f516bffa98cb500198093ba8fd891023f7996912966bebcae2b551ff02cd8ca3851a20f0c7ec75ad63b6079fad61f456c87ab635d6bac82f1ec677584e6509b7e3ef60d57074bbc374ca149e1d21ff5b619e7").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("e7eef2fefccc477fefb673fd6fffff7ffee8edaf17ffaf05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 39,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ad006f217629cfbd4388af2a65e89dbe6ca861a66f3ec2ac2455a7a7f34e0971d520b9e4a52d809f1c88911a0b72462c16138da404541a6ba5f05bf180d1e534df6a9cba080a378c72efef3427807daacf9655a10dcf3ad83e62373f80199085").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ffb775fbdd78dabb76e7f3fdf9fffdece9d7d87e7ffdf60b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 58,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9012b1cd59553bd23cf80ff09c2370395d8ba50e5c83b9de3a942ac9da9a9faac24a596c1889bb73add4521c76c7c17d01d61014a6d82777eb3d9e7995364cf86fc303e8791fa1398ce852395ce0114780be541a107521c059366a689afea4d2").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ff7acccfdb7b6f67fdcfeb3fecfbfecdfb5f5e95bffdfc0d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 13,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("839bd5a9a48c639602e925d71929584fbf17078647f96b40bd9235e63248453d2c3d4ab43e141d3153f741351d3abf2e12bdcef75fb914d6054c33aa4f2acaf74bd0a0463ab756c687a1aa869b0bfec0d02fed1afef7482cab242c52211ad742").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5fbdfffaafcf7df9cbefbdefbb17f793cdf9f6b7ed7dde0d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 3,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("85fe189d01aa1b37398a677949729b64108de2ba2512e127021de259e2bddbf0f1eb5506161d8992a89b588dccdcf8e40bd5751e44f706643096371f7783130e1fc1062743d9ffba1a1a07dcc6ebafae01956b14ebe8c6d68a90f08fda2afeda").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("717de578eefe4f996dfd3bfd335ddfffffdfffbfdfe3d70d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 46,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("82987e6a654b7cfc46ef493766b6f6e8a4b266ce35d25676599b30614dafd8ef4c3826fde35bd352cdcf63beb009174f0e7556a1db08491bcce0a5887335d7efca855fa7f96e5d17baa704eadc718aa53f23d5d4f17c5a01a61184b2e166d023").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("d69dffbc2f7e1b7fdffd6f5fe976fdddfaed72ffed6eff0d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 51,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("85022430e4241bbb69ddc87e26b7be3371744d2dd9bf33d090faae6f8a6e27d3d35c9eab23014cae65a503beb953849e0be8cdb970e7151bc6e2bac72513a97f2a8c5c5d79faa5e65c6e3d97451211b7fd49a355c2598fe58d27941fa4270128").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5dd77cfedbafcdff37d87673afecfd5dafffd77b92ffff0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 45,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a6e8241cd36bf783b8e3c3edef5610c5ea7cc790988b0f78cf1f65fda8e38b3e108a783d75c9f1585e18bdba1bfe04c00d33a79b54bb0afffd6c0676711ffa6e43e1410760cc575996d3fe28c11f3185b76f457b46dc28d6ea8b67ac9f2bbd3c").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fdb375f0af7fef63ffffc6f5fcd7bdeff7b64afabe7b7f0e").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 23,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8c16347a5185d57cfc7bb39477fbad54f570d72d4e54ff21a85b0fd0a766a7a3f392ccb286803cb963d830f9b51231741298eb70ef8a1531f3f471eaa1504672c7b72df59fce9def7388661c02e243ab1de42debe1f2f464a89a5c93ed294a0b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ede7fafb87dfefcfffb7ff7a1ef9dfd63fe377c73deba60f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 36,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("adc4c435faddeb6ffad59315332e644f1bab3673fb19258a1f1e9205ff37c79f1584d7be20093542c81912b1bff9c39803137cf734b572ac65aa2fe561c1699f161313333ee3e1ca1dffb5a8771151eb8169a001d0b34c3f1485b5b59b52115c").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bfba7df6dee6ffffefdfefbdbc67fe73932a5b91edfdf60f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 35,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8ed04832db7a3e16cfd95efd136da27627910134f73845730821983f3805dc2503883d2828f1da2a185b5bff7a6ba5ec003d1329525dc9ba975ab33445cf228c388dfe4a1de799638ac9726c54d161beecba91cef2878911b4d7a2c7aa9417e9").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("aed5fbb5baba7def96de7f7fed67bdedfcbff7fb2fdf7807").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 19,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b52843b55f5dd70d675233ec54b0bef5a1b8601ddcc125d40ddc8b803b60b0cc2089662357221ffd8e6e36516c2f5c7d0473d64bf0a43e40f129f5abb1eafa4e936e6591a34f3f7e7ecc1158d3a7231c4cb374305116d4136880ed24e8a8d656").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("faa79defca7bbb97adffffbe76eebdce713fb5ee7fff7706").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 49,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a019fd84e5691bb46abd0d62552ec58383262449362b090b7855963b62a5240d1c1cfcc92f9f9026f29c86cdda37375f0e5c2e14e875a8f3807161ba3745b9c36584c0257b7e02e6323ae175569e0292139fd508504b3f1ea0542d5b23aaa623").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ecfcbeddddfd6bbdf6d8e57ddee3fbbf3b7edf6fddd7cf05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 26,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("85b640cae7861d8eaf1ad0a05ad061b74e059c1ee3024412b2cc505c48ca52d2f604524ceffd111989a313a3394f517518b8700fde86a3e961c5940fad4270af997c4cb68cc5b99dcbb440222e90767514fe2a191185fd0b636de012ff103afb").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bb17b7bfef6d6f943dbdfbf7f2ef71fd3fddb557fef8bf0e").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 61,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ae791919196848b8bd4530636e71d712f612d66bb967b694c15c6388d747bbd0bc5d01e7f4a23225fc4f979cdd096d041659b4dbdf09bd3a967a2c78caa9552d3c2c114709f15cfd25444986087a0df892698beee9e29502f9368e1c3d9f2a7b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("9bbe2f7bff639efcd797baf17be4ffefae7f13f7fff63f0e").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 12,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("85b29af63dbd84118c09a9fc74758b45f7533e52e18ef3c71d4d15a263c2dd763de9304d22adb3bbfdf422b0f5ec9f6403a095cdd211c1323c233a346cce27c64603ed68bad73fb09fdf897e93b2dd0206ac290f43556d9fb745b9dd59da439c").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("8fdff77d7df3bdbfb557dfffd7a7a61df1bfadfe7acc9b0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 15,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a76344801b25bd9bcbaa2e4136a8cb693311b446afe1d0c78834578c8b87d1b533a674a2dccddf5ad6d8b6e48d0c6a330bcd27b89ee81b338a46ba2647029e1d433b183c67472d4b18fdb2bddf0f98fc2b7d3aac5ecfbc22dc4c8e7b169790cb").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("8feeff383a93ffff6fb7f1beeff7afdfed1fe3927beef505").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 57,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("906d6264b712e0cab4db371d86930d5fca6dfd7da25976d25066ded3288067d392919ff5c0dbb1e4a3edbc7a2f5a2eda1935218c9106dc2aff050005c47837810c22bba2a8cd43311e778e7266fd9dec321a6bc67f48976d2c9a73dbf7069830").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fa757bf93f7f7bdf9cdf752fddcaeb6e771f97fd7cff6e07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 52,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b681befeb30524e28533facd113d802f6335a561c4e7c95656e9ba006231fb43dfb685e846d017d7edf19f0995515b781747f01aa2b3caeba44b8b602188f3d642e6586a22180a7a93cd568904ceb818fe0dacaa0dfb5f434e26697038a761d3").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("4f7d5fbdfface7aff7fa5765f3bf7d277771caf79d4fff0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 55,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a9bb18ebff060374d613cca4ef89a94efdfc0f1e1337c502b3de3c8c565ae45be88f732b347dd4e72a3f0e9daa73b2dc0344ad01afe69172023f7c417e854519e730629bcc4e413326755971a4521559b493a8fdb856b50e0e7a7b580d20884b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("6fbfbcbfa4fa6fffebdeaafff2c1a7ffbb6f3feff4b3f304").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 6,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a5bff7bfafaf2a6ef4ab9751509a134619837ba235c8897bb3dbd9566326a370ea9118913e6a0500e3de43b83fdf1d590feda44dbc7de7bc206e4eb864a2b0ce92e18bd8a8c0ecf69e207c7dfc555e5d4bce4df8a46cf314cfb1f8a9f0081bb6").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("efffe174eae75defa3adfe7f32eeff2acdfdf5ff98fe7f0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 5,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ab81e8eb935a5501e3407fd41420d099d84ba30baf4af654e972a7da4795e1342596e6a5480c511126df78ab7f2a1a7e0192cf5f76de70bab89e43adf21dfc954d3ef9f05470cf42aa04e43678fd57757b3073acc6a1b204a60241e0e1800d91").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("6bf3ffbf33d3d53bffedbceccbb9f36dfffffff4d97f8805").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 9,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8529c059358c8c3ebbe3bccf7a4c5b53aa35c6858bcf69cf188cff09a5660737453f29b85431b0d08d60d32679df6f5a00b571228f588f8a1c475faa09f4f352ab88f8c1c3e9b2316528ed5572fd2b421fd6b787586542e921598e91c99b886d").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ffdd17b5fefcadf8f77cf17974c7f3b7df3df9bde9ff670b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 33,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a00455810c6fc9940c5606ccf2cc8e35cbbcb7c34acf65bc10342f8b62c52cb88bd6b8cac5702ae301783cb5362721f80c3df397a09af0b6d25a5cb73c9a0a1745271a64196360b5f4e4eab28f72dde22c3bf6e0a88c7909875e9564c83cd446").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ff1fe5af53efbbffbb577b1773d87fe7bddf733579ce7f0b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 20,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b7c0580684cc1808030ff25ef4ea90e62f628f2c5710900d913e6ff4a060bca1ca0d422bdaa553369d8bd1257cd9b6a0186467f070526c32b13b473b7b2bc896d14d467ed815d18b05fdf0e377327d1926b605f29ece9e159f399a64bc013486").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("becd6efbcdfddb37efbeff0efb77d967b2deff55fbf29e06").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 32,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b0f2e1094a58f2a6b4778973786627f0282e682b2a58b96a788b418db5ed81c81ac1b16d658eb79b28e30c1e9f4a16720188868da0a4dfac47306539ca49e0903a22a11403fde8726b09716c82cca7f0362eecc48aeea0884b436e7ef61d539b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("7d57cb32e6ef1d59ffffc8d7fffd39777f5defdebfbaa907").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 16,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b0b1c0d46ba4a2451c68edb06f9fe3ebd57297c4d514b0421f71c1ed70dc411d7723b1922ea77a93ef2d0a6de36cca2d0c7f49038e8502fd538c779d121a20b5afcd3eab2e12341cf319b92f3dc53e4d09472099ad110273021efff8aa0b167a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("572ff97f7f46b15fbff6db17f9c375ffe3bb9f3f7b57f707").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 24,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b22f80c57c19b9a4a2fda3f5592d3d59937a684e95a2bc1535747a8a090b12ebccc995dea13ea2168bd0ae969d292ca618a0c584d223affc11cdad533597c1361704cf80e7c779e88e7d9dc4ead4d9a83142a2296496e642e21226f627036806").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ced9e52135daef976c2efb3c6daf3fefe3fbfdff5d7fff07").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 62,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8d776e4539fba97aefb0ec183051f751229c09cbdefae701ec429d14f209e5c88ee2fd69b90ceb5d38a423f01e888cbb140332ffda7da8e46b5f68a854ee73a4c901f4ff172ea902c75036dac986e04b4410ffbb2df6acc5b39a05440a64b683").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ec936ebff7c2f0eb80e17eaffecfffefff3b7aed697ecb0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 7,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9035dddcbd2359a4aeae3967da9649438e1851c25cfaea8e377404d78b3cce95ba4a6cfaf87374824f9f0fd18bbd7484056dad4a7dc99634f41d5638416ddf14a4c2630344ef078dd1380926a97fb6e4781e8c1ae509de9956931ebe54db14d0").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bdb96fbf0cdff7f5ceef6b3e49377fd6d175bf6d1e7bfd05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 37,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("90fe8ea42123b867514efc767b5d1d8ae9dc2701933014be65e503d34b8cb67a02c5e5011a06ef91802d0bdd5537fff51639d59b584b0a90b8e77c71432604026ddf4340bfa1b17e07c7b072bd2abcb5f0bee4e2b52a91535ad50410bcb838ad").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("bb17b73fcf6d6f942dbdfad6f2ee71fd1fddb557f6fabf0c").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 61,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8b31c91e80e8303b46dc55e14bfcd2d518498444051798239040de05f34b2935ed85df4f37ba7b01c99288c130e5e11a13361b59fa4b8d6054330f282d8156fcce4f12eb464e65624ccf2df74548fbbc837168b759f9ba67176ffcfbe81c3caa").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("9cfe3d9fb7f7dec1aa3ebfe7f5fe076f83edf7f2533eb20e").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 60,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8263e10702bd83e3ad61bc1a0968b36b43d6ed8a2ca652b21987a6d315319a19b8123cde88bb2dd1269270075befe76a0482ae903fd67dc64598bf6867d90dd202d33b02baf9192fbe10ef0de8444d73550eaf51de8784ed7b1fa79658584b22").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("edabedf2dba9f6a6317779f726d1abefffadd5cbf3bbc50f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 0,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b0c8ee7f6baba5cd42a8365f30736b0dd01845e022f8812c284a31768f98379c8f9560de77ae776babb7304c54c0165d15a1a1363846b2c0c6044faafdf7141dbdf31aa9b958d07e6fd4c3283bf49614eead7c948ad04460a5c7d42ab61b04c0").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("cadf7c7dbf73e6a6ba2f44e5bddaa79d9ea745fef1dfdb0d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 56,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8fdd2059c6799cf49e44cf851784b9161ae627e83a018039399fbeca5e1cc7598bad37116ac792d4e7ff5d3911bdebac09c9d86a40a8dfbf58cbae24f64b311d01cb8100fde6fa4b12af107e01a98bdfc2ca9c333d7b32b6bebbd5dd35508688").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("e8bcb8cdddbd69b9f6d8e56d9e43fb3f3b7edf6fddd6cf04").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 26,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("99ad0a195839d02fc1dbf092d9c69eb16ce49250fc64d9b42e128faddf604530fc50da155f791012bdfbd0ccae2d25fd00194c518cbc88adab1143434ba6d5a36e47604abd02d9857964ce477ee685740b6e24ba6b101ee66ec3923e685afbc1").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ac9f67c17bbf9bf61fb5e4fbccfe65fd74f377f74c4cad09").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 31,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ab54454a2c0fa59b172374b340f6a5c5bc9aef5148be57d36a44154629141f90bc4cf473e47eaecb37c2a4e00e45344f0eb94478c9d86315ee2ff33aac52e5f77debe613276d6576454709b89c91f115c5e7093acbb7a990fbb582ddb5272a20").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("147e1c9737f752c10a1abd61f5fe056d8248f582523ea008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 60,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a048e10cc373bf8c7a63d47de50aea413cc94aed7b610f214993f24dc43241151b8bc27cbce49e0d41eaae845b35e9f215cc73438b01d445301d02ca7cb589d4fde0d7b9d5025d32f54e9e368fc537bee014f46db8065919b1494eb2c9f03a3b").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("b6d99072643a0a4a0e01bb800057ebac0988182d00b50006").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 4,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a9f6a79434e66c3f8c7154e8292ffae7de055c3510bd2e8c9c2be9c4bf9a2467277ba448515fe8f0adc3fa84321ffc02121be61a60967dfef6ea489ed8742a235949567536d01665b4642616510ea9b1c7d94c1d2b965881543f54e5ba7eaa2d").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("525840c9832b6100c4c9413b84c05a0991174e851ed51008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 13,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b48d51f6e30589cac7bfdb68312ec2a0ef4ab14d6bb770c1737ec14f700b8e84e0275980fdf37a869a6539be9abadeb60a3bc372c5c6f473381b3fb9d0d94882fd0e480311c82f10b7aece7da6cdecd5d51e4ca47e8c2f2e781712a5453cf600").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("e90221299f2ed5223544627082cc72051bcc058190024208").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 10,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("846f3a3dacf676768976478ca17fb09abe8cac2eaa3e8bbf086a9ea85f6ecefde5b6bb2f287b5c97abf2e99b36e9e2d30d8d62fb5c0fbf5e985ce74ca7a048cddb4fd0b74668f95392f644d59c69af7508c4c43476cf209009d0ca4cd3ad379a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("0f70100104e3de6a30462e0c03b06364008a4362ca94130c").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 41,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b0d19d7a70bae19ebcd9d24398fdfd954792555e425dac30d44c4448c4e621a1a2204aed550a4f0b416abca1fa692fe318b2df3fd52cc2dfcc75e11126dff2f5bff5b9fe0ad5426472ab6de21f3647238289abb8f89bdb1a0ae1172304e13473").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("8924a0000ea161aac9a0fd84a8df610171087220c6800807").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 11,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("add1ff0aee96a24501fd0106239276333ca8f22fbb1c205c9cfe74943c000d6ec47b44719791de3a3911c5c66ce425911593ae5d22e6ea14fe825c9cd9f841311ba27be1ec50bd1421b36c9cbb1aa21f7449456005aaf34d22c2fff458cc87a4").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("0040b98010020162048c2f43a4273d8558bf73c22e091004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 19,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("91ae021043ebff6538ef17ebda90b08947511951530d162190e033532b874f2c941356a8b04f6c220bbfff5f6efbdc110c99bae1597f4aaf7cd0c6034637b9923fb0929c5d5a95d0eb593bca7224a62736f514ec988bed2ed4339dbdc0b0de12").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("90513e241a98118a4c088c074c28e4480143103b0a2e9204").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 29,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ac89481cf6a9ac260088fc7c1a3ff729612961422ab82325302275cda09e22311ee90d6fbfb537cee96b0c779a13c3b6021993654a485948ac3c7d4da8af7d5a6ef92ded1e7e72607479d5baffe5fc55eed678fe36254c79fc6efb3267f59314").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("51e4247d08215818422264482100a4e3056f502c9004150d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 25,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a92c33069c550643a51486c8b072e589fe173ad1adcaaad52067f1c3c117e73b4b3219fc2a544b5440f9a957c68e7f330583f6ff6c0b3445ec99cefa7dedd4ddbb033d56ea29f321930f1ac996d14bd5c58b9f05ddbde4ac63d91b5ff5ab0c26").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("139481943c24047c04814394062614061281856115001905").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 59,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b656bd40db418a0dc942c546376357c7140ba981296bf340adebff29178dc96d700c525d5693f19e594adefe48e316ba0e0f9bd5a2f656e9824f88aa470e85eb6da29dd9d414ddd532234b0984667911857c69f0b963b129313ff3653a9ac7ee").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("42da5024181046a00a29006089d006000e2300c280094308").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 56,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("95eb921e0076e55307966c8786bc8e89c6ba2e4163bb69b6f84fdb403f94caaea7c5b89a8b6c6854b481a9d05fd20a4f0e1d5c333e60cc6049ad528aab90a9b97faeb7144007839d4276a4904614a3fafeabf5a8ba452f5c74e0f987acd3147e").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("211160d05020421834068078001a11088112805022acb009").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 58,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a29488d15491e206ec841f8c7754a20d64791c8f7a128d0a088df7276e57e618cf4cf81e431629ddb230fde9d1f05f980fbd58d8820eb59efd44faf8c066706e202ea09af503b7c4e3443aeccc42b09fc496d7a7c7e54c68471090d5ab87a1ea").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("8238034a040116a80600104062005d0c2624121445000c0c").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 12,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9141c0f19d26321e0e6245cb2a619c0ea0dc1fae173ad76015ec5ad7382e01a11b35329a88e2443294285ef4f9395f4713f6ff445d8bac890fd4900153a684eb5d61bc498057652a16397c49406550947430b2f040e1429ab9a3c9f2b61dd9a9").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("1000802140843206020000a001a0402a4400910202028208").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 22,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("83e129dbdde41a63cd32cd76f06e4c80ad122dfd34d65aa75dde58b7a5af5e97cc82508b1df7c535b61764e403e8350d0e60a36f128333bcd24d0c4cb0f2951b177fa606ede65b35fb7634cff1115a160b6daec8bde36c2255e0ed399061ebe3").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("04a41000028242004420081000402c081000c80802480004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 14,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b28fe0a4f029c2e6209e68686fcc888993e5264128702fce78ab9c3fb05a6b12205278990a2cafe1677a9551e63883d9061d8ee549d56d21191acb117fe1ae7a91b7a97ba7046eedd11ade555b2108dca445473cdc9c710821b3903b7ebdc3a3").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("040100000480c00108022100008815450020400814010a04").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 62,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b9a1b0cbe58fccd30327a7253023a31f0a3225e48a48127f7672c4fc04c197be57af402ea497e5b92a5317887b4b475b19407084b44bfada27d2736c7b1a99b43d9b6fc33906973a8a381611e31076c654f6228972792a4c9e6b5699a985e479").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000880001203040000003082008502c2040002003a080004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 57,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("83f52430c2d7e95093be39706afba48fcd9c7c70d828df653e608bd2d91077f859a6d8e3e681196d02a525126754a55b19c2a25605c00984829c45d5adc0ee4af5f06b76d816b00b432af0e56eae45ccd1950ca75920cd3564230d93331e4a20").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("0401443a0101080400000000100040084201000180620008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 63,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a1939d7649e9f1afd8f343378ccfbd71f45d3f5ab877e95f7b27652db180db8dc6578dca6ef6af56848115d9b089db0712f38bf929a9135adebe6c6fb9a5425ab237a8c1538d238a7893e4a5ec5a65916d7b395e628299ee65a326e95113f538").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("0000a44000048090304000000400024040100042c0808408").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 40,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("ab94701678b0b7460335895a6288b47466adcb880054d3621cf0eea0d650af97b323e450731b6624ed76b6be1a3019b4066bfd4665f130b377b5879c8d1ec34a17228827112dc1bc2a05100442d03b7dae839494e5a1cc3a69c81c75d2e3ba74").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000042001200040018280800204080400040030110010a08").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 28,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("94fe6a626c58ec8f0cc2b87302563ffaa11fad62ba5c99c9f2d9997878e619ab4c64b717917acab6ed1732220d8ef416086faf14f51c848c4fa95261e04bffeec4a7b8425ad84c4c0f7e6d78b49ed7379501f0a25b09b4df33f5656835a32dfd").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("0044400a0100078008108000001000200300212000200408").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 8,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b755c70b32c88e816354942ec0eb30c919349dc7b37b42094cb748f8525c57b7f80b8306db6368a52e3eb8104002d05f092e73b69b6e01197723fa7bcc087bc7fb0dec9fc7325cc58f0a5db784a93b2091b6b505b223c503a128a30dc7cff3ac").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("680008000c2441000800000201004000088008400c010008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 50,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("acc03b7b38eed5e7ddf2a411f0538a67b8b48dbe2b709a19bd93c48f5c7b230a5a67fa54031c1a2dbd11cb972c9bad6803e407844cefa95bc83a1118dad09412335fd6d0406e0b4ef8b08db505c998477a8a6715cf28b12ae47aa51147fb08a4").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("100400000400000140010018802420004a40008000420c08").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 51,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8b144f1b3faa0011932073e6e5a147866c226697ff8fac473f3a189f6ab0c1f31760e673aa7e83a1175c01c30944422404bb237fc8f1d2c6716af96372f1bef0e121d03941e68e54c4e1ec1892a02c40e1ce97b13beca8d7e532fdff55e4e9a9").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("a4100008440020008001002020010820800000000120c008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 7,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8067888a1b88b145afcde932545844acc07c26ed5bebd798d9a758727b00f97c4d5c83d85a205735ba4a50330e45e02c0f8b845af3c151b2cf84e13a4357fe87860acbdded3fed31b0a1f1af18722d0ba29b1e19bb3ad85c48fa5fde27af3346").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("800000800050002802000000041040002042204080022308").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 18,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("955cdbfb7bc4c3fbcc929010ad4b40fe28256bcaa8669a7c1bfc8349cc5a2e7697c5b6421955d653e1d51926e3083f131368ea4600131ed350413e3ecc55e653675849a126882add461327aa63daaf28c3f7eff08dfc451c024037e7dde8021a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("008e428000020010000100002000c0000003001000444008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 53,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("816b3808c04adf81735ad642e7c056843f8804d8b38350adaa8fe24b9462a79dd6b1052ae0e9766b8cca559ce08cec2f08a68e3ad8a788443672ed2186a1039ec3ff93ebf5e98824b0944eae7ab3e693f3667bff3744ab1d27495684b28109f7").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("020a00200048000112400044001000090000200000890004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 42,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a08af599ed9d5668d09bc0265c42ee320695049742fc86529775d526da0a7a24c09bee202a8ad80ee8f52a48e62fb5560ddeb2a750089341c3e0c37e5d56bbf8b1e4c9b954e12a32d4c3ff94c5003f5b708c7b5610f1c0c2f22bf42812e25fcd").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000000000004810204152010104002401000400000028009").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 36,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("812d18744bb2146f76101857cc852b71d42914195d590b46a5dd23cc248b67271dd141b714a65dac3e4e4d257454f87717de9db1c3130891a46e0b273572be8aefaa9aa81aa72588dd2856bdf91b49eb3ebda47d05b83a8ff52c1c0fb859f19a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("04140000400000a20a100060001000810020002100080008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 31,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("aadd4cff48e0c84c440e795d9f8984b93b9b0b7a4f591902b21652495cfc21d2c6200db433e171c25d180a2c0c4062530d2b3d699dad94bb8674cdc6dab62d8ef965a1d61c2e7e0ddbdf417fdfb918a44fda56776b65b1c7ce0e15ad83110330").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("002000012280000011400800403032005802000000000004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 9,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8cb3c29e08b0f71131a59cb852aac196cfb9528138eb3094f34fb2cb78928cea3e83007503a33591dff5f1af4a55a19e106a67647857aa383f70388dd47fbb5f67918a388b3b2ff6ee537a5c5dd1a748f8fe3fdabd03ca6f04eaa982d7f374c5").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00000200020000020001200000204620000820010042140a").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 43,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("93e59e7e766593826dfc38a4853cb66e7b8f56ccf478b553fc793399dfce84aacfa9f10f981692b8062bd51b379bbabf08c754158dc72268ffd3d2cdd551c287d8d8b69ec1f4a5ddae4dc9e61cd0d6ed91fac8074ae741ee977ff22928686496").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000010800040000408c80810400000080100010010004904").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 54,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9642ef2a293ef42ae99ea9aac66483a242e1a9518e23213ace2a4008c9ed4d7d142598a85d6634732ca790d2b8cfd7870da5e490575518ea2e9c990fb57b79283df8c22c2a52834dbd48f9405440af964b49d5767452016c08decfdb78aeefa0").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("500000400008006000000010020840010000804000504408").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 45,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a344899fc0b980ebf4fd9bd908ae41b79f7669c177c834e55edf943695f03d4d385ba489e8a74cf50b955d0f507ea4951109517a4a97ee8c6238e2d3981c13a5d86985bcbbd105b6b2f5dd90009b5d5c1892e9b4e21091f110ab8d5f72834ed5").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00000000210028002620020000008000202200000048020a").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 23,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9738f7c09afbb6a455da0f8db2dace3e2b37223a5bf364b9b6ea3fed13c60c4018756933d7eb19c67e1d2af2809b4c410c96910a91b428259118a9fa160c3e6f543c891240bec9f989820be08489d2df4f817678d97f9a728e744b3e0fa5057a").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("020004000040000810001011100080000200000040000b08").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 48,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a16c9593df6894826d1c74de4bebb5b51fcffa42b7642b8c2c4a482df40067b94f3465081849e32b42416dd36f709f5f1183f3b2bd6f36fda9d7d7ddfd97057130aca9a8de619627dd7e27781fae48b00f6199358a2500339e0632f0ba8905be").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000000090029060080000800100000802000300001000008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 2,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8a09a94d70bfbdf89152cbdd3ac3521b98980351b2d7b36aa5d76ef2e53feaf98ce4c79021fa67d4997e6bcfd5411b4319527a339b326fb6db5fdaaa40ec51263dafe15e036e31dae102f58441b9b3da57bedf5f669776ce7292af5ee3b76215").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("005820404000000108200000100000010000020044020008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 30,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a79892db4d61a7a71393a97bffb5f7c16f72f4996e884f879d1081a079f0a6e8d557aa5df84795af104b0b413a0526d314f8caa66ab4605a32b49db179ceeb1dd829abcb2da92fbbc2fd1f03260aaeda41ec485b9358845310eaeb1a3b970eb9").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("080008080000001202028000404080840800000000020004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 6,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("86b3450dece3b584787c95592734f83f041b16fb88cd200cdd22fe5e768d812f8b80147c6a6ab357b08616287724dee70f96ae2a4646afcc3b81f9d2821f22b52b3a7038bcdf2a04ac50cbcc6923db1bc744f740572ac233f037c4e99409b4fa").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00008000000410040000040100000002c400408008800108").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 5,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("93a9c541b5990579537c1a2962e17db5724cc27f654f2253ab5272467ca885b56f6777b2b02c478910c342fabf3cb0cc04af1ba5169af53cc5dd4a82359f3075f660fafe9bdfbd174e073a443b61c23522bed91f6d1faa12675719e5a5293682").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000000000808001100020000400001004030200004402004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 49,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("80b040e27eb73fb29f289592cc18245974b05be209d0a554a142bc9a0ce8883cb730f0279c2b47b9b7a27da99237be8216848ac7dfd584a9ea5b7cd02d3b88752602f58cbe4d6952605223ca3d0f0819ba4ff004ff4e608efd743c9598f3dbb7").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("c00800000000200000100000000244400001080180010004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 21,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("813eee7a211d4d35c014d2ff3b0609ce1855aaee3e416564750a4b8e4fa7c9edf8ed3cca796a1200ff3d85172545bbb60eaa76bcaa073df2b4b9ce7b57bf7631b9894f04596fb15ad99cd5374ea67f1b01d9579933d9078afbb38cff8ec1c1be").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("420100000400240044040000000110040000000800000108").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 33,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("917eba27e4a063dd3f3d18a0f020d17f08b6ecbb6c73f174ffb73b3f60205488f37c7d4b76a55fde84a0fec21c0519930d779c0a7bcaef1ac171b67d8753fa426e5db10b45d93bb95cc87b6c2150e703c9b40b2cad6cb49bce864def869a1795").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00410000080000000020006000400000040006004002c008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 46,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8b9ee759d0568fdc80222eb6f0d1065cff7da625501aa305495992296bc582c9a10e45390de83a14378978f2a483749218b3d1384c4fb812e0bc5510b17e39d7fea620862714ffa52782bee2dae3153215356bd41754162958577ac56671c891").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("050140000000000000300004000000010b00008001000008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 27,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8b014824a1935006262285c203e4e2abdfe24634423edb36d1ddef6ce69ce2862a45fc1329edcda680ea8df1bf016e91182932f24c14b44b35de17b14d5c2f77f029f4ead9be84c823412ac8543d710555d707697cd83b3c5f6055d291a026db").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000002000103000000021010000040200000008408001004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 34,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9984e634dde38f2a52c42a18e53928d502df9d17c078428b8656d4fd701e51e601aab2a72c639f48f2e98f16f577cc3f02367a2c9f941dd38f92016f377ddbf4d32a643881c0b004332c116b21a4c0d6a62e022f2d936d689df5d886ed0b3182").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("800082002000000200000000020400110000210000000008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 15,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("93e3b7574afe15055e2c9e8e73678deeb6d167b200d139d53ae498c37b72c96c6628443a1da430404ccf27203f33f00e0bd1aaafb9690ef73479fb17bb4a41c674f3852ac70b9560454fcbb82d9dba6a032e0c98c65ed21eebc015da419f8004").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("08000100000000000c000000008000042000000000222004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 16,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("87a21ed38670416cac92a41709542089d8438653122d952cc1787a20ce41bbdf1db5705cefc2a404209cb8ef131a4ddb16e4274acda77e53d614793a9da531ba2411acb95f8bc140f8ad33464dbee4fe44be0d5327386d9183a93e163c30298f").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00000000000000000020200000800180200100004000010c").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 0,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b80173cb046af8718c91cd86cbd35370e0b2cf0bd2b5a26f8f7870a7aae80ca2b83c524f42d4890f1ee4b0fecb7dda95022875904ba82507e888f9f157fe97b6013c4bbc658f97188103427aaa57b7458520515740526b27327e5a42329e18ca").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000000400042400000000000100200004020000000100008").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 3,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a4f9606bfa022b15ce7457ca62b5b72a4e3dfa516cfc7ecb04f6dd1b0ca7f33b544abdc85d740540c23db50285dc2fab16e6be196f3a63ee7e66f4b972ac6c77e417847e21f85b0eb40d61486b36321bbecd921a626a6f3cf9ca22be7d2f1b91").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000040200000100000004004000010000000001001000108").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 20,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8e4d609187b067de811c1c454f285f4d86ce7385c2b1e2dd5b2af191df0085c3734bdc17a34ddf803992f633f72979ca0aaf535caa320a1f8da9c7bbd7b182d69be40fd1a2927664f8fe5420ee8e120a3c86699c266614d5dc320876213af728").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000000000240000000000810000000200000000000000004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980062,
                            index: 24,
                            beacon_block_root: hex!("c835fa6fcfa00d1b0cfa3ec24b3c4671be93538f19aef7cf54065982ab6ae7e0").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("89fd432723e23aa9ebd2371c92789ffb65efada5b5b7d1d00bcf6eba4b148d9cc879661687ef48be61aa7cae840714200e497f54068ad31d395e1162b85d2b9b1f3ff51bad0fa4eda951d5f6bbd44b6ea68f014246f7dee15eb23ee744d0eb7c").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("00400000000040000008000000000004003000a000010004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980061,
                            index: 2,
                            beacon_block_root: hex!("d61058d752a46fb21a1d313b6a981c90a694247673f009e9429047d9aa2510b6").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("83b38cb8a28ea704fc667836f716d30e261c64441cf9f08604af6fb5ac0188053ca73a7c5404ab07338d5ca8f4f4f451114795e2d1ca6999df241138bf2b1fb3b6859933a87afabccae6f0a25a8ec9ec79f9e1d225e635dc4a82f16bae6d8f75").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("000400000000040408000200080010000000001000040004").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980061,
                            index: 5,
                            beacon_block_root: hex!("d61058d752a46fb21a1d313b6a981c90a694247673f009e9429047d9aa2510b6").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("878ae7cdd643e8c9d584401e27ab9c9683c07e3c498cc098fccd7afd56e06acb2912456fc24e64f5101e859c918341f712cfc325901287ffd3bdde373cced5ac4a233919dbe7f93acda75e6610541216d0c884c9406adc0cf62f353fbeaec5b9").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ad1ddb5ef2ffdb3eacfd5fff8e8fdffbbbfffa73df77980b").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 38,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("922ccf3859ef423e7dfad7ee593a7d80fe89954f75534e24b42cdac42bf8332c07135d62ba199ca29b771d69949bda65058a617fa8e99dfdd5523b73494d1f6e22fd33ae55f7979d5895a6d6ef7694acde929fa0c7a6121aa17dcce8c0ad9736").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("f9ffefff77ffdd97ece8c4e36c7afba797fef769cfe7ad0f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 43,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("9122b83d333e560435872124c4efd21a4e814cf6cd798ef7ce42273b6e80c9d235570b8967399752bc16189413c3bdf20b0e2a611474d9f593db45c993f1671daaa0e15d83c339b9e6682ea04723caf482de9eb26e5cf07441b3de9ec48566c4").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5cfa0ecadfefd05fffebfed7c9ae75eeffbd9d97fbf3790f").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 13,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("91947275b031ab3baa76d9b76bf4cd0f44b99d3920286ddb705717b8eb30ae0f52f432eea362904887fd1bf23903c54315aedbb64d9691c60ea1ecaa659c43da77bb6c46c55f4be0ab0da7047e8d27a76403faa1466252946f3d9a650abd712e").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("fe7e3dfc7c7d7ffe7fdffda43f377c62f8ef0b7b8eedbf05").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 42,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8984c5dd35b60de771007fe41b50ee8313fdce5b14c30d92302b6c5d5ca59d98ca0ca4e96c7d6fe766524b9621c9345f03e90c0af9ec41986005f830d76112b8a9541f731975ee4d2a5c62678f4a68c4e42d43a6b90110ae56d3ed0511131220").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ffaee9ffff46b1fbfdf5ece9f267ad5e76cfe77877cfb60d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 20,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8ade6112f8e5e3c274ec973597ad984cad14af674d038c62b5a949d41d11f7411b18103780eb36e3aba47a1bbe873926127745fc47ce2e03e5fb6355199f4c7a4683f008a63297590f61d59d752a01420efc66a3dca1270cede233486d16c2cc").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("75f87ffb39defb1a723549eefbdfffeafc1d77ff7373eb0e").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 36,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("b647c44b93ae84089a4c6aff4673fbb209067cf9e62b8e08893f42056b697a810d703b8cdf159463a354891713797bbe0ad8f9087e863d4fd00c6097cdf9aabb8b22ce3fa30b2dd06e488674929e84b9e1843af735d58ed8fc5f2841bd48de67").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("5f0dafe5fdadefef7c8a6f793fe369b7dbb1fdfb5fb3f30d").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 22,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("a47a20e267dcbb1e160852ecc5cdcea95bb0a22599200915b551f5cd9d11e992aaf26d3b2b7862de1772035ce986860607c05e50271b4e85597eac6368c07f7b8a7575806069b17427235b2cf9bd1e57e52bb315f664194e3c14e1fb1ee230ed").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("dca5b2e667f1dd65f7ab5fb6b5dffe6fefe7677fcae8f607").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 39,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("8412e3dc1380aa42d605cd999ac380b78c870c9c98a8eda8b3e0588cc828148654b60a013c3251c843b25065fb317055009912967f985ea6507a19f8a79ca625f575af41cb57b8ac263ecefdbbf5bf92603273d940b567036caf734bbde20c7d").to_vec().try_into().expect("signature too long"),
                    },
                    Attestation{
                        aggregation_bits: hex!("ad0ddb5e323fd33eacfd5fff8e8fdffbbbeffa73df779808").to_vec().try_into().expect("aggregation bits too long"),
                        data: AttestationData{
                            slot: 3980033,
                            index: 38,
                            beacon_block_root: hex!("6579e774c93bb16470c6b9bc5931b7d066e6f295bed99a4ea2a360fd946c954a").into(),
                            source: Checkpoint{
                                epoch: 124374,
                                root: hex!("eb9260adbd0bea25d82a2ede826930351daea986104164255c4747f312bd0d3e").into()
                            },
                            target: Checkpoint{
                                epoch: 124376,
                                root: hex!("78dfb693a3eb30be4b37829a7a18d1e9080e93eac0f950c7112630add80c8878").into()
                            },
                        },
                        signature: hex!("96ca186cdb8dd8aaaec0a2b375ffc98013ed690e0ef176ae2cb04e5c94b245b1f94d80ee30bb4b0d53f573b42631eacf0e24056230e337ab61779c55bbe310cf86b0cb1f1a36eb138d11cd594bd5995193f940459f4c01221a7956940995717b").to_vec().try_into().expect("signature too long"),
                    },
                ].try_into().expect("too many attestations"),
                deposits: vec![
                ].try_into().expect("too many deposits"),
                voluntary_exits:vec![
                ].try_into().expect("too many voluntary exits"),
                sync_aggregate: SyncAggregate{
                    sync_committee_bits: hex!("05efefd2f7f477f7ed46476cbeff7bc37db995cffbb5effbfffab66fad73bfcf354efffdfdbff7bfebdff5f5fffbfbfe37dfd6deffffdbf52ddde7f77fd9d7b9").to_vec().try_into().expect("committee bits too long"),
                    sync_committee_signature: hex!("8f333d43b32fdf7b7d7c4d3c1bf08b50984fd45fa8e41398c2a34a788f1a12cfdd18c98d303b435461d5aa1bb9f102780530ddfb0f7d3961414b289d0d6f143a99e80f8f16d8bb8af9747a843cc1252bdc810edb2d165cf6caa9af091990321d").to_vec().try_into().expect("signature too long"),
                }.try_into().expect("too many voluntary exits"),
                execution_payload: ExecutionPayload{
                    parent_hash: hex!("55697d6c38beecef74431d4324e62ff48b7ad3c574137524daba15ae9b604ec6").into(),
                    fee_recipient: hex!("b64a30399f7f6b0c154c2e7af0a3ec7b0a5b131a").to_vec().try_into().expect("fee recipient too long"),
                    state_root: hex!("542658178e5928d26b1fb4758cde084aff0dd70f02220958db7fec44dacd5e18").into(),
                    receipts_root: hex!("3b2a55ecff24d398d21bd5ae84448158cf0bd5dd3ea569b69b25c7fec6fa5c74").into(),
                    logs_bloom: hex!("ac281213c08010153614c02280ebaf00009ba004a741031a8a930d8108601c13a0469475c428d224400288dc06e182d100229c96435fe040808c2e0da46050e24420314546326488a80812491820807244c5009061c0971848220485c28020820c23194eae0cd20851a3910100940d412804840be5da069211006154244b984c0504f1045669a18405d073132891508711095dd11424c7080601916231200914022cc248361543e3cd048804ac8500e1ea02410840c416065b9c581926e04214002064560210280114a1509198a5946c01c020c0c04292d4060113026a182910211492720d620b0df6f0100842910884c8a790002008a24844926213106908cb").to_vec().to_vec().try_into().expect("logs bloom too long"),
                    prev_randao: hex!("05d1cd36accde0afc97cdd24b744fa16cf9215fdf3c2e712577c8d47015c3bd3").into(),
                    block_number: 7667947,
                    gas_limit: 30000000,
                    gas_used: 21045017,
                    timestamp: 1664268756,
                    extra_data: hex!("496c6c756d696e61746520446d6f63726174697a6520447374726962757465").to_vec().try_into().expect("extra data too long"),
                    base_fee_per_gas: U256::from(1137 as u32),
                    block_hash: hex!("3ed713fdc47edf20b07d893ab23affe82b36e6f81dd9d0051e86b67d80a2a3d2").into(),
                    transactions_root: hex!("7550686a83fe717ecb51f8f79a30444a3d5b13f8690ead2b8b2ca6ddd7ff8a36").into(),
                }
            },
        },
        block_body_root: hex!("ad1b3be000eab0cd26d809a7f50372213c9d0b8a8ea0dd762cb0f9c817b0908d").into(),
        sync_aggregate: SyncAggregate{
            sync_committee_bits: hex!("05efefd2f7f4f777ed46476cbeff7bc37db995cffbb5effbfffab66fed73bfcf354efffdfdbff7bfebdff575fffbfbfe37dfd6deffffdff52dd5e7ff7fd9d7b9").to_vec().try_into().expect("too many pubkeys"),
            sync_committee_signature: hex!("859326de544fa047c553ab7d1f65e19474daf1f147b5c59c2bfd8a0bc489b6dbae1d70260a9a6a1ffc98175cf7636bb50898f505392374d84bad4f22834a80bb0dd69687733978820eec008dd387d8c8fb557650708f308ae88f9b470cf766e1").to_vec().try_into().expect("signature too long"),
        },
        fork_version: hex!("02001020").into(),
    }
}