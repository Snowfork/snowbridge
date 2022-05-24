use crate::{BeaconHeader, SyncCommittee, ForkData, SigningData};

use ssz_rs_derive::SimpleSerialize;
use ssz_rs::{Deserialize, Sized, SimpleSerialize as SimpleSerializeTrait, Bitlist};
use ssz_rs::prelude::{Vector, List};
use sp_std::convert::TryInto;
use sp_std::iter::FromIterator;
use sp_std::prelude::*;

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZBeaconBlockHeader {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZForkData {
    pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZSigningData {
	pub object_root: [u8; 32],
	pub domain: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZSyncCommittee {
	pub pubkeys: Vector<Vector<u8, 48>, 512>,
	pub aggregate_pubkey: Vector<u8, 48>,
}

#[derive(Debug)]
pub enum MerkleizationError {
    HashTreeRootError,
    HashTreeRootInvalidBytes,
    InvalidLength
}

pub fn get_beacon_header(beacon_header: BeaconHeader) -> Result<SSZBeaconBlockHeader, MerkleizationError> {
    Ok(SSZBeaconBlockHeader{
        slot: beacon_header.slot,
        proposer_index: beacon_header.proposer_index,
        parent_root: beacon_header.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        state_root: beacon_header.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        body_root: beacon_header.body_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn hash_tree_root_beacon_header(beacon_header: BeaconHeader) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(get_beacon_header(beacon_header)?)
}

pub fn hash_tree_root_sync_committee(sync_committee: SyncCommittee) -> Result<[u8; 32], MerkleizationError> {
    let mut pubkeys_vec = Vec::new();

    for pubkey in sync_committee.pubkeys.iter() {
        let conv_pubkey = Vector::<u8, 48>::from_iter(pubkey.0);

        pubkeys_vec.push(conv_pubkey);
    }

    let pubkeys = Vector::<Vector::<u8, 48>, 512>::from_iter(pubkeys_vec.clone());

    let agg = Vector::<u8, 48>::from_iter(sync_committee.aggregate_pubkey.0);

    hash_tree_root(SSZSyncCommittee{ 
        pubkeys: pubkeys, 
        aggregate_pubkey: agg,
    })
}

pub fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZForkData{ 
        current_version: fork_data.current_version, 
        genesis_validators_root: fork_data.genesis_validators_root
    })
}

pub fn hash_tree_root_signing_data(signing_data: SigningData) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZSigningData{ 
        object_root: signing_data.object_root.into(),
        domain: signing_data.domain.into(),
    })
}

pub fn hash_tree_root<T: SimpleSerializeTrait>(mut object: T) -> Result<[u8; 32], MerkleizationError> {
    match object.hash_tree_root() {
        Ok(node)=> node.as_bytes().try_into().map_err(|_| MerkleizationError::HashTreeRootInvalidBytes), 
        Err(_e) => Err(MerkleizationError::HashTreeRootError)
    }
}

#[cfg(test)]
mod tests {
    use crate::merklization;
    use crate as ethereum_beacon_client;
    use frame_support::{assert_ok};
    use sp_core::H256;

    use hex_literal::hex;


    #[test]
    pub fn test_hash_tree_root_beacon_header() {
        let hash_root = merklization::hash_tree_root_beacon_header(
            ethereum_beacon_client::BeaconHeader {
                slot: 3,
                proposer_index: 2,
                parent_root: hex!(
                    "796ea53efb534eab7777809cc5ee2d84e7f25024b9d0c4d7e5bcaab657e4bdbd"
                )
                .into(),
                state_root: hex!(
                    "ba3ff080912be5c9c158b2e962c1b39a91bc0615762ba6fa2ecacafa94e9ae0a"
                )
                .into(),
                body_root: hex!(
                    "a18d7fcefbb74a177c959160e0ee89c23546482154e6831237710414465dcae5"
                )
                .into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("7d42595818709e805dd2fa710a2d2c1f62576ef1ab7273941ac9130fb94b91f7")
        );
    }

    #[test]
    pub fn test_hash_tree_root_beacon_header_more_test_values() {
        let hash_root = merklization::hash_tree_root_beacon_header(
            ethereum_beacon_client::BeaconHeader {
                slot: 3476424,
                proposer_index: 314905,
                parent_root: hex!(
                    "c069d7b49cffd2b815b0fb8007eb9ca91202ea548df6f3db60000f29b2489f28"
                )
                .into(),
                state_root: hex!(
                    "444d293e4533501ee508ad608783a7d677c3c566f001313e8a02ce08adf590a3"
                )
                .into(),
                body_root: hex!(
                    "6508a0241047f21ba88f05d05b15534156ab6a6f8e029a9a5423da429834e04a"
                )
                .into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("0aa41166ff01e58e111ac8c42309a738ab453cf8d7285ed8477b1c484acb123e")
        );
    }

    #[test]
    pub fn test_hash_tree_root_sync_committee() {
        let hash_root = merklization::hash_tree_root_sync_committee(
            ethereum_beacon_client::SyncCommittee { 
                pubkeys: vec![
                    ethereum_beacon_client::PublicKey(hex!("592ad40fcec5c0e70f4d6663e3b480e181db52820f69878e3153fb6532eb493d20818d0a5db416df916d4f026f40c713").into()),
                    ethereum_beacon_client::PublicKey(hex!("fd9697146d92b66331f5e4f0a8e40805f39d3dd3480b0f825b94c455036d3d9eff40267d1e1768435079e5cead9ee88b").into()),
                    ethereum_beacon_client::PublicKey(hex!("9174ef2d8f23190c4e7ff6da77b715e2f80e55ef58c48f980fff6b1a363ac36e8e03e626ecf066443ecff3b5b8b09603").into()),
                    ethereum_beacon_client::PublicKey(hex!("7374240fe290230714325d3e6686c91ad79417cb4b170f00479b5d37e2c46607d8dc39b851141b87a5008d90938e82e7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d5c0166b30874667ddd825fff632f1af95b58c0207e04ff0f657d371dd04b1d22127bd22e6c5fbd531271dd192c5e3af").into()),
                    ethereum_beacon_client::PublicKey(hex!("594c0812a81867ca44d55a45a33d33be1d63feb15bd438b252b63bc6f4f3a89ebf5759827a1f8757f2be85f9c25a5bce").into()),
                    ethereum_beacon_client::PublicKey(hex!("82956ee4399ef8cb97298c2c250d697154e24b884a48614d12e624502d0fa3edbbf5221e2f4681f846080195a9a85996").into()),
                    ethereum_beacon_client::PublicKey(hex!("7b865684ff738da3ba8b6a6927fd2a3dea1bb3ab42416733136426985fdd973fc80232c8f8ff7c303a010a221e1f9339").into()),
                    ethereum_beacon_client::PublicKey(hex!("e30df7b9dc4d666dc2073cc9b682a81ea1e0dc09e96b151ad36074878a37eb0bf5c0d9942d4cece1dd4b32d2399a5ddd").into()),
                    ethereum_beacon_client::PublicKey(hex!("54d569acbbe1d06735b5b727029bf08fd2aed8322888db7a6930f9b827a2a0ac57a940a7adbba5a1f9d12fac5c33c265").into()),
                    ethereum_beacon_client::PublicKey(hex!("aef15f3d7cd04342d1c5ede2cb7a51ae1bb9d4db55babce4b6fa097d97c61d6b9e02fee49d2d4b7741f469a38d7c46d1").into()),
                    ethereum_beacon_client::PublicKey(hex!("d56544f0397c0f92e4f31237e780deb6b6cec0e6a0c8dba06a91e06b8bff2d2603342b2cf79fc54f0c47dcc9ccd6064b").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e89b0b766d0b04de35423331d8928f5cab2e94e63c843fb7ef0641d1741e53312f7911566e715e1e44f66b9e24e2128").into()),
                    ethereum_beacon_client::PublicKey(hex!("6bfdd061268631a959a57eb6eeae7e10954c8aec58097c3f259f6ada84ffb6208eb1f9f490fcdfbfbd28d3c9cbef046e").into()),
                    ethereum_beacon_client::PublicKey(hex!("dcd844cf90f873b94f6954fb348d75a5ab862f068501fb8e89d069803bf8d071fbf986da257db2c7200f22fadadd2350").into()),
                    ethereum_beacon_client::PublicKey(hex!("ad25d61f1e4902f7c66eba8d897ad54f460f9a00ced0846513841b6276b54b1ba2ac0654bee5d66c239250522a3859ee").into()),
                    ethereum_beacon_client::PublicKey(hex!("1dd6f1c0ff54f1cabb4be851201073295e6adcaada87268f5e2afe9a8bfb78321b6b3b4562884a74f0b5f9083ceb33ce").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c27cce73f31bfe20c96f3f4865c4a0792ccdebadb501accc3ee4d43d7f18c0387cf08eb0669f8e736e1ce71fa0fad5d").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f419b8494f7d0a2bfcb802c8cb0453fae02962a923817516ff30dee3342966040cc47463328638b894c18d2116593aa").into()),
                    ethereum_beacon_client::PublicKey(hex!("3c9ff7a14e6cee517bffd523fe1bce809c9b92f14f5b68120d630ddc76033d56007dcb785c336616421b3418d220e9d6").into()),
                    ethereum_beacon_client::PublicKey(hex!("67d7a8b53c1095d5c82a686e67457e88e19aabac3852279d4e5e21f77bc61a9885dc922bd44571eea3905d27894c2b0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e6215e87002db22be50bd9aeb85f4219e3bb5dc2d136b1c8cb732934b615095630909ae8362fca32fe74e17a93bc84c").into()),
                    ethereum_beacon_client::PublicKey(hex!("775425b7dd5308588a1d79e5b016946054c4d2977ef1a0ad54529ee34ba9b9a707632380c5499c6f42be0e11ae91f047").into()),
                    ethereum_beacon_client::PublicKey(hex!("41005eab7e00694116bcdcd479a5257e7b6f8ac992ef36fbd0b575cdce58ec45ec7d5c7d66fb3540daaa8f775442cc5c").into()),
                    ethereum_beacon_client::PublicKey(hex!("33c01eb38a6e1514b636b6bbe91a67c0889e3bb47cd8ed5082b1ff0460565cd3343991e4ffeddc25d8814191501ecb94").into()),
                    ethereum_beacon_client::PublicKey(hex!("69b1da814d330416ec0c0f54b8a24672fafa5bc64b8916da774a56aa8c9e68f69eba9bf6618934fcb35cf86773305d7e").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e53f3a4a02d7f41db7c41258b9a20e9b90cf7c49e9e23a7f28f9de697a758926fc06f05d7d2d11fb9ec3d6c178b175e").into()),
                    ethereum_beacon_client::PublicKey(hex!("66fcc2ea21cf64f61be9a8ba4f95c99a11d5e184a3b292311aa01948694a85b3824f9ad51eaf2f5a3111a1593e8ab169").into()),
                    ethereum_beacon_client::PublicKey(hex!("aa1d785d10c58c8acacc9282948ecc6837687eef3de96ab05d3eea8e58d0f791362cbc63e217a1c8ff3c97430ff709fb").into()),
                    ethereum_beacon_client::PublicKey(hex!("a3dccd49dc50279f425cc26c1c53244eacb2d3030999ca967d0b75d23b31d1dc57816387155a84eaf5b9ce6dc85a2411").into()),
                    ethereum_beacon_client::PublicKey(hex!("bbee48cb2ac01e8e78a992ad3ad3e771ee56079c88d747a35001c5a17bb865e524b76bca8970264fc6a3b31cc42d7223").into()),
                    ethereum_beacon_client::PublicKey(hex!("881d43305e064c3b91e7ebd23f792eb300f44bdadd59ed4c318d1c1c43f3c6e19f1a480ce869902a07da86abe15f6409").into()),
                    ethereum_beacon_client::PublicKey(hex!("59211736fa864d763f33b79609c42bb7c009f7563ff0262571acc68d93628a5db70bf0902dfa2586a383f06c9f6fb1f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("3e91b39420a2d698c427cb47973d9def12be2829c799c75f75d8eb23aa0f9a1564a978fcb0f6b3d6d97bd2f3c49ea068").into()),
                    ethereum_beacon_client::PublicKey(hex!("34456b26307fdeccb0596d87967a13c6520044954075893f20e206c11f1fef917802df99740eff63b4f95fb03678175d").into()),
                    ethereum_beacon_client::PublicKey(hex!("46b33cf4e60e94c62f0d60d19b6be1b3c5b66543c2f0046f342b3a411d45ac6ed5511de898c456ec0d7ab95ad38415c5").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7275fc4e89be58d8f4191bb0163c0be0739a46e48a1287f11ac5f1a1078e91529d771f320fa39ba6ce921805347c4b0").into()),
                    ethereum_beacon_client::PublicKey(hex!("05da2f1a0b043f495ecc34694d6980bcd3140f6b012b1855edf8409c0485573c32f4d5cf84d6b924f74e0c4614e26191").into()),
                    ethereum_beacon_client::PublicKey(hex!("f68f046604a0e4fb66457bf1aefe8366e187f58403de9d2dafe649ee0b94a88f0605df07db572142899c1950ef19b611").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fdd4c89e6750e7f9be4db00a67af0298574bcb1262e20baaba20d9876af9384d0672a278e7a592509f6aaa68d6ea39c").into()),
                    ethereum_beacon_client::PublicKey(hex!("d160e657ded2ed693ba0aeceafce00a45b6f6918ecb37559533910db5aa2dbd7a4ca2d00f8f500b4bc3fdf8ff28a00f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("c80f5c3907a3aeb71834f15b9142230adf0dfa0346974dee0efe4fe126304d8f40e731d17de257c5ded5337034d4e157").into()),
                    ethereum_beacon_client::PublicKey(hex!("774b9a91584563e453be5f389a7c803be33c9043b72860107a36917fd086605be53b04ca506ce4201272ca67326f133d").into()),
                    ethereum_beacon_client::PublicKey(hex!("35a346d536f08a6814d4eaa516ee6d76790c1d441a134bf6ed90efa634acf805b5e7f6f3aa21036ba6e73ffa9d1fd71d").into()),
                    ethereum_beacon_client::PublicKey(hex!("39748096a77d9632237541184c8b5470feb99d823c3f76d297fd47d9107a0d99f00ceae4ff8d563d8184e1aae33f0037").into()),
                    ethereum_beacon_client::PublicKey(hex!("833f50691636c04cf0568166ca2b69a2e71265b6454a7cc2542a2942614964672c2813b2c02b1ae50078f9e87d7dd2b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fb2185421e6bbe56bfd27f3adc10bce06f6374028ecfe4c59b0f21a2697565e7a40399e19886e3b646a1a74bc34f0f4").into()),
                    ethereum_beacon_client::PublicKey(hex!("8926ca48be3126257a5fecc6efdb623b70fb0a7fb8086351f877849bb868f0c69a0ca8f1cfb97a67bd51a479dde4a1dc").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a977485281b39819ae110aaf19e825af8621dc7c2565417c22e8285840974eeb6de83c23102996bca3e419b00ca73ab").into()),
                    ethereum_beacon_client::PublicKey(hex!("26294201dcbfea5675180a5d5baeda262e0fe10c971842268224e9df1572d21975a210b0cc33a46222519a40441dad9c").into()),
                    ethereum_beacon_client::PublicKey(hex!("e90202aa6ac88c6a1f3ec2ab9bc2267b10fc3a5766e38664ad0a8b6328ddbfd5738cb3e884a97527696c76d59c70a4f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("d1aa645377c0b2de860f36aae0f8122cc7756ad92bcc3b0235cc6b89704b7d5c01c1584f6681137f92de2403918a7b8a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ed05586aaea0dbecf5a7ea958cea78ad2ea01ed27fcd6d1202129f26c2d189c6525b575de4256fcd6ce0da575299ac4").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ec65e0317b32719a41798302a3b64a184bdf1b56d3315aa9d8bbc4b4ee60d8c26fc7e2d4e384873263704e462ee82c0").into()),
                    ethereum_beacon_client::PublicKey(hex!("150aa17e6997bff5b0f630d1558e24c4276f57f2a286c0adef5b6e5b04eb2f7f490e3c05be745ac75e28e9938189eda7").into()),
                    ethereum_beacon_client::PublicKey(hex!("7817e17b91f98c23c10c193679a379c339dcec598a0866908341e20ba08e558f49973c40e33d8b189fb75a93f7a11a5f").into()),
                    ethereum_beacon_client::PublicKey(hex!("92a239b186bc2f64e17d4e6fa9e1e7e72d23e0da131e1464aa364ad20829dd95483e2c3ec8a66c937130e5769fc7969c").into()),
                    ethereum_beacon_client::PublicKey(hex!("c030e963a0e9cd438f3f342dd35bddb98756d9ad1b383b34f3984b4e45e0ad511281ec57b876d2aeda07306ad9148e1d").into()),
                    ethereum_beacon_client::PublicKey(hex!("2df883b5ea2fc447d970d37ab1ed00afc05877087f827c6b0972d91e0bb6e4b2e31fa0d7a3433c5e49259278793196a4").into()),
                    ethereum_beacon_client::PublicKey(hex!("3e9179efb4b7db1fc7c3c14a8ef08c823c724ec9c75d63dc4d3670b062a288f24d608de490f5a090584bae36ef787742").into()),
                    ethereum_beacon_client::PublicKey(hex!("4561a292b2d11d365941221cf4b35a6c0e01cd20355db2cb634d57afa570b15ca03d21a9ff313522d40fcd3e7c093a22").into()),
                    ethereum_beacon_client::PublicKey(hex!("a7b1c4ecf19324d3579c43217173b1d6c72c5efd572a9fdf468a959cad0688c161025468702751ac31fe40bc3d8c09ab").into()),
                    ethereum_beacon_client::PublicKey(hex!("b7948546b20c78627d5315341ec1dedc75c1fc8c8ded9a11b6c43dd089bb3c4e4b4a01d5c052466114ba03b35852c5be").into()),
                    ethereum_beacon_client::PublicKey(hex!("3d76765417d16c953b267e094560a639bf00089b6ab74b7a35c8dd97994553c6ccb7feb1623023d1953264b79bfa7eff").into()),
                    ethereum_beacon_client::PublicKey(hex!("e9021d4e9b8b82de6714155a7aebe86bbec9afe96d59c34b8f0da7d3ae26b7710d317267145ab3f9e4b6de1e9a9a1ba2").into()),
                    ethereum_beacon_client::PublicKey(hex!("451f7ade7504aad289faa5d47aa4f3961b511ad66fc29c86a175cd8b86550cfa80f461ba1d07087545f52f5c26796398").into()),
                    ethereum_beacon_client::PublicKey(hex!("ef5a4dff11da3106b57f545bffa7903f5a76e6f11e3b7373e4dd9cf8692985d280ac799d7bbff0b7e1209b2a289117f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c9ce4d6a5e7c62ba628861b8b2a125d4ddebdb2643c89e47bc7a74e1176d6c53347d81fa9cb610de72d7382bc42cbc9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9be385450781312e856fcdd2d5f33cfd9f41e0139a7c1fff20066e7a44da4d98bff0a20507677ae0aab046577a20b258").into()),
                    ethereum_beacon_client::PublicKey(hex!("dfc86865deca183ddb0a3a5e239b4a96a5bf0dfc9d0238654d9f98aa499cb24ee27ed53c8a330823e38915340896cb19").into()),
                    ethereum_beacon_client::PublicKey(hex!("12611949e77eae716e210fda053c4581ffd8c746f71378a088a4d9098900de1b403798a17e4ffb99e7c09331b766954a").into()),
                    ethereum_beacon_client::PublicKey(hex!("03c33353b56c37fd5d7ecb9a2621e806c78f1aeb81e48a56a7e80c4ac13bfff3817a457a2ef7be88d84a3b7643049b84").into()),
                    ethereum_beacon_client::PublicKey(hex!("6b346a9bd9f64f3648aaf5667adca9d58297612563a9de50d65cb2327b13eb1447b4df2914391585a51cd1006d6f8b9a").into()),
                    ethereum_beacon_client::PublicKey(hex!("54711b443b7269027f5d8785b15ad00330648c0af885d4e55c57e805ed0a58014d6a904f15168c0845960c88e6484390").into()),
                    ethereum_beacon_client::PublicKey(hex!("6e465cb124b6c7b22a6fc3cecfa6285286b87499fd68962c179530abfb7633b8d277aef04a5da5a67244ac7d221454a6").into()),
                    ethereum_beacon_client::PublicKey(hex!("40d43031189d535ebad1f67244e819a472947cd90f7d9ea5ad8713955d7757a04363045e12d650a0537b57f64c06265b").into()),
                    ethereum_beacon_client::PublicKey(hex!("aae725a2e72b93105c0dfa90bec925a9342cb385c98cd55bd43d310b30df2e6e3c68316a8684b9027ecfa074259aedb5").into()),
                    ethereum_beacon_client::PublicKey(hex!("6c16482741c737d33381fd406b119f5ffeaa56c2afb11feb22af75179aaa093349eef9ecc84fc2a3f8d803bb9dc49343").into()),
                    ethereum_beacon_client::PublicKey(hex!("e9df20640774b6edd1d0af5ca87fa11e11c9dbc4c723d4380e97350e1b2080871e34961e3bf1a966b4271251c4fae4c4").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2db5407eb2f083c176594527266189ac064df732e02f209f5bf5cb8e392ae0ff1f9e2dbca5e938807dfa1c91dd5d278").into()),
                    ethereum_beacon_client::PublicKey(hex!("06ca35839eacbddf3141777b627d7f0d8e75a0f0ee50d30c639dbc0e86275e561566af9ff4af282e24fe2ce12bdb1293").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2b1596c0b5095eea22d3a1e248f74ec4cc1d4ab5323a53f04a454b11bf4cbcbb7d5e8f67d1232ab9ca7a2dc821dea21").into()),
                    ethereum_beacon_client::PublicKey(hex!("9985a2a35d177203f370d1ac24c1581aa7e46cda8eca5c35e6dd49425f1d691c4c6820ddb413f93067ea382f532631d3").into()),
                    ethereum_beacon_client::PublicKey(hex!("29d11321174d8f1f5ecb3d28bfcdd0506bccfb447892d3920bb55ec7fa087263cb5b4d322d40cb63833cf125398535a2").into()),
                    ethereum_beacon_client::PublicKey(hex!("d6edd7af27555832b630f4c7403bf7ee83200b9fc91ee1acb15d32d9567a1c32ccf2205589f64bcd641c7d83af96d1c2").into()),
                    ethereum_beacon_client::PublicKey(hex!("9549d61156db24fa2fec8178dda6e000be4dc4d0402b6e59739ccc38a3dc0a5e58dcc0c701cd439d100553ce8a00b8db").into()),
                    ethereum_beacon_client::PublicKey(hex!("99c3db60f2ea4d0237583f4e1f5051c31b0b998c8a3983e31abc498c76449234b608a2210cdbb2af57f435f720d3ff21").into()),
                    ethereum_beacon_client::PublicKey(hex!("3a30a77d28767ca707aed12587f9c6850c3ba15b7881f0b78ca67639b6e08c9f74196d3dc91d41a16aa404f401bde9d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("aaf332dda99e8734a0d6d25092553a053da334eaaa4a1c27cf4eb11ee73052dcab482dc4ede1151909d40a26053ad49c").into()),
                    ethereum_beacon_client::PublicKey(hex!("ffc3623230153c5187b976a9335ac8990280f0ae6299dbccee9936989b9ba254801c8eb2b54991500027c0369479d26f").into()),
                    ethereum_beacon_client::PublicKey(hex!("6d4c0adeda48e24b598b84d9ca0d4b888bd6125ac3973809c4c583284c9ac84a7bf1afc5e0d10b0e37250b25df966205").into()),
                    ethereum_beacon_client::PublicKey(hex!("779270764bd2804d044c4a634e5ff0fcd157e986767a2a16911faaf9324db71be8db672fa3d3e67d726549bfbca2bbe3").into()),
                    ethereum_beacon_client::PublicKey(hex!("84da89873c5e06161f9fbed011be93579ce6543dfb0d848d9450cf1f4c932f82b2661e2c1e28ccc8c85e06a5b1d13704").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b9f404b44c809ce70a964b465e90da3aad04be36cf444f80aaff7adabd36ce6e4b1521f49f7d37493219cbd80ee152c").into()),
                    ethereum_beacon_client::PublicKey(hex!("3c39774228c98051b2e2660d3229746857c006c2d0cee7bd0c8c9c4626fe6c83cc21d7113a7d46c32da00281a9ae71c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("67e903a9f8de3c19d4dd0593aec5efff05d0e1bca96baef18d0ae0ce43acfdb06ed50e5fc2569bef166cea66b0ddc1c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("5238b6a910f190c29c2de8251cc9b1093e533185b21f83a7354823ec05716ccd39198c225f05ec5a86f739ba3381b843").into()),
                    ethereum_beacon_client::PublicKey(hex!("c0296847fe8eab3547971b2381fa96e4f12c9aa31c757421bb32081a250aa9ded1ea4e9a0632c01144211c52277fc3f5").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c212df6d5613ff8412acb4ee33b03f3a82c084b5ef7bb36df27f9a2af9c93bb18203175c4726db1a2aa28c2e1a3a501").into()),
                    ethereum_beacon_client::PublicKey(hex!("6914ba0678a28189fdc6f008ea5050392d1221f0456833dee8033bc327838810077e8a6a8dc924717396b6fa3c2a1072").into()),
                    ethereum_beacon_client::PublicKey(hex!("daafc547bf228ab800104aa5c3bca8f7e4ce89f2d4ba4e54c2c0b3e4dd26ac505550db8106f97058e8fca5bbbcfa09ce").into()),
                    ethereum_beacon_client::PublicKey(hex!("abc0e3a7b44c2643d00328a760a002d280e022ccb390694978822f9c4b426497fdf0de9f12696d9428d2d0bc90c25a55").into()),
                    ethereum_beacon_client::PublicKey(hex!("71e0902422bc35043cfa16869b2136df6f71459d9cc92739ddd750e9f3aeb9e5cdad9943a4885b4b6c3f54fe536c0c27").into()),
                    ethereum_beacon_client::PublicKey(hex!("20988e1b6b1440e5252a289666d60ded7e0cba4f4f8ba0092a7930a0a1c55da458e7ef86dc52c4713a0fc9cae96b053b").into()),
                    ethereum_beacon_client::PublicKey(hex!("4d55dea5157a5eb5abb901f211b826643fa002a1307f6409b77c49649b2802b5b9eb2e5f168a0359b31efdc4295eb40d").into()),
                    ethereum_beacon_client::PublicKey(hex!("14eabe5536492eda3c9e43f2895216aa16fbe63b99efd81614e97d5b359637d555e514cfafc620a52134f53cdd4fd72f").into()),
                    ethereum_beacon_client::PublicKey(hex!("62d1c771b75b6514cb1dc7cc2bfd7f9629be147ad24cdefe615361fbb20d197994dc505816b01294c023412767a371f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("ddc91125bf98f9db27cd1ea3bac1a1cd5477ab1054252fefb95d41dc9e0c2f5b194bce3085d910860ad9f32be964ca47").into()),
                    ethereum_beacon_client::PublicKey(hex!("457a01c1596df8407e7619fd3b9b7e1e7743bbc0144bb514cc6934ac98081d6e4269b23ac40989ecedebcf3b010f6888").into()),
                    ethereum_beacon_client::PublicKey(hex!("9954a816a68b2af847367844790fd4d7460d2bb6130cea83f7c8798c92aa27a5e104d20e269fb48823c66f6951e96e38").into()),
                    ethereum_beacon_client::PublicKey(hex!("47fa6c841bfcd44a0368f25b4a8faefc37cfc9a749800f0888a99d9c490944150fe23ab6290352efbccec55561b2b96d").into()),
                    ethereum_beacon_client::PublicKey(hex!("99e0ab54807fc7e848ce2f02f0b57ba752761566836abd95cd8806f83623260945c30316b712f270d3cfb63faa970d7e").into()),
                    ethereum_beacon_client::PublicKey(hex!("a9f647d23fc073c845d36f840a7747a91a8d7fe93c727292527280e2ce91668f278d0295291a658886b47fccecc892d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("ae228cbd2c63bf265dd7af534114ccc0fa180b6b9fd6f5e6d69f3aa266b876139d27df8bc06e19e6850988fd43b223ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a442b6fddd08efdd2b22074b2daa8978200c960e766dfab3891f4d0f7c8a80e4c9aefdb0fa9acdd4ef57e700a1e17d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("98c319e76300e2df833003b96293dea393bda83cbbfabf211ca9e74c2d037e4b592893a915eef4b3945c57b155cacdcd").into()),
                    ethereum_beacon_client::PublicKey(hex!("ee9549cfd64bfb3c14054c115f51c9edfa11aef90a27a05d698dcf09f5fbcc61d24469e64640ae272ac176a8400150d3").into()),
                    ethereum_beacon_client::PublicKey(hex!("40e6e84b13c80f7c9152f7b217dbf10aba2c83f95a457dcd29d2e775cdd9d3c8d2978ac5d10c6b42bb3bda636c5cb268").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b47a82f1977a0e4f4f6f72f1763204cb8600ca49a6d2d8471ac9bb24b7fba58cb008bca19acaa9df4cb054bf5744461").into()),
                    ethereum_beacon_client::PublicKey(hex!("b934915b598c4ce455ed2db152eada53a69aae79f180c019fc4b111adc25ea6066f0a7d64dd7fcd21cd991266b8ded3f").into()),
                    ethereum_beacon_client::PublicKey(hex!("3665f08c33894e0c486099387a341addff19434e5d994b0fc3cdaa3132cf7deedabb9cd412a4d2ad23193738f828ee5a").into()),
                    ethereum_beacon_client::PublicKey(hex!("70d43455c1979c7ef47a2af28cdd3cbfeec99381ff3b0b265ab0ac2822e76fcb3b8d150918a2844dd4ec7f2e5e3e3d6e").into()),
                    ethereum_beacon_client::PublicKey(hex!("dc4a7161925eb9d638abf47f7c52f11d6c5a4c0b6bd22acdedc90fa1aa4d38d8d2e3135fa9336047881245e6dfc02ffd").into()),
                    ethereum_beacon_client::PublicKey(hex!("74e3a9222eaeda103b4e258de5fb904712ec32a7508464445c3b850a8faae3e7baf49e9d94bbce113a324bb7326259ae").into()),
                    ethereum_beacon_client::PublicKey(hex!("d1c4f1797492b5174b1ce79cf0e35dedced7925f3ce2f63e8817c280b2ab1387a60fc182fc063e12f0b3c186b4875b71").into()),
                    ethereum_beacon_client::PublicKey(hex!("5efa696e2ad91bf24134c6791ea91e32a17536f6d9b724906a6d82356b1ca2d288830300cdc3c4fd713e253dfaf370d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("4fa67acfdeff8cb4c7773ff11dc106b1225e4ec44bdb8857d402366202df51e6049b6d9a3a13aa9b13dabc9b2fba9c6b").into()),
                    ethereum_beacon_client::PublicKey(hex!("b220a5224e453335f170cfe7dc8eb1e86dd1746dfa8cde4abc115621f8042ef789ba08ac0b460ae58c38df5523b83cb2").into()),
                    ethereum_beacon_client::PublicKey(hex!("eccf1b85afbfb973aba29c817e18041b54e8ca72d663a0c46c074e0138a2bb73c192ff49df3a3dbc7a3d98f322984aed").into()),
                    ethereum_beacon_client::PublicKey(hex!("83e87639baf3593a6f1c095a362d1b103190a9f31f1f008619259c148de4cdfc02b14490fd7d5db36555c74caf81d48d").into()),
                    ethereum_beacon_client::PublicKey(hex!("66cd922fd2adcf753d0a9f7101dc80fdd5e47355309fba4c837778992c5cdccb6128674a11b2f1fb12ed9114a7504197").into()),
                    ethereum_beacon_client::PublicKey(hex!("582b88e41895a018714532430022de480ef109055200d17263c7e452804351953f4a2d10082f61a7532ac124a90e764a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b05174dcd3bd22b82b68c5e43ddc51e5cb0eb491801654a8701758a6f25075803a9e83bc06f29539b129817aa1124f68").into()),
                    ethereum_beacon_client::PublicKey(hex!("a270ce53f2101193cebf79564da6762881c7ab76f5b2e51adccbc64875c0c276e6eddaa701dcdf9d5fc508ed7d144b13").into()),
                    ethereum_beacon_client::PublicKey(hex!("9da80283fcf4ae1a382efe93f4a0251b2f1e9c4cecced6ff82e489e5b795551e5551e90ad84513872713ab99e97569f4").into()),
                    ethereum_beacon_client::PublicKey(hex!("d2627b8cea52cf7efc4bfee5e0aeccd3f5f74e9d96b827b7ef01f56be0a3ea155bcf39d02b5994cd239c390ff3ddd12a").into()),
                    ethereum_beacon_client::PublicKey(hex!("691423de74e583d34ced94b0cb7e9d90eb0badc034b4bd57b2522a05c584560de877c2713e960cface373250795efbb1").into()),
                    ethereum_beacon_client::PublicKey(hex!("b900d27e26a4ac2cc592afe43b55b127f7e212cb9cc271bcf773a14b9701831f2d26937230c13e54b8a26792b38ef852").into()),
                    ethereum_beacon_client::PublicKey(hex!("4c3cfd02e0a08d23d27399c128ae13aef5c3d6de95a01df35ce650f3381315c426c7d55d74ee29028902ec830e7914a8").into()),
                    ethereum_beacon_client::PublicKey(hex!("85044d41872397360a07c728cc23c6608fb8659556b909881b4d4cbc0855c8086236ed557a2d29f20f10f93eaa9e853b").into()),
                    ethereum_beacon_client::PublicKey(hex!("83f7cc9ce77ac055459febe75e112c1977eb0bbe10cee86660f61b2ffe2f21b88ac819bf5a9f75025e1efef5f535059b").into()),
                    ethereum_beacon_client::PublicKey(hex!("03515113d9ea4ff33ca8d39f245c537d4e208ca8051233f2e8e6310b11113c041ec6b4cb056a7c6719535dc90a0d1a51").into()),
                    ethereum_beacon_client::PublicKey(hex!("dd60a5aebff91b751904af5d41a9a7a889d056d6d3aa5d9a51cb53f185b55b9d360612062d981feda2c49d990a3b7ed7").into()),
                    ethereum_beacon_client::PublicKey(hex!("8632b93ea8088006f21b819c6b37d62f5de8734b36f890216eb4e64a03b317a26712b649e0b1a6707144918dbb2d1992").into()),
                    ethereum_beacon_client::PublicKey(hex!("84c7925a563467db0bd502db87ec182fb01c3bb4981eaf933dcd68bbc3ab4a088a956f31990aef5371d038fe68c57ed4").into()),
                    ethereum_beacon_client::PublicKey(hex!("f03add2f7f65b9299129a6725ebbe8a8f80b2256bb3984fc574cd970a58f7fef3eb12d816ba1a64896faf1cbc2dc4b40").into()),
                    ethereum_beacon_client::PublicKey(hex!("792f40be7464ada3b8283405993ddd5c374df5acdc1810da4992c69507d4c19d6aa3ebbe5511c300793dead4f2981f85").into()),
                    ethereum_beacon_client::PublicKey(hex!("27a7a587ff7d68c1f6e5a6e51913aeed47f007da3d6914516a8b1cea931a613eebf6e251a1ab96ae5d49de2208f92f6d").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a1f80834c47b2b128197fcf0439b540b3dfa3f6a6239af5566f7d6712e36aac04f7b1c1c401d0d68bf261a801e57b43").into()),
                    ethereum_beacon_client::PublicKey(hex!("916006e47fd50e504916cf847e4355c4b209b4173f73a0f0d73da3d7cd3ea01ac5c0702a2e505a56f09ffab0c7afe77d").into()),
                    ethereum_beacon_client::PublicKey(hex!("07ee37c12280ad367d8b3e125d59cd4e223bd030182fa6ef655f38cc1d126603d228d10b0d0725d9238f62674c72b783").into()),
                    ethereum_beacon_client::PublicKey(hex!("99e840e2b15ea4c8a0af9df164ffaecdd767e0ca3b74b16cbe68efa10c1eb427b13af36880ac3fb2cd43eb5ac4c0fee0").into()),
                    ethereum_beacon_client::PublicKey(hex!("be333e4367358a301c56106ecf7f73a195d43a6d3a617382f02c9fa76a686a52859226b40a7294c11d79dbfb538fdb1d").into()),
                    ethereum_beacon_client::PublicKey(hex!("eecd34ffd24036c47d3730ad7db0419c4bb068db585d78fbd2abc1082302309cfd2a63a52b889be1a4c4a3b706b69313").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e377a7c137ee0941f12c283f7219d0366fced6ad4487f232ca1e0259e23c6529a3efc6e407009be0e8f9e422a2d21f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d47f9bca0f2e9d9c630475b61fdb3eb166d7d13fde030265ef7111f882a31e60c9af2038c82896ba66db8d0dafe59342").into()),
                    ethereum_beacon_client::PublicKey(hex!("b23ff52d6a2581fe6666ebaa9e974720fa61aa76339348439cf6b438501ada02011e359920a332f4ca313aa9d6793e33").into()),
                    ethereum_beacon_client::PublicKey(hex!("a6cfedbeddecd02af2adccc34388e903bc902cafcb69250c35a72cc23e6b6fdd354f025374939aaf83b829a9c6a7d630").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab01e7831103a563c6c71ff62bf9ae6f12018f5e13a9c77bdc723e7e6c00273a9c06ce12e7376705c346d6977777b8f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("97ad201ddd583922b820d75257194d9bed542563266478d1ffd4258714e4fe0a579bdcdd869c3d1609059817dcbdc97b").into()),
                    ethereum_beacon_client::PublicKey(hex!("7dc8dc3502d8c806467b11604c358f2f4a8b373089dc8d747730a1b404c4632602496746ca009a1933571515db72fc37").into()),
                    ethereum_beacon_client::PublicKey(hex!("98ee3f9ea03840eab045c327afc9e5c6ca29252e15e430425f806c5107410c17f367a3318481f15541f39f0c8830c2f5").into()),
                    ethereum_beacon_client::PublicKey(hex!("6631a1b7f57469accf1d6f62ebf71101b0c63086b60eaecd8a4cc610fe6d147b36568cd43514cb00a939d3e3e9f5f64c").into()),
                    ethereum_beacon_client::PublicKey(hex!("c04c762b6ca4b00710620a0a823f0d45f0b70de6de17d11f17197a894912f0c2472b15a4a6d77b0da150f177fa671d43").into()),
                    ethereum_beacon_client::PublicKey(hex!("59a29648bd074ba0b58b1be1b6e624c39108818f24a2c205654846c8a5deaa5dd8223528287660c44380c4577464d989").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2d9ed5e331ac7a154da9a9fb8afae24128834e694691ccc045aa6c7626a34b5cb5b27a4febaf65e5c6e5a4ff01cb761").into()),
                    ethereum_beacon_client::PublicKey(hex!("97252851d18da63138229bf5579c403992e70787b184c00a92c52715d1c2ece7ad8dbff4e0395bcc59af34b90c1a55cb").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a129cdf6a308aa8945aa5976d37d694653e8a8b5e9c1045e9b2998bb95985de384844fa4c93f0e293ac02d2c0fea10b").into()),
                    ethereum_beacon_client::PublicKey(hex!("39ae46323e61ce8fc7c00a5a1dea76dab333e45e28cda9ddf4b3f070a0788554edd233f6f9698cdb789978bf6d6a1c50").into()),
                    ethereum_beacon_client::PublicKey(hex!("005e9f18435fd8619cd7b172fd9cd5922333e050f6c6e225d6b9d4f7f8758cf464a2aaaf659da1b99115be05fa2d712a").into()),
                    ethereum_beacon_client::PublicKey(hex!("5d18ed5c1b7873925bf6f977f311b9106ff21080ec9589ead61105c3056cddf654bec257ff1f52fa60bb55c15a7684a3").into()),
                    ethereum_beacon_client::PublicKey(hex!("50713a00b7143befadbd9c13eda936246c7518fb6eb1a180a3f198d1b671ff2ba0daa577db1f8145b8517a6ee992dd11").into()),
                    ethereum_beacon_client::PublicKey(hex!("d55f23269ee3df4f0c809bf0a4492b0ee45f8e54ef61aad3662ce835663064d52cdc8b0428c1d02657271b19b588b770").into()),
                    ethereum_beacon_client::PublicKey(hex!("8f4bc64bc204ad66dcf4d217fd2298cca01198c6ab187604c0142b83ee4b96380d240f95b44142b09c101fcd7719b932").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2eff3dedf0fc5b96fa92335a273fbe1380aa14fb305d3ddecfb45c9d1205341c5247f1c05d01ec74dc518b2faa42cc2").into()),
                    ethereum_beacon_client::PublicKey(hex!("506caeaedee1bafaac3184596f7a3048671c48d2c000784fce0e5414785be22e9cda50f91b66234e8709a0158d8a125a").into()),
                    ethereum_beacon_client::PublicKey(hex!("a66df170d6df7a816586c38b63fa44e1d2d342cfd479b3d98e3dfa0e51302202005e52d69ea498f5b38697fc21335e47").into()),
                    ethereum_beacon_client::PublicKey(hex!("a8efb7f4c8e7adf628ab6ce6d41aee80037d300a3cafe39381fc0cda4178f31b342833af90cef41dad8ccb6f35c4e918").into()),
                    ethereum_beacon_client::PublicKey(hex!("9f6205e44f238add8772e8512c4a6113ce68668d7d0b5de793b0c8311c7abc8b9d0866c2d8f8468b8b64bed0e8bc5b0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("a105bb8c171fc59f169842e19ca67b65ecf65d49bdafb0c15e6feaab3aab276ab98bdbccae7c5687fc86a810bd696ec3").into()),
                    ethereum_beacon_client::PublicKey(hex!("a62b760282c17398d441b3b28d5a662fb4d422d3f1d19eb32e802e1762e46fd7bdb0249fac79d27a4ffe93210d132836").into()),
                    ethereum_beacon_client::PublicKey(hex!("8cf9bccae19f190d1878500d1fc2b7189185cc7de38432e798634f27e8c5dd4c63913b67c83633789f93ef931948b402").into()),
                    ethereum_beacon_client::PublicKey(hex!("1c0b102f99254eb527951f2e5919b554cd89f9d179db24c0f6fa50eeaaaa1a6e09e2265f836f20c0d01ecb7fba042879").into()),
                    ethereum_beacon_client::PublicKey(hex!("85baee3657309168bd587ee03fbe3098e64dbeb5501f232b7ee8dfa7e194c97e0b07f491c63ab50dcebc26e787c55eaf").into()),
                    ethereum_beacon_client::PublicKey(hex!("990e8094b73cd024c969bcff6579cf384d772309ae5bb67f73ccb8c14fc1772488112d6da06ed1bdc259192047d3c1b1").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5f7fccb1715529b08f0f4bd19e2dee442e05dc034ce83abaf5d3a8b730781fdce5f191b3eec88138a778d54f7eecb0b").into()),
                    ethereum_beacon_client::PublicKey(hex!("5edf0543d2fb0d0da48b6513315324a187ccb121e4dc1cb0e20aed9aeed4c010493dfe6eb29e40a8a1cb2a99d66c5082").into()),
                    ethereum_beacon_client::PublicKey(hex!("8c45c9c4dbc4ab72011de24e77694fed0faaa79d0942c945254bbb9801c6e02cfc34be2a2c7d4e3ed88974da4d4913bb").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ad4a4bd60fe71810354ecfc6596bb49e65968580ceae6ec119ab5bcd40f678c9ae3dcf8514f42bc6eccd19753909da9").into()),
                    ethereum_beacon_client::PublicKey(hex!("5021bf3b9596e9609e20742bf919d6c7c4f08b741516956fd27e3a4ca3e6700fe97836d7912b6296c0bf036255ebaf53").into()),
                    ethereum_beacon_client::PublicKey(hex!("7d11292b615863b3c82f2cdee2504d4e91ad1c67bf0886e577605aa5cbdbc754b7517d97302afbaf797e4b8bf4bfe290").into()),
                    ethereum_beacon_client::PublicKey(hex!("74d036f2fe387100e93ee79ea8b4b925941e97a55f65adef72a5b09d7d7c7d04d8fa00de40355849357a5a794c6e92c3").into()),
                    ethereum_beacon_client::PublicKey(hex!("d369e227e7f80d1c0b5ed8b56906493f400e8df32c27c0948bf880410a01d4c3be9391ffd98bea94084590161fc40e95").into()),
                    ethereum_beacon_client::PublicKey(hex!("baa4425995be82cb626d5a46a581e201e6655567995fb9ecd1eeff3895c424d6540cde53cde94e30542bec8547c5abae").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e72aedc36a9aad2203f1346f2276f0cbe0e6e2c14e2958acd2350a833355a3a487d8fc00bb62c3bb3ad572b8fa4c80a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2cb79e71a08b9c7587412bf3fb0719f104d35758a130ad9c5db5eb963eb9b0ca48b1818f0d2e458d77b7cc1a0c731ed2").into()),
                    ethereum_beacon_client::PublicKey(hex!("0ccda1e12b2ec9b5b51ad7f9b2da7b7b9c5ca3ce1691255632ae139fb20c390934b52dc802e983a175f77f3601ffbabf").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc6a02131d9a559f2cf478f4caa2f9bcbdae303f6cb255d09586ecdfcdd52a6139f78ae9fffc37aac26993fda0602d83").into()),
                    ethereum_beacon_client::PublicKey(hex!("bc4e870034b5e059ab8e1a757dfe24558ad529c31149b6ead6fd660327df23175238e97e5d8ec2e5d4adf3c89dcb20da").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e971ea94ed788af8d9ae4c2800c1861a7a18a1c170d9b9918a5b811ef8d9cfc1523120078aa875c7a8ce97f7d8e0d95").into()),
                    ethereum_beacon_client::PublicKey(hex!("6ae848fe0254343869d0fb87bda7a9b0a8adb01fdac894721905ad8113116abe8f5cd50cc2eb4466c25060fc22285524").into()),
                    ethereum_beacon_client::PublicKey(hex!("2bbb0cd0c14b385077ebf9a4d4ff94bf25a2dddeadfd28665b0bae88b6d1706983b3cd5869cd55a6ed9fb8d05ea7e658").into()),
                    ethereum_beacon_client::PublicKey(hex!("42fe0da184a452ebdb5a5a9eb66aa78c804fcc763272ad38fed2e1e2916a98aec184a1a2f28100ea45e5cd4396e04a04").into()),
                    ethereum_beacon_client::PublicKey(hex!("ed4f8e38168ddf72fcb40185ab825c137bc3f165c045468f22b94ef1d7c544830ec912c0d514fc6888fb6a00b1dd48ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("9d158487f2cfcb3dd2e5fb772e8818a436226584113631b160a5e63f7d8631a1449bb2dd807798c6fc3cbabdafdbe2f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("14cf0527cacc56af5bc9a41d446dc51f12f20ff88862fa93b16fa8472f3834c0879050f3502c98eb46684584758557eb").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b9410630e1b016180aa5aef885464e442d7939a05442afc24139901a3644316c4421d8ec62f84b8bfd80cc4ee07dcdc").into()),
                    ethereum_beacon_client::PublicKey(hex!("c7f7e27d9febe978673bd7407e2b29a8a08200151cb1e4c80306eb2324aea1ff44d84761c748d146aad5affbe7c8dbb2").into()),
                    ethereum_beacon_client::PublicKey(hex!("7d932b16654517f7ae6a24ad2504121a4c2ef42cf06f3c793b50a26cdb25b335dfc35b4239774ba850331ee5083cafb5").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb826f00ef57af37846019ebefe7048743567e66afba9f205a4dd823193b9e83e01eec9e5df9cd95bbe15001e88ed9d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f25e8e8e25274a3aa1fd7a075f476fc265be02585f3c4d36730dfa01cb4ec82d0765ed1cbccd8489ef5a4ee1621e84b").into()),
                    ethereum_beacon_client::PublicKey(hex!("589b1616f4a3d3180605bdd2011fee71686f331c6bd25f2a8e533311788b8dd49bb474fbd6df67a80ca6a39276d6b882").into()),
                    ethereum_beacon_client::PublicKey(hex!("7347fa5cd94995bedea7822bbc63dff63dc56435a6262cb311c7bd15393ab12434868b9695d6ae0ed8c1ce8823c542eb").into()),
                    ethereum_beacon_client::PublicKey(hex!("429faa123dcac7087ed3ffdee6bcfd0da9f4581da2b767a11d880b0399cc487b87079e4a435ca580736c2d4507841848").into()),
                    ethereum_beacon_client::PublicKey(hex!("886b8b424f8b73c4cf21e763bcc0bdaf214027778d20c0a7aec33d6df166db959031dad3bb1ae9b8451cce440a66cb1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("da4f352e0fae62db98c4851f0d724e415a199acecc39bf1701237a5fbe0d12957a8f9641780f11905404fb7d2824a2aa").into()),
                    ethereum_beacon_client::PublicKey(hex!("a52c9f919eef29dbd5b7afe6c4b320ae667c598ec96767f4a9d2b98847b561d18906947584377b5390ddd521fa39c850").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab0e0ce404986c8e021cb294603d95633dfe9dfdf1cb64ade41ac1ea1c7d4dc257502987543285fbc39aac05554ac947").into()),
                    ethereum_beacon_client::PublicKey(hex!("ea7032ce742d6a994b3e0fd402c5b33c97b4c00f2b65c393a29bb012c4a118a382e7b3187de7c6c97598576c65f99799").into()),
                    ethereum_beacon_client::PublicKey(hex!("cfc2d1985cac5a70574824953a34b2d553ef9dafc418b1cc0c925d21a72c4a8c779bb432d9622a0ce81161c32f295f2c").into()),
                    ethereum_beacon_client::PublicKey(hex!("862998ca5b4ceef04dc506cac25f9713308d61209cf9159fbd86c7f72c568d6dc30e4a41271f6098ad6468138c8df230").into()),
                    ethereum_beacon_client::PublicKey(hex!("0ea0ed2df34520c90e2c77b4280dcd2f8146d30d90abda608cdacf72386e65e72d3dd01c8aae38915cac31efe1c5f8ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("0903d2a58e3b23a08059bbdfd0e7921efb1b9f6ced498ddbe1ee4395ffdc0f76d5d62c608faac91d1af5557f61622cbe").into()),
                    ethereum_beacon_client::PublicKey(hex!("9cecb4609044d57b1a317115b51d903f924f7c719d7f3652c74df2db479a87d480d42e13dd413441d9ceda78f7a2bcb8").into()),
                    ethereum_beacon_client::PublicKey(hex!("00d1b9c442153143dd39352584f1dbb650b227830f16e17689252b30a1562c4b444d6ee2477ba869071b7b0da5c59c1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e758e5a89f0cbcc2e20a1ba24645a06d5fd807b7b61905b5c94337a5c8ff1bce7c376052db3c0d5d9f5627b185e05d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("ed2815d73a0e4eb8363dc0702791708d1414eace45cc72efec789b316f492615951c426ef3acd1b2b1b25d89a226659f").into()),
                    ethereum_beacon_client::PublicKey(hex!("5ec91e9b10f34a536bd19abbd3a8b5d8f449610bb29ead21a3d70214c4070c62495ab957e6562cc0b74249b5845951d1").into()),
                    ethereum_beacon_client::PublicKey(hex!("11470a00ce1b3b826c6b09dccdba674c2170faf2ba2493b99db913170bd32c230fc286a1c6ba91d8983276a3ab055782").into()),
                    ethereum_beacon_client::PublicKey(hex!("5cd57023a718919f95c57884e5f11d710dd8f368e3704c857aeec8832630d2a5931a98bd16217333a33a7a1689c3d967").into()),
                    ethereum_beacon_client::PublicKey(hex!("1bbbb1db07d866a6327a9d6fdcebbec692579b24dc40ffb92cd1f126b0b236be20e7a38e68f95bbc98328ee2152b492f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2d44c95d616d7557386995ea9d41aa85f2b5ecd59b7be030dedabcbcee59518beda22fc880e1b22d9490eab6dcb9104").into()),
                    ethereum_beacon_client::PublicKey(hex!("6674895bb092d1e3cc3417ea2883b6a183b369a94993c191b557004544cf10515a08078e88a3d50a40138c4a085d44de").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ed96cc214daffb4791b36556ffcfd86c530b897545a3115fe1e96b9a9b245a8ce13d1f79fd564676e412ca585e7a12f").into()),
                    ethereum_beacon_client::PublicKey(hex!("89c8e60701ea5ceb56c21ed2bb214cf11f5228809bd87d90cd498475b441538719ad9333a4518ff7de428da357cdef1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("693e71cd15c563d5991f59b9272e56a0708d9c6153e5f2e4c360edd2216f0f34c84a0958657d9951f07adb59cede3872").into()),
                    ethereum_beacon_client::PublicKey(hex!("a321ac46c76d814041715969badb8387aae44605c5cb37603c1b5583cf86ddfcade68fcf527d7b311a671c34a74fb63f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8b9dbe413c3daafc8af4d1a993693b2c2212300cfd0b845ad4fe2737987a94df2e674d09f3b696f86facd30ff51c3346").into()),
                    ethereum_beacon_client::PublicKey(hex!("919691e12cf32158e2a713d307a0c5c5cd12d7c599298507b5e862789ac06c53aa2887597726515e2b0c6b2bc1755931").into()),
                    ethereum_beacon_client::PublicKey(hex!("d7615e4bcff4c2ea320096157c9f6983bf7356e402d1ada1c533d9020a41b2c059c7b0998080c9cecae0429590126f4d").into()),
                    ethereum_beacon_client::PublicKey(hex!("46b75aa174b464929c3dad0612ffab3ac4a93d183acabf5c63b4028de74c54657bba088cacec2314f5faf010a0e0857b").into()),
                    ethereum_beacon_client::PublicKey(hex!("a6af7399d8d063427e7418186671c1865b92e341d1d30e2c2370d58bf7216d9ea45e8bc560247f4a12a3518a7d20bfa2").into()),
                    ethereum_beacon_client::PublicKey(hex!("aac38787d8d4967b681471588deb83e6076a75ba6f41acf9be9b1dc37a035612519b8ad3d40c3e9cb6705a50b26de504").into()),
                    ethereum_beacon_client::PublicKey(hex!("ac422b79df1dc95d28ae4e672fbcf3f3899e1154cfa49b557bf6bfdbdeceaa3035dc845e70fe13d97e984d641d61e670").into()),
                    ethereum_beacon_client::PublicKey(hex!("5465c9ca4ec21d1175f9cf57be625b8b81bfb1e397be86b65f12342223aabf10e933c52d2ff62c0c3cff5ecae48703de").into()),
                    ethereum_beacon_client::PublicKey(hex!("0002c940e0259645dbd45f8dae6e890badb365aebe711718ec289521683072036407d97173b75a1240460f016bd5568d").into()),
                    ethereum_beacon_client::PublicKey(hex!("03de8d3749469cf060c6cdb97e62faf4419213cdf2e200d3573af61a19c7cf99c227972e5eb76b1f7c0def16678c8a37").into()),
                    ethereum_beacon_client::PublicKey(hex!("327ca1925d2e483c14afcccfa717d2fca8d2317d1b9e1deb19143b154e3dd5a3a95a8771c22eec1400bfa3ef74d03b67").into()),
                    ethereum_beacon_client::PublicKey(hex!("6d7cd4ca1e676132f4a36df58a857017da927a84ff9a3b581041a80e9c16d1d1bbff804e9da701d44616a3ffe066f8dd").into()),
                    ethereum_beacon_client::PublicKey(hex!("65bef5d86a3052820aa4805ca39233e335d5df1ada012fd302ddf8e2b23638de3ac804551abbe17fbe847d29fc3ee94c").into()),
                    ethereum_beacon_client::PublicKey(hex!("171576e109389210615c80b32ba7a943717880e597d6f54e8ae7c91ab057eeab331abf182469e0bec0dc30fbbc069d9f").into()),
                    ethereum_beacon_client::PublicKey(hex!("7e0d9cf753824cd4eef7989d352e14245ac976aa7da97432de070ef286d4471a2326736b2e6e74e05e4fb501636a623b").into()),
                    ethereum_beacon_client::PublicKey(hex!("5b4480902f197a8dde0a96fd411d49c461396e9e2e6b668541aebc0c57ef2adc015184cc5d8c9a5dc1e208fc7441d8fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5b05f65db381d5b78d6abc874a96f330819bb19fe8f9c6f984030bbac9a0dfa2b23e5cd1f547d5fbe492b668013e51d").into()),
                    ethereum_beacon_client::PublicKey(hex!("81478b4f9dbcfea2e7d9e58fa722eb4e2a28f11ef2b8971ac901cc3513fb2bb3b06cfd636a48056a238b3d2f2d22bf39").into()),
                    ethereum_beacon_client::PublicKey(hex!("a598e546cb075bedfa78c70740378270c40df45b743d3129ab92a1890805af4092292ff40cfad3a1438825f0f3069b20").into()),
                    ethereum_beacon_client::PublicKey(hex!("29010a78762071447b3f6d3b523f0f9c770e742bbc6f93cdff5bc9c9316961ee8c50ed46405d2f7b2d30547092afd9f0").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5584c4416396a1a0fee5f61d72fe2f959adedf469f6d4dfa6636e8260606817b88bfe8b9f26bbe2102df26f633c9535").into()),
                    ethereum_beacon_client::PublicKey(hex!("7f1fd7878f3a4d9eed3430f9ff89024b3ce386f2e5b796f844c29a557774ae129f00ed0bb01a0fe8ec2b7e7a84fcb493").into()),
                    ethereum_beacon_client::PublicKey(hex!("446d473edee8b199ae9054d79dfec55c125b2450795234d9439af2ebaf2af4f56a05b09ae2d632df9aa8b0a0d180513b").into()),
                    ethereum_beacon_client::PublicKey(hex!("2827d2414a76ecbd48643eb5ee7d14f0db6ae4e90575227f9b2e6a6ffc5ab315d57337fd30c7f259568f1dcb12ad84f6").into()),
                    ethereum_beacon_client::PublicKey(hex!("d751a8ac677460557ba93ba408b43acbb88ba332fd901897303a4db4ed63659420f5997f8476f3be9fe7e20e7e94e35e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b8b7d1b69ddfdd838e99855caf768f7a2c0839e028758684e5bae7fd794d39b0c01b18e58edcab5864bef1ffcf9bc0e7").into()),
                    ethereum_beacon_client::PublicKey(hex!("a975ba4683293cddf7df96d86d27df93e8d3af3d8d9358a72aaa520aeed28d600fd1629327d2149c2a6fd2008c86e783").into()),
                    ethereum_beacon_client::PublicKey(hex!("a9db57c59cd11ac63b77f775b5a534e6bbc49c36cf14c5eb50d8c515d9db609932ebd38992d206b758f5bdf79fe9e19b").into()),
                    ethereum_beacon_client::PublicKey(hex!("1854a1702e2d88bf2599923b37e3ae7a215eb328f4411c6a027af018a389c2c48aef4da5c7ee5ca607a2df6504b063e5").into()),
                    ethereum_beacon_client::PublicKey(hex!("bde2165260aeec319eeb21d6e1887526c80073fa4cd645601ab18d89a273ae1f2bdfd001adec1c6a207e89f6df4b52d5").into()),
                    ethereum_beacon_client::PublicKey(hex!("ea1c1ab4b439976dc0a83a21d5950f0b23f4cd3f98fc74ee87e563ded51de6814a9858d726b72913fcc17326f8df1280").into()),
                    ethereum_beacon_client::PublicKey(hex!("23f245c03e8068492208f3be3eb7aa9b692be64d5a2fbbe67b970033d30f57bce82767e45c69ab73dc8979f3f42fd875").into()),
                    ethereum_beacon_client::PublicKey(hex!("04affc5e3bfb32e9da7cc1c8313885e3bc445f6bb07f98a8e8bdb6d8805cd97f824e82ed632781d76af6adaae9e2e35c").into()),
                    ethereum_beacon_client::PublicKey(hex!("317972cc1bf6bd50564ea1d4063dcac55b1ed54c33afe12a867e7a7217f09e761d3794de130005662abfc90e0ae1d9f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e19957a89d1bb94ce7f5c66e66d1a738a974fb4d8ae0b9446d3e7455ccf9d39013d04d3c0818eb7b28e3ff188b6c8b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("a7e4692ef517d6a222235971d63f8a9865fd2bf64679886b5865c8d93a3e92471595e6fa41d5306190aa755678069cf9").into()),
                    ethereum_beacon_client::PublicKey(hex!("aca2e3bef92b1caccfc99ccb13a5369c168d9e614035bee8005f94df9debd42fefdefb63085580c418c62cc87c54c9c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("f407b443364414d4687e9c262edd8968f8227453e21a2dab0df192a398142eec25355d557bbe7617f7d6820c34afbb4b").into()),
                    ethereum_beacon_client::PublicKey(hex!("06226636518893196f04d110ef44234da6c03d42969cedc3b028cd4e2a2823380e37492f9d4082c8a840960e8f851193").into()),
                    ethereum_beacon_client::PublicKey(hex!("dbb0eec956c4c76e9289df2c3e11a4f63b747093c9ac9ccc425be23cb61236c4782696267dceb77dfe15e67608c53518").into()),
                    ethereum_beacon_client::PublicKey(hex!("12a0893a2fb2755fb06ac21c6fc1e9aacc94853a572ef142439e3933561f4bef2a3a068ab908156bb08045193c81b482").into()),
                    ethereum_beacon_client::PublicKey(hex!("2f4754f6755804d9e183db0bd0559c7914fcd1621818347711d9781ad538c989fe0943962918d3f48fd1f0ba2943ef9c").into()),
                    ethereum_beacon_client::PublicKey(hex!("daf3c4a105e481fc6ed7d7958cd6664a322ffc422bdb79837a3a2217a1c8953e610012b85e8684129ee214a1ed22c450").into()),
                    ethereum_beacon_client::PublicKey(hex!("e17993d7a4890b01dd06c17c449121ab6464381159116495549ebe5691fc8f789dd9f451205b6469da4ee5cb08bbccdd").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c6d39813d221f76daec53a4c0d856a6d81634c7eb9fcd4995000e6924ce5dfddd9a2f8bb7fa03588509d6f26cfe4b87").into()),
                    ethereum_beacon_client::PublicKey(hex!("4dbfc314db89655c3abf449f02501328775fc927486c6165617468ff9706fe4da6515801508745e182bb01c3dcccff0c").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e78991588e809efdda4449cb641bb5c454c3323bb58d2ee36180b95bd38b5eb463a2ecea1d416f92434ab04c9b9d7b9").into()),
                    ethereum_beacon_client::PublicKey(hex!("8a4110d6e92c555aac0723575ade90028ac3bf3dba1e3d883b87e2d435644af1bf313a13aac4cf0de87a2074396a14bb").into()),
                    ethereum_beacon_client::PublicKey(hex!("e0c4efa0dd03954d62d199ad7986369946f5639ef6ba4ad2488f1778130e80deb3edca8ea8a764d14b8e26c8acd92d58").into()),
                    ethereum_beacon_client::PublicKey(hex!("84fd88d078841abaddb870ad7b4af018f89fbc870861f07ded34fbff3dda200f08c0baf3735de6f217ec2f22fb21167c").into()),
                    ethereum_beacon_client::PublicKey(hex!("57e5fc66ff7e8a6979b50182f51b949d4f6546aa9971edb222db251d53e35575d72c475ac7a3f89287be169a167c975d").into()),
                    ethereum_beacon_client::PublicKey(hex!("a941e1bbdfe8075dd8dcdd897825b9ae0700d6406c30cdf948cadf8c4e8f965d268cc7f139e14662b07375d729ab50ae").into()),
                    ethereum_beacon_client::PublicKey(hex!("80eab08327a584e92c7b407ddfbf2f2089304fa645025b64d320ad805faf848acdb0e50a4a83619c7387632def5c2c7d").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e336fbad48495662425322650229205ea9a0027f2fddf322e20c0c96cad584ff0d10e715ebc92a0f16df3626f0f50ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("2675aac1369cdf82eda6070a4c26978b6cbdc087e2708031e499c864bf74f9b160a39c3133a7a54cf5010fea77a4b9e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("c8551cebdbb6bf0cde14db43cc93e33f7011bc7ab20d0ee83aac1f1576136ba201e6eabdf89dd7d9b7301bc5d3e27667").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ae7a17e2b3ae4db193c76edacfd835642c10c212686c2aaff001d61c0b2be5c0a597b61e4f5c34d75035fd314d61cb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9b9dd6dc2ef294459927e4051e7b601a6a615aa245d2234b85550c02fedf4a7d708a8d42d74b486b7618e56122eeda37").into()),
                    ethereum_beacon_client::PublicKey(hex!("44ec4ad5f5d593a1d87b29437502f81ba83844070ed7768da1fc2c0d01b71e0bf81896f41b91eb30895556f4a8c9ebd6").into()),
                    ethereum_beacon_client::PublicKey(hex!("fcb1e1d118f1642150d8886ca4b8dc9a22cebe090b9fb3ed2a083bbbe7c4a1f294ca5baa0e7290aa3a451240165e39ff").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc0f69f26185206d0a2ba0a81884907e68734f75a2c405b09fa6321d7d3c57758f95a30c303f8fcd552016473d61edc3").into()),
                    ethereum_beacon_client::PublicKey(hex!("ffb1ec77a144c1763d674c3f0ee3a68ce07a06ba800c5847cec05e2116eadb76ad04f878aca1958bb2caee4a83c4a89d").into()),
                    ethereum_beacon_client::PublicKey(hex!("2c1bdcf7927b6c8ef8102500acf5dd5a4311e2aeac743e18b6a1b9bb40aef3355e0186b1070a08011b4f7990c10c1eac").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ec2e6ab03f661b05ff18fb1581dd54f06cbd51402da28d8c0af5e014e29bcb7931cb1e11119d169219fb23f8fded28a").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb77ccf790d6a01fc15ec8cc05c556f205dd9c876c5f82c70c25cee59e4aea3d9f7796a46bb9427064d5ffee880b492c").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c9df8d5b6a34a9ced1cd09c2bf4f70599f60c335a0c6771c5f04ec232cc08efd83f0944d3589956f60d252d845feb05").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ab8e23133e45863418fe65445330a3b6c3011df8bac6cf93b5926b086b9dcd21049e015c7a8603e5e6f6cb1c7d82183").into()),
                    ethereum_beacon_client::PublicKey(hex!("640a25a74ac3a4b04a5b07677cd2ad37632e9b388ad7a33527034e88fba9d9fec741ededd0a0b24eeb6a14e26b79eb2b").into()),
                    ethereum_beacon_client::PublicKey(hex!("c019daa772c4b1e662fba4c7b8d747d063560cb8dcf096e8e76bd53c3313f1d798c8e6521ed3a9521b76d29589317c8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("d3a15585a4cf0f70a4efe499c743f9a65fdae640a7aa9f544fe64ae0f063d1ddbb8ddbb00190393cd90d3c6888c0208e").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e479a68d186de0b5f09520a57cd307b06486c58c62df60f4e0b59e7b6f37363a72525ec413748c08678d332454abf39").into()),
                    ethereum_beacon_client::PublicKey(hex!("587fdc44cd4e64605683aa5310cc25f5de61ba55e6092e0412073a57120b434362fbf166ca2899b005eadd041fb3c671").into()),
                    ethereum_beacon_client::PublicKey(hex!("79a8839537b861061451227ecf69ec2107ddb6639b58ed21e6c4a662f902b051a1c85c6560803be39c380107a377f9a0").into()),
                    ethereum_beacon_client::PublicKey(hex!("b76886ba071f3d36b68d859c92bc51d4e5c911996a85d9743e92a195fb8c431fb74acda134bcd0f0d26b919a76b4b55d").into()),
                    ethereum_beacon_client::PublicKey(hex!("20d812559299d8a04a6a94bf925fae22c76da82814e41d0c993f8bf33cc3ebf806f6cd8e0661ba17a94e8dbc62bb0c9d").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5f0c2bfe5146258675a244f09181e13b0433e363d7a9140965607f73dab103ba7326241f488a4d59684c02865d55635").into()),
                    ethereum_beacon_client::PublicKey(hex!("c3d5d83f97faa9478167ec0a1ebe77c7a63ed6143d1a72dd2068499660943811b218941c58d741da20cffecac5c55990").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b14f01c2fbd576354c92b5a0df3ce10b145929004d48971cd0e756c5f56b04aeeb34516f3a144d03f002d0967ef7cfe").into()),
                    ethereum_beacon_client::PublicKey(hex!("b260fb83e43dbf1a33d6cb248a915d68f34643f5a8314679def67fea3b177111a946b930e357c5bd9f2a2387aad080b7").into()),
                    ethereum_beacon_client::PublicKey(hex!("39312b58753e00d10569bf6ef24381f6617ee240ba544b1818bf872b6e879a54a2812af995faf863d87c291f74971c13").into()),
                    ethereum_beacon_client::PublicKey(hex!("b0cc618f13bdc63f993bfd9c481f6cb7811cf95f5f0245f79005fbcbbb8fc778a8c987434f14cbb93a45abf4b8c947d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("2cc5db08ba74d8a775034fc0bd5c406404fd4d7676d44943e63c96bacbc39962f6d3e3500b1c644f77e712228fdea2cf").into()),
                    ethereum_beacon_client::PublicKey(hex!("49eb863bd11a2c4724a777b9d460eda0fda2c4becc672a803c48de9f96989568cb364099ac564dbeb3e92afa67f60395").into()),
                    ethereum_beacon_client::PublicKey(hex!("999f338eca9bd5f056804756a7d0d6fe583aa7f7dd649bf1f8f67968099cbd0b18940ec5067a7ab25e01e23051f2087c").into()),
                    ethereum_beacon_client::PublicKey(hex!("a01a49f1bf7fade0df36ae023fcd9cab0c1f4f03b5e706f1f56f6fd8039ec23de612c1dd733f9a4389d4eb36d56e07fc").into()),
                    ethereum_beacon_client::PublicKey(hex!("e336096c0ed659fcd9f1cc977ce102476e2dbc86d6fb5e02acc8917a54e5b5d622255df54b120c3346b7e5ba16520929").into()),
                    ethereum_beacon_client::PublicKey(hex!("91a23149c6166ccce189870bbe0d9bc8127794f7ff5457b81684a2b3d188f39dbdc4b93b72e9523a1900ba6f39a85ea3").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fa3362b8142ac359635289f2fcc49ee26ea5e4713b09a627e092fb6b959dc47a73c9a46659ae3d262983b31b1fb8d93").into()),
                    ethereum_beacon_client::PublicKey(hex!("b8b4ada2cf1781b82769de6345f553fe4efa941a6812eccbb97f1569b4c850c3c0389965f9b3c76b0ca10236aaf8d3a4").into()),
                    ethereum_beacon_client::PublicKey(hex!("c948ada289c233b3424fce65c7b2080b8a8784523d6f2b96f58fbc378c2a02020ca52e86a4cad6572c54bf5c7e31c51a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ac7ccc2588fe8659ce1b151aa71e412147c034d7cc6131d1ad227541a42d5bcb287a56ba541d4a56b0d6bd62822c1de").into()),
                    ethereum_beacon_client::PublicKey(hex!("00996c99f81b91725e1f69fe003a1740fc6f8f86e3669c6269a79f73856659111c67a9d6c909feccd1ed13031ef67f7b").into()),
                    ethereum_beacon_client::PublicKey(hex!("f565da15259bce6a355e272514500a64f769bbe064e039a47684905a4e2e42b3c568a1430a576bae5a896cdbf14998b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("dc5bdfda4dc4267ad8270129bc97a7d5ccaa8bbbb50088e1e834c9b5bd3fd77bfb87ed2e1e4b8046f8dd969ea48b5a07").into()),
                    ethereum_beacon_client::PublicKey(hex!("227623b84b9ef970d38f0a6db223d994e9732b43a9db5df589b3580e063b5b6ebe39eea0fde1b49de24607bb972ec7a9").into()),
                    ethereum_beacon_client::PublicKey(hex!("79722813e02f532e1ce955472b3b92a0c65170041c66db8810644088ac1f9d7d5ec9d486892e4a2f8ffd585c39372e28").into()),
                    ethereum_beacon_client::PublicKey(hex!("d2db351f4afaa5d229fe2e51186f02930c2bf1714c7cc22fad5014f06d340fcf323526583faaec4b7b44c12446f53fbd").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5c89944408497894df0906be5b3672400018951be3b3a96a891b26dd8a6c6623f70ca5d07d1881e168188ce7b0cddda").into()),
                    ethereum_beacon_client::PublicKey(hex!("1ca436bf1d6c5c448b641a280d8710c80b7d54243a1818e6e20bfc840ec5e1d6132f0e8ba9b9a9dec71e44a4547c5ab7").into()),
                    ethereum_beacon_client::PublicKey(hex!("36bcb436a5b15d3f6fda0fd15a428e408e4e8b5dd14524a495e671f3ac601751a08398fed5da5af8a466aef08c5b4f54").into()),
                    ethereum_beacon_client::PublicKey(hex!("6bff71d59b403ea90033ce90656b620bb9fc24a0016656c2dd7bfb3b0313647a0a938b62f723a9950d1deba7f6fb7fe8").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7ae2b7067ab87306721ea27eb772fadf6171f44578e00c750a328a89cf6307d26cdf130d4f3fd775bde1945f90bc88b").into()),
                    ethereum_beacon_client::PublicKey(hex!("be23ea95fe9f4c3e0ce3f100bb987ce1217ac3a13ba7f36e7548a4b39295a239810070e373a3b192a104c4098b3fd1bf").into()),
                    ethereum_beacon_client::PublicKey(hex!("e4a3b7ae6102ddfb885af9f00b1ea9635800585ec63859d950488d61541e5aa21156e9bd59b8eccb5ed199e0137bdabf").into()),
                    ethereum_beacon_client::PublicKey(hex!("e4315f35a3c07600f3fcd7189b7b13bdf0acf49a62ed3feb7c7ee5d144a815d914671fbf6ca7e4774e81e9f23df58594").into()),
                    ethereum_beacon_client::PublicKey(hex!("fac458b869238514af57043e5ffacf21f41c65c8ebad331891d0d9ea0e5bf7cf812a1c6a8464040268c1c3649bdf9573").into()),
                    ethereum_beacon_client::PublicKey(hex!("b79731448d23697f2ad9e46d45d5294c579552050249c30986fec03aee25418e43bd21bb7e3c60b1cfb7e516c8457b25").into()),
                    ethereum_beacon_client::PublicKey(hex!("0df17cb29cc7d29836b52f814df0a8925d40bc8600eb508483ac46625496f78dd0168ae443814f568a86fdc4df31345a").into()),
                    ethereum_beacon_client::PublicKey(hex!("59be6134134684c549242957ebdb7b5d70feb9deaacbb9a7025f7c42332679badbd3a5f6dedea499bd1a70c26673aad1").into()),
                    ethereum_beacon_client::PublicKey(hex!("680e7397fec66aed567d8c8be353ee84af202fcca51722baf7f0efef64efec475a029f3c9d512d68d50814aaec677c76").into()),
                    ethereum_beacon_client::PublicKey(hex!("2dc1781e3f3ba5da4111c00fe219d57107edc8fda24a17425109f0f5357ddfc884a1511a811a8afda823a5d0d24f618c").into()),
                    ethereum_beacon_client::PublicKey(hex!("1d87d8bf86a62958992b2563206c2ba9ec840820dabba6dd2207ec37c8e2bf9d8ee3a32b41e7547c484cf9e142892eba").into()),
                    ethereum_beacon_client::PublicKey(hex!("f161e244d6c811eb53b3c4fbb3594b58b67b2d88da032e948b747c7b3bff3d6a0a8d8c645288e4d977882d974ea0e6fd").into()),
                    ethereum_beacon_client::PublicKey(hex!("8879dbdcb46f7cf04858250390dcb8f2c6d50f467a50ebda76b13e3065a441545a806ffe1ae8b1f558a1c614d3838958").into()),
                    ethereum_beacon_client::PublicKey(hex!("861e73a81cbddf6952f0f78306d0c08eeb9ca2e4bf0ae1c15c8f85b09a25232054ae4a3947df4418811a4adb202b9948").into()),
                    ethereum_beacon_client::PublicKey(hex!("2c636d623b2c1f429cfe607bca34d030b672f4dd5f179261145820566827ee73c0d8bfc0a4b1f859f9705f41a331d192").into()),
                    ethereum_beacon_client::PublicKey(hex!("1f8a1df647ac82a65d5e84e2370aea8e136896735ad7a79c5fdc02cbb7f395a0d1bf2e12b05e6f3602fc2eedba9aeae5").into()),
                    ethereum_beacon_client::PublicKey(hex!("fb04c8d5b7d81c2cf8b97b0b44c4b6c5f9e11e7ebeafd7e93ce1069db6383f3b4a2b545bb45ee515fe329ccdc546c0b1").into()),
                    ethereum_beacon_client::PublicKey(hex!("8b42f9688b1b86f723c971ddf1049aebbfa0e8d07ea2d2c336228e75bfd08b68196e85f407f2b56ab99099bf52ecb071").into()),
                    ethereum_beacon_client::PublicKey(hex!("8879a7c6dea69e0f219b871a532631b9ab1232afdf39018d74211f1992683e69dad55c6dcea230116c2ed6543a9f490f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8cbcb5718b3402dd26d178babae08d6ef0c34c023e6ddc1efb06d38bb010c114b72d762fbb3791f77565668dd77b9b1b").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab284c824a8ecb4035d8dca505b7b1ecbf9e1c773b27739b208a0edc17857e83b6a338e8ffdc855fce5b5bb3f02c5c0d").into()),
                    ethereum_beacon_client::PublicKey(hex!("6b4b1f780e1ef845065da669986b0bf4e84391989344007d0b5de73d90e8a9477ae475276c292dff360fc72aa7d63dd9").into()),
                    ethereum_beacon_client::PublicKey(hex!("b28a37f6816e165cf446f2f3f16efdd9aafb7ef2c8dc485ed75c67d1d1fb18668ad3ff60929320ca7a105caa0274c902").into()),
                    ethereum_beacon_client::PublicKey(hex!("d647e816ffe87d232539a781b42e22a9cf526d741d4657e5cf9a140e28c386c5053346311103b355a4776b833b20e2d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9b5707472b103f1e90760dcf1cf9e3f8a48eb3f8f0f6456d210e71dc8a2110922075dd3368b8c641513d3befe62aed2f").into()),
                    ethereum_beacon_client::PublicKey(hex!("e5f013fb8c9bbbf73fdc36d5ff49433bca7cfdbd3510de610ef966b90ea8a14beec7c37298b26fc79b762cb0056e39a8").into()),
                    ethereum_beacon_client::PublicKey(hex!("22f9523123f339946c4f3d30b854e214fa1778bb0c81ce64597d0666c31c4a694fa59ad916c52b470b148c798db9de5d").into()),
                    ethereum_beacon_client::PublicKey(hex!("3351cf4cbcda72d40d43a433e837eefa8f7a291a2e21d8f787c04cbc2921956930a4cf7f653b47e6025198d1d7f84e86").into()),
                    ethereum_beacon_client::PublicKey(hex!("9264b32abbdac89b424ddd852ce1f5f81136b72247ec9a81e78417bdfd575dc5242b56d42d029480e96926657e9883ec").into()),
                    ethereum_beacon_client::PublicKey(hex!("e1b34041c02cb47e9b98977f87479973b5f0c126a8e50fa3da15ee6c5ea0fbc3ea96ba3c3ea41352895bda6bc43be154").into()),
                    ethereum_beacon_client::PublicKey(hex!("fa04e07fde6350b0783872277aa7ddf0dcadb71c417d6f5ad5fd5f996e4b8c1529a4c1b9549bf1b9470d812bed2f0219").into()),
                    ethereum_beacon_client::PublicKey(hex!("53226ebe9468f67f6cf5029718436f1d0927b69b7a4ce65461434464623ac8b8cc435c79bb893aa1eb9b8e61282b7b81").into()),
                    ethereum_beacon_client::PublicKey(hex!("06cef8a94117cb83cdfa690e17c6469f5dfa2ba21d15bfd45a85de5e492af0ef092c6a92a156b458f2530786dcc59f0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8bd32db5735956a9ccf4d8735943fb2a23034c879cd95f3f80c4de0b9cc20078c2c13630bc28280e080530b797bd96c6").into()),
                    ethereum_beacon_client::PublicKey(hex!("df497ee408435c72a92be5f7e4a19d72eb5162de18d5b7dcd38716be0d18750b55165b2682bb917c7095978826973f6a").into()),
                    ethereum_beacon_client::PublicKey(hex!("c3c02a5675a6e54e2a0d98911fca10818c1e8ab62d504518b4f54a1eaa496f2a412911d31b14e887806ab9cf92ad3180").into()),
                    ethereum_beacon_client::PublicKey(hex!("4c9f52876e3691d0ed44abfccb13bd3958ed6132dc0b6c3388e027005831575ec1c274800d30f8b45e11bb5b6996651a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b3dbdce3b154993a517a460cfbb4e480b0b793d4da76869e6cfa042379085cb38ee99e646f75507bdf3f13322a6d1f97").into()),
                    ethereum_beacon_client::PublicKey(hex!("cb7f417bb8eb482be13bb476cbb72d12a3428005f14569e8e391ec14b0488310c6a7ea00d0127b9ba0423c951fd6f0cb").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ba292f9d3821a2a1bd88266f0f204c202dc71ed36fe27d465f3174b76d82fb19460237d1a26e7f17e1f9235374a7f60").into()),
                    ethereum_beacon_client::PublicKey(hex!("c623a9e36119a1ab0e01740286a6081d9fb817c72ccca4f9d2df59a08e3a83043491f44abe09ada15720a1964ed20b59").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ef3a922f83c584f56140985bbed54c93db4ac627eb73a424edbb50485efd8d280e5a97a01ce43ccd6758ebe09181317").into()),
                    ethereum_beacon_client::PublicKey(hex!("dac5d8f1639830cc2b1b17506c0078aca75139d784b499ea9cbb4223a6fa6f931b04fb30391d3855678409a60772afc6").into()),
                    ethereum_beacon_client::PublicKey(hex!("339237bccca58405048e28a90228e07cc03392de6caec556ab825db2e5627f372a84ab0e36e21e89aa25e2df9deaa004").into()),
                    ethereum_beacon_client::PublicKey(hex!("b1ca31182f1dc721417fe6dba5c9196bccddb0c2d5dd8aba9878c8c2374c1fd69a60c8e03fe8e166940e7a8098cbb35f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2f884459430cb3723f2d90e9de32d3ef8f01af0a885695c5dd4d0513010e4b5d4b9db12d6dc350555f29179c93ab4f6").into()),
                    ethereum_beacon_client::PublicKey(hex!("4659ad1a5e30c78808437382e98c164affea4fd333a6e989fbf6c06c6124d4e986f43e53624f886c1bbb55222e2767de").into()),
                    ethereum_beacon_client::PublicKey(hex!("da532030048916d9004d965be1fe05f6b400bcb5e8d1ec601f32ffb3b0a1ba4dc264552683e6543616423de2e5f0bd27").into()),
                    ethereum_beacon_client::PublicKey(hex!("773b6e459689d297f3d7a9276eb38bf0d52dbcd6cd191844f1dc77cf4203e562daa08613b10853363046dd6e9fc5816b").into()),
                    ethereum_beacon_client::PublicKey(hex!("f67d682f27e19b5826865cc53b3902087bb4f7f463791717f9fbf7922d366fd6092816426855dcdabf63c50ca128adcd").into()),
                    ethereum_beacon_client::PublicKey(hex!("b7a1683fb46c615283239bdecffa64a424bf50386ae05f21a99b2dbc865c1040f23c7f9b822c07935aa117abaf3b5b0e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b3292c1567e69cfda914af4a88b9b3866da19b75b73cf26e9dc4913605db825caf32cfc682a5a2cf44174f852ae18f53").into()),
                    ethereum_beacon_client::PublicKey(hex!("e86de28f261e5d99d1018787536891768cb3a637c058f3a907d77bacf585a868472885a9d9f040f604d91ca0df38ae07").into()),
                    ethereum_beacon_client::PublicKey(hex!("964c67c3f0ba0181be168f06a3987092752913e3dc74bf6cf959b3a4f2d495f9b9fd1719db92365c93a287d59c76d0cc").into()),
                    ethereum_beacon_client::PublicKey(hex!("ecc3011b11c8531a3210e521905ea5dae9fa513dd065f58bdf8c3cd2a0a188c199ff2f9a28eb4ec9a552270b9d9d1751").into()),
                    ethereum_beacon_client::PublicKey(hex!("8217dc346983f0b346c75fb52ed58555b9ff50b1def6241f98577602a4b1b920e7bcb648f24d0bfb0d09342c07469905").into()),
                    ethereum_beacon_client::PublicKey(hex!("48472011869e7f1cccbab985aa626feb791a046d0909e5bc8a7f835c1f2784d0b9ffb94814befcf6bd70d319a5afab07").into()),
                    ethereum_beacon_client::PublicKey(hex!("05524c57200fda8ae5584c903d99cb0f4e8814f432320a47cd6ef6e96c593c35c674c064c7458cde498111a3e323397e").into()),
                    ethereum_beacon_client::PublicKey(hex!("48b61fd688a92d00f31e84fd61b1d17e912ebaddd9a6c2ccd828b29513161205748e3e8b0ab73f612497d59d6ad0221e").into()),
                    ethereum_beacon_client::PublicKey(hex!("8f73077c785aed209e6243846f227576efeddde353549df21b9a04ce427a5c8d0620925bb483d4e894c8410c882159ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab4c22f55fe68b5445569ebbfb81ee9230488437f1881b4cee7cce94dca1eac2918a12923daef44b60cd36284584d48c").into()),
                    ethereum_beacon_client::PublicKey(hex!("9be505314d4b077b96420c2db86fc897d91d1ead77a1c342127be77e692c789d7cc2b5f5a8c8efebe8a02d9cd211a131").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e118d95bbd4a352b244e25aa8d583b9b3d27bebc1567fe913c044fb16c84c6445068cd5a4a3de68598c0c94d3f09769").into()),
                    ethereum_beacon_client::PublicKey(hex!("ce56a0b1299d8790a6c0c42acd868d0fa35741331542e187c17f42155bd25988f5eaf31c6eb8879dac1dea235f3bd51b").into()),
                    ethereum_beacon_client::PublicKey(hex!("79448cc53228295b30d5fff2a474276ba7a544d5726b5f35158b4ce8734f011eb71a2a5ef96cbf7ea6fcd5ff5c4aa7fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("07a4a1e44de9bc9df1230fa27442e70745349ee65929915a012c894f342aa03b2d7f246ed0c6073654a8ef8e7375933c").into()),
                    ethereum_beacon_client::PublicKey(hex!("42b3ad402a73b649665692b4a4436c321bf7de4f620fc4d4c36f78ae84191ccc320411752bc5b9b6ddc6d9a0736bd431").into()),
                    ethereum_beacon_client::PublicKey(hex!("d6ceb58d704a23ccfa67094a7dc489d8462ea2e804da64d13cea5d3a15977d1d7887c0b6fda094da3068e0789020cfb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e991c3af0a037e5c72c9ee8573cf73a2b3070cfc14677e8276b469b0a5e241883676b612a097e8e35bc85686fef0cc9").into()),
                    ethereum_beacon_client::PublicKey(hex!("09b8965928378c34d661830080e2248a8f4d3ebf8d276d0cf46d463572994f8e1a8838af077b8aa3bb36c01ef1572040").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a969a3725ab6159d65dacfe0deeda2c3f68573ede5edd41169c5cc40ce2db7c8be23ccee5c3b84a7017e828981c0fb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9fc543c1ffbb92090af9fb95ad4fffe8c1e42f2361b88541def0a437d4a1852a52322cbc849470f511e85dd2b1d0239a").into()),
                    ethereum_beacon_client::PublicKey(hex!("6be2fd1a32d4e5b24551f4bbd4bc58036d287ecf06ed08f83e2e5ff8e1060bf16335db0aee63a9f3b91e0b5d73e0754d").into()),
                    ethereum_beacon_client::PublicKey(hex!("43dfab2e051a407901c72067b8908960ac7dc26267e7d8139e1c2f14765ffb9d423473c5908c02c6b916ddd324b1ff3c").into()),
                    ethereum_beacon_client::PublicKey(hex!("efc620ef4ad1c17992e385bb309c72384df98e5b554b234d81c4163585c2fb8f950967ff7e38bdb871c6837a6275d81a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2db3b9183484bc2dcf2a95006966df60f1489221bb9bdeddcd2bd4a6c47eec5c8c32a0e84d6aa8063cf989ff75134d26").into()),
                    ethereum_beacon_client::PublicKey(hex!("c19f9a4ccfe34103c5d844dbdd5a855770f7b51ed07c326019c2378f3a64aec64523d6f6fd4035993f520cd50ed327b0").into()),
                    ethereum_beacon_client::PublicKey(hex!("e3d13221be67584ec7a210f48269c2225b016a7e76c5deb0bc889390cce68d9f9c9cb885236d726b3908a71bae4d949e").into()),
                    ethereum_beacon_client::PublicKey(hex!("6f06816cae1258cc8afa8d568624adb7f4d4c4a092046b1419bee96f7a89be275a86550ff7e28d08107ebf90eeb1a3cd").into()),
                    ethereum_beacon_client::PublicKey(hex!("b21bd20249cdda7ec9c9091ab4a9c82da6b29bebc9b93863ac9be37ff68e31bf096d6102a62a673b4f3993265a4e7269").into()),
                    ethereum_beacon_client::PublicKey(hex!("4bcfd90138307d30bcd433a0c4dd93a3b9149ddb3b18f37b83dbf8afe4874b41e72d71f9fa5bbc110c986643c09d7b8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("32d84a7fc81c4cad9cf0e24025542484add41c2956893a2dc559475fc49828e1411f8c645efe2ced0dbf33eee577ffa0").into()),
                    ethereum_beacon_client::PublicKey(hex!("53016f6f7b251a963983ea7faa6acf1716305a73081556ed688a10de520680175bcc0bff7db6f970c8af73993865782b").into()),
                    ethereum_beacon_client::PublicKey(hex!("ee17f2750bd819d575413211932d135b40ad2238449828db386e95e5f00d175e7e5d45ee9b17043a3345c0b35a1eaee6").into()),
                    ethereum_beacon_client::PublicKey(hex!("2bfea1200d1e1c316d0f4ad3a9a812bc3f512f05234c0c255ed769d1c8e6b39eb37f187bd6ee682067b325fefb73a6d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("328a22d8f820948445c2f675bbd08977e849251c6a88ca09fdb973d7bac30f548b48206a1f0878ed51a1e1842c27f291").into()),
                    ethereum_beacon_client::PublicKey(hex!("5de4775658c19b18e6e57c87b80e78c7a5d98f800ac286a549cdcda95b2cdbe129162ceba3ff5fa30c6ef51c08c72c70").into()),
                    ethereum_beacon_client::PublicKey(hex!("0025f9d4c482ea697b48487b2f9598a56baaadb20a8132305785c1fe74c4e08736616025148303c732da5b6f72403687").into()),
                    ethereum_beacon_client::PublicKey(hex!("5864cb038a65dd8d60ca8d5381486e030bfa6b4e0f1576165964fa0b91b6fb15ce2519f6f3b60a46d8af658f0f70b115").into()),
                    ethereum_beacon_client::PublicKey(hex!("e0e24e558af53a16356c8a64f36323499b86ea074b64998a34093c888d33534a1b950345715b5e6ea4c0c8886546f021").into()),
                    ethereum_beacon_client::PublicKey(hex!("b1f58c8fb7bde85054e647f4d37ab48907903a575cfdd7ebf18f2ba031d8a0d55abf88700b56f1dd80f7b43985e2ea83").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ef3e2050988e8f9115377b3870541b6cdafa0c41d5cf60243fc2aa0e5bcc3e5a26e56afa204bd46fcd448bd32557d71").into()),
                    ethereum_beacon_client::PublicKey(hex!("0097a885e5134288487ac7c74e816ec7134f29991d3ad82c1fe8a2aa37a0d0657375a0d9703b944a783f696da26b507e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b57073f55798da723cbae804377e6ef3f3d652703f31837f38ce0cbed8ad6986a878f9e8b02ff7664740f5d33133a408").into()),
                    ethereum_beacon_client::PublicKey(hex!("d090df80de6d303e7b6a12463826fe6f189fd73b56cbe233a677fec9bae042a8eaebb7b543b6ecfe0f894c64340ce73d").into()),
                    ethereum_beacon_client::PublicKey(hex!("fbd2771eb7afe646e753e1143fc71c6242d08d15f3eb4661f64a57432b244347d0224e528c95acb45c0b115a6bc32eef").into()),
                    ethereum_beacon_client::PublicKey(hex!("90c1ccec5b07e1dfa7b11ed197ca5750e912bdbc08cd154285988657c17d3a9dd2d6a0e2a0637c4111a64b664c0dc2d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5de3fdccc7d764cbaac240297160ab8c03006d9e1c58ba5e238723df8938ad41513493ac70bab651e17fde8761faa02").into()),
                    ethereum_beacon_client::PublicKey(hex!("af0eb8983937b20e9a58efcbefcc38edd2e06ba855f936c919ea8b87840e42b8c52a7c27c989eadaa05fa76bf660d32f").into()),
                    ethereum_beacon_client::PublicKey(hex!("b9e9c1b2423e75a9c1b94820e5a4af02b80976b6c8b137fe84c43685f6a3221fbe156049e79a4956297decf4a11a5fe3").into()),
                    ethereum_beacon_client::PublicKey(hex!("909824631d1d54108084f51262b6de477db008882286ff0fe05311678a1a51eb266c012fb2f98ae77c5717b93a8d52c7").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c4bcabd58b280f525c603f74b36c7bc27055b17e9e9db6a2c3fadd9a2e81d7836cf2928a92d0d92828828a43dd9f207").into()),
                    ethereum_beacon_client::PublicKey(hex!("6ec32e0bee65b47405cdc2e18920b3e44375411e8bd36f0e8176bbf68caa48eaa7ae70ac440f12b2ea58ae1be864d36e").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f8b71ae2cd16f79792b6f9d84da3472a879784bfb9af668ee94f04b135a3cf5ba63ddf93965cffac77dc47f6db95978").into()),
                    ethereum_beacon_client::PublicKey(hex!("9c1aececbe3fdb59b247977f848885b15f7ef304d0f80e82255c0e0127ab69a0cf16d40969c2d9a865cd5bbdd0a3f545").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e04d97939ac15c12d4f8fbefa99a92db45127e8de1d8e54ecdc3431431dffc7cdb9075dc6727fa84cd7f48a5818efdf").into()),
                    ethereum_beacon_client::PublicKey(hex!("2b919e7b0859fc003180bb10cb9394fe482121ad6c36ef9980b2e9eda3f4503558be73e92a86c9b808a7201741d146d7").into()),
                    ethereum_beacon_client::PublicKey(hex!("deac924f8c0d3bf2c48bddcb580e405d292369aa3099427f46b9ee542b9ae101ce992a3c0fae25d4af37322194db2383").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc22f6509ca9f0670ffaa559fd93f9bebba3c779fbbf6f835d416f8784bae7c3eb1768b75b7e568f6f269207fe18719a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5a9edf1fa24a06c51e7e28e6b1b849492850873db39ea98ce6fe7ba2ceebbdeb11c4aac7678f22f5952d9f6f094eeb7").into()),
                    ethereum_beacon_client::PublicKey(hex!("216a07dc89189a198be0a2218bcc886f8fc23f58dcc4d805b5d54bab19c1a1f9903fe05a23b0e07f68c229360cd90aa5").into()),
                    ethereum_beacon_client::PublicKey(hex!("692aaaff1e314ed4aa0030f23f5ba4792fcae751f63aaa2821a8ef4980f87c04f5fd5f4fa4fa4760207b6bd685a75387").into()),
                    ethereum_beacon_client::PublicKey(hex!("f73631bc70eaf9b3791c3ace9bd2a98ba01f5d12b4956cd3ce54abc31326937abeda7cb46b7242ae1c11e59cf49a299c").into()),
                    ethereum_beacon_client::PublicKey(hex!("ecb748df4e998ffcfd528ad0a1182fd5b3b6ad2bd13b27f269f588abfa86dad4f7e3f47eaeff96a4cb6ff2394fb85673").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e9db5aecb6e77f567ee6c1c69c87fbb4c05d3dadc84ff3fde2aded0e7719be3e944c79c574da604a5d6db4afdb8e83d").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c8331dae5e6dea66381992ff095ef389d91fc567834b6ccabba82504b4a64fd224dd6bc64a21525bc72b54e89c65947").into()),
                    ethereum_beacon_client::PublicKey(hex!("00f3a46c1d5a7ddfc21542ae6bfa0d7d11b817e3b1cec928989eab479d188c0ace06422e0eef06ca4cb3b6bb2ae743a1").into()),
                    ethereum_beacon_client::PublicKey(hex!("e6d91059c8923dcd8c1ba7d204eb26129d8e1cb615560f3361fdb336a21a9b8d388d0c4ec5c8c1125c74b651b8a87447").into()),
                    ethereum_beacon_client::PublicKey(hex!("1b81e683c1c8951ef9f2d60f8d6981ff76cabd9f244b76fa95935767ae72aaa11439ca748d4ffa86c420e845c9c091d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("a939e87acaf2364d0eb20c6e5fb34ca840ec59d24c3890faaebb4091003b95e8591a06d2e97040e08f62ae109041c30e").into()),
                    ethereum_beacon_client::PublicKey(hex!("0273315dab1ba0569d894d2d6ea1bfe337ad96c2ba698251cb6ce07fec84edce2d007bfca8484d97fcd495a984113576").into()),
                    ethereum_beacon_client::PublicKey(hex!("5b57c54165d85997ee71fddab0a6307c7c91853e2a7dff55f9fb9f819f34ba96bac7649d58c3fcec3089960e70c3f739").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a9724785a156b5cbc04bd023250ebecf5eca79c867182ff74909e0a3aa36e561f7a7e5a9d9824f3b1f0de7ac7e3e66a").into()),
                    ethereum_beacon_client::PublicKey(hex!("bbc1ba4a1744686d05840bdc2f8598dc1836e303ddfe2de3bbc289dd65e316db0ac9825dd52b461d070be6037183af21").into()),
                    ethereum_beacon_client::PublicKey(hex!("a76a6e761dab25f53ba65a6339e658a99fee356513491e6a9c8bf5fd1adfdaa82a61c5c3eab29aaaf18d26d7a68cf7db").into()),
                    ethereum_beacon_client::PublicKey(hex!("e3059e67ab8e7a9008fa04b2783ae24cec8bc367f243f0829b997bbc6384e25c9826e6f7645e886473f601f7cc64e567").into()),
                    ethereum_beacon_client::PublicKey(hex!("3a663435112ca1274e865197c6602d9f5d02e8b7e289f6c1131c5d6b0810a5ddf88dd5c1d76eef1d4442fa4833f82b4a").into()),
                    ethereum_beacon_client::PublicKey(hex!("623fe7520964ed389f3225eb8b9576ce0dacc940de80c268698eebcc3c91b84abc6dcb8d349dd72cdc277a07556e797d").into()),
                    ethereum_beacon_client::PublicKey(hex!("78ca48e60d53c1b1847429d4cce2eb429009c58047764c07bf9fde84a5a3476f364f8dafe8bb167b2cccf2f416472a60").into()),
                    ethereum_beacon_client::PublicKey(hex!("47226fb6d8fae3bbf7bed1597bd84569a357bd53d949cdde3992c7f81643724ee9f72b62696c6396b4c637259e73a868").into()),
                    ethereum_beacon_client::PublicKey(hex!("0bb1b6692e799e18e77336b76c946464ebd1e558f894919cd997c086642e9af5f12c854ce6deb94ec527cb7e56a0829c").into()),
                    ethereum_beacon_client::PublicKey(hex!("8c5a4d571bc5749fb63c4c79134d0a39a15cf1dc6c5be170a516c523e1106453cfd45f04d0ec7445ee06bd72e7b2cb00").into()),
                    ethereum_beacon_client::PublicKey(hex!("27ad8315984488f302122db80849256283fbe2a36b292453b41718a5014668b57d47b0ea16c863fcf83675a8840bff96").into()),
                    ethereum_beacon_client::PublicKey(hex!("56209effafa71998e158bd7dc4a1150a2eb6a01f8c5ee7755ba0fc878db9a6bc6f69173f1a622500228581e0c4775023").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7b891dc458730c26317850bdc4968c5cea62b81d558d52a6e822b357e7bc81807465fc60970ea675f5dcbab6392e3c6").into()),
                    ethereum_beacon_client::PublicKey(hex!("b34b3572d23f0dd027675e218364bad1e9bf3a3d9390e381658218afbc814efe6189f81e58b73e62c754d99c987b6ebb").into()),
                    ethereum_beacon_client::PublicKey(hex!("8faa50f5a25c0add8d9609af6c4e62a900016cd73d9767300e86e273dd5b893e1497a15c06efdb841ef03fe3f20bbe25").into()),
                    ethereum_beacon_client::PublicKey(hex!("dd02eac3e5f9279daed0f72b333571d199836098879a24afe7c1966baa788b27d7cfb3b9d84fceab4e484f895df3f28f").into()),
                    ethereum_beacon_client::PublicKey(hex!("50d78c9e44d1b5bcb0f0b6f65c7a8e1f915144b8112e8ed9375c0911f41398573f5acc99d765ba8f5a0d2f47434f4914").into()),
                    ethereum_beacon_client::PublicKey(hex!("3b338724435e729ec971de9e8e5b250e3d915cd7c2248c3007ed98d9245edacc0097f9a863e120b2d9dcb7fb747eb7cc").into()),
                    ethereum_beacon_client::PublicKey(hex!("6a3708c1f41d66f0e65e7eb17e826d4f1fcce407ed89742f14c355a35d02ae82c1c7958b4268613e6e8216ab0557fb69").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e8317360c663966d6ac37784e2f0fa2f7a0dbdfe3676b92d693ba921eb9cb5c5fd4d6d659bcb5bbb9e7abca4d9c522d").into()),
                    ethereum_beacon_client::PublicKey(hex!("41a42ac00447280d00e2cbf959164915f1db6b8d50650c5b8a5cde7ab03b57a9202f55e5d26c4702a701c97eaa889e6f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8d95af7e59b360612767991da1d059c6b03c845e748f137fe7c478d1dd9ab38e7eff333ba846a8042e4704570d611ce4").into()),
                    ethereum_beacon_client::PublicKey(hex!("f1a1e3f9220e944d36e7f31d5a7e8d0e74b5a5a86e284b5812e004abe6ba6f05764a1590465022c95d326d523ff712e2").into()),
                    ethereum_beacon_client::PublicKey(hex!("693ddb61295e04972b014d57f90ab4d0f4cabe17ddbcf1d2f2bbd9138a0415c3882274874ec3d58eb75c3ca589e7c01e").into()),
                    ethereum_beacon_client::PublicKey(hex!("3931ce237b2d97f62efff4139b6aa896ecae55cbbaa55362e4167da818c1781bd6c52880f770c62234bcd250811c1e8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c0f8be111b73b732a9742a3f6ddb0082b30b71c252ac9848279bdf22ee4c1a38a16c8e2493afc8d544c0da582e843d3e").into()),
                    ethereum_beacon_client::PublicKey(hex!("d58e670e2e65ecec6cfaee5cef86b02f3213d6c2ecc5e985a463212a437a658ce0e36eea6b5c558111e65a3aede73144").into()),
                    ethereum_beacon_client::PublicKey(hex!("4f8d870bf79765b1452408dab2e37f4fc0287606c2d5f6bfa8eb7ab0040e86116034692b3c54d11a99932f9cb9bc66e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("925088675e26e9a11354152c1b22f3c1674b6c5f654ca4e77558ac862adfe1a43fe06fb99cd38b0337d1647de2cfdf79").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb18de6ff89704f52022d088323752047cd0ebf8670930e153bcb6d4ed51dc83c18e4a314ec173e8967a124351e1d5e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("a8d1375f9ea2516837196a47d08fbe9a52eb79c9707e31db7b97acf3f776ba04025b81834d306cdbf894defd9181cdc1").into()),
                    ethereum_beacon_client::PublicKey(hex!("d8e8ab56a33db91ccaf7cd72c18104cb5fe4a31aa9e1d130ac8cf5db32a72461b6c02161db5e0b7c9f65cfe72fb1e1c7").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a66d617a402e671d7b3a49b237716e1306e71c2c8cabd97468797205549565380d941c978e09719e5412cabbb5e04ec").into()),
                    ethereum_beacon_client::PublicKey(hex!("3d0574c09ef94ed797ddfeaad8f41220c7ed7ab480e77fe8d8c63a5b034ff865f9ca16ba02051f58eb8571753152132d").into()),
                    ethereum_beacon_client::PublicKey(hex!("6361aa7f1474475631fb92ad48cd0482048bf72de7bbadbf2ef89b9ceb4ca3a9e4465832ea8ca5f7f7eaf6013249f89f").into()),
                    ethereum_beacon_client::PublicKey(hex!("d149bffce1398c121f5ed9f513ec95032fe0ddfdf50345f0dd2bc728fc44931212616b2a55c2e63633a9fc04b978023b").into()),
                    ethereum_beacon_client::PublicKey(hex!("289fa13f9df0e6248bee1bbfa97fd76acc9efa8c6fa2f5e1a879a469017d2dcdd3de19c95b221f97d56796fbcc698fd4").into()),
                    ethereum_beacon_client::PublicKey(hex!("974f3db12fab335af96fa2d01001017985b275b8f3256a7876027aa6eda3cebe84da40174927fd7473067af25899af45").into()),
                    ethereum_beacon_client::PublicKey(hex!("a300a56297ac25e75abf4302418b31600507ebf8c363bf7a02797c0d0b3d9c8486c78c8a27435501875bb132bbedd0fd").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c0571e571d341a5eb35154ba00b1e70a970473164c877755a45d500edffbdc14e989fd74b60f7b20ab698c1b5665c3c").into()),
                    ethereum_beacon_client::PublicKey(hex!("028ff156516b95616daef35d6ebde8c97adc49f1b47e5e1d37b7190422b19c54e25eae492e0a08ecd4e57d0b256a13de").into()),
                    ethereum_beacon_client::PublicKey(hex!("6c49980ca82c1d0698d0b96fe122e8f0862b1cddbbc42d225e4f11566357ff3101a112386a3d10f66763750f06db26ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c2336c7e90bab593b0bad69b08fee00170532e88ee4f2457b67bc58a5dc7372b16ef6a9ac06adc38f4d0b2af7e24385").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e58db501e1bab0679a9bf2f0e334b1d6c02b9299fd3bcc86fd5f0237a7c762541d2202518ea533558ff346c3a04b95f").into()),
                    ethereum_beacon_client::PublicKey(hex!("bea17ae6fa254e824988874a16610a2a1855bf3a4cd906ce51d70efb320bed0d38d2fea4e686a1c843a79135322da2ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("5555e6d620cdab05722518e1425a7a1a7b2bf11e66256a4102311c179a7a641058fcd3f7d3472f0f3bb63a6b869eb185").into()),
                    ethereum_beacon_client::PublicKey(hex!("b31fe8175f233d9eef4ece8bee9aed74c398b6a33b4e0763f3ec8f6a823dc06504f83ad6efc0efb4d36638752b6ef9d7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d71b61fed4c5180aa3db48050d26910b85923908d8b008492c0ce79cb537876ab4ec3e5502793ecabfae0ba0e7b186fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("9ce91664331a36ff4de3ef5209ebae7929fa9ea9a7e4b16659f672cbdcc258ea44cdd37ac3111e87413c07b707577990").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c4f8769855c0025a645d9e4cc3715cacecab680efb6022dde8680241b9c60616aa712307923d909f1c8a0e8c78cfa35").into()),
                    ethereum_beacon_client::PublicKey(hex!("d4a4f235827663f9a59739749c0c0309d27b9322c314544cfe82b55ab806f6fb4915037e6bf63245d3b16812c3fcbc78").into()),
                    ethereum_beacon_client::PublicKey(hex!("917f3a5749a5e944b2b84b2a863a98bd7020b97e2045639229543c725314bfdbc54397d629cb34bc8715f60dc0d0e01c").into())
                ], 
                aggregate_pubkey: ethereum_beacon_client::PublicKey(hex!("6d11763ae7f45b8b77916988126e200f7be7f754abe03a27134456f8a1671ae172eddf182d185ffacf557d23ba267ddd").into())
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("24409c991386e5d43bcecf871dc1fa395013f0293c86766877f745a408148a3a")
        );
    }

    #[test]
    pub fn test_hash_tree_root_fork_data() {
        let hash_root = merklization::hash_tree_root_fork_data(
            ethereum_beacon_client::ForkData {
                current_version: hex!("83f38a34").into(),
                genesis_validators_root: hex!("22370bbbb358800f5711a10ea9845284272d8493bed0348cab87b8ab1e127930").into()
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("57c12c4246bc7152b174b51920506bf943eff9c7ffa50b9533708e9cc1f680fc")
        );
    }

    #[test]
    pub fn test_hash_tree_root_signing_data() {
        let hash_root = merklization::hash_tree_root_signing_data(
            ethereum_beacon_client::SigningData {
                object_root: hex!("63654cbe64fc07853f1198c165dd3d49c54fc53bc417989bbcc66da15f850c54").into(),
                domain: hex!("037da907d1c3a03c0091b2254e1480d9b1783476e228ab29adaaa8f133e08f7a").into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("b9eb2caf2d691b183c2d57f322afe505c078cd08101324f61c3641714789a54e")
        );
    }
}