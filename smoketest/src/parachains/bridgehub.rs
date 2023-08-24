#[allow(dead_code, unused_imports, non_camel_case_types)]
#[allow(clippy::all)]
pub mod api {
    use super::api as root_mod;
    pub static PALLETS: [&str; 29usize] = [
        "System",
        "ParachainSystem",
        "Timestamp",
        "ParachainInfo",
        "Balances",
        "TransactionPayment",
        "Authorship",
        "CollatorSelection",
        "Session",
        "Aura",
        "AuraExt",
        "XcmpQueue",
        "PolkadotXcm",
        "CumulusXcm",
        "DmpQueue",
        "Utility",
        "Multisig",
        "BridgeWococoGrandpa",
        "BridgeWococoParachain",
        "BridgeWococoMessages",
        "BridgeRococoGrandpa",
        "BridgeRococoParachain",
        "BridgeRococoMessages",
        "BridgeRelayers",
        "EthereumInboundQueue",
        "EthereumOutboundQueue",
        "EthereumBeaconClient",
        "EthereumControl",
        "MessageQueue",
    ];
    #[derive(:: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug)]
    pub enum Event {
        #[codec(index = 0)]
        System(system::Event),
        #[codec(index = 1)]
        ParachainSystem(parachain_system::Event),
        #[codec(index = 10)]
        Balances(balances::Event),
        #[codec(index = 11)]
        TransactionPayment(transaction_payment::Event),
        #[codec(index = 21)]
        CollatorSelection(collator_selection::Event),
        #[codec(index = 22)]
        Session(session::Event),
        #[codec(index = 30)]
        XcmpQueue(xcmp_queue::Event),
        #[codec(index = 31)]
        PolkadotXcm(polkadot_xcm::Event),
        #[codec(index = 32)]
        CumulusXcm(cumulus_xcm::Event),
        #[codec(index = 33)]
        DmpQueue(dmp_queue::Event),
        #[codec(index = 40)]
        Utility(utility::Event),
        #[codec(index = 36)]
        Multisig(multisig::Event),
        #[codec(index = 41)]
        BridgeWococoGrandpa(bridge_wococo_grandpa::Event),
        #[codec(index = 42)]
        BridgeWococoParachain(bridge_wococo_parachain::Event),
        #[codec(index = 46)]
        BridgeWococoMessages(bridge_wococo_messages::Event),
        #[codec(index = 43)]
        BridgeRococoGrandpa(bridge_rococo_grandpa::Event),
        #[codec(index = 44)]
        BridgeRococoParachain(bridge_rococo_parachain::Event),
        #[codec(index = 45)]
        BridgeRococoMessages(bridge_rococo_messages::Event),
        #[codec(index = 47)]
        BridgeRelayers(bridge_relayers::Event),
        #[codec(index = 48)]
        EthereumInboundQueue(ethereum_inbound_queue::Event),
        #[codec(index = 49)]
        EthereumOutboundQueue(ethereum_outbound_queue::Event),
        #[codec(index = 50)]
        EthereumBeaconClient(ethereum_beacon_client::Event),
        #[codec(index = 51)]
        EthereumControl(ethereum_control::Event),
        #[codec(index = 60)]
        MessageQueue(message_queue::Event),
    }
    pub mod system {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Remark {
                pub remark: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct SetHeapPages {
                pub pages: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetCode {
                pub code: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetCodeWithoutChecks {
                pub code: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetStorage {
                pub items: ::std::vec::Vec<(
                    ::std::vec::Vec<::core::primitive::u8>,
                    ::std::vec::Vec<::core::primitive::u8>,
                )>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct KillStorage {
                pub keys: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct KillPrefix {
                pub prefix: ::std::vec::Vec<::core::primitive::u8>,
                pub subkeys: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RemarkWithEvent {
                pub remark: ::std::vec::Vec<::core::primitive::u8>,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::remark`]."]
                pub fn remark(
                    &self,
                    remark: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<Remark> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "remark",
                        Remark { remark },
                        [
                            101u8, 80u8, 195u8, 226u8, 224u8, 247u8, 60u8, 128u8, 3u8, 101u8, 51u8,
                            147u8, 96u8, 126u8, 76u8, 230u8, 194u8, 227u8, 191u8, 73u8, 160u8,
                            146u8, 87u8, 147u8, 243u8, 28u8, 228u8, 116u8, 224u8, 181u8, 129u8,
                            160u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_heap_pages`]."]
                pub fn set_heap_pages(
                    &self,
                    pages: ::core::primitive::u64,
                ) -> ::subxt::tx::StaticTxPayload<SetHeapPages> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "set_heap_pages",
                        SetHeapPages { pages },
                        [
                            43u8, 103u8, 128u8, 49u8, 156u8, 136u8, 11u8, 204u8, 80u8, 6u8, 244u8,
                            86u8, 171u8, 44u8, 140u8, 225u8, 142u8, 198u8, 43u8, 87u8, 26u8, 45u8,
                            125u8, 222u8, 165u8, 254u8, 172u8, 158u8, 39u8, 178u8, 86u8, 87u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_code`]."]
                pub fn set_code(
                    &self,
                    code: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<SetCode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "set_code",
                        SetCode { code },
                        [
                            27u8, 104u8, 244u8, 205u8, 188u8, 254u8, 121u8, 13u8, 106u8, 120u8,
                            244u8, 108u8, 97u8, 84u8, 100u8, 68u8, 26u8, 69u8, 93u8, 128u8, 107u8,
                            4u8, 3u8, 142u8, 13u8, 134u8, 196u8, 62u8, 113u8, 181u8, 14u8, 40u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_code_without_checks`]."]
                pub fn set_code_without_checks(
                    &self,
                    code: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<SetCodeWithoutChecks> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "set_code_without_checks",
                        SetCodeWithoutChecks { code },
                        [
                            102u8, 160u8, 125u8, 235u8, 30u8, 23u8, 45u8, 239u8, 112u8, 148u8,
                            159u8, 158u8, 42u8, 93u8, 206u8, 94u8, 80u8, 250u8, 66u8, 195u8, 60u8,
                            40u8, 142u8, 169u8, 183u8, 80u8, 80u8, 96u8, 3u8, 231u8, 99u8, 216u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_storage`]."]
                pub fn set_storage(
                    &self,
                    items: ::std::vec::Vec<(
                        ::std::vec::Vec<::core::primitive::u8>,
                        ::std::vec::Vec<::core::primitive::u8>,
                    )>,
                ) -> ::subxt::tx::StaticTxPayload<SetStorage> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "set_storage",
                        SetStorage { items },
                        [
                            74u8, 43u8, 106u8, 255u8, 50u8, 151u8, 192u8, 155u8, 14u8, 90u8, 19u8,
                            45u8, 165u8, 16u8, 235u8, 242u8, 21u8, 131u8, 33u8, 172u8, 119u8, 78u8,
                            140u8, 10u8, 107u8, 202u8, 122u8, 235u8, 181u8, 191u8, 22u8, 116u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::kill_storage`]."]
                pub fn kill_storage(
                    &self,
                    keys: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                ) -> ::subxt::tx::StaticTxPayload<KillStorage> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "kill_storage",
                        KillStorage { keys },
                        [
                            174u8, 174u8, 13u8, 174u8, 75u8, 138u8, 128u8, 235u8, 222u8, 216u8,
                            85u8, 18u8, 198u8, 1u8, 138u8, 70u8, 19u8, 108u8, 209u8, 41u8, 228u8,
                            67u8, 130u8, 230u8, 160u8, 207u8, 11u8, 180u8, 139u8, 242u8, 41u8,
                            15u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::kill_prefix`]."]
                pub fn kill_prefix(
                    &self,
                    prefix: ::std::vec::Vec<::core::primitive::u8>,
                    subkeys: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<KillPrefix> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "kill_prefix",
                        KillPrefix { prefix, subkeys },
                        [
                            203u8, 116u8, 217u8, 42u8, 154u8, 215u8, 77u8, 217u8, 13u8, 22u8,
                            193u8, 2u8, 128u8, 115u8, 179u8, 115u8, 187u8, 218u8, 129u8, 34u8,
                            80u8, 4u8, 173u8, 120u8, 92u8, 35u8, 237u8, 112u8, 201u8, 207u8, 200u8,
                            48u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::remark_with_event`]."]
                pub fn remark_with_event(
                    &self,
                    remark: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<RemarkWithEvent> {
                    ::subxt::tx::StaticTxPayload::new(
                        "System",
                        "remark_with_event",
                        RemarkWithEvent { remark },
                        [
                            123u8, 225u8, 180u8, 179u8, 144u8, 74u8, 27u8, 85u8, 101u8, 75u8,
                            134u8, 44u8, 181u8, 25u8, 183u8, 158u8, 14u8, 213u8, 56u8, 225u8,
                            136u8, 88u8, 26u8, 114u8, 178u8, 43u8, 176u8, 43u8, 240u8, 84u8, 116u8,
                            46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "Event for the System pallet."]
        pub type Event = runtime_types::frame_system::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An extrinsic completed successfully."]
            pub struct ExtrinsicSuccess {
                pub dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
            }
            impl ::subxt::events::StaticEvent for ExtrinsicSuccess {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "ExtrinsicSuccess";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An extrinsic failed."]
            pub struct ExtrinsicFailed {
                pub dispatch_error: runtime_types::sp_runtime::DispatchError,
                pub dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
            }
            impl ::subxt::events::StaticEvent for ExtrinsicFailed {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "ExtrinsicFailed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "`:code` was updated."]
            pub struct CodeUpdated;
            impl ::subxt::events::StaticEvent for CodeUpdated {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "CodeUpdated";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A new account was created."]
            pub struct NewAccount {
                pub account: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for NewAccount {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "NewAccount";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An account was reaped."]
            pub struct KilledAccount {
                pub account: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for KilledAccount {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "KilledAccount";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "On on-chain remark happened."]
            pub struct Remarked {
                pub sender: ::subxt::utils::AccountId32,
                pub hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for Remarked {
                const PALLET: &'static str = "System";
                const EVENT: &'static str = "Remarked";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The full account information for a particular account ID."]
                pub fn account(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_system::AccountInfo<
                            ::core::primitive::u32,
                            runtime_types::pallet_balances::types::AccountData<
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "Account",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            248u8, 178u8, 160u8, 222u8, 45u8, 231u8, 115u8, 164u8, 98u8, 184u8,
                            174u8, 206u8, 149u8, 190u8, 175u8, 34u8, 202u8, 230u8, 69u8, 218u8,
                            83u8, 43u8, 170u8, 41u8, 106u8, 77u8, 233u8, 97u8, 114u8, 14u8, 155u8,
                            131u8,
                        ],
                    )
                }
                #[doc = " The full account information for a particular account ID."]
                pub fn account_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_system::AccountInfo<
                            ::core::primitive::u32,
                            runtime_types::pallet_balances::types::AccountData<
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "Account",
                        Vec::new(),
                        [
                            248u8, 178u8, 160u8, 222u8, 45u8, 231u8, 115u8, 164u8, 98u8, 184u8,
                            174u8, 206u8, 149u8, 190u8, 175u8, 34u8, 202u8, 230u8, 69u8, 218u8,
                            83u8, 43u8, 170u8, 41u8, 106u8, 77u8, 233u8, 97u8, 114u8, 14u8, 155u8,
                            131u8,
                        ],
                    )
                }
                #[doc = " Total extrinsics count for the current block."]
                pub fn extrinsic_count(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "ExtrinsicCount",
                        vec![],
                        [
                            223u8, 60u8, 201u8, 120u8, 36u8, 44u8, 180u8, 210u8, 242u8, 53u8,
                            222u8, 154u8, 123u8, 176u8, 249u8, 8u8, 225u8, 28u8, 232u8, 4u8, 136u8,
                            41u8, 151u8, 82u8, 189u8, 149u8, 49u8, 166u8, 139u8, 9u8, 163u8, 231u8,
                        ],
                    )
                }
                #[doc = " The current weight for the block."]
                pub fn block_weight(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_support::dispatch::PerDispatchClass<
                            runtime_types::sp_weights::weight_v2::Weight,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "BlockWeight",
                        vec![],
                        [
                            120u8, 67u8, 71u8, 163u8, 36u8, 202u8, 52u8, 106u8, 143u8, 155u8,
                            144u8, 87u8, 142u8, 241u8, 232u8, 183u8, 56u8, 235u8, 27u8, 237u8,
                            20u8, 202u8, 33u8, 85u8, 189u8, 0u8, 28u8, 52u8, 198u8, 40u8, 219u8,
                            54u8,
                        ],
                    )
                }
                #[doc = " Total length (in bytes) for all extrinsics put together, for the current block."]
                pub fn all_extrinsics_len(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "AllExtrinsicsLen",
                        vec![],
                        [
                            202u8, 145u8, 209u8, 225u8, 40u8, 220u8, 174u8, 74u8, 93u8, 164u8,
                            254u8, 248u8, 254u8, 192u8, 32u8, 117u8, 96u8, 149u8, 53u8, 145u8,
                            219u8, 64u8, 234u8, 18u8, 217u8, 200u8, 203u8, 141u8, 145u8, 28u8,
                            134u8, 60u8,
                        ],
                    )
                }
                #[doc = " Map of block numbers to block hashes."]
                pub fn block_hash(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "BlockHash",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            50u8, 112u8, 176u8, 239u8, 175u8, 18u8, 205u8, 20u8, 241u8, 195u8,
                            21u8, 228u8, 186u8, 57u8, 200u8, 25u8, 38u8, 44u8, 106u8, 20u8, 168u8,
                            80u8, 76u8, 235u8, 12u8, 51u8, 137u8, 149u8, 200u8, 4u8, 220u8, 237u8,
                        ],
                    )
                }
                #[doc = " Map of block numbers to block hashes."]
                pub fn block_hash_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "BlockHash",
                        Vec::new(),
                        [
                            50u8, 112u8, 176u8, 239u8, 175u8, 18u8, 205u8, 20u8, 241u8, 195u8,
                            21u8, 228u8, 186u8, 57u8, 200u8, 25u8, 38u8, 44u8, 106u8, 20u8, 168u8,
                            80u8, 76u8, 235u8, 12u8, 51u8, 137u8, 149u8, 200u8, 4u8, 220u8, 237u8,
                        ],
                    )
                }
                #[doc = " Extrinsics data for the current block (maps an extrinsic's index to its data)."]
                pub fn extrinsic_data(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "ExtrinsicData",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            210u8, 224u8, 211u8, 186u8, 118u8, 210u8, 185u8, 194u8, 238u8, 211u8,
                            254u8, 73u8, 67u8, 184u8, 31u8, 229u8, 168u8, 125u8, 98u8, 23u8, 241u8,
                            59u8, 49u8, 86u8, 126u8, 9u8, 114u8, 163u8, 160u8, 62u8, 50u8, 67u8,
                        ],
                    )
                }
                #[doc = " Extrinsics data for the current block (maps an extrinsic's index to its data)."]
                pub fn extrinsic_data_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "ExtrinsicData",
                        Vec::new(),
                        [
                            210u8, 224u8, 211u8, 186u8, 118u8, 210u8, 185u8, 194u8, 238u8, 211u8,
                            254u8, 73u8, 67u8, 184u8, 31u8, 229u8, 168u8, 125u8, 98u8, 23u8, 241u8,
                            59u8, 49u8, 86u8, 126u8, 9u8, 114u8, 163u8, 160u8, 62u8, 50u8, 67u8,
                        ],
                    )
                }
                #[doc = " The current block number being processed. Set by `execute_block`."]
                pub fn number(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "Number",
                        vec![],
                        [
                            228u8, 96u8, 102u8, 190u8, 252u8, 130u8, 239u8, 172u8, 126u8, 235u8,
                            246u8, 139u8, 208u8, 15u8, 88u8, 245u8, 141u8, 232u8, 43u8, 204u8,
                            36u8, 87u8, 211u8, 141u8, 187u8, 68u8, 236u8, 70u8, 193u8, 235u8,
                            164u8, 191u8,
                        ],
                    )
                }
                #[doc = " Hash of the previous block."]
                pub fn parent_hash(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "ParentHash",
                        vec![],
                        [
                            232u8, 206u8, 177u8, 119u8, 38u8, 57u8, 233u8, 50u8, 225u8, 49u8,
                            169u8, 176u8, 210u8, 51u8, 231u8, 176u8, 234u8, 186u8, 188u8, 112u8,
                            15u8, 152u8, 195u8, 232u8, 201u8, 97u8, 208u8, 249u8, 9u8, 163u8, 69u8,
                            36u8,
                        ],
                    )
                }
                #[doc = " Digest of the current block, also part of the block header."]
                pub fn digest(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::sp_runtime::generic::digest::Digest,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "Digest",
                        vec![],
                        [
                            83u8, 141u8, 200u8, 132u8, 182u8, 55u8, 197u8, 122u8, 13u8, 159u8,
                            31u8, 42u8, 60u8, 191u8, 89u8, 221u8, 242u8, 47u8, 199u8, 213u8, 48u8,
                            216u8, 131u8, 168u8, 245u8, 82u8, 56u8, 190u8, 62u8, 69u8, 96u8, 37u8,
                        ],
                    )
                }
                #[doc = " Events deposited for the current block."]
                #[doc = ""]
                #[doc = " NOTE: The item is unbound and should therefore never be read on chain."]
                #[doc = " It could otherwise inflate the PoV size of a block."]
                #[doc = ""]
                #[doc = " Events have a large in-memory size. Box the events to not go out-of-memory"]
                #[doc = " just in case someone still reads them from within the runtime."]
                pub fn events(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<
                            runtime_types::frame_system::EventRecord<
                                runtime_types::bridge_hub_rococo_runtime::RuntimeEvent,
                                ::subxt::utils::H256,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "Events",
                        vec![],
                        [
                            206u8, 182u8, 99u8, 187u8, 178u8, 239u8, 249u8, 91u8, 241u8, 8u8,
                            176u8, 191u8, 57u8, 29u8, 184u8, 177u8, 209u8, 186u8, 253u8, 79u8,
                            229u8, 106u8, 54u8, 20u8, 140u8, 251u8, 76u8, 111u8, 64u8, 235u8,
                            135u8, 102u8,
                        ],
                    )
                }
                #[doc = " The number of events in the `Events<T>` list."]
                pub fn event_count(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "EventCount",
                        vec![],
                        [
                            236u8, 93u8, 90u8, 177u8, 250u8, 211u8, 138u8, 187u8, 26u8, 208u8,
                            203u8, 113u8, 221u8, 233u8, 227u8, 9u8, 249u8, 25u8, 202u8, 185u8,
                            161u8, 144u8, 167u8, 104u8, 127u8, 187u8, 38u8, 18u8, 52u8, 61u8, 66u8,
                            112u8,
                        ],
                    )
                }
                #[doc = " Mapping between a topic (represented by T::Hash) and a vector of indexes"]
                #[doc = " of events in the `<Events<T>>` list."]
                #[doc = ""]
                #[doc = " All topic vectors have deterministic storage locations depending on the topic. This"]
                #[doc = " allows light-clients to leverage the changes trie storage tracking mechanism and"]
                #[doc = " in case of changes fetch the list of events of interest."]
                #[doc = ""]
                #[doc = " The value has the type `(T::BlockNumber, EventIndex)` because if we used only just"]
                #[doc = " the `EventIndex` then in case if the topic has the same contents on the next block"]
                #[doc = " no notification will be triggered thus the event might be lost."]
                pub fn event_topics(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<(::core::primitive::u32, ::core::primitive::u32)>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "EventTopics",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            205u8, 90u8, 142u8, 190u8, 176u8, 37u8, 94u8, 82u8, 98u8, 1u8, 129u8,
                            63u8, 246u8, 101u8, 130u8, 58u8, 216u8, 16u8, 139u8, 196u8, 154u8,
                            111u8, 110u8, 178u8, 24u8, 44u8, 183u8, 176u8, 232u8, 82u8, 223u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " Mapping between a topic (represented by T::Hash) and a vector of indexes"]
                #[doc = " of events in the `<Events<T>>` list."]
                #[doc = ""]
                #[doc = " All topic vectors have deterministic storage locations depending on the topic. This"]
                #[doc = " allows light-clients to leverage the changes trie storage tracking mechanism and"]
                #[doc = " in case of changes fetch the list of events of interest."]
                #[doc = ""]
                #[doc = " The value has the type `(T::BlockNumber, EventIndex)` because if we used only just"]
                #[doc = " the `EventIndex` then in case if the topic has the same contents on the next block"]
                #[doc = " no notification will be triggered thus the event might be lost."]
                pub fn event_topics_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<(::core::primitive::u32, ::core::primitive::u32)>,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "EventTopics",
                        Vec::new(),
                        [
                            205u8, 90u8, 142u8, 190u8, 176u8, 37u8, 94u8, 82u8, 98u8, 1u8, 129u8,
                            63u8, 246u8, 101u8, 130u8, 58u8, 216u8, 16u8, 139u8, 196u8, 154u8,
                            111u8, 110u8, 178u8, 24u8, 44u8, 183u8, 176u8, 232u8, 82u8, 223u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " Stores the `spec_version` and `spec_name` of when the last runtime upgrade happened."]
                pub fn last_runtime_upgrade(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_system::LastRuntimeUpgradeInfo,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "LastRuntimeUpgrade",
                        vec![],
                        [
                            52u8, 37u8, 117u8, 111u8, 57u8, 130u8, 196u8, 14u8, 99u8, 77u8, 91u8,
                            126u8, 178u8, 249u8, 78u8, 34u8, 9u8, 194u8, 92u8, 105u8, 113u8, 81u8,
                            185u8, 127u8, 245u8, 184u8, 60u8, 29u8, 234u8, 182u8, 96u8, 196u8,
                        ],
                    )
                }
                #[doc = " True if we have upgraded so that `type RefCount` is `u32`. False (default) if not."]
                pub fn upgraded_to_u32_ref_count(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "UpgradedToU32RefCount",
                        vec![],
                        [
                            171u8, 88u8, 244u8, 92u8, 122u8, 67u8, 27u8, 18u8, 59u8, 175u8, 175u8,
                            178u8, 20u8, 150u8, 213u8, 59u8, 222u8, 141u8, 32u8, 107u8, 3u8, 114u8,
                            83u8, 250u8, 180u8, 233u8, 152u8, 54u8, 187u8, 99u8, 131u8, 204u8,
                        ],
                    )
                }
                #[doc = " True if we have upgraded so that AccountInfo contains three types of `RefCount`. False"]
                #[doc = " (default) if not."]
                pub fn upgraded_to_triple_ref_count(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "UpgradedToTripleRefCount",
                        vec![],
                        [
                            90u8, 33u8, 56u8, 86u8, 90u8, 101u8, 89u8, 133u8, 203u8, 56u8, 201u8,
                            210u8, 244u8, 232u8, 150u8, 18u8, 51u8, 105u8, 14u8, 230u8, 103u8,
                            155u8, 246u8, 99u8, 53u8, 207u8, 225u8, 128u8, 186u8, 76u8, 40u8,
                            185u8,
                        ],
                    )
                }
                #[doc = " The execution phase of the block."]
                pub fn execution_phase(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::frame_system::Phase>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "System",
                        "ExecutionPhase",
                        vec![],
                        [
                            230u8, 183u8, 221u8, 135u8, 226u8, 223u8, 55u8, 104u8, 138u8, 224u8,
                            103u8, 156u8, 222u8, 99u8, 203u8, 199u8, 164u8, 168u8, 193u8, 133u8,
                            201u8, 155u8, 63u8, 95u8, 17u8, 206u8, 165u8, 123u8, 161u8, 33u8,
                            172u8, 93u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Block & extrinsics weights: base values and limits."]
                pub fn block_weights(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_system::limits::BlockWeights,
                    >,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "BlockWeights",
                        [
                            118u8, 253u8, 239u8, 217u8, 145u8, 115u8, 85u8, 86u8, 172u8, 248u8,
                            139u8, 32u8, 158u8, 126u8, 172u8, 188u8, 197u8, 105u8, 145u8, 235u8,
                            171u8, 50u8, 31u8, 225u8, 167u8, 187u8, 241u8, 87u8, 6u8, 17u8, 234u8,
                            185u8,
                        ],
                    )
                }
                #[doc = " The maximum length of a block (in bytes)."]
                pub fn block_length(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::frame_system::limits::BlockLength,
                    >,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "BlockLength",
                        [
                            116u8, 184u8, 225u8, 228u8, 207u8, 203u8, 4u8, 220u8, 234u8, 198u8,
                            150u8, 108u8, 205u8, 87u8, 194u8, 131u8, 229u8, 51u8, 140u8, 4u8, 47u8,
                            12u8, 200u8, 144u8, 153u8, 62u8, 51u8, 39u8, 138u8, 205u8, 203u8,
                            236u8,
                        ],
                    )
                }
                #[doc = " Maximum number of block number to block hash mappings to keep (oldest pruned first)."]
                pub fn block_hash_count(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "BlockHashCount",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The weight of runtime database operations the runtime can invoke."]
                pub fn db_weight(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::sp_weights::RuntimeDbWeight>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "DbWeight",
                        [
                            124u8, 162u8, 190u8, 149u8, 49u8, 177u8, 162u8, 231u8, 62u8, 167u8,
                            199u8, 181u8, 43u8, 232u8, 185u8, 116u8, 195u8, 51u8, 233u8, 223u8,
                            20u8, 129u8, 246u8, 13u8, 65u8, 180u8, 64u8, 9u8, 157u8, 59u8, 245u8,
                            118u8,
                        ],
                    )
                }
                #[doc = " Get the chain's current version."]
                pub fn version(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::sp_version::RuntimeVersion>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "Version",
                        [
                            93u8, 98u8, 57u8, 243u8, 229u8, 8u8, 234u8, 231u8, 72u8, 230u8, 139u8,
                            47u8, 63u8, 181u8, 17u8, 2u8, 220u8, 231u8, 104u8, 237u8, 185u8, 143u8,
                            165u8, 253u8, 188u8, 76u8, 147u8, 12u8, 170u8, 26u8, 74u8, 200u8,
                        ],
                    )
                }
                #[doc = " The designated SS58 prefix of this chain."]
                #[doc = ""]
                #[doc = " This replaces the \"ss58Format\" property declared in the chain spec. Reason is"]
                #[doc = " that the runtime should know about the prefix in order to make use of it as"]
                #[doc = " an identifier of the chain."]
                pub fn ss58_prefix(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u16>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "System",
                        "SS58Prefix",
                        [
                            116u8, 33u8, 2u8, 170u8, 181u8, 147u8, 171u8, 169u8, 167u8, 227u8,
                            41u8, 144u8, 11u8, 236u8, 82u8, 100u8, 74u8, 60u8, 184u8, 72u8, 169u8,
                            90u8, 208u8, 135u8, 15u8, 117u8, 10u8, 123u8, 128u8, 193u8, 29u8, 70u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod parachain_system {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetValidationData {
                pub data:
                    runtime_types::cumulus_primitives_parachain_inherent::ParachainInherentData,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SudoSendUpwardMessage {
                pub message: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AuthorizeUpgrade {
                pub code_hash: ::subxt::utils::H256,
                pub check_version: ::core::primitive::bool,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct EnactAuthorizedUpgrade {
                pub code: ::std::vec::Vec<::core::primitive::u8>,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_validation_data`]."]
                pub fn set_validation_data(
                    &self,
                    data : runtime_types :: cumulus_primitives_parachain_inherent :: ParachainInherentData,
                ) -> ::subxt::tx::StaticTxPayload<SetValidationData> {
                    ::subxt::tx::StaticTxPayload::new(
                        "ParachainSystem",
                        "set_validation_data",
                        SetValidationData { data },
                        [
                            200u8, 80u8, 163u8, 177u8, 184u8, 117u8, 61u8, 203u8, 244u8, 214u8,
                            106u8, 151u8, 128u8, 131u8, 254u8, 120u8, 254u8, 76u8, 104u8, 39u8,
                            215u8, 227u8, 233u8, 254u8, 26u8, 62u8, 17u8, 42u8, 19u8, 127u8, 108u8,
                            242u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::sudo_send_upward_message`]."]
                pub fn sudo_send_upward_message(
                    &self,
                    message: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<SudoSendUpwardMessage> {
                    ::subxt::tx::StaticTxPayload::new(
                        "ParachainSystem",
                        "sudo_send_upward_message",
                        SudoSendUpwardMessage { message },
                        [
                            127u8, 79u8, 45u8, 183u8, 190u8, 205u8, 184u8, 169u8, 255u8, 191u8,
                            86u8, 154u8, 134u8, 25u8, 249u8, 63u8, 47u8, 194u8, 108u8, 62u8, 60u8,
                            170u8, 81u8, 240u8, 113u8, 48u8, 181u8, 171u8, 95u8, 63u8, 26u8, 222u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::authorize_upgrade`]."]
                pub fn authorize_upgrade(
                    &self,
                    code_hash: ::subxt::utils::H256,
                    check_version: ::core::primitive::bool,
                ) -> ::subxt::tx::StaticTxPayload<AuthorizeUpgrade> {
                    ::subxt::tx::StaticTxPayload::new(
                        "ParachainSystem",
                        "authorize_upgrade",
                        AuthorizeUpgrade {
                            code_hash,
                            check_version,
                        },
                        [
                            208u8, 115u8, 62u8, 35u8, 70u8, 223u8, 65u8, 57u8, 216u8, 44u8, 169u8,
                            249u8, 90u8, 112u8, 17u8, 208u8, 30u8, 131u8, 102u8, 131u8, 240u8,
                            217u8, 230u8, 214u8, 145u8, 198u8, 55u8, 13u8, 217u8, 51u8, 178u8,
                            141u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::enact_authorized_upgrade`]."]
                pub fn enact_authorized_upgrade(
                    &self,
                    code: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<EnactAuthorizedUpgrade> {
                    ::subxt::tx::StaticTxPayload::new(
                        "ParachainSystem",
                        "enact_authorized_upgrade",
                        EnactAuthorizedUpgrade { code },
                        [
                            43u8, 157u8, 1u8, 230u8, 134u8, 72u8, 230u8, 35u8, 159u8, 13u8, 201u8,
                            134u8, 184u8, 94u8, 167u8, 13u8, 108u8, 157u8, 145u8, 166u8, 119u8,
                            37u8, 51u8, 121u8, 252u8, 255u8, 48u8, 251u8, 126u8, 152u8, 247u8, 5u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::cumulus_pallet_parachain_system::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The validation function has been scheduled to apply."]
            pub struct ValidationFunctionStored;
            impl ::subxt::events::StaticEvent for ValidationFunctionStored {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "ValidationFunctionStored";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "The validation function was applied as of the contained relay chain block number."]
            pub struct ValidationFunctionApplied {
                pub relay_chain_block_num: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for ValidationFunctionApplied {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "ValidationFunctionApplied";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The relay-chain aborted the upgrade process."]
            pub struct ValidationFunctionDiscarded;
            impl ::subxt::events::StaticEvent for ValidationFunctionDiscarded {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "ValidationFunctionDiscarded";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An upgrade has been authorized."]
            pub struct UpgradeAuthorized {
                pub code_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for UpgradeAuthorized {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "UpgradeAuthorized";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "Some downward messages have been received and will be processed."]
            pub struct DownwardMessagesReceived {
                pub count: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for DownwardMessagesReceived {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "DownwardMessagesReceived";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward messages were processed using the given weight."]
            pub struct DownwardMessagesProcessed {
                pub weight_used: runtime_types::sp_weights::weight_v2::Weight,
                pub dmq_head: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for DownwardMessagesProcessed {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "DownwardMessagesProcessed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An upward message was sent to the relay chain."]
            pub struct UpwardMessageSent {
                pub message_hash: ::core::option::Option<[::core::primitive::u8; 32usize]>,
            }
            impl ::subxt::events::StaticEvent for UpwardMessageSent {
                const PALLET: &'static str = "ParachainSystem";
                const EVENT: &'static str = "UpwardMessageSent";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " In case of a scheduled upgrade, this storage field contains the validation code to be applied."]
                #[doc = ""]
                #[doc = " As soon as the relay chain gives us the go-ahead signal, we will overwrite the [`:code`][sp_core::storage::well_known_keys::CODE]"]
                #[doc = " which will result the next block process with the new validation code. This concludes the upgrade process."]
                pub fn pending_validation_code(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "PendingValidationCode",
                        vec![],
                        [
                            162u8, 35u8, 108u8, 76u8, 160u8, 93u8, 215u8, 84u8, 20u8, 249u8, 57u8,
                            187u8, 88u8, 161u8, 15u8, 131u8, 213u8, 89u8, 140u8, 20u8, 227u8,
                            204u8, 79u8, 176u8, 114u8, 119u8, 8u8, 7u8, 64u8, 15u8, 90u8, 92u8,
                        ],
                    )
                }
                #[doc = " Validation code that is set by the parachain and is to be communicated to collator and"]
                #[doc = " consequently the relay-chain."]
                #[doc = ""]
                #[doc = " This will be cleared in `on_initialize` of each new block if no other pallet already set"]
                #[doc = " the value."]
                pub fn new_validation_code(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "NewValidationCode",
                        vec![],
                        [
                            224u8, 174u8, 53u8, 106u8, 240u8, 49u8, 48u8, 79u8, 219u8, 74u8, 142u8,
                            166u8, 92u8, 204u8, 244u8, 200u8, 43u8, 169u8, 177u8, 207u8, 190u8,
                            106u8, 180u8, 65u8, 245u8, 131u8, 134u8, 4u8, 53u8, 45u8, 76u8, 3u8,
                        ],
                    )
                }
                #[doc = " The [`PersistedValidationData`] set for this block."]
                #[doc = " This value is expected to be set only once per block and it's never stored"]
                #[doc = " in the trie."]
                pub fn validation_data(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::polkadot_primitives::v5::PersistedValidationData<
                            ::subxt::utils::H256,
                            ::core::primitive::u32,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "ValidationData",
                        vec![],
                        [
                            112u8, 58u8, 240u8, 81u8, 219u8, 110u8, 244u8, 186u8, 251u8, 90u8,
                            195u8, 217u8, 229u8, 102u8, 233u8, 24u8, 109u8, 96u8, 219u8, 72u8,
                            139u8, 93u8, 58u8, 140u8, 40u8, 110u8, 167u8, 98u8, 199u8, 12u8, 138u8,
                            131u8,
                        ],
                    )
                }
                #[doc = " Were the validation data set to notify the relay chain?"]
                pub fn did_set_validation_code(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "DidSetValidationCode",
                        vec![],
                        [
                            89u8, 83u8, 74u8, 174u8, 234u8, 188u8, 149u8, 78u8, 140u8, 17u8, 92u8,
                            165u8, 243u8, 87u8, 59u8, 97u8, 135u8, 81u8, 192u8, 86u8, 193u8, 187u8,
                            113u8, 22u8, 108u8, 83u8, 242u8, 208u8, 174u8, 40u8, 49u8, 245u8,
                        ],
                    )
                }
                #[doc = " The relay chain block number associated with the last parachain block."]
                pub fn last_relay_chain_block_number(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "LastRelayChainBlockNumber",
                        vec![],
                        [
                            68u8, 121u8, 6u8, 159u8, 181u8, 94u8, 151u8, 215u8, 225u8, 244u8, 4u8,
                            158u8, 216u8, 85u8, 55u8, 228u8, 197u8, 35u8, 200u8, 33u8, 29u8, 182u8,
                            17u8, 83u8, 59u8, 63u8, 25u8, 180u8, 132u8, 23u8, 97u8, 252u8,
                        ],
                    )
                }
                #[doc = " An option which indicates if the relay-chain restricts signalling a validation code upgrade."]
                #[doc = " In other words, if this is `Some` and [`NewValidationCode`] is `Some` then the produced"]
                #[doc = " candidate will be invalid."]
                #[doc = ""]
                #[doc = " This storage item is a mirror of the corresponding value for the current parachain from the"]
                #[doc = " relay-chain. This value is ephemeral which means it doesn't hit the storage. This value is"]
                #[doc = " set after the inherent."]
                pub fn upgrade_restriction_signal(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::core::option::Option<
                            runtime_types::polkadot_primitives::v5::UpgradeRestriction,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "UpgradeRestrictionSignal",
                        vec![],
                        [
                            61u8, 3u8, 26u8, 6u8, 88u8, 114u8, 109u8, 63u8, 7u8, 115u8, 245u8,
                            198u8, 73u8, 234u8, 28u8, 228u8, 126u8, 27u8, 151u8, 18u8, 133u8, 54u8,
                            144u8, 149u8, 246u8, 43u8, 83u8, 47u8, 77u8, 238u8, 10u8, 196u8,
                        ],
                    )
                }
                #[doc = " The state proof for the last relay parent block."]
                #[doc = ""]
                #[doc = " This field is meant to be updated each block with the validation data inherent. Therefore,"]
                #[doc = " before processing of the inherent, e.g. in `on_initialize` this data may be stale."]
                #[doc = ""]
                #[doc = " This data is also absent from the genesis."]
                pub fn relay_state_proof(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::sp_trie::storage_proof::StorageProof,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "RelayStateProof",
                        vec![],
                        [
                            35u8, 124u8, 167u8, 221u8, 162u8, 145u8, 158u8, 186u8, 57u8, 154u8,
                            225u8, 6u8, 176u8, 13u8, 178u8, 195u8, 209u8, 122u8, 221u8, 26u8,
                            155u8, 126u8, 153u8, 246u8, 101u8, 221u8, 61u8, 145u8, 211u8, 236u8,
                            48u8, 130u8,
                        ],
                    )
                }
                #[doc = " The snapshot of some state related to messaging relevant to the current parachain as per"]
                #[doc = " the relay parent."]
                #[doc = ""]
                #[doc = " This field is meant to be updated each block with the validation data inherent. Therefore,"]
                #[doc = " before processing of the inherent, e.g. in `on_initialize` this data may be stale."]
                #[doc = ""]
                #[doc = " This data is also absent from the genesis."]                pub fn relevant_messaging_state (& self ,) -> :: subxt :: storage :: address :: StaticStorageAddress :: < :: subxt :: metadata :: DecodeStaticType < runtime_types :: cumulus_pallet_parachain_system :: relay_state_snapshot :: MessagingStateSnapshot > , :: subxt :: storage :: address :: Yes , () , () >{
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "RelevantMessagingState",
                        vec![],
                        [
                            106u8, 163u8, 234u8, 149u8, 52u8, 27u8, 151u8, 140u8, 211u8, 216u8,
                            211u8, 43u8, 46u8, 91u8, 73u8, 109u8, 220u8, 228u8, 215u8, 24u8, 6u8,
                            250u8, 231u8, 34u8, 195u8, 105u8, 24u8, 94u8, 21u8, 139u8, 22u8, 28u8,
                        ],
                    )
                }
                #[doc = " The parachain host configuration that was obtained from the relay parent."]
                #[doc = ""]
                #[doc = " This field is meant to be updated each block with the validation data inherent. Therefore,"]
                #[doc = " before processing of the inherent, e.g. in `on_initialize` this data may be stale."]
                #[doc = ""]
                #[doc = " This data is also absent from the genesis."]
                pub fn host_configuration(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::polkadot_primitives::v5::AbridgedHostConfiguration,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "HostConfiguration",
                        vec![],
                        [
                            104u8, 200u8, 30u8, 202u8, 119u8, 204u8, 233u8, 20u8, 67u8, 199u8,
                            47u8, 166u8, 254u8, 152u8, 10u8, 187u8, 240u8, 255u8, 148u8, 201u8,
                            134u8, 41u8, 130u8, 201u8, 112u8, 65u8, 68u8, 103u8, 56u8, 123u8,
                            178u8, 113u8,
                        ],
                    )
                }
                #[doc = " The last downward message queue chain head we have observed."]
                #[doc = ""]
                #[doc = " This value is loaded before and saved after processing inbound downward messages carried"]
                #[doc = " by the system inherent."]
                pub fn last_dmq_mqc_head(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::cumulus_primitives_parachain_inherent::MessageQueueChain,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "LastDmqMqcHead",
                        vec![],
                        [
                            176u8, 255u8, 246u8, 125u8, 36u8, 120u8, 24u8, 44u8, 26u8, 64u8, 236u8,
                            210u8, 189u8, 237u8, 50u8, 78u8, 45u8, 139u8, 58u8, 141u8, 112u8,
                            253u8, 178u8, 198u8, 87u8, 71u8, 77u8, 248u8, 21u8, 145u8, 187u8, 52u8,
                        ],
                    )
                }
                #[doc = " The message queue chain heads we have observed per each channel incoming channel."]
                #[doc = ""]
                #[doc = " This value is loaded before and saved after processing inbound downward messages carried"]
                #[doc = " by the system inherent."]
                pub fn last_hrmp_mqc_heads(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::subxt::utils::KeyedVec<
                            runtime_types::polkadot_parachain::primitives::Id,
                            runtime_types::cumulus_primitives_parachain_inherent::MessageQueueChain,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "LastHrmpMqcHeads",
                        vec![],
                        [
                            55u8, 179u8, 35u8, 16u8, 173u8, 0u8, 122u8, 179u8, 236u8, 98u8, 9u8,
                            112u8, 11u8, 219u8, 241u8, 89u8, 131u8, 198u8, 64u8, 139u8, 103u8,
                            158u8, 77u8, 107u8, 83u8, 236u8, 255u8, 208u8, 47u8, 61u8, 219u8,
                            240u8,
                        ],
                    )
                }
                #[doc = " Number of downward messages processed in a block."]
                #[doc = ""]
                #[doc = " This will be cleared in `on_initialize` of each new block."]
                pub fn processed_downward_messages(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "ProcessedDownwardMessages",
                        vec![],
                        [
                            48u8, 177u8, 84u8, 228u8, 101u8, 235u8, 181u8, 27u8, 66u8, 55u8, 50u8,
                            146u8, 245u8, 223u8, 77u8, 132u8, 178u8, 80u8, 74u8, 90u8, 166u8, 81u8,
                            109u8, 25u8, 91u8, 69u8, 5u8, 69u8, 123u8, 197u8, 160u8, 146u8,
                        ],
                    )
                }
                #[doc = " HRMP watermark that was set in a block."]
                #[doc = ""]
                #[doc = " This will be cleared in `on_initialize` of each new block."]
                pub fn hrmp_watermark(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "HrmpWatermark",
                        vec![],
                        [
                            189u8, 59u8, 183u8, 195u8, 69u8, 185u8, 241u8, 226u8, 62u8, 204u8,
                            230u8, 77u8, 102u8, 75u8, 86u8, 157u8, 249u8, 140u8, 219u8, 72u8, 94u8,
                            64u8, 176u8, 72u8, 34u8, 205u8, 114u8, 103u8, 231u8, 233u8, 206u8,
                            111u8,
                        ],
                    )
                }
                #[doc = " HRMP messages that were sent in a block."]
                #[doc = ""]
                #[doc = " This will be cleared in `on_initialize` of each new block."]
                pub fn hrmp_outbound_messages(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<
                            runtime_types::polkadot_core_primitives::OutboundHrmpMessage<
                                runtime_types::polkadot_parachain::primitives::Id,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "HrmpOutboundMessages",
                        vec![],
                        [
                            74u8, 86u8, 173u8, 248u8, 90u8, 230u8, 71u8, 225u8, 127u8, 164u8,
                            221u8, 62u8, 146u8, 13u8, 73u8, 9u8, 98u8, 168u8, 6u8, 14u8, 97u8,
                            166u8, 45u8, 70u8, 62u8, 210u8, 9u8, 32u8, 83u8, 18u8, 4u8, 201u8,
                        ],
                    )
                }
                #[doc = " Upward messages that were sent in a block."]
                #[doc = ""]
                #[doc = " This will be cleared in `on_initialize` of each new block."]
                pub fn upward_messages(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "UpwardMessages",
                        vec![],
                        [
                            129u8, 208u8, 187u8, 36u8, 48u8, 108u8, 135u8, 56u8, 204u8, 60u8,
                            100u8, 158u8, 113u8, 238u8, 46u8, 92u8, 228u8, 41u8, 178u8, 177u8,
                            208u8, 195u8, 148u8, 149u8, 127u8, 21u8, 93u8, 92u8, 29u8, 115u8, 10u8,
                            248u8,
                        ],
                    )
                }
                #[doc = " Upward messages that are still pending and not yet send to the relay chain."]
                pub fn pending_upward_messages(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "PendingUpwardMessages",
                        vec![],
                        [
                            223u8, 46u8, 224u8, 227u8, 222u8, 119u8, 225u8, 244u8, 59u8, 87u8,
                            127u8, 19u8, 217u8, 237u8, 103u8, 61u8, 6u8, 210u8, 107u8, 201u8,
                            117u8, 25u8, 85u8, 248u8, 36u8, 231u8, 28u8, 202u8, 41u8, 140u8, 208u8,
                            254u8,
                        ],
                    )
                }
                #[doc = " The number of HRMP messages we observed in `on_initialize` and thus used that number for"]
                #[doc = " announcing the weight of `on_initialize` and `on_finalize`."]
                pub fn announced_hrmp_messages_per_candidate(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "AnnouncedHrmpMessagesPerCandidate",
                        vec![],
                        [
                            132u8, 61u8, 162u8, 129u8, 251u8, 243u8, 20u8, 144u8, 162u8, 73u8,
                            237u8, 51u8, 248u8, 41u8, 127u8, 171u8, 180u8, 79u8, 137u8, 23u8, 66u8,
                            134u8, 106u8, 222u8, 182u8, 154u8, 0u8, 145u8, 184u8, 156u8, 36u8,
                            97u8,
                        ],
                    )
                }
                #[doc = " The weight we reserve at the beginning of the block for processing XCMP messages. This"]
                #[doc = " overrides the amount set in the Config trait."]
                pub fn reserved_xcmp_weight_override(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::sp_weights::weight_v2::Weight,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "ReservedXcmpWeightOverride",
                        vec![],
                        [
                            180u8, 90u8, 34u8, 178u8, 1u8, 242u8, 211u8, 97u8, 100u8, 34u8, 39u8,
                            42u8, 142u8, 249u8, 236u8, 194u8, 244u8, 164u8, 96u8, 54u8, 98u8, 46u8,
                            92u8, 196u8, 185u8, 51u8, 231u8, 234u8, 249u8, 143u8, 244u8, 64u8,
                        ],
                    )
                }
                #[doc = " The weight we reserve at the beginning of the block for processing DMP messages. This"]
                #[doc = " overrides the amount set in the Config trait."]
                pub fn reserved_dmp_weight_override(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::sp_weights::weight_v2::Weight,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "ReservedDmpWeightOverride",
                        vec![],
                        [
                            90u8, 122u8, 168u8, 240u8, 95u8, 195u8, 160u8, 109u8, 175u8, 170u8,
                            227u8, 44u8, 139u8, 176u8, 32u8, 161u8, 57u8, 233u8, 56u8, 55u8, 123u8,
                            168u8, 174u8, 96u8, 159u8, 62u8, 186u8, 186u8, 17u8, 70u8, 57u8, 246u8,
                        ],
                    )
                }
                #[doc = " The next authorized upgrade, if there is one."]
                pub fn authorized_upgrade(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::cumulus_pallet_parachain_system::CodeUpgradeAuthorization,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "AuthorizedUpgrade",
                        vec![],
                        [
                            12u8, 212u8, 71u8, 191u8, 89u8, 101u8, 195u8, 3u8, 23u8, 180u8, 233u8,
                            52u8, 53u8, 133u8, 207u8, 94u8, 58u8, 43u8, 221u8, 236u8, 161u8, 41u8,
                            30u8, 194u8, 125u8, 2u8, 118u8, 152u8, 197u8, 49u8, 34u8, 33u8,
                        ],
                    )
                }
                #[doc = " A custom head data that should be returned as result of `validate_block`."]
                #[doc = ""]
                #[doc = " See `Pallet::set_custom_validation_head_data` for more information."]
                pub fn custom_validation_head_data(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainSystem",
                        "CustomValidationHeadData",
                        vec![],
                        [
                            189u8, 150u8, 234u8, 128u8, 111u8, 27u8, 173u8, 92u8, 109u8, 4u8, 98u8,
                            103u8, 158u8, 19u8, 16u8, 5u8, 107u8, 135u8, 126u8, 170u8, 62u8, 64u8,
                            149u8, 80u8, 33u8, 17u8, 83u8, 22u8, 176u8, 118u8, 26u8, 223u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod timestamp {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Set {
                #[codec(compact)]
                pub now: ::core::primitive::u64,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set`]."]
                pub fn set(
                    &self,
                    now: ::core::primitive::u64,
                ) -> ::subxt::tx::StaticTxPayload<Set> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Timestamp",
                        "set",
                        Set { now },
                        [
                            6u8, 97u8, 172u8, 236u8, 118u8, 238u8, 228u8, 114u8, 15u8, 115u8,
                            102u8, 85u8, 66u8, 151u8, 16u8, 33u8, 187u8, 17u8, 166u8, 88u8, 127u8,
                            214u8, 182u8, 51u8, 168u8, 88u8, 43u8, 101u8, 185u8, 8u8, 1u8, 28u8,
                        ],
                    )
                }
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Current time for the current block."]
                pub fn now(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Timestamp",
                        "Now",
                        vec![],
                        [
                            148u8, 53u8, 50u8, 54u8, 13u8, 161u8, 57u8, 150u8, 16u8, 83u8, 144u8,
                            221u8, 59u8, 75u8, 158u8, 130u8, 39u8, 123u8, 106u8, 134u8, 202u8,
                            185u8, 83u8, 85u8, 60u8, 41u8, 120u8, 96u8, 210u8, 34u8, 2u8, 250u8,
                        ],
                    )
                }
                #[doc = " Did the timestamp get updated in this block?"]
                pub fn did_update(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Timestamp",
                        "DidUpdate",
                        vec![],
                        [
                            70u8, 13u8, 92u8, 186u8, 80u8, 151u8, 167u8, 90u8, 158u8, 232u8, 175u8,
                            13u8, 103u8, 135u8, 2u8, 78u8, 16u8, 6u8, 39u8, 158u8, 167u8, 85u8,
                            27u8, 47u8, 122u8, 73u8, 127u8, 26u8, 35u8, 168u8, 72u8, 204u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " The minimum period between blocks. Beware that this is different to the *expected*"]
                #[doc = " period that the block production apparatus provides. Your chosen consensus system will"]
                #[doc = " generally work with this to determine a sensible block time. e.g. For Aura, it will be"]
                #[doc = " double this period on default settings."]
                pub fn minimum_period(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Timestamp",
                        "MinimumPeriod",
                        [
                            128u8, 214u8, 205u8, 242u8, 181u8, 142u8, 124u8, 231u8, 190u8, 146u8,
                            59u8, 226u8, 157u8, 101u8, 103u8, 117u8, 249u8, 65u8, 18u8, 191u8,
                            103u8, 119u8, 53u8, 85u8, 81u8, 96u8, 220u8, 42u8, 184u8, 239u8, 42u8,
                            246u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod parachain_info {
        use super::root_mod;
        use super::runtime_types;
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                pub fn parachain_id(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::polkadot_parachain::primitives::Id,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "ParachainInfo",
                        "ParachainId",
                        vec![],
                        [
                            151u8, 191u8, 241u8, 118u8, 192u8, 47u8, 166u8, 151u8, 217u8, 240u8,
                            165u8, 232u8, 51u8, 113u8, 243u8, 1u8, 89u8, 240u8, 11u8, 1u8, 77u8,
                            104u8, 12u8, 56u8, 17u8, 135u8, 214u8, 19u8, 114u8, 135u8, 66u8, 76u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod balances {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct TransferAllowDeath {
                pub dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub value: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetBalanceDeprecated {
                pub who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub new_free: ::core::primitive::u128,
                #[codec(compact)]
                pub old_reserved: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceTransfer {
                pub source: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                pub dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub value: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct TransferKeepAlive {
                pub dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub value: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct TransferAll {
                pub dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                pub keep_alive: ::core::primitive::bool,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceUnreserve {
                pub who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                pub amount: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UpgradeAccounts {
                pub who: ::std::vec::Vec<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Transfer {
                pub dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub value: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceSetBalance {
                pub who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                #[codec(compact)]
                pub new_free: ::core::primitive::u128,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::transfer_allow_death`]."]
                pub fn transfer_allow_death(
                    &self,
                    dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    value: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<TransferAllowDeath> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "transfer_allow_death",
                        TransferAllowDeath { dest, value },
                        [
                            234u8, 130u8, 149u8, 36u8, 235u8, 112u8, 159u8, 189u8, 104u8, 148u8,
                            108u8, 230u8, 25u8, 198u8, 71u8, 158u8, 112u8, 3u8, 162u8, 25u8, 145u8,
                            252u8, 44u8, 63u8, 47u8, 34u8, 47u8, 158u8, 61u8, 14u8, 120u8, 255u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_balance_deprecated`]."]
                pub fn set_balance_deprecated(
                    &self,
                    who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    new_free: ::core::primitive::u128,
                    old_reserved: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<SetBalanceDeprecated> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "set_balance_deprecated",
                        SetBalanceDeprecated {
                            who,
                            new_free,
                            old_reserved,
                        },
                        [
                            240u8, 107u8, 184u8, 206u8, 78u8, 106u8, 115u8, 152u8, 130u8, 56u8,
                            156u8, 176u8, 105u8, 27u8, 176u8, 187u8, 49u8, 171u8, 229u8, 79u8,
                            254u8, 248u8, 8u8, 162u8, 134u8, 12u8, 89u8, 100u8, 137u8, 102u8,
                            132u8, 158u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_transfer`]."]
                pub fn force_transfer(
                    &self,
                    source: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    value: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<ForceTransfer> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "force_transfer",
                        ForceTransfer {
                            source,
                            dest,
                            value,
                        },
                        [
                            79u8, 174u8, 212u8, 108u8, 184u8, 33u8, 170u8, 29u8, 232u8, 254u8,
                            195u8, 218u8, 221u8, 134u8, 57u8, 99u8, 6u8, 70u8, 181u8, 227u8, 56u8,
                            239u8, 243u8, 158u8, 157u8, 245u8, 36u8, 162u8, 11u8, 237u8, 147u8,
                            15u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::transfer_keep_alive`]."]
                pub fn transfer_keep_alive(
                    &self,
                    dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    value: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<TransferKeepAlive> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "transfer_keep_alive",
                        TransferKeepAlive { dest, value },
                        [
                            112u8, 179u8, 75u8, 168u8, 193u8, 221u8, 9u8, 82u8, 190u8, 113u8,
                            253u8, 13u8, 130u8, 134u8, 170u8, 216u8, 136u8, 111u8, 242u8, 220u8,
                            202u8, 112u8, 47u8, 79u8, 73u8, 244u8, 226u8, 59u8, 240u8, 188u8,
                            210u8, 208u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::transfer_all`]."]
                pub fn transfer_all(
                    &self,
                    dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    keep_alive: ::core::primitive::bool,
                ) -> ::subxt::tx::StaticTxPayload<TransferAll> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "transfer_all",
                        TransferAll { dest, keep_alive },
                        [
                            46u8, 129u8, 29u8, 177u8, 221u8, 107u8, 245u8, 69u8, 238u8, 126u8,
                            145u8, 26u8, 219u8, 208u8, 14u8, 80u8, 149u8, 1u8, 214u8, 63u8, 67u8,
                            201u8, 144u8, 45u8, 129u8, 145u8, 174u8, 71u8, 238u8, 113u8, 208u8,
                            34u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_unreserve`]."]
                pub fn force_unreserve(
                    &self,
                    who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    amount: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<ForceUnreserve> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "force_unreserve",
                        ForceUnreserve { who, amount },
                        [
                            160u8, 146u8, 137u8, 76u8, 157u8, 187u8, 66u8, 148u8, 207u8, 76u8,
                            32u8, 254u8, 82u8, 215u8, 35u8, 161u8, 213u8, 52u8, 32u8, 98u8, 102u8,
                            106u8, 234u8, 123u8, 6u8, 175u8, 184u8, 188u8, 174u8, 106u8, 176u8,
                            78u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::upgrade_accounts`]."]
                pub fn upgrade_accounts(
                    &self,
                    who: ::std::vec::Vec<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<UpgradeAccounts> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "upgrade_accounts",
                        UpgradeAccounts { who },
                        [
                            164u8, 61u8, 119u8, 24u8, 165u8, 46u8, 197u8, 59u8, 39u8, 198u8, 228u8,
                            96u8, 228u8, 45u8, 85u8, 51u8, 37u8, 5u8, 75u8, 40u8, 241u8, 163u8,
                            86u8, 228u8, 151u8, 217u8, 47u8, 105u8, 203u8, 103u8, 207u8, 4u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::transfer`]."]
                pub fn transfer(
                    &self,
                    dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    value: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<Transfer> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "transfer",
                        Transfer { dest, value },
                        [
                            111u8, 222u8, 32u8, 56u8, 171u8, 77u8, 252u8, 29u8, 194u8, 155u8,
                            200u8, 192u8, 198u8, 81u8, 23u8, 115u8, 236u8, 91u8, 218u8, 114u8,
                            107u8, 141u8, 138u8, 100u8, 237u8, 21u8, 58u8, 172u8, 3u8, 20u8, 216u8,
                            38u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_set_balance`]."]
                pub fn force_set_balance(
                    &self,
                    who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                    new_free: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<ForceSetBalance> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Balances",
                        "force_set_balance",
                        ForceSetBalance { who, new_free },
                        [
                            237u8, 4u8, 41u8, 58u8, 62u8, 179u8, 160u8, 4u8, 50u8, 71u8, 178u8,
                            36u8, 130u8, 130u8, 92u8, 229u8, 16u8, 245u8, 169u8, 109u8, 165u8,
                            72u8, 94u8, 70u8, 196u8, 136u8, 37u8, 94u8, 140u8, 215u8, 125u8, 125u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_balances::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An account was created with some free balance."]
            pub struct Endowed {
                pub account: ::subxt::utils::AccountId32,
                pub free_balance: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Endowed {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Endowed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An account was removed whose balance was non-zero but below ExistentialDeposit,"]
            #[doc = "resulting in an outright loss."]
            pub struct DustLost {
                pub account: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for DustLost {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "DustLost";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Transfer succeeded."]
            pub struct Transfer {
                pub from: ::subxt::utils::AccountId32,
                pub to: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Transfer {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Transfer";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A balance was set by root."]
            pub struct BalanceSet {
                pub who: ::subxt::utils::AccountId32,
                pub free: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for BalanceSet {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "BalanceSet";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was reserved (moved from free to reserved)."]
            pub struct Reserved {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Reserved {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Reserved";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was unreserved (moved from reserved to free)."]
            pub struct Unreserved {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Unreserved {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Unreserved";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was moved from the reserve of the first account to the second account."]
            #[doc = "Final argument indicates the destination balance type."]
            pub struct ReserveRepatriated {
                pub from: ::subxt::utils::AccountId32,
                pub to: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
                pub destination_status:
                    runtime_types::frame_support::traits::tokens::misc::BalanceStatus,
            }
            impl ::subxt::events::StaticEvent for ReserveRepatriated {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "ReserveRepatriated";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was deposited (e.g. for transaction fees)."]
            pub struct Deposit {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Deposit {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Deposit";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was withdrawn from the account (e.g. for transaction fees)."]
            pub struct Withdraw {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Withdraw {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Withdraw";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was removed from the account (e.g. for misbehavior)."]
            pub struct Slashed {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Slashed {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Slashed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was minted into an account."]
            pub struct Minted {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Minted {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Minted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was burned from an account."]
            pub struct Burned {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Burned {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Burned";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was suspended from an account (it can be restored later)."]
            pub struct Suspended {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Suspended {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Suspended";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some amount was restored into an account."]
            pub struct Restored {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Restored {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Restored";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An account was upgraded."]
            pub struct Upgraded {
                pub who: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for Upgraded {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Upgraded";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "Total issuance was increased by `amount`, creating a credit to be balanced."]
            pub struct Issued {
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Issued {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Issued";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "Total issuance was decreased by `amount`, creating a debt to be balanced."]
            pub struct Rescinded {
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Rescinded {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Rescinded";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was locked."]
            pub struct Locked {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Locked {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Locked";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was unlocked."]
            pub struct Unlocked {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Unlocked {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Unlocked";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was frozen."]
            pub struct Frozen {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Frozen {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Frozen";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some balance was thawed."]
            pub struct Thawed {
                pub who: ::subxt::utils::AccountId32,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for Thawed {
                const PALLET: &'static str = "Balances";
                const EVENT: &'static str = "Thawed";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The total units issued in the system."]
                pub fn total_issuance(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "TotalIssuance",
                        vec![],
                        [
                            1u8, 206u8, 252u8, 237u8, 6u8, 30u8, 20u8, 232u8, 164u8, 115u8, 51u8,
                            156u8, 156u8, 206u8, 241u8, 187u8, 44u8, 84u8, 25u8, 164u8, 235u8,
                            20u8, 86u8, 242u8, 124u8, 23u8, 28u8, 140u8, 26u8, 73u8, 231u8, 51u8,
                        ],
                    )
                }
                #[doc = " The total units of outstanding deactivated balance in the system."]
                pub fn inactive_issuance(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "InactiveIssuance",
                        vec![],
                        [
                            74u8, 203u8, 111u8, 142u8, 225u8, 104u8, 173u8, 51u8, 226u8, 12u8,
                            85u8, 135u8, 41u8, 206u8, 177u8, 238u8, 94u8, 246u8, 184u8, 250u8,
                            140u8, 213u8, 91u8, 118u8, 163u8, 111u8, 211u8, 46u8, 204u8, 160u8,
                            154u8, 21u8,
                        ],
                    )
                }
                #[doc = " The Balances pallet example of storing the balance of an account."]
                #[doc = ""]
                #[doc = " # Example"]
                #[doc = ""]
                #[doc = " ```nocompile"]
                #[doc = "  impl pallet_balances::Config for Runtime {"]
                #[doc = "    type AccountStore = StorageMapShim<Self::Account<Runtime>, frame_system::Provider<Runtime>, AccountId, Self::AccountData<Balance>>"]
                #[doc = "  }"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " You can also store the balance of an account in the `System` pallet."]
                #[doc = ""]
                #[doc = " # Example"]
                #[doc = ""]
                #[doc = " ```nocompile"]
                #[doc = "  impl pallet_balances::Config for Runtime {"]
                #[doc = "   type AccountStore = System"]
                #[doc = "  }"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " But this comes with tradeoffs, storing account balances in the system pallet stores"]
                #[doc = " `frame_system` data alongside the account data contrary to storing account balances in the"]
                #[doc = " `Balances` pallet, which uses a `StorageMap` to store balances data only."]
                #[doc = " NOTE: This is only used in the case that this pallet is used to store balances."]
                pub fn account(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_balances::types::AccountData<::core::primitive::u128>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Account",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            109u8, 250u8, 18u8, 96u8, 139u8, 232u8, 4u8, 139u8, 133u8, 239u8, 30u8,
                            237u8, 73u8, 209u8, 143u8, 160u8, 94u8, 248u8, 124u8, 43u8, 224u8,
                            165u8, 11u8, 6u8, 176u8, 144u8, 189u8, 161u8, 174u8, 210u8, 56u8,
                            225u8,
                        ],
                    )
                }
                #[doc = " The Balances pallet example of storing the balance of an account."]
                #[doc = ""]
                #[doc = " # Example"]
                #[doc = ""]
                #[doc = " ```nocompile"]
                #[doc = "  impl pallet_balances::Config for Runtime {"]
                #[doc = "    type AccountStore = StorageMapShim<Self::Account<Runtime>, frame_system::Provider<Runtime>, AccountId, Self::AccountData<Balance>>"]
                #[doc = "  }"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " You can also store the balance of an account in the `System` pallet."]
                #[doc = ""]
                #[doc = " # Example"]
                #[doc = ""]
                #[doc = " ```nocompile"]
                #[doc = "  impl pallet_balances::Config for Runtime {"]
                #[doc = "   type AccountStore = System"]
                #[doc = "  }"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " But this comes with tradeoffs, storing account balances in the system pallet stores"]
                #[doc = " `frame_system` data alongside the account data contrary to storing account balances in the"]
                #[doc = " `Balances` pallet, which uses a `StorageMap` to store balances data only."]
                #[doc = " NOTE: This is only used in the case that this pallet is used to store balances."]
                pub fn account_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_balances::types::AccountData<::core::primitive::u128>,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Account",
                        Vec::new(),
                        [
                            109u8, 250u8, 18u8, 96u8, 139u8, 232u8, 4u8, 139u8, 133u8, 239u8, 30u8,
                            237u8, 73u8, 209u8, 143u8, 160u8, 94u8, 248u8, 124u8, 43u8, 224u8,
                            165u8, 11u8, 6u8, 176u8, 144u8, 189u8, 161u8, 174u8, 210u8, 56u8,
                            225u8,
                        ],
                    )
                }
                #[doc = " Any liquidity locks on some account balances."]
                #[doc = " NOTE: Should only be accessed when setting, changing and freeing a lock."]
                pub fn locks(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
                            runtime_types::pallet_balances::types::BalanceLock<
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Locks",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            216u8, 253u8, 87u8, 73u8, 24u8, 218u8, 35u8, 0u8, 244u8, 134u8, 195u8,
                            58u8, 255u8, 64u8, 153u8, 212u8, 210u8, 232u8, 4u8, 122u8, 90u8, 212u8,
                            136u8, 14u8, 127u8, 232u8, 8u8, 192u8, 40u8, 233u8, 18u8, 250u8,
                        ],
                    )
                }
                #[doc = " Any liquidity locks on some account balances."]
                #[doc = " NOTE: Should only be accessed when setting, changing and freeing a lock."]
                pub fn locks_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
                            runtime_types::pallet_balances::types::BalanceLock<
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Locks",
                        Vec::new(),
                        [
                            216u8, 253u8, 87u8, 73u8, 24u8, 218u8, 35u8, 0u8, 244u8, 134u8, 195u8,
                            58u8, 255u8, 64u8, 153u8, 212u8, 210u8, 232u8, 4u8, 122u8, 90u8, 212u8,
                            136u8, 14u8, 127u8, 232u8, 8u8, 192u8, 40u8, 233u8, 18u8, 250u8,
                        ],
                    )
                }
                #[doc = " Named reserves on some account balances."]
                pub fn reserves(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::ReserveData<
                                [::core::primitive::u8; 8usize],
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Reserves",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            17u8, 32u8, 191u8, 46u8, 76u8, 220u8, 101u8, 100u8, 42u8, 250u8, 128u8,
                            167u8, 117u8, 44u8, 85u8, 96u8, 105u8, 216u8, 16u8, 147u8, 74u8, 55u8,
                            183u8, 94u8, 160u8, 177u8, 26u8, 187u8, 71u8, 197u8, 187u8, 163u8,
                        ],
                    )
                }
                #[doc = " Named reserves on some account balances."]
                pub fn reserves_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::ReserveData<
                                [::core::primitive::u8; 8usize],
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Reserves",
                        Vec::new(),
                        [
                            17u8, 32u8, 191u8, 46u8, 76u8, 220u8, 101u8, 100u8, 42u8, 250u8, 128u8,
                            167u8, 117u8, 44u8, 85u8, 96u8, 105u8, 216u8, 16u8, 147u8, 74u8, 55u8,
                            183u8, 94u8, 160u8, 177u8, 26u8, 187u8, 71u8, 197u8, 187u8, 163u8,
                        ],
                    )
                }
                #[doc = " Holds on account balances."]
                pub fn holds(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::IdAmount<
                                runtime_types::bridge_hub_rococo_runtime::RuntimeHoldReason,
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Holds",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            209u8, 212u8, 29u8, 29u8, 197u8, 118u8, 65u8, 232u8, 89u8, 192u8,
                            255u8, 71u8, 50u8, 146u8, 96u8, 255u8, 32u8, 91u8, 147u8, 161u8, 213u8,
                            215u8, 63u8, 134u8, 233u8, 70u8, 227u8, 132u8, 224u8, 30u8, 70u8,
                            187u8,
                        ],
                    )
                }
                #[doc = " Holds on account balances."]
                pub fn holds_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::IdAmount<
                                runtime_types::bridge_hub_rococo_runtime::RuntimeHoldReason,
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Holds",
                        Vec::new(),
                        [
                            209u8, 212u8, 29u8, 29u8, 197u8, 118u8, 65u8, 232u8, 89u8, 192u8,
                            255u8, 71u8, 50u8, 146u8, 96u8, 255u8, 32u8, 91u8, 147u8, 161u8, 213u8,
                            215u8, 63u8, 134u8, 233u8, 70u8, 227u8, 132u8, 224u8, 30u8, 70u8,
                            187u8,
                        ],
                    )
                }
                #[doc = " Freeze locks on account balances."]
                pub fn freezes(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::IdAmount<
                                (),
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Freezes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            211u8, 24u8, 237u8, 217u8, 47u8, 230u8, 147u8, 39u8, 112u8, 209u8,
                            193u8, 47u8, 242u8, 13u8, 241u8, 0u8, 100u8, 45u8, 116u8, 130u8, 246u8,
                            196u8, 50u8, 134u8, 135u8, 112u8, 206u8, 1u8, 12u8, 53u8, 106u8, 131u8,
                        ],
                    )
                }
                #[doc = " Freeze locks on account balances."]
                pub fn freezes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_balances::types::IdAmount<
                                (),
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Balances",
                        "Freezes",
                        Vec::new(),
                        [
                            211u8, 24u8, 237u8, 217u8, 47u8, 230u8, 147u8, 39u8, 112u8, 209u8,
                            193u8, 47u8, 242u8, 13u8, 241u8, 0u8, 100u8, 45u8, 116u8, 130u8, 246u8,
                            196u8, 50u8, 134u8, 135u8, 112u8, 206u8, 1u8, 12u8, 53u8, 106u8, 131u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " The minimum amount required to keep an account open. MUST BE GREATER THAN ZERO!"]
                #[doc = ""]
                #[doc = " If you *really* need it to be zero, you can enable the feature `insecure_zero_ed` for"]
                #[doc = " this pallet. However, you do so at your own risk: this will open up a major DoS vector."]
                #[doc = " In case you have multiple sources of provider references, you may also get unexpected"]
                #[doc = " behaviour if you set this to zero."]
                #[doc = ""]
                #[doc = " Bottom line: Do yourself a favour and make it at least one!"]
                pub fn existential_deposit(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Balances",
                        "ExistentialDeposit",
                        [
                            84u8, 157u8, 140u8, 4u8, 93u8, 57u8, 29u8, 133u8, 105u8, 200u8, 214u8,
                            27u8, 144u8, 208u8, 218u8, 160u8, 130u8, 109u8, 101u8, 54u8, 210u8,
                            136u8, 71u8, 63u8, 49u8, 237u8, 234u8, 15u8, 178u8, 98u8, 148u8, 156u8,
                        ],
                    )
                }
                #[doc = " The maximum number of locks that should exist on an account."]
                #[doc = " Not strictly enforced, but used for weight estimation."]
                pub fn max_locks(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Balances",
                        "MaxLocks",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The maximum number of named reserves that can exist on an account."]
                pub fn max_reserves(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Balances",
                        "MaxReserves",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The maximum number of holds that can exist on an account at any time."]
                pub fn max_holds(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Balances",
                        "MaxHolds",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The maximum number of individual freeze locks that can exist on an account at any time."]
                pub fn max_freezes(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Balances",
                        "MaxFreezes",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod transaction_payment {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_transaction_payment::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,"]
            #[doc = "has been paid by `who`."]
            pub struct TransactionFeePaid {
                pub who: ::subxt::utils::AccountId32,
                pub actual_fee: ::core::primitive::u128,
                pub tip: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for TransactionFeePaid {
                const PALLET: &'static str = "TransactionPayment";
                const EVENT: &'static str = "TransactionFeePaid";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                pub fn next_fee_multiplier(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::sp_arithmetic::fixed_point::FixedU128,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "TransactionPayment",
                        "NextFeeMultiplier",
                        vec![],
                        [
                            210u8, 0u8, 206u8, 165u8, 183u8, 10u8, 206u8, 52u8, 14u8, 90u8, 218u8,
                            197u8, 189u8, 125u8, 113u8, 216u8, 52u8, 161u8, 45u8, 24u8, 245u8,
                            237u8, 121u8, 41u8, 106u8, 29u8, 45u8, 129u8, 250u8, 203u8, 206u8,
                            180u8,
                        ],
                    )
                }
                pub fn storage_version(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_transaction_payment::Releases,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "TransactionPayment",
                        "StorageVersion",
                        vec![],
                        [
                            219u8, 243u8, 82u8, 176u8, 65u8, 5u8, 132u8, 114u8, 8u8, 82u8, 176u8,
                            200u8, 97u8, 150u8, 177u8, 164u8, 166u8, 11u8, 34u8, 12u8, 12u8, 198u8,
                            58u8, 191u8, 186u8, 221u8, 221u8, 119u8, 181u8, 253u8, 154u8, 228u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " A fee mulitplier for `Operational` extrinsics to compute \"virtual tip\" to boost their"]
                #[doc = " `priority`"]
                #[doc = ""]
                #[doc = " This value is multipled by the `final_fee` to obtain a \"virtual tip\" that is later"]
                #[doc = " added to a tip component in regular `priority` calculations."]
                #[doc = " It means that a `Normal` transaction can front-run a similarly-sized `Operational`"]
                #[doc = " extrinsic (with no tip), by including a tip value greater than the virtual tip."]
                #[doc = ""]
                #[doc = " ```rust,ignore"]
                #[doc = " // For `Normal`"]
                #[doc = " let priority = priority_calc(tip);"]
                #[doc = ""]
                #[doc = " // For `Operational`"]
                #[doc = " let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;"]
                #[doc = " let priority = priority_calc(tip + virtual_tip);"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " Note that since we use `final_fee` the multiplier applies also to the regular `tip`"]
                #[doc = " sent with the transaction. So, not only does the transaction get a priority bump based"]
                #[doc = " on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`"]
                #[doc = " transactions."]
                pub fn operational_fee_multiplier(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u8>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "TransactionPayment",
                        "OperationalFeeMultiplier",
                        [
                            141u8, 130u8, 11u8, 35u8, 226u8, 114u8, 92u8, 179u8, 168u8, 110u8,
                            28u8, 91u8, 221u8, 64u8, 4u8, 148u8, 201u8, 193u8, 185u8, 66u8, 226u8,
                            114u8, 97u8, 79u8, 62u8, 212u8, 202u8, 114u8, 237u8, 228u8, 183u8,
                            165u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod authorship {
        use super::root_mod;
        use super::runtime_types;
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Author of current block."]
                pub fn author(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Authorship",
                        "Author",
                        vec![],
                        [
                            149u8, 42u8, 33u8, 147u8, 190u8, 207u8, 174u8, 227u8, 190u8, 110u8,
                            25u8, 131u8, 5u8, 167u8, 237u8, 188u8, 188u8, 33u8, 177u8, 126u8,
                            181u8, 49u8, 126u8, 118u8, 46u8, 128u8, 154u8, 95u8, 15u8, 91u8, 103u8,
                            113u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod collator_selection {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetInvulnerables {
                pub new: ::std::vec::Vec<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct SetDesiredCandidates {
                pub max: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct SetCandidacyBond {
                pub bond: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RegisterAsCandidate;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct LeaveIntent;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AddInvulnerable {
                pub who: ::subxt::utils::AccountId32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RemoveInvulnerable {
                pub who: ::subxt::utils::AccountId32,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_invulnerables`]."]
                pub fn set_invulnerables(
                    &self,
                    new: ::std::vec::Vec<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetInvulnerables> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "set_invulnerables",
                        SetInvulnerables { new },
                        [
                            120u8, 177u8, 166u8, 239u8, 2u8, 102u8, 76u8, 143u8, 218u8, 130u8,
                            168u8, 152u8, 200u8, 107u8, 221u8, 30u8, 252u8, 18u8, 108u8, 147u8,
                            81u8, 251u8, 183u8, 185u8, 0u8, 184u8, 100u8, 251u8, 95u8, 168u8, 26u8,
                            142u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_desired_candidates`]."]
                pub fn set_desired_candidates(
                    &self,
                    max: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<SetDesiredCandidates> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "set_desired_candidates",
                        SetDesiredCandidates { max },
                        [
                            181u8, 32u8, 138u8, 37u8, 254u8, 213u8, 197u8, 224u8, 82u8, 26u8, 3u8,
                            113u8, 11u8, 146u8, 251u8, 35u8, 250u8, 202u8, 209u8, 2u8, 231u8,
                            176u8, 216u8, 124u8, 125u8, 43u8, 52u8, 126u8, 150u8, 140u8, 20u8,
                            113u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_candidacy_bond`]."]
                pub fn set_candidacy_bond(
                    &self,
                    bond: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<SetCandidacyBond> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "set_candidacy_bond",
                        SetCandidacyBond { bond },
                        [
                            42u8, 173u8, 79u8, 226u8, 224u8, 202u8, 70u8, 185u8, 125u8, 17u8,
                            123u8, 99u8, 107u8, 163u8, 67u8, 75u8, 110u8, 65u8, 248u8, 179u8, 39u8,
                            177u8, 135u8, 186u8, 66u8, 237u8, 30u8, 73u8, 163u8, 98u8, 81u8, 152u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::register_as_candidate`]."]
                pub fn register_as_candidate(
                    &self,
                ) -> ::subxt::tx::StaticTxPayload<RegisterAsCandidate> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "register_as_candidate",
                        RegisterAsCandidate {},
                        [
                            63u8, 11u8, 114u8, 142u8, 89u8, 78u8, 120u8, 214u8, 22u8, 215u8, 125u8,
                            60u8, 203u8, 89u8, 141u8, 126u8, 124u8, 167u8, 70u8, 240u8, 85u8,
                            253u8, 34u8, 245u8, 67u8, 46u8, 240u8, 195u8, 57u8, 81u8, 138u8, 69u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::leave_intent`]."]
                pub fn leave_intent(&self) -> ::subxt::tx::StaticTxPayload<LeaveIntent> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "leave_intent",
                        LeaveIntent {},
                        [
                            217u8, 3u8, 35u8, 71u8, 152u8, 203u8, 203u8, 212u8, 25u8, 113u8, 158u8,
                            124u8, 161u8, 154u8, 32u8, 47u8, 116u8, 134u8, 11u8, 201u8, 154u8,
                            40u8, 138u8, 163u8, 184u8, 188u8, 33u8, 237u8, 219u8, 40u8, 63u8,
                            221u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::add_invulnerable`]."]
                pub fn add_invulnerable(
                    &self,
                    who: ::subxt::utils::AccountId32,
                ) -> ::subxt::tx::StaticTxPayload<AddInvulnerable> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "add_invulnerable",
                        AddInvulnerable { who },
                        [
                            159u8, 175u8, 217u8, 28u8, 231u8, 252u8, 194u8, 220u8, 157u8, 161u8,
                            221u8, 229u8, 176u8, 129u8, 163u8, 48u8, 222u8, 228u8, 134u8, 98u8,
                            186u8, 95u8, 201u8, 182u8, 253u8, 62u8, 228u8, 237u8, 162u8, 59u8,
                            118u8, 50u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::remove_invulnerable`]."]
                pub fn remove_invulnerable(
                    &self,
                    who: ::subxt::utils::AccountId32,
                ) -> ::subxt::tx::StaticTxPayload<RemoveInvulnerable> {
                    ::subxt::tx::StaticTxPayload::new(
                        "CollatorSelection",
                        "remove_invulnerable",
                        RemoveInvulnerable { who },
                        [
                            211u8, 227u8, 103u8, 96u8, 140u8, 105u8, 42u8, 168u8, 233u8, 31u8,
                            221u8, 48u8, 53u8, 137u8, 30u8, 27u8, 63u8, 167u8, 137u8, 241u8, 195u8,
                            108u8, 185u8, 113u8, 229u8, 30u8, 52u8, 58u8, 67u8, 154u8, 85u8, 47u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_collator_selection::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "New Invulnerables were set."]
            pub struct NewInvulnerables {
                pub invulnerables: ::std::vec::Vec<::subxt::utils::AccountId32>,
            }
            impl ::subxt::events::StaticEvent for NewInvulnerables {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "NewInvulnerables";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A new Invulnerable was added."]
            pub struct InvulnerableAdded {
                pub account_id: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for InvulnerableAdded {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "InvulnerableAdded";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An Invulnerable was removed."]
            pub struct InvulnerableRemoved {
                pub account_id: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for InvulnerableRemoved {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "InvulnerableRemoved";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "The number of desired candidates was set."]
            pub struct NewDesiredCandidates {
                pub desired_candidates: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for NewDesiredCandidates {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "NewDesiredCandidates";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "The candidacy bond was set."]
            pub struct NewCandidacyBond {
                pub bond_amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for NewCandidacyBond {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "NewCandidacyBond";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A new candidate joined."]
            pub struct CandidateAdded {
                pub account_id: ::subxt::utils::AccountId32,
                pub deposit: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for CandidateAdded {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "CandidateAdded";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A candidate was removed."]
            pub struct CandidateRemoved {
                pub account_id: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for CandidateRemoved {
                const PALLET: &'static str = "CollatorSelection";
                const EVENT: &'static str = "CandidateRemoved";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The invulnerable, permissioned collators. This list must be sorted."]
                pub fn invulnerables(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::subxt::utils::AccountId32,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "Invulnerables",
                        vec![],
                        [
                            215u8, 62u8, 140u8, 81u8, 0u8, 189u8, 182u8, 139u8, 32u8, 42u8, 20u8,
                            223u8, 81u8, 212u8, 100u8, 97u8, 146u8, 253u8, 75u8, 123u8, 240u8,
                            125u8, 249u8, 62u8, 226u8, 70u8, 57u8, 206u8, 16u8, 74u8, 52u8, 72u8,
                        ],
                    )
                }
                #[doc = " The (community, limited) collation candidates."]
                pub fn candidates(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::pallet_collator_selection::pallet::CandidateInfo<
                                ::subxt::utils::AccountId32,
                                ::core::primitive::u128,
                            >,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "Candidates",
                        vec![],
                        [
                            28u8, 116u8, 232u8, 94u8, 147u8, 216u8, 214u8, 30u8, 26u8, 241u8, 68u8,
                            108u8, 165u8, 107u8, 89u8, 136u8, 111u8, 239u8, 150u8, 42u8, 210u8,
                            214u8, 192u8, 234u8, 29u8, 41u8, 157u8, 169u8, 120u8, 126u8, 192u8,
                            32u8,
                        ],
                    )
                }
                #[doc = " Last block authored by collator."]
                pub fn last_authored_block(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "LastAuthoredBlock",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            53u8, 30u8, 243u8, 31u8, 228u8, 231u8, 175u8, 153u8, 204u8, 241u8,
                            76u8, 147u8, 6u8, 202u8, 255u8, 89u8, 30u8, 129u8, 85u8, 92u8, 10u8,
                            97u8, 177u8, 129u8, 88u8, 196u8, 7u8, 255u8, 74u8, 52u8, 28u8, 0u8,
                        ],
                    )
                }
                #[doc = " Last block authored by collator."]
                pub fn last_authored_block_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "LastAuthoredBlock",
                        Vec::new(),
                        [
                            53u8, 30u8, 243u8, 31u8, 228u8, 231u8, 175u8, 153u8, 204u8, 241u8,
                            76u8, 147u8, 6u8, 202u8, 255u8, 89u8, 30u8, 129u8, 85u8, 92u8, 10u8,
                            97u8, 177u8, 129u8, 88u8, 196u8, 7u8, 255u8, 74u8, 52u8, 28u8, 0u8,
                        ],
                    )
                }
                #[doc = " Desired number of candidates."]
                #[doc = ""]
                #[doc = " This should ideally always be less than [`Config::MaxCandidates`] for weights to be correct."]
                pub fn desired_candidates(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "DesiredCandidates",
                        vec![],
                        [
                            161u8, 170u8, 254u8, 76u8, 112u8, 146u8, 144u8, 7u8, 177u8, 152u8,
                            146u8, 60u8, 143u8, 237u8, 1u8, 168u8, 176u8, 33u8, 103u8, 35u8, 39u8,
                            233u8, 107u8, 253u8, 47u8, 183u8, 11u8, 86u8, 230u8, 13u8, 127u8,
                            133u8,
                        ],
                    )
                }
                #[doc = " Fixed amount to deposit to become a collator."]
                #[doc = ""]
                #[doc = " When a collator calls `leave_intent` they immediately receive the deposit back."]
                pub fn candidacy_bond(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "CollatorSelection",
                        "CandidacyBond",
                        vec![],
                        [
                            1u8, 153u8, 211u8, 74u8, 138u8, 178u8, 81u8, 9u8, 205u8, 117u8, 102u8,
                            182u8, 56u8, 184u8, 56u8, 62u8, 193u8, 82u8, 224u8, 218u8, 253u8,
                            194u8, 250u8, 55u8, 220u8, 107u8, 157u8, 175u8, 62u8, 35u8, 224u8,
                            183u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod session {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetKeys {
                pub keys: runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                pub proof: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct PurgeKeys;
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_keys`]."]
                pub fn set_keys(
                    &self,
                    keys: runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                    proof: ::std::vec::Vec<::core::primitive::u8>,
                ) -> ::subxt::tx::StaticTxPayload<SetKeys> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Session",
                        "set_keys",
                        SetKeys { keys, proof },
                        [
                            199u8, 56u8, 39u8, 236u8, 44u8, 88u8, 207u8, 0u8, 187u8, 195u8, 218u8,
                            94u8, 126u8, 128u8, 37u8, 162u8, 216u8, 223u8, 36u8, 165u8, 18u8, 37u8,
                            16u8, 72u8, 136u8, 28u8, 134u8, 230u8, 231u8, 48u8, 230u8, 122u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::purge_keys`]."]
                pub fn purge_keys(&self) -> ::subxt::tx::StaticTxPayload<PurgeKeys> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Session",
                        "purge_keys",
                        PurgeKeys {},
                        [
                            200u8, 255u8, 4u8, 213u8, 188u8, 92u8, 99u8, 116u8, 163u8, 152u8, 29u8,
                            35u8, 133u8, 119u8, 246u8, 44u8, 91u8, 31u8, 145u8, 23u8, 213u8, 64u8,
                            71u8, 242u8, 207u8, 239u8, 231u8, 37u8, 61u8, 63u8, 190u8, 35u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_session::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "New session has happened. Note that the argument is the session index, not the"]
            #[doc = "block number as the type might suggest."]
            pub struct NewSession {
                pub session_index: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for NewSession {
                const PALLET: &'static str = "Session";
                const EVENT: &'static str = "NewSession";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The current set of validators."]
                pub fn validators(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<::subxt::utils::AccountId32>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "Validators",
                        vec![],
                        [
                            144u8, 235u8, 200u8, 43u8, 151u8, 57u8, 147u8, 172u8, 201u8, 202u8,
                            242u8, 96u8, 57u8, 76u8, 124u8, 77u8, 42u8, 113u8, 218u8, 220u8, 230u8,
                            32u8, 151u8, 152u8, 172u8, 106u8, 60u8, 227u8, 122u8, 118u8, 137u8,
                            68u8,
                        ],
                    )
                }
                #[doc = " Current index of the session."]
                pub fn current_index(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "CurrentIndex",
                        vec![],
                        [
                            148u8, 179u8, 159u8, 15u8, 197u8, 95u8, 214u8, 30u8, 209u8, 251u8,
                            183u8, 231u8, 91u8, 25u8, 181u8, 191u8, 143u8, 252u8, 227u8, 80u8,
                            159u8, 66u8, 194u8, 67u8, 113u8, 74u8, 111u8, 91u8, 218u8, 187u8,
                            130u8, 40u8,
                        ],
                    )
                }
                #[doc = " True if the underlying economic identities or weighting behind the validators"]
                #[doc = " has changed in the queued validator set."]
                pub fn queued_changed(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "QueuedChanged",
                        vec![],
                        [
                            105u8, 140u8, 235u8, 218u8, 96u8, 100u8, 252u8, 10u8, 58u8, 221u8,
                            244u8, 251u8, 67u8, 91u8, 80u8, 202u8, 152u8, 42u8, 50u8, 113u8, 200u8,
                            247u8, 59u8, 213u8, 77u8, 195u8, 1u8, 150u8, 220u8, 18u8, 245u8, 46u8,
                        ],
                    )
                }
                #[doc = " The queued keys for the next session. When the next session begins, these keys"]
                #[doc = " will be used to determine the validator's session keys."]
                pub fn queued_keys(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<(
                            ::subxt::utils::AccountId32,
                            runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                        )>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "QueuedKeys",
                        vec![],
                        [
                            42u8, 134u8, 252u8, 233u8, 29u8, 69u8, 168u8, 107u8, 77u8, 70u8, 80u8,
                            189u8, 149u8, 227u8, 77u8, 74u8, 100u8, 175u8, 10u8, 162u8, 145u8,
                            105u8, 85u8, 196u8, 169u8, 195u8, 116u8, 255u8, 112u8, 122u8, 112u8,
                            133u8,
                        ],
                    )
                }
                #[doc = " Indices of disabled validators."]
                #[doc = ""]
                #[doc = " The vec is always kept sorted so that we can find whether a given validator is"]
                #[doc = " disabled using binary search. It gets cleared when `on_session_ending` returns"]
                #[doc = " a new set of identities."]
                pub fn disabled_validators(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u32>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "DisabledValidators",
                        vec![],
                        [
                            135u8, 22u8, 22u8, 97u8, 82u8, 217u8, 144u8, 141u8, 121u8, 240u8,
                            189u8, 16u8, 176u8, 88u8, 177u8, 31u8, 20u8, 242u8, 73u8, 104u8, 11u8,
                            110u8, 214u8, 34u8, 52u8, 217u8, 106u8, 33u8, 174u8, 174u8, 198u8,
                            84u8,
                        ],
                    )
                }
                #[doc = " The next session keys for a validator."]
                pub fn next_keys(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "NextKeys",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            21u8, 0u8, 237u8, 42u8, 156u8, 77u8, 229u8, 211u8, 105u8, 8u8, 231u8,
                            5u8, 246u8, 188u8, 69u8, 143u8, 202u8, 240u8, 252u8, 253u8, 106u8,
                            37u8, 51u8, 244u8, 206u8, 199u8, 249u8, 37u8, 17u8, 102u8, 20u8, 246u8,
                        ],
                    )
                }
                #[doc = " The next session keys for a validator."]
                pub fn next_keys_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "NextKeys",
                        Vec::new(),
                        [
                            21u8, 0u8, 237u8, 42u8, 156u8, 77u8, 229u8, 211u8, 105u8, 8u8, 231u8,
                            5u8, 246u8, 188u8, 69u8, 143u8, 202u8, 240u8, 252u8, 253u8, 106u8,
                            37u8, 51u8, 244u8, 206u8, 199u8, 249u8, 37u8, 17u8, 102u8, 20u8, 246u8,
                        ],
                    )
                }
                #[doc = " The owner of a key. The key is the `KeyTypeId` + the encoded key."]
                pub fn key_owner(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::sp_core::crypto::KeyTypeId>,
                    _1: impl ::std::borrow::Borrow<[::core::primitive::u8]>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "KeyOwner",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            &(_0.borrow(), _1.borrow()),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            4u8, 91u8, 25u8, 84u8, 250u8, 201u8, 174u8, 129u8, 201u8, 58u8, 197u8,
                            199u8, 137u8, 240u8, 118u8, 33u8, 99u8, 2u8, 195u8, 57u8, 53u8, 172u8,
                            0u8, 148u8, 203u8, 144u8, 149u8, 64u8, 135u8, 254u8, 242u8, 215u8,
                        ],
                    )
                }
                #[doc = " The owner of a key. The key is the `KeyTypeId` + the encoded key."]
                pub fn key_owner_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Session",
                        "KeyOwner",
                        Vec::new(),
                        [
                            4u8, 91u8, 25u8, 84u8, 250u8, 201u8, 174u8, 129u8, 201u8, 58u8, 197u8,
                            199u8, 137u8, 240u8, 118u8, 33u8, 99u8, 2u8, 195u8, 57u8, 53u8, 172u8,
                            0u8, 148u8, 203u8, 144u8, 149u8, 64u8, 135u8, 254u8, 242u8, 215u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod aura {
        use super::root_mod;
        use super::runtime_types;
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The current authority set."]
                pub fn authorities(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::sp_consensus_aura::sr25519::app_sr25519::Public,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Aura",
                        "Authorities",
                        vec![],
                        [
                            199u8, 89u8, 94u8, 48u8, 249u8, 35u8, 105u8, 90u8, 15u8, 86u8, 218u8,
                            85u8, 22u8, 236u8, 228u8, 36u8, 137u8, 64u8, 236u8, 171u8, 242u8,
                            217u8, 91u8, 240u8, 205u8, 205u8, 226u8, 16u8, 147u8, 235u8, 181u8,
                            41u8,
                        ],
                    )
                }
                #[doc = " The current slot of this block."]
                #[doc = ""]
                #[doc = " This will be set in `on_initialize`."]
                pub fn current_slot(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::sp_consensus_slots::Slot>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Aura",
                        "CurrentSlot",
                        vec![],
                        [
                            139u8, 237u8, 185u8, 137u8, 251u8, 179u8, 69u8, 167u8, 133u8, 168u8,
                            204u8, 64u8, 178u8, 123u8, 92u8, 250u8, 119u8, 190u8, 208u8, 178u8,
                            208u8, 176u8, 124u8, 187u8, 74u8, 165u8, 33u8, 78u8, 161u8, 206u8, 8u8,
                            108u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod aura_ext {
        use super::root_mod;
        use super::runtime_types;
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Serves as cache for the authorities."]
                #[doc = ""]
                #[doc = " The authorities in AuRa are overwritten in `on_initialize` when we switch to a new session,"]
                #[doc = " but we require the old authorities to verify the seal when validating a PoV. This will always"]
                #[doc = " be updated to the latest AuRa authorities in `on_finalize`."]
                pub fn authorities(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::sp_consensus_aura::sr25519::app_sr25519::Public,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "AuraExt",
                        "Authorities",
                        vec![],
                        [
                            199u8, 89u8, 94u8, 48u8, 249u8, 35u8, 105u8, 90u8, 15u8, 86u8, 218u8,
                            85u8, 22u8, 236u8, 228u8, 36u8, 137u8, 64u8, 236u8, 171u8, 242u8,
                            217u8, 91u8, 240u8, 205u8, 205u8, 226u8, 16u8, 147u8, 235u8, 181u8,
                            41u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod xcmp_queue {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ServiceOverweight {
                pub index: ::core::primitive::u64,
                pub weight_limit: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SuspendXcmExecution;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ResumeXcmExecution;
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct UpdateSuspendThreshold {
                pub new: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct UpdateDropThreshold {
                pub new: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct UpdateResumeThreshold {
                pub new: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UpdateThresholdWeight {
                pub new: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UpdateWeightRestrictDecay {
                pub new: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UpdateXcmpMaxIndividualWeight {
                pub new: runtime_types::sp_weights::weight_v2::Weight,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::service_overweight`]."]
                pub fn service_overweight(
                    &self,
                    index: ::core::primitive::u64,
                    weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ServiceOverweight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "service_overweight",
                        ServiceOverweight {
                            index,
                            weight_limit,
                        },
                        [
                            121u8, 236u8, 235u8, 23u8, 210u8, 238u8, 238u8, 122u8, 15u8, 86u8,
                            34u8, 119u8, 105u8, 100u8, 214u8, 236u8, 117u8, 39u8, 254u8, 235u8,
                            189u8, 15u8, 72u8, 74u8, 225u8, 134u8, 148u8, 126u8, 31u8, 203u8,
                            144u8, 106u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::suspend_xcm_execution`]."]
                pub fn suspend_xcm_execution(
                    &self,
                ) -> ::subxt::tx::StaticTxPayload<SuspendXcmExecution> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "suspend_xcm_execution",
                        SuspendXcmExecution {},
                        [
                            139u8, 76u8, 166u8, 86u8, 106u8, 144u8, 16u8, 47u8, 105u8, 185u8, 7u8,
                            7u8, 63u8, 14u8, 250u8, 236u8, 99u8, 121u8, 101u8, 143u8, 28u8, 175u8,
                            108u8, 197u8, 226u8, 43u8, 103u8, 92u8, 186u8, 12u8, 51u8, 153u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::resume_xcm_execution`]."]
                pub fn resume_xcm_execution(
                    &self,
                ) -> ::subxt::tx::StaticTxPayload<ResumeXcmExecution> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "resume_xcm_execution",
                        ResumeXcmExecution {},
                        [
                            67u8, 111u8, 47u8, 237u8, 79u8, 42u8, 90u8, 56u8, 245u8, 2u8, 20u8,
                            23u8, 33u8, 121u8, 135u8, 50u8, 204u8, 147u8, 195u8, 80u8, 177u8,
                            202u8, 8u8, 160u8, 164u8, 138u8, 64u8, 252u8, 178u8, 63u8, 102u8,
                            245u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_suspend_threshold`]."]
                pub fn update_suspend_threshold(
                    &self,
                    new: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<UpdateSuspendThreshold> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_suspend_threshold",
                        UpdateSuspendThreshold { new },
                        [
                            155u8, 120u8, 9u8, 228u8, 110u8, 62u8, 233u8, 36u8, 57u8, 85u8, 19u8,
                            67u8, 246u8, 88u8, 81u8, 116u8, 243u8, 236u8, 174u8, 130u8, 8u8, 246u8,
                            254u8, 97u8, 155u8, 207u8, 123u8, 60u8, 164u8, 14u8, 196u8, 97u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_drop_threshold`]."]
                pub fn update_drop_threshold(
                    &self,
                    new: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<UpdateDropThreshold> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_drop_threshold",
                        UpdateDropThreshold { new },
                        [
                            146u8, 177u8, 164u8, 96u8, 247u8, 182u8, 229u8, 175u8, 194u8, 101u8,
                            186u8, 168u8, 94u8, 114u8, 172u8, 119u8, 35u8, 222u8, 175u8, 21u8,
                            67u8, 61u8, 216u8, 144u8, 194u8, 10u8, 181u8, 62u8, 166u8, 198u8,
                            138u8, 243u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_resume_threshold`]."]
                pub fn update_resume_threshold(
                    &self,
                    new: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<UpdateResumeThreshold> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_resume_threshold",
                        UpdateResumeThreshold { new },
                        [
                            231u8, 128u8, 80u8, 179u8, 61u8, 50u8, 103u8, 209u8, 103u8, 55u8,
                            101u8, 113u8, 150u8, 10u8, 202u8, 7u8, 0u8, 77u8, 58u8, 4u8, 227u8,
                            17u8, 225u8, 112u8, 121u8, 203u8, 184u8, 113u8, 231u8, 156u8, 174u8,
                            154u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_threshold_weight`]."]
                pub fn update_threshold_weight(
                    &self,
                    new: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<UpdateThresholdWeight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_threshold_weight",
                        UpdateThresholdWeight { new },
                        [
                            14u8, 144u8, 112u8, 207u8, 195u8, 208u8, 184u8, 164u8, 94u8, 41u8, 8u8,
                            58u8, 180u8, 80u8, 239u8, 39u8, 210u8, 159u8, 114u8, 169u8, 152u8,
                            176u8, 26u8, 161u8, 32u8, 43u8, 250u8, 156u8, 56u8, 21u8, 43u8, 159u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_weight_restrict_decay`]."]
                pub fn update_weight_restrict_decay(
                    &self,
                    new: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<UpdateWeightRestrictDecay> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_weight_restrict_decay",
                        UpdateWeightRestrictDecay { new },
                        [
                            42u8, 53u8, 83u8, 191u8, 51u8, 227u8, 210u8, 193u8, 142u8, 218u8,
                            244u8, 177u8, 19u8, 87u8, 148u8, 177u8, 231u8, 197u8, 196u8, 255u8,
                            41u8, 130u8, 245u8, 139u8, 107u8, 212u8, 90u8, 161u8, 82u8, 248u8,
                            160u8, 223u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_xcmp_max_individual_weight`]."]
                pub fn update_xcmp_max_individual_weight(
                    &self,
                    new: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<UpdateXcmpMaxIndividualWeight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "XcmpQueue",
                        "update_xcmp_max_individual_weight",
                        UpdateXcmpMaxIndividualWeight { new },
                        [
                            148u8, 185u8, 89u8, 36u8, 152u8, 220u8, 248u8, 233u8, 236u8, 82u8,
                            170u8, 111u8, 225u8, 142u8, 25u8, 211u8, 72u8, 248u8, 250u8, 14u8,
                            45u8, 72u8, 78u8, 95u8, 92u8, 196u8, 245u8, 104u8, 112u8, 128u8, 27u8,
                            109u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::cumulus_pallet_xcmp_queue::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some XCM was executed ok."]
            pub struct Success {
                pub message_hash: [::core::primitive::u8; 32usize],
                pub message_id: [::core::primitive::u8; 32usize],
                pub weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for Success {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "Success";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some XCM failed."]
            pub struct Fail {
                pub message_hash: [::core::primitive::u8; 32usize],
                pub message_id: [::core::primitive::u8; 32usize],
                pub error: runtime_types::xcm::v3::traits::Error,
                pub weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for Fail {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "Fail";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Bad XCM version used."]
            pub struct BadVersion {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for BadVersion {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "BadVersion";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Bad XCM format used."]
            pub struct BadFormat {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for BadFormat {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "BadFormat";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An HRMP message was sent to a sibling parachain."]
            pub struct XcmpMessageSent {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for XcmpMessageSent {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "XcmpMessageSent";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An XCM exceeded the individual message weight budget."]
            pub struct OverweightEnqueued {
                pub sender: runtime_types::polkadot_parachain::primitives::Id,
                pub sent_at: ::core::primitive::u32,
                pub index: ::core::primitive::u64,
                pub required: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for OverweightEnqueued {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "OverweightEnqueued";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An XCM from the overweight queue was executed with the given actual weight used."]
            pub struct OverweightServiced {
                pub index: ::core::primitive::u64,
                pub used: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for OverweightServiced {
                const PALLET: &'static str = "XcmpQueue";
                const EVENT: &'static str = "OverweightServiced";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Status of the inbound XCMP channels."]
                pub fn inbound_xcmp_status(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<
                            runtime_types::cumulus_pallet_xcmp_queue::InboundChannelDetails,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "InboundXcmpStatus",
                        vec![],
                        [
                            183u8, 198u8, 237u8, 153u8, 132u8, 201u8, 87u8, 182u8, 121u8, 164u8,
                            129u8, 241u8, 58u8, 192u8, 115u8, 152u8, 7u8, 33u8, 95u8, 51u8, 2u8,
                            176u8, 144u8, 12u8, 125u8, 83u8, 92u8, 198u8, 211u8, 101u8, 28u8, 50u8,
                        ],
                    )
                }
                #[doc = " Inbound aggregate XCMP messages. It can only be one per ParaId/block."]
                pub fn inbound_xcmp_messages(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                    _1: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "InboundXcmpMessages",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                        ],
                        [
                            157u8, 232u8, 222u8, 97u8, 218u8, 96u8, 96u8, 90u8, 216u8, 205u8, 39u8,
                            130u8, 109u8, 152u8, 127u8, 57u8, 54u8, 63u8, 104u8, 135u8, 33u8,
                            175u8, 197u8, 166u8, 238u8, 22u8, 137u8, 162u8, 226u8, 199u8, 87u8,
                            25u8,
                        ],
                    )
                }
                #[doc = " Inbound aggregate XCMP messages. It can only be one per ParaId/block."]
                pub fn inbound_xcmp_messages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "InboundXcmpMessages",
                        Vec::new(),
                        [
                            157u8, 232u8, 222u8, 97u8, 218u8, 96u8, 96u8, 90u8, 216u8, 205u8, 39u8,
                            130u8, 109u8, 152u8, 127u8, 57u8, 54u8, 63u8, 104u8, 135u8, 33u8,
                            175u8, 197u8, 166u8, 238u8, 22u8, 137u8, 162u8, 226u8, 199u8, 87u8,
                            25u8,
                        ],
                    )
                }
                #[doc = " The non-empty XCMP channels in order of becoming non-empty, and the index of the first"]
                #[doc = " and last outbound message. If the two indices are equal, then it indicates an empty"]
                #[doc = " queue and there must be a non-`Ok` `OutboundStatus`. We assume queues grow no greater"]
                #[doc = " than 65535 items. Queue indices for normal messages begin at one; zero is reserved in"]
                #[doc = " case of the need to send a high-priority signal message this block."]
                #[doc = " The bool is true if there is a signal message waiting to be sent."]
                pub fn outbound_xcmp_status(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<
                            runtime_types::cumulus_pallet_xcmp_queue::OutboundChannelDetails,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "OutboundXcmpStatus",
                        vec![],
                        [
                            238u8, 120u8, 185u8, 141u8, 82u8, 159u8, 41u8, 68u8, 204u8, 15u8, 46u8,
                            152u8, 144u8, 74u8, 250u8, 83u8, 71u8, 105u8, 54u8, 53u8, 226u8, 87u8,
                            14u8, 202u8, 58u8, 160u8, 54u8, 162u8, 239u8, 248u8, 227u8, 116u8,
                        ],
                    )
                }
                #[doc = " The messages outbound in a given XCMP channel."]
                pub fn outbound_xcmp_messages(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                    _1: impl ::std::borrow::Borrow<::core::primitive::u16>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "OutboundXcmpMessages",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                        ],
                        [
                            50u8, 182u8, 237u8, 191u8, 106u8, 67u8, 54u8, 1u8, 17u8, 107u8, 70u8,
                            90u8, 202u8, 8u8, 63u8, 184u8, 171u8, 111u8, 192u8, 196u8, 7u8, 31u8,
                            186u8, 68u8, 31u8, 63u8, 71u8, 61u8, 83u8, 223u8, 79u8, 200u8,
                        ],
                    )
                }
                #[doc = " The messages outbound in a given XCMP channel."]
                pub fn outbound_xcmp_messages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "OutboundXcmpMessages",
                        Vec::new(),
                        [
                            50u8, 182u8, 237u8, 191u8, 106u8, 67u8, 54u8, 1u8, 17u8, 107u8, 70u8,
                            90u8, 202u8, 8u8, 63u8, 184u8, 171u8, 111u8, 192u8, 196u8, 7u8, 31u8,
                            186u8, 68u8, 31u8, 63u8, 71u8, 61u8, 83u8, 223u8, 79u8, 200u8,
                        ],
                    )
                }
                #[doc = " Any signal messages waiting to be sent."]
                pub fn signal_messages(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "SignalMessages",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            156u8, 242u8, 186u8, 89u8, 177u8, 195u8, 90u8, 121u8, 94u8, 106u8,
                            222u8, 78u8, 19u8, 162u8, 179u8, 96u8, 38u8, 113u8, 209u8, 148u8, 29u8,
                            110u8, 106u8, 167u8, 162u8, 96u8, 221u8, 20u8, 33u8, 179u8, 168u8,
                            142u8,
                        ],
                    )
                }
                #[doc = " Any signal messages waiting to be sent."]
                pub fn signal_messages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::core::primitive::u8>>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "SignalMessages",
                        Vec::new(),
                        [
                            156u8, 242u8, 186u8, 89u8, 177u8, 195u8, 90u8, 121u8, 94u8, 106u8,
                            222u8, 78u8, 19u8, 162u8, 179u8, 96u8, 38u8, 113u8, 209u8, 148u8, 29u8,
                            110u8, 106u8, 167u8, 162u8, 96u8, 221u8, 20u8, 33u8, 179u8, 168u8,
                            142u8,
                        ],
                    )
                }
                #[doc = " The configuration which controls the dynamics of the outbound queue."]
                pub fn queue_config(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::cumulus_pallet_xcmp_queue::QueueConfigData,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "QueueConfig",
                        vec![],
                        [
                            229u8, 217u8, 163u8, 35u8, 141u8, 111u8, 58u8, 130u8, 249u8, 46u8,
                            241u8, 94u8, 112u8, 24u8, 219u8, 28u8, 142u8, 237u8, 150u8, 19u8, 70u8,
                            51u8, 229u8, 40u8, 40u8, 249u8, 187u8, 69u8, 104u8, 249u8, 150u8,
                            190u8,
                        ],
                    )
                }
                #[doc = " The messages that exceeded max individual message weight budget."]
                #[doc = ""]
                #[doc = " These message stay in this storage map until they are manually dispatched via"]
                #[doc = " `service_overweight`."]
                pub fn overweight(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u64>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<(
                        runtime_types::polkadot_parachain::primitives::Id,
                        ::core::primitive::u32,
                        ::std::vec::Vec<::core::primitive::u8>,
                    )>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "Overweight",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            222u8, 249u8, 232u8, 110u8, 117u8, 229u8, 165u8, 164u8, 219u8, 219u8,
                            149u8, 204u8, 25u8, 78u8, 204u8, 116u8, 111u8, 114u8, 120u8, 222u8,
                            56u8, 77u8, 122u8, 147u8, 108u8, 15u8, 94u8, 161u8, 212u8, 50u8, 7u8,
                            7u8,
                        ],
                    )
                }
                #[doc = " The messages that exceeded max individual message weight budget."]
                #[doc = ""]
                #[doc = " These message stay in this storage map until they are manually dispatched via"]
                #[doc = " `service_overweight`."]
                pub fn overweight_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<(
                        runtime_types::polkadot_parachain::primitives::Id,
                        ::core::primitive::u32,
                        ::std::vec::Vec<::core::primitive::u8>,
                    )>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "Overweight",
                        Vec::new(),
                        [
                            222u8, 249u8, 232u8, 110u8, 117u8, 229u8, 165u8, 164u8, 219u8, 219u8,
                            149u8, 204u8, 25u8, 78u8, 204u8, 116u8, 111u8, 114u8, 120u8, 222u8,
                            56u8, 77u8, 122u8, 147u8, 108u8, 15u8, 94u8, 161u8, 212u8, 50u8, 7u8,
                            7u8,
                        ],
                    )
                }
                #[doc = "Counter for the related counted storage map"]
                pub fn counter_for_overweight(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "CounterForOverweight",
                        vec![],
                        [
                            148u8, 226u8, 248u8, 107u8, 165u8, 97u8, 218u8, 160u8, 127u8, 48u8,
                            185u8, 251u8, 35u8, 137u8, 119u8, 251u8, 151u8, 167u8, 189u8, 66u8,
                            80u8, 74u8, 134u8, 129u8, 222u8, 180u8, 51u8, 182u8, 50u8, 110u8, 10u8,
                            43u8,
                        ],
                    )
                }
                #[doc = " The number of overweight messages ever recorded in `Overweight`. Also doubles as the next"]
                #[doc = " available free overweight index."]
                pub fn overweight_count(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "OverweightCount",
                        vec![],
                        [
                            102u8, 180u8, 196u8, 148u8, 115u8, 62u8, 46u8, 238u8, 97u8, 116u8,
                            117u8, 42u8, 14u8, 5u8, 72u8, 237u8, 230u8, 46u8, 150u8, 126u8, 89u8,
                            64u8, 233u8, 166u8, 180u8, 137u8, 52u8, 233u8, 252u8, 255u8, 36u8,
                            20u8,
                        ],
                    )
                }
                #[doc = " Whether or not the XCMP queue is suspended from executing incoming XCMs or not."]
                pub fn queue_suspended(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::bool>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "XcmpQueue",
                        "QueueSuspended",
                        vec![],
                        [
                            23u8, 37u8, 48u8, 112u8, 222u8, 17u8, 252u8, 65u8, 160u8, 217u8, 218u8,
                            30u8, 2u8, 1u8, 204u8, 0u8, 251u8, 17u8, 138u8, 197u8, 164u8, 50u8,
                            122u8, 0u8, 31u8, 238u8, 147u8, 213u8, 30u8, 132u8, 184u8, 215u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod polkadot_xcm {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Send {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct TeleportAssets {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                pub fee_asset_item: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReserveTransferAssets {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                pub fee_asset_item: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Execute {
                pub message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
                pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceXcmVersion {
                pub location:
                    ::std::boxed::Box<runtime_types::xcm::v3::multilocation::MultiLocation>,
                pub version: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceDefaultXcmVersion {
                pub maybe_xcm_version: ::core::option::Option<::core::primitive::u32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceSubscribeVersionNotify {
                pub location: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceUnsubscribeVersionNotify {
                pub location: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct LimitedReserveTransferAssets {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                pub fee_asset_item: ::core::primitive::u32,
                pub weight_limit: runtime_types::xcm::v3::WeightLimit,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct LimitedTeleportAssets {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                pub fee_asset_item: ::core::primitive::u32,
                pub weight_limit: runtime_types::xcm::v3::WeightLimit,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceSuspension {
                pub suspended: ::core::primitive::bool,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::send`]."]
                pub fn send(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    message: runtime_types::xcm::VersionedXcm,
                ) -> ::subxt::tx::StaticTxPayload<Send> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "send",
                        Send {
                            dest: ::std::boxed::Box::new(dest),
                            message: ::std::boxed::Box::new(message),
                        },
                        [
                            246u8, 35u8, 227u8, 112u8, 223u8, 7u8, 44u8, 186u8, 60u8, 225u8, 153u8,
                            249u8, 104u8, 51u8, 123u8, 227u8, 143u8, 65u8, 232u8, 209u8, 178u8,
                            104u8, 70u8, 56u8, 230u8, 14u8, 75u8, 83u8, 250u8, 160u8, 9u8, 39u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::teleport_assets`]."]
                pub fn teleport_assets(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    beneficiary: runtime_types::xcm::VersionedMultiLocation,
                    assets: runtime_types::xcm::VersionedMultiAssets,
                    fee_asset_item: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<TeleportAssets> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "teleport_assets",
                        TeleportAssets {
                            dest: ::std::boxed::Box::new(dest),
                            beneficiary: ::std::boxed::Box::new(beneficiary),
                            assets: ::std::boxed::Box::new(assets),
                            fee_asset_item,
                        },
                        [
                            187u8, 42u8, 2u8, 96u8, 105u8, 125u8, 74u8, 53u8, 2u8, 21u8, 31u8,
                            160u8, 201u8, 197u8, 157u8, 190u8, 40u8, 145u8, 5u8, 99u8, 194u8, 41u8,
                            114u8, 60u8, 165u8, 186u8, 15u8, 226u8, 85u8, 113u8, 159u8, 136u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::reserve_transfer_assets`]."]
                pub fn reserve_transfer_assets(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    beneficiary: runtime_types::xcm::VersionedMultiLocation,
                    assets: runtime_types::xcm::VersionedMultiAssets,
                    fee_asset_item: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<ReserveTransferAssets> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "reserve_transfer_assets",
                        ReserveTransferAssets {
                            dest: ::std::boxed::Box::new(dest),
                            beneficiary: ::std::boxed::Box::new(beneficiary),
                            assets: ::std::boxed::Box::new(assets),
                            fee_asset_item,
                        },
                        [
                            249u8, 177u8, 76u8, 204u8, 186u8, 165u8, 16u8, 186u8, 129u8, 239u8,
                            65u8, 252u8, 9u8, 132u8, 32u8, 164u8, 117u8, 177u8, 40u8, 21u8, 196u8,
                            246u8, 147u8, 2u8, 95u8, 110u8, 68u8, 162u8, 148u8, 9u8, 59u8, 170u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::execute`]."]
                pub fn execute(
                    &self,
                    message: runtime_types::xcm::VersionedXcm,
                    max_weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<Execute> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "execute",
                        Execute {
                            message: ::std::boxed::Box::new(message),
                            max_weight,
                        },
                        [
                            102u8, 41u8, 146u8, 29u8, 241u8, 205u8, 95u8, 153u8, 228u8, 141u8,
                            11u8, 228u8, 13u8, 44u8, 75u8, 204u8, 174u8, 35u8, 155u8, 104u8, 204u8,
                            82u8, 239u8, 98u8, 249u8, 187u8, 193u8, 1u8, 122u8, 88u8, 162u8, 200u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_xcm_version`]."]
                pub fn force_xcm_version(
                    &self,
                    location: runtime_types::xcm::v3::multilocation::MultiLocation,
                    version: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<ForceXcmVersion> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "force_xcm_version",
                        ForceXcmVersion {
                            location: ::std::boxed::Box::new(location),
                            version,
                        },
                        [
                            66u8, 73u8, 103u8, 208u8, 142u8, 219u8, 59u8, 169u8, 212u8, 160u8,
                            39u8, 28u8, 137u8, 54u8, 5u8, 106u8, 237u8, 48u8, 242u8, 130u8, 91u8,
                            190u8, 43u8, 80u8, 127u8, 42u8, 228u8, 172u8, 131u8, 154u8, 249u8,
                            195u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_default_xcm_version`]."]
                pub fn force_default_xcm_version(
                    &self,
                    maybe_xcm_version: ::core::option::Option<::core::primitive::u32>,
                ) -> ::subxt::tx::StaticTxPayload<ForceDefaultXcmVersion> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "force_default_xcm_version",
                        ForceDefaultXcmVersion { maybe_xcm_version },
                        [
                            38u8, 36u8, 59u8, 231u8, 18u8, 79u8, 76u8, 9u8, 200u8, 125u8, 214u8,
                            166u8, 37u8, 99u8, 111u8, 161u8, 135u8, 2u8, 133u8, 157u8, 165u8, 18u8,
                            152u8, 81u8, 209u8, 255u8, 137u8, 237u8, 28u8, 126u8, 224u8, 141u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_subscribe_version_notify`]."]
                pub fn force_subscribe_version_notify(
                    &self,
                    location: runtime_types::xcm::VersionedMultiLocation,
                ) -> ::subxt::tx::StaticTxPayload<ForceSubscribeVersionNotify> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "force_subscribe_version_notify",
                        ForceSubscribeVersionNotify {
                            location: ::std::boxed::Box::new(location),
                        },
                        [
                            236u8, 37u8, 153u8, 26u8, 174u8, 187u8, 154u8, 38u8, 179u8, 223u8,
                            130u8, 32u8, 128u8, 30u8, 148u8, 229u8, 7u8, 185u8, 174u8, 9u8, 96u8,
                            215u8, 189u8, 178u8, 148u8, 141u8, 249u8, 118u8, 7u8, 238u8, 1u8, 49u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_unsubscribe_version_notify`]."]
                pub fn force_unsubscribe_version_notify(
                    &self,
                    location: runtime_types::xcm::VersionedMultiLocation,
                ) -> ::subxt::tx::StaticTxPayload<ForceUnsubscribeVersionNotify> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "force_unsubscribe_version_notify",
                        ForceUnsubscribeVersionNotify {
                            location: ::std::boxed::Box::new(location),
                        },
                        [
                            154u8, 169u8, 145u8, 211u8, 185u8, 71u8, 9u8, 63u8, 3u8, 158u8, 187u8,
                            173u8, 115u8, 166u8, 100u8, 66u8, 12u8, 40u8, 198u8, 40u8, 213u8,
                            104u8, 95u8, 183u8, 215u8, 53u8, 94u8, 158u8, 106u8, 56u8, 149u8, 52u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::limited_reserve_transfer_assets`]."]
                pub fn limited_reserve_transfer_assets(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    beneficiary: runtime_types::xcm::VersionedMultiLocation,
                    assets: runtime_types::xcm::VersionedMultiAssets,
                    fee_asset_item: ::core::primitive::u32,
                    weight_limit: runtime_types::xcm::v3::WeightLimit,
                ) -> ::subxt::tx::StaticTxPayload<LimitedReserveTransferAssets> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "limited_reserve_transfer_assets",
                        LimitedReserveTransferAssets {
                            dest: ::std::boxed::Box::new(dest),
                            beneficiary: ::std::boxed::Box::new(beneficiary),
                            assets: ::std::boxed::Box::new(assets),
                            fee_asset_item,
                            weight_limit,
                        },
                        [
                            131u8, 191u8, 89u8, 27u8, 236u8, 142u8, 130u8, 129u8, 245u8, 95u8,
                            159u8, 96u8, 252u8, 80u8, 28u8, 40u8, 128u8, 55u8, 41u8, 123u8, 22u8,
                            18u8, 0u8, 236u8, 77u8, 68u8, 135u8, 181u8, 40u8, 47u8, 92u8, 240u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::limited_teleport_assets`]."]
                pub fn limited_teleport_assets(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    beneficiary: runtime_types::xcm::VersionedMultiLocation,
                    assets: runtime_types::xcm::VersionedMultiAssets,
                    fee_asset_item: ::core::primitive::u32,
                    weight_limit: runtime_types::xcm::v3::WeightLimit,
                ) -> ::subxt::tx::StaticTxPayload<LimitedTeleportAssets> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "limited_teleport_assets",
                        LimitedTeleportAssets {
                            dest: ::std::boxed::Box::new(dest),
                            beneficiary: ::std::boxed::Box::new(beneficiary),
                            assets: ::std::boxed::Box::new(assets),
                            fee_asset_item,
                            weight_limit,
                        },
                        [
                            234u8, 19u8, 104u8, 174u8, 98u8, 159u8, 205u8, 110u8, 240u8, 78u8,
                            186u8, 138u8, 236u8, 116u8, 104u8, 215u8, 57u8, 178u8, 166u8, 208u8,
                            197u8, 113u8, 101u8, 56u8, 23u8, 56u8, 84u8, 14u8, 173u8, 70u8, 211u8,
                            201u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_suspension`]."]
                pub fn force_suspension(
                    &self,
                    suspended: ::core::primitive::bool,
                ) -> ::subxt::tx::StaticTxPayload<ForceSuspension> {
                    ::subxt::tx::StaticTxPayload::new(
                        "PolkadotXcm",
                        "force_suspension",
                        ForceSuspension { suspended },
                        [
                            147u8, 1u8, 117u8, 148u8, 8u8, 14u8, 53u8, 167u8, 85u8, 184u8, 25u8,
                            183u8, 52u8, 197u8, 12u8, 135u8, 45u8, 88u8, 13u8, 27u8, 218u8, 31u8,
                            80u8, 27u8, 183u8, 36u8, 0u8, 243u8, 235u8, 85u8, 75u8, 81u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_xcm::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Execution of an XCM message was attempted."]
            pub struct Attempted {
                pub outcome: runtime_types::xcm::v3::traits::Outcome,
            }
            impl ::subxt::events::StaticEvent for Attempted {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "Attempted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A XCM message was sent."]
            pub struct Sent {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub message: runtime_types::xcm::v3::Xcm,
                pub message_id: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for Sent {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "Sent";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response received which does not match a registered query. This may be because a"]
            #[doc = "matching query was never registered, it may be because it is a duplicate response, or"]
            #[doc = "because the query timed out."]
            pub struct UnexpectedResponse {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for UnexpectedResponse {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "UnexpectedResponse";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response has been received and is ready for taking with `take_response`. There is"]
            #[doc = "no registered notification call."]
            pub struct ResponseReady {
                pub query_id: ::core::primitive::u64,
                pub response: runtime_types::xcm::v3::Response,
            }
            impl ::subxt::events::StaticEvent for ResponseReady {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "ResponseReady";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response has been received and query is removed. The registered notification has"]
            #[doc = "been dispatched and executed successfully."]
            pub struct Notified {
                pub query_id: ::core::primitive::u64,
                pub pallet_index: ::core::primitive::u8,
                pub call_index: ::core::primitive::u8,
            }
            impl ::subxt::events::StaticEvent for Notified {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "Notified";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response has been received and query is removed. The registered notification could"]
            #[doc = "not be dispatched because the dispatch weight is greater than the maximum weight"]
            #[doc = "originally budgeted by this runtime for the query result."]
            pub struct NotifyOverweight {
                pub query_id: ::core::primitive::u64,
                pub pallet_index: ::core::primitive::u8,
                pub call_index: ::core::primitive::u8,
                pub actual_weight: runtime_types::sp_weights::weight_v2::Weight,
                pub max_budgeted_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for NotifyOverweight {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "NotifyOverweight";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response has been received and query is removed. There was a general error with"]
            #[doc = "dispatching the notification call."]
            pub struct NotifyDispatchError {
                pub query_id: ::core::primitive::u64,
                pub pallet_index: ::core::primitive::u8,
                pub call_index: ::core::primitive::u8,
            }
            impl ::subxt::events::StaticEvent for NotifyDispatchError {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "NotifyDispatchError";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Query response has been received and query is removed. The dispatch was unable to be"]
            #[doc = "decoded into a `Call`; this might be due to dispatch function having a signature which"]
            #[doc = "is not `(origin, QueryId, Response)`."]
            pub struct NotifyDecodeFailed {
                pub query_id: ::core::primitive::u64,
                pub pallet_index: ::core::primitive::u8,
                pub call_index: ::core::primitive::u8,
            }
            impl ::subxt::events::StaticEvent for NotifyDecodeFailed {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "NotifyDecodeFailed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Expected query response has been received but the origin location of the response does"]
            #[doc = "not match that expected. The query remains registered for a later, valid, response to"]
            #[doc = "be received and acted upon."]
            pub struct InvalidResponder {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
                pub expected_location:
                    ::core::option::Option<runtime_types::xcm::v3::multilocation::MultiLocation>,
            }
            impl ::subxt::events::StaticEvent for InvalidResponder {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "InvalidResponder";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Expected query response has been received but the expected origin location placed in"]
            #[doc = "storage by this runtime previously cannot be decoded. The query remains registered."]
            #[doc = ""]
            #[doc = "This is unexpected (since a location placed in storage in a previously executing"]
            #[doc = "runtime should be readable prior to query timeout) and dangerous since the possibly"]
            #[doc = "valid response will be dropped. Manual governance intervention is probably going to be"]
            #[doc = "needed."]
            pub struct InvalidResponderVersion {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for InvalidResponderVersion {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "InvalidResponderVersion";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            #[doc = "Received query response has been read and removed."]
            pub struct ResponseTaken {
                pub query_id: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for ResponseTaken {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "ResponseTaken";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some assets have been placed in an asset trap."]
            pub struct AssetsTrapped {
                pub hash: ::subxt::utils::H256,
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub assets: runtime_types::xcm::VersionedMultiAssets,
            }
            impl ::subxt::events::StaticEvent for AssetsTrapped {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "AssetsTrapped";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An XCM version change notification message has been attempted to be sent."]
            #[doc = ""]
            #[doc = "The cost of sending it (borne by the chain) is included."]
            pub struct VersionChangeNotified {
                pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub result: ::core::primitive::u32,
                pub cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                pub message_id: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for VersionChangeNotified {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "VersionChangeNotified";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The supported version of a location has been changed. This might be through an"]
            #[doc = "automatic notification or a manual intervention."]
            pub struct SupportedVersionChanged {
                pub location: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub version: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for SupportedVersionChanged {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "SupportedVersionChanged";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A given location which had a version change subscription was dropped owing to an error"]
            #[doc = "sending the notification to it."]
            pub struct NotifyTargetSendFail {
                pub location: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
                pub error: runtime_types::xcm::v3::traits::Error,
            }
            impl ::subxt::events::StaticEvent for NotifyTargetSendFail {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "NotifyTargetSendFail";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A given location which had a version change subscription was dropped owing to an error"]
            #[doc = "migrating the location to our new XCM format."]
            pub struct NotifyTargetMigrationFail {
                pub location: runtime_types::xcm::VersionedMultiLocation,
                pub query_id: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for NotifyTargetMigrationFail {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "NotifyTargetMigrationFail";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Expected query response has been received but the expected querier location placed in"]
            #[doc = "storage by this runtime previously cannot be decoded. The query remains registered."]
            #[doc = ""]
            #[doc = "This is unexpected (since a location placed in storage in a previously executing"]
            #[doc = "runtime should be readable prior to query timeout) and dangerous since the possibly"]
            #[doc = "valid response will be dropped. Manual governance intervention is probably going to be"]
            #[doc = "needed."]
            pub struct InvalidQuerierVersion {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for InvalidQuerierVersion {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "InvalidQuerierVersion";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Expected query response has been received but the querier location of the response does"]
            #[doc = "not match the expected. The query remains registered for a later, valid, response to"]
            #[doc = "be received and acted upon."]
            pub struct InvalidQuerier {
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub query_id: ::core::primitive::u64,
                pub expected_querier: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub maybe_actual_querier:
                    ::core::option::Option<runtime_types::xcm::v3::multilocation::MultiLocation>,
            }
            impl ::subxt::events::StaticEvent for InvalidQuerier {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "InvalidQuerier";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A remote has requested XCM version change notification from us and we have honored it."]
            #[doc = "A version information message is sent to them and its cost is included."]
            pub struct VersionNotifyStarted {
                pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                pub message_id: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for VersionNotifyStarted {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "VersionNotifyStarted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "We have requested that a remote chain send us XCM version change notifications."]
            pub struct VersionNotifyRequested {
                pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                pub message_id: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for VersionNotifyRequested {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "VersionNotifyRequested";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "We have requested that a remote chain stops sending us XCM version change notifications."]
            pub struct VersionNotifyUnrequested {
                pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                pub message_id: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for VersionNotifyUnrequested {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "VersionNotifyUnrequested";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Fees were paid from a location for an operation (often for using `SendXcm`)."]
            pub struct FeesPaid {
                pub paying: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub fees: runtime_types::xcm::v3::multiasset::MultiAssets,
            }
            impl ::subxt::events::StaticEvent for FeesPaid {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "FeesPaid";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some assets have been claimed from an asset trap"]
            pub struct AssetsClaimed {
                pub hash: ::subxt::utils::H256,
                pub origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                pub assets: runtime_types::xcm::VersionedMultiAssets,
            }
            impl ::subxt::events::StaticEvent for AssetsClaimed {
                const PALLET: &'static str = "PolkadotXcm";
                const EVENT: &'static str = "AssetsClaimed";
            }
        }
    }
    pub mod cumulus_xcm {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::cumulus_pallet_xcm::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message is invalid XCM."]
            #[doc = "\\[ id \\]"]
            pub struct InvalidFormat(pub [::core::primitive::u8; 32usize]);
            impl ::subxt::events::StaticEvent for InvalidFormat {
                const PALLET: &'static str = "CumulusXcm";
                const EVENT: &'static str = "InvalidFormat";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message is unsupported version of XCM."]
            #[doc = "\\[ id \\]"]
            pub struct UnsupportedVersion(pub [::core::primitive::u8; 32usize]);
            impl ::subxt::events::StaticEvent for UnsupportedVersion {
                const PALLET: &'static str = "CumulusXcm";
                const EVENT: &'static str = "UnsupportedVersion";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message executed with the given outcome."]
            #[doc = "\\[ id, outcome \\]"]
            pub struct ExecutedDownward(
                pub [::core::primitive::u8; 32usize],
                pub runtime_types::xcm::v3::traits::Outcome,
            );
            impl ::subxt::events::StaticEvent for ExecutedDownward {
                const PALLET: &'static str = "CumulusXcm";
                const EVENT: &'static str = "ExecutedDownward";
            }
        }
    }
    pub mod dmp_queue {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ServiceOverweight {
                pub index: ::core::primitive::u64,
                pub weight_limit: runtime_types::sp_weights::weight_v2::Weight,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::service_overweight`]."]
                pub fn service_overweight(
                    &self,
                    index: ::core::primitive::u64,
                    weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ServiceOverweight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "DmpQueue",
                        "service_overweight",
                        ServiceOverweight {
                            index,
                            weight_limit,
                        },
                        [
                            121u8, 236u8, 235u8, 23u8, 210u8, 238u8, 238u8, 122u8, 15u8, 86u8,
                            34u8, 119u8, 105u8, 100u8, 214u8, 236u8, 117u8, 39u8, 254u8, 235u8,
                            189u8, 15u8, 72u8, 74u8, 225u8, 134u8, 148u8, 126u8, 31u8, 203u8,
                            144u8, 106u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::cumulus_pallet_dmp_queue::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message is invalid XCM."]
            pub struct InvalidFormat {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for InvalidFormat {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "InvalidFormat";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message is unsupported version of XCM."]
            pub struct UnsupportedVersion {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for UnsupportedVersion {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "UnsupportedVersion";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message executed with the given outcome."]
            pub struct ExecutedDownward {
                pub message_hash: [::core::primitive::u8; 32usize],
                pub message_id: [::core::primitive::u8; 32usize],
                pub outcome: runtime_types::xcm::v3::traits::Outcome,
            }
            impl ::subxt::events::StaticEvent for ExecutedDownward {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "ExecutedDownward";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The weight limit for handling downward messages was reached."]
            pub struct WeightExhausted {
                pub message_hash: [::core::primitive::u8; 32usize],
                pub message_id: [::core::primitive::u8; 32usize],
                pub remaining_weight: runtime_types::sp_weights::weight_v2::Weight,
                pub required_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for WeightExhausted {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "WeightExhausted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message is overweight and was placed in the overweight queue."]
            pub struct OverweightEnqueued {
                pub message_hash: [::core::primitive::u8; 32usize],
                pub message_id: [::core::primitive::u8; 32usize],
                pub overweight_index: ::core::primitive::u64,
                pub required_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for OverweightEnqueued {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "OverweightEnqueued";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Downward message from the overweight queue was executed."]
            pub struct OverweightServiced {
                pub overweight_index: ::core::primitive::u64,
                pub weight_used: runtime_types::sp_weights::weight_v2::Weight,
            }
            impl ::subxt::events::StaticEvent for OverweightServiced {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "OverweightServiced";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The maximum number of downward messages was reached."]
            pub struct MaxMessagesExhausted {
                pub message_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for MaxMessagesExhausted {
                const PALLET: &'static str = "DmpQueue";
                const EVENT: &'static str = "MaxMessagesExhausted";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The configuration."]
                pub fn configuration(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::cumulus_pallet_dmp_queue::ConfigData,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "Configuration",
                        vec![],
                        [
                            243u8, 180u8, 26u8, 3u8, 173u8, 150u8, 126u8, 197u8, 225u8, 243u8,
                            179u8, 184u8, 88u8, 108u8, 243u8, 209u8, 11u8, 225u8, 144u8, 33u8,
                            11u8, 22u8, 103u8, 112u8, 143u8, 54u8, 58u8, 236u8, 92u8, 90u8, 150u8,
                            155u8,
                        ],
                    )
                }
                #[doc = " The page index."]
                pub fn page_index(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::cumulus_pallet_dmp_queue::PageIndexData,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "PageIndex",
                        vec![],
                        [
                            94u8, 132u8, 34u8, 67u8, 10u8, 22u8, 235u8, 96u8, 168u8, 26u8, 57u8,
                            200u8, 130u8, 218u8, 37u8, 71u8, 28u8, 119u8, 78u8, 107u8, 209u8,
                            120u8, 190u8, 2u8, 101u8, 215u8, 122u8, 187u8, 94u8, 38u8, 255u8,
                            234u8,
                        ],
                    )
                }
                #[doc = " The queue pages."]
                pub fn pages(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<(
                            ::core::primitive::u32,
                            ::std::vec::Vec<::core::primitive::u8>,
                        )>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "Pages",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            228u8, 86u8, 33u8, 107u8, 248u8, 4u8, 223u8, 175u8, 222u8, 25u8, 204u8,
                            42u8, 235u8, 21u8, 215u8, 91u8, 167u8, 14u8, 133u8, 151u8, 190u8, 57u8,
                            138u8, 208u8, 79u8, 244u8, 132u8, 14u8, 48u8, 247u8, 171u8, 108u8,
                        ],
                    )
                }
                #[doc = " The queue pages."]
                pub fn pages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<(
                            ::core::primitive::u32,
                            ::std::vec::Vec<::core::primitive::u8>,
                        )>,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "Pages",
                        Vec::new(),
                        [
                            228u8, 86u8, 33u8, 107u8, 248u8, 4u8, 223u8, 175u8, 222u8, 25u8, 204u8,
                            42u8, 235u8, 21u8, 215u8, 91u8, 167u8, 14u8, 133u8, 151u8, 190u8, 57u8,
                            138u8, 208u8, 79u8, 244u8, 132u8, 14u8, 48u8, 247u8, 171u8, 108u8,
                        ],
                    )
                }
                #[doc = " The overweight messages."]
                pub fn overweight(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u64>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<(
                        ::core::primitive::u32,
                        ::std::vec::Vec<::core::primitive::u8>,
                    )>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "Overweight",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            222u8, 85u8, 143u8, 49u8, 42u8, 248u8, 138u8, 163u8, 46u8, 199u8,
                            188u8, 61u8, 137u8, 135u8, 127u8, 146u8, 210u8, 254u8, 121u8, 42u8,
                            112u8, 114u8, 22u8, 228u8, 207u8, 207u8, 245u8, 175u8, 152u8, 140u8,
                            225u8, 237u8,
                        ],
                    )
                }
                #[doc = " The overweight messages."]
                pub fn overweight_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<(
                        ::core::primitive::u32,
                        ::std::vec::Vec<::core::primitive::u8>,
                    )>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "Overweight",
                        Vec::new(),
                        [
                            222u8, 85u8, 143u8, 49u8, 42u8, 248u8, 138u8, 163u8, 46u8, 199u8,
                            188u8, 61u8, 137u8, 135u8, 127u8, 146u8, 210u8, 254u8, 121u8, 42u8,
                            112u8, 114u8, 22u8, 228u8, 207u8, 207u8, 245u8, 175u8, 152u8, 140u8,
                            225u8, 237u8,
                        ],
                    )
                }
                #[doc = "Counter for the related counted storage map"]
                pub fn counter_for_overweight(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "DmpQueue",
                        "CounterForOverweight",
                        vec![],
                        [
                            148u8, 226u8, 248u8, 107u8, 165u8, 97u8, 218u8, 160u8, 127u8, 48u8,
                            185u8, 251u8, 35u8, 137u8, 119u8, 251u8, 151u8, 167u8, 189u8, 66u8,
                            80u8, 74u8, 134u8, 129u8, 222u8, 180u8, 51u8, 182u8, 50u8, 110u8, 10u8,
                            43u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod utility {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Batch {
                pub calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AsDerivative {
                pub index: ::core::primitive::u16,
                pub call: ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct BatchAll {
                pub calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct DispatchAs {
                pub as_origin:
                    ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::OriginCaller>,
                pub call: ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceBatch {
                pub calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct WithWeight {
                pub call: ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                pub weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::batch`]."]
                pub fn batch(
                    &self,
                    calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                ) -> ::subxt::tx::StaticTxPayload<Batch> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "batch",
                        Batch { calls },
                        [
                            46u8, 104u8, 127u8, 65u8, 87u8, 57u8, 37u8, 149u8, 73u8, 191u8, 162u8,
                            91u8, 242u8, 27u8, 31u8, 209u8, 106u8, 77u8, 50u8, 79u8, 32u8, 85u8,
                            166u8, 15u8, 117u8, 134u8, 35u8, 52u8, 51u8, 182u8, 134u8, 101u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::as_derivative`]."]
                pub fn as_derivative(
                    &self,
                    index: ::core::primitive::u16,
                    call: runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                ) -> ::subxt::tx::StaticTxPayload<AsDerivative> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "as_derivative",
                        AsDerivative {
                            index,
                            call: ::std::boxed::Box::new(call),
                        },
                        [
                            232u8, 94u8, 53u8, 249u8, 132u8, 6u8, 168u8, 69u8, 18u8, 15u8, 44u8,
                            38u8, 32u8, 7u8, 199u8, 244u8, 43u8, 31u8, 234u8, 163u8, 128u8, 234u8,
                            92u8, 90u8, 193u8, 233u8, 110u8, 214u8, 100u8, 65u8, 197u8, 174u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::batch_all`]."]
                pub fn batch_all(
                    &self,
                    calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                ) -> ::subxt::tx::StaticTxPayload<BatchAll> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "batch_all",
                        BatchAll { calls },
                        [
                            9u8, 217u8, 118u8, 39u8, 114u8, 39u8, 0u8, 158u8, 60u8, 69u8, 146u8,
                            69u8, 253u8, 2u8, 245u8, 78u8, 3u8, 37u8, 87u8, 22u8, 168u8, 51u8,
                            201u8, 186u8, 177u8, 235u8, 247u8, 113u8, 49u8, 193u8, 133u8, 38u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::dispatch_as`]."]
                pub fn dispatch_as(
                    &self,
                    as_origin: runtime_types::bridge_hub_rococo_runtime::OriginCaller,
                    call: runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                ) -> ::subxt::tx::StaticTxPayload<DispatchAs> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "dispatch_as",
                        DispatchAs {
                            as_origin: ::std::boxed::Box::new(as_origin),
                            call: ::std::boxed::Box::new(call),
                        },
                        [
                            211u8, 27u8, 115u8, 80u8, 174u8, 171u8, 21u8, 28u8, 254u8, 194u8,
                            167u8, 182u8, 83u8, 195u8, 28u8, 187u8, 253u8, 31u8, 25u8, 151u8,
                            146u8, 219u8, 185u8, 82u8, 199u8, 114u8, 207u8, 227u8, 15u8, 231u8,
                            199u8, 92u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::force_batch`]."]
                pub fn force_batch(
                    &self,
                    calls: ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                ) -> ::subxt::tx::StaticTxPayload<ForceBatch> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "force_batch",
                        ForceBatch { calls },
                        [
                            34u8, 98u8, 210u8, 255u8, 63u8, 12u8, 189u8, 236u8, 132u8, 107u8,
                            167u8, 78u8, 234u8, 255u8, 143u8, 205u8, 239u8, 115u8, 146u8, 14u8,
                            100u8, 1u8, 199u8, 209u8, 50u8, 88u8, 117u8, 46u8, 166u8, 68u8, 164u8,
                            67u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::with_weight`]."]
                pub fn with_weight(
                    &self,
                    call: runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                    weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<WithWeight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Utility",
                        "with_weight",
                        WithWeight {
                            call: ::std::boxed::Box::new(call),
                            weight,
                        },
                        [
                            72u8, 246u8, 141u8, 242u8, 193u8, 93u8, 225u8, 2u8, 192u8, 130u8,
                            250u8, 111u8, 117u8, 113u8, 223u8, 56u8, 232u8, 162u8, 196u8, 135u8,
                            0u8, 80u8, 221u8, 22u8, 239u8, 27u8, 102u8, 98u8, 178u8, 235u8, 102u8,
                            162u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_utility::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Batch of dispatches did not complete fully. Index of first failing dispatch given, as"]
            #[doc = "well as the error."]
            pub struct BatchInterrupted {
                pub index: ::core::primitive::u32,
                pub error: runtime_types::sp_runtime::DispatchError,
            }
            impl ::subxt::events::StaticEvent for BatchInterrupted {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "BatchInterrupted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Batch of dispatches completed fully with no error."]
            pub struct BatchCompleted;
            impl ::subxt::events::StaticEvent for BatchCompleted {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "BatchCompleted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Batch of dispatches completed but has errors."]
            pub struct BatchCompletedWithErrors;
            impl ::subxt::events::StaticEvent for BatchCompletedWithErrors {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "BatchCompletedWithErrors";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A single item within a Batch of dispatches has completed with no error."]
            pub struct ItemCompleted;
            impl ::subxt::events::StaticEvent for ItemCompleted {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "ItemCompleted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A single item within a Batch of dispatches has completed with error."]
            pub struct ItemFailed {
                pub error: runtime_types::sp_runtime::DispatchError,
            }
            impl ::subxt::events::StaticEvent for ItemFailed {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "ItemFailed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A call was dispatched."]
            pub struct DispatchedAs {
                pub result: ::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
            }
            impl ::subxt::events::StaticEvent for DispatchedAs {
                const PALLET: &'static str = "Utility";
                const EVENT: &'static str = "DispatchedAs";
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " The limit on the number of batched calls."]
                pub fn batched_calls_limit(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Utility",
                        "batched_calls_limit",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod multisig {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AsMultiThreshold1 {
                pub other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                pub call: ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AsMulti {
                pub threshold: ::core::primitive::u16,
                pub other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                pub maybe_timepoint: ::core::option::Option<
                    runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                >,
                pub call: ::std::boxed::Box<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ApproveAsMulti {
                pub threshold: ::core::primitive::u16,
                pub other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                pub maybe_timepoint: ::core::option::Option<
                    runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                >,
                pub call_hash: [::core::primitive::u8; 32usize],
                pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct CancelAsMulti {
                pub threshold: ::core::primitive::u16,
                pub other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                pub timepoint: runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                pub call_hash: [::core::primitive::u8; 32usize],
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::as_multi_threshold_1`]."]
                pub fn as_multi_threshold_1(
                    &self,
                    other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    call: runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                ) -> ::subxt::tx::StaticTxPayload<AsMultiThreshold1> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Multisig",
                        "as_multi_threshold_1",
                        AsMultiThreshold1 {
                            other_signatories,
                            call: ::std::boxed::Box::new(call),
                        },
                        [
                            245u8, 25u8, 221u8, 230u8, 82u8, 142u8, 139u8, 2u8, 131u8, 165u8,
                            221u8, 255u8, 201u8, 27u8, 44u8, 126u8, 218u8, 15u8, 19u8, 48u8, 251u8,
                            113u8, 223u8, 95u8, 70u8, 132u8, 208u8, 42u8, 190u8, 32u8, 121u8,
                            193u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::as_multi`]."]
                pub fn as_multi(
                    &self,
                    threshold: ::core::primitive::u16,
                    other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    maybe_timepoint: ::core::option::Option<
                        runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                    >,
                    call: runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                    max_weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<AsMulti> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Multisig",
                        "as_multi",
                        AsMulti {
                            threshold,
                            other_signatories,
                            maybe_timepoint,
                            call: ::std::boxed::Box::new(call),
                            max_weight,
                        },
                        [
                            115u8, 159u8, 55u8, 89u8, 178u8, 64u8, 25u8, 213u8, 251u8, 217u8,
                            252u8, 54u8, 191u8, 139u8, 103u8, 96u8, 148u8, 90u8, 20u8, 108u8, 45u8,
                            183u8, 174u8, 28u8, 46u8, 90u8, 228u8, 228u8, 32u8, 75u8, 14u8, 154u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::approve_as_multi`]."]
                pub fn approve_as_multi(
                    &self,
                    threshold: ::core::primitive::u16,
                    other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    maybe_timepoint: ::core::option::Option<
                        runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                    >,
                    call_hash: [::core::primitive::u8; 32usize],
                    max_weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ApproveAsMulti> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Multisig",
                        "approve_as_multi",
                        ApproveAsMulti {
                            threshold,
                            other_signatories,
                            maybe_timepoint,
                            call_hash,
                            max_weight,
                        },
                        [
                            133u8, 113u8, 121u8, 66u8, 218u8, 219u8, 48u8, 64u8, 211u8, 114u8,
                            163u8, 193u8, 164u8, 21u8, 140u8, 218u8, 253u8, 237u8, 240u8, 126u8,
                            200u8, 213u8, 184u8, 50u8, 187u8, 182u8, 30u8, 52u8, 142u8, 72u8,
                            210u8, 101u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::cancel_as_multi`]."]
                pub fn cancel_as_multi(
                    &self,
                    threshold: ::core::primitive::u16,
                    other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    timepoint: runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                    call_hash: [::core::primitive::u8; 32usize],
                ) -> ::subxt::tx::StaticTxPayload<CancelAsMulti> {
                    ::subxt::tx::StaticTxPayload::new(
                        "Multisig",
                        "cancel_as_multi",
                        CancelAsMulti {
                            threshold,
                            other_signatories,
                            timepoint,
                            call_hash,
                        },
                        [
                            30u8, 25u8, 186u8, 142u8, 168u8, 81u8, 235u8, 164u8, 82u8, 209u8, 66u8,
                            129u8, 209u8, 78u8, 172u8, 9u8, 163u8, 222u8, 125u8, 57u8, 2u8, 43u8,
                            169u8, 174u8, 159u8, 167u8, 25u8, 226u8, 254u8, 110u8, 80u8, 216u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_multisig::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A new multisig operation has begun."]
            pub struct NewMultisig {
                pub approving: ::subxt::utils::AccountId32,
                pub multisig: ::subxt::utils::AccountId32,
                pub call_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for NewMultisig {
                const PALLET: &'static str = "Multisig";
                const EVENT: &'static str = "NewMultisig";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A multisig operation has been approved by someone."]
            pub struct MultisigApproval {
                pub approving: ::subxt::utils::AccountId32,
                pub timepoint: runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                pub multisig: ::subxt::utils::AccountId32,
                pub call_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for MultisigApproval {
                const PALLET: &'static str = "Multisig";
                const EVENT: &'static str = "MultisigApproval";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A multisig operation has been executed."]
            pub struct MultisigExecuted {
                pub approving: ::subxt::utils::AccountId32,
                pub timepoint: runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                pub multisig: ::subxt::utils::AccountId32,
                pub call_hash: [::core::primitive::u8; 32usize],
                pub result: ::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
            }
            impl ::subxt::events::StaticEvent for MultisigExecuted {
                const PALLET: &'static str = "Multisig";
                const EVENT: &'static str = "MultisigExecuted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A multisig operation has been cancelled."]
            pub struct MultisigCancelled {
                pub cancelling: ::subxt::utils::AccountId32,
                pub timepoint: runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                pub multisig: ::subxt::utils::AccountId32,
                pub call_hash: [::core::primitive::u8; 32usize],
            }
            impl ::subxt::events::StaticEvent for MultisigCancelled {
                const PALLET: &'static str = "Multisig";
                const EVENT: &'static str = "MultisigCancelled";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The set of open multisig operations."]
                pub fn multisigs(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                    _1: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_multisig::Multisig<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                            ::subxt::utils::AccountId32,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Multisig",
                        "Multisigs",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                        ],
                        [
                            69u8, 153u8, 186u8, 204u8, 117u8, 95u8, 119u8, 182u8, 220u8, 87u8, 8u8,
                            15u8, 123u8, 83u8, 5u8, 188u8, 115u8, 121u8, 163u8, 96u8, 218u8, 3u8,
                            106u8, 44u8, 44u8, 187u8, 46u8, 238u8, 80u8, 203u8, 175u8, 155u8,
                        ],
                    )
                }
                #[doc = " The set of open multisig operations."]
                pub fn multisigs_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_multisig::Multisig<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                            ::subxt::utils::AccountId32,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "Multisig",
                        "Multisigs",
                        Vec::new(),
                        [
                            69u8, 153u8, 186u8, 204u8, 117u8, 95u8, 119u8, 182u8, 220u8, 87u8, 8u8,
                            15u8, 123u8, 83u8, 5u8, 188u8, 115u8, 121u8, 163u8, 96u8, 218u8, 3u8,
                            106u8, 44u8, 44u8, 187u8, 46u8, 238u8, 80u8, 203u8, 175u8, 155u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " The base amount of currency needed to reserve for creating a multisig execution or to"]
                #[doc = " store a dispatch call for later."]
                #[doc = ""]
                #[doc = " This is held for an additional storage item whose value size is"]
                #[doc = " `4 + sizeof((BlockNumber, Balance, AccountId))` bytes and whose key size is"]
                #[doc = " `32 + sizeof(AccountId)` bytes."]
                pub fn deposit_base(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Multisig",
                        "DepositBase",
                        [
                            84u8, 157u8, 140u8, 4u8, 93u8, 57u8, 29u8, 133u8, 105u8, 200u8, 214u8,
                            27u8, 144u8, 208u8, 218u8, 160u8, 130u8, 109u8, 101u8, 54u8, 210u8,
                            136u8, 71u8, 63u8, 49u8, 237u8, 234u8, 15u8, 178u8, 98u8, 148u8, 156u8,
                        ],
                    )
                }
                #[doc = " The amount of currency needed per unit threshold when creating a multisig execution."]
                #[doc = ""]
                #[doc = " This is held for adding 32 bytes more into a pre-existing storage value."]
                pub fn deposit_factor(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Multisig",
                        "DepositFactor",
                        [
                            84u8, 157u8, 140u8, 4u8, 93u8, 57u8, 29u8, 133u8, 105u8, 200u8, 214u8,
                            27u8, 144u8, 208u8, 218u8, 160u8, 130u8, 109u8, 101u8, 54u8, 210u8,
                            136u8, 71u8, 63u8, 49u8, 237u8, 234u8, 15u8, 178u8, 98u8, 148u8, 156u8,
                        ],
                    )
                }
                #[doc = " The maximum amount of signatories allowed in the multisig."]
                pub fn max_signatories(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "Multisig",
                        "MaxSignatories",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_wococo_grandpa {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SubmitFinalityProof {
                pub finality_target: ::std::boxed::Box<
                    runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                >,
                pub justification:
                    runtime_types::bp_header_chain::justification::GrandpaJustification<
                        runtime_types::sp_runtime::generic::header::Header<
                            ::core::primitive::u32,
                            runtime_types::sp_runtime::traits::BlakeTwo256,
                        >,
                    >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Initialize {
                pub init_data: runtime_types::bp_header_chain::InitializationData<
                    runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::submit_finality_proof`]."]
                pub fn submit_finality_proof(
                    &self,
                    finality_target: runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                    justification : runtime_types :: bp_header_chain :: justification :: GrandpaJustification < runtime_types :: sp_runtime :: generic :: header :: Header < :: core :: primitive :: u32 , runtime_types :: sp_runtime :: traits :: BlakeTwo256 > >,
                ) -> ::subxt::tx::StaticTxPayload<SubmitFinalityProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoGrandpa",
                        "submit_finality_proof",
                        SubmitFinalityProof {
                            finality_target: ::std::boxed::Box::new(finality_target),
                            justification,
                        },
                        [
                            243u8, 151u8, 137u8, 216u8, 68u8, 155u8, 240u8, 189u8, 151u8, 109u8,
                            146u8, 206u8, 153u8, 172u8, 229u8, 34u8, 235u8, 91u8, 233u8, 74u8,
                            53u8, 0u8, 244u8, 197u8, 10u8, 245u8, 30u8, 180u8, 163u8, 147u8, 121u8,
                            252u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::initialize`]."]
                pub fn initialize(
                    &self,
                    init_data: runtime_types::bp_header_chain::InitializationData<
                        runtime_types::sp_runtime::generic::header::Header<
                            ::core::primitive::u32,
                            runtime_types::sp_runtime::traits::BlakeTwo256,
                        >,
                    >,
                ) -> ::subxt::tx::StaticTxPayload<Initialize> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoGrandpa",
                        "initialize",
                        Initialize { init_data },
                        [
                            135u8, 85u8, 211u8, 216u8, 17u8, 111u8, 188u8, 136u8, 164u8, 189u8,
                            64u8, 137u8, 228u8, 131u8, 176u8, 129u8, 27u8, 14u8, 58u8, 27u8, 72u8,
                            254u8, 101u8, 217u8, 210u8, 178u8, 195u8, 187u8, 15u8, 110u8, 78u8,
                            103u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoGrandpa",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoGrandpa",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_grandpa::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Best finalized chain header has been updated to the header with given number and hash."]
            pub struct UpdatedBestFinalizedHeader {
                pub number: ::core::primitive::u32,
                pub hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for UpdatedBestFinalizedHeader {
                const PALLET: &'static str = "BridgeWococoGrandpa";
                const EVENT: &'static str = "UpdatedBestFinalizedHeader";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Number mandatory headers that we may accept in the current block for free (returning"]
                #[doc = " `Pays::No`)."]
                #[doc = ""]
                #[doc = " If the `FreeMandatoryHeadersRemaining` hits zero, all following mandatory headers in the"]
                #[doc = " current block are accepted with fee (`Pays::Yes` is returned)."]
                #[doc = ""]
                #[doc = " The `FreeMandatoryHeadersRemaining` is an ephemeral value that is set to"]
                #[doc = " `MaxFreeMandatoryHeadersPerBlock` at each block initialization and is killed on block"]
                #[doc = " finalization. So it never ends up in the storage trie."]
                pub fn free_mandatory_headers_remaining(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "FreeMandatoryHeadersRemaining",
                        vec![],
                        [
                            0u8, 224u8, 249u8, 242u8, 233u8, 176u8, 22u8, 146u8, 231u8, 123u8,
                            181u8, 166u8, 59u8, 33u8, 68u8, 34u8, 228u8, 155u8, 69u8, 177u8, 147u8,
                            70u8, 202u8, 200u8, 188u8, 230u8, 122u8, 91u8, 83u8, 232u8, 38u8, 68u8,
                        ],
                    )
                }
                #[doc = " Hash of the header used to bootstrap the pallet."]
                pub fn initial_hash(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "InitialHash",
                        vec![],
                        [
                            174u8, 39u8, 103u8, 193u8, 12u8, 0u8, 178u8, 128u8, 1u8, 176u8, 217u8,
                            69u8, 175u8, 181u8, 65u8, 105u8, 213u8, 19u8, 146u8, 159u8, 12u8,
                            228u8, 136u8, 242u8, 26u8, 186u8, 136u8, 2u8, 235u8, 162u8, 249u8,
                            247u8,
                        ],
                    )
                }
                #[doc = " Hash of the best finalized header."]
                pub fn best_finalized(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::HeaderId<
                            ::subxt::utils::H256,
                            ::core::primitive::u32,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "BestFinalized",
                        vec![],
                        [
                            117u8, 63u8, 104u8, 89u8, 13u8, 71u8, 15u8, 76u8, 96u8, 71u8, 250u8,
                            159u8, 18u8, 199u8, 158u8, 213u8, 55u8, 96u8, 121u8, 2u8, 220u8, 244u8,
                            242u8, 127u8, 183u8, 63u8, 95u8, 175u8, 21u8, 23u8, 101u8, 194u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported hashes. Ordered by the insertion time."]
                pub fn imported_hashes(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "ImportedHashes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            28u8, 27u8, 54u8, 5u8, 190u8, 52u8, 90u8, 217u8, 92u8, 228u8, 169u8,
                            132u8, 147u8, 0u8, 78u8, 146u8, 169u8, 234u8, 47u8, 93u8, 144u8, 2u8,
                            201u8, 223u8, 161u8, 175u8, 45u8, 150u8, 45u8, 237u8, 218u8, 58u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported hashes. Ordered by the insertion time."]
                pub fn imported_hashes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "ImportedHashes",
                        Vec::new(),
                        [
                            28u8, 27u8, 54u8, 5u8, 190u8, 52u8, 90u8, 217u8, 92u8, 228u8, 169u8,
                            132u8, 147u8, 0u8, 78u8, 146u8, 169u8, 234u8, 47u8, 93u8, 144u8, 2u8,
                            201u8, 223u8, 161u8, 175u8, 45u8, 150u8, 45u8, 237u8, 218u8, 58u8,
                        ],
                    )
                }
                #[doc = " Current ring buffer position."]
                pub fn imported_hashes_pointer(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "ImportedHashesPointer",
                        vec![],
                        [
                            159u8, 83u8, 35u8, 45u8, 27u8, 249u8, 155u8, 131u8, 181u8, 196u8,
                            224u8, 26u8, 92u8, 132u8, 127u8, 237u8, 13u8, 142u8, 196u8, 147u8,
                            221u8, 216u8, 11u8, 78u8, 190u8, 241u8, 201u8, 96u8, 74u8, 185u8,
                            208u8, 42u8,
                        ],
                    )
                }
                #[doc = " Relevant fields of imported headers."]
                pub fn imported_headers(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_header_chain::StoredHeaderData<
                            ::core::primitive::u32,
                            ::subxt::utils::H256,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "ImportedHeaders",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            14u8, 120u8, 104u8, 228u8, 94u8, 125u8, 44u8, 30u8, 1u8, 32u8, 28u8,
                            65u8, 74u8, 131u8, 170u8, 109u8, 126u8, 28u8, 74u8, 125u8, 37u8, 137u8,
                            213u8, 37u8, 139u8, 224u8, 42u8, 60u8, 113u8, 141u8, 103u8, 223u8,
                        ],
                    )
                }
                #[doc = " Relevant fields of imported headers."]
                pub fn imported_headers_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_header_chain::StoredHeaderData<
                            ::core::primitive::u32,
                            ::subxt::utils::H256,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "ImportedHeaders",
                        Vec::new(),
                        [
                            14u8, 120u8, 104u8, 228u8, 94u8, 125u8, 44u8, 30u8, 1u8, 32u8, 28u8,
                            65u8, 74u8, 131u8, 170u8, 109u8, 126u8, 28u8, 74u8, 125u8, 37u8, 137u8,
                            213u8, 37u8, 139u8, 224u8, 42u8, 60u8, 113u8, 141u8, 103u8, 223u8,
                        ],
                    )
                }
                #[doc = " The current GRANDPA Authority set."]
                pub fn current_authority_set(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_bridge_grandpa::storage_types::StoredAuthoritySet,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "CurrentAuthoritySet",
                        vec![],
                        [
                            249u8, 40u8, 229u8, 120u8, 141u8, 219u8, 206u8, 5u8, 54u8, 121u8,
                            207u8, 77u8, 8u8, 80u8, 105u8, 107u8, 151u8, 111u8, 82u8, 119u8, 8u8,
                            31u8, 104u8, 82u8, 92u8, 156u8, 37u8, 160u8, 235u8, 64u8, 62u8, 94u8,
                        ],
                    )
                }
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume it. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoGrandpa",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Maximal number of \"free\" mandatory header transactions per block."]
                #[doc = ""]
                #[doc = " To be able to track the bridged chain, the pallet requires all headers that are"]
                #[doc = " changing GRANDPA authorities set at the bridged chain (we call them mandatory)."]
                #[doc = " So it is a common good deed to submit mandatory headers to the pallet. However, if the"]
                #[doc = " bridged chain gets compromised, its validators may generate as many mandatory headers"]
                #[doc = " as they want. And they may fill the whole block (at this chain) for free. This constants"]
                #[doc = " limits number of calls that we may refund in a single block. All calls above this"]
                #[doc = " limit are accepted, but are not refunded."]
                pub fn max_free_mandatory_headers_per_block(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoGrandpa",
                        "MaxFreeMandatoryHeadersPerBlock",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " Maximal number of finalized headers to keep in the storage."]
                #[doc = ""]
                #[doc = " The setting is there to prevent growing the on-chain state indefinitely. Note"]
                #[doc = " the setting does not relate to block numbers - we will simply keep as much items"]
                #[doc = " in the storage, so it doesn't guarantee any fixed timeframe for finality headers."]
                #[doc = ""]
                #[doc = " Incautious change of this constant may lead to orphan entries in the runtime storage."]
                pub fn headers_to_keep(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoGrandpa",
                        "HeadersToKeep",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_wococo_parachain {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SubmitParachainHeads {
                pub at_relay_block: (::core::primitive::u32, ::subxt::utils::H256),
                pub parachains: ::std::vec::Vec<(
                    runtime_types::bp_polkadot_core::parachains::ParaId,
                    ::subxt::utils::H256,
                )>,
                pub parachain_heads_proof:
                    runtime_types::bp_polkadot_core::parachains::ParaHeadsProof,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::submit_parachain_heads`]."]
                pub fn submit_parachain_heads(
                    &self,
                    at_relay_block: (::core::primitive::u32, ::subxt::utils::H256),
                    parachains: ::std::vec::Vec<(
                        runtime_types::bp_polkadot_core::parachains::ParaId,
                        ::subxt::utils::H256,
                    )>,
                    parachain_heads_proof : runtime_types :: bp_polkadot_core :: parachains :: ParaHeadsProof,
                ) -> ::subxt::tx::StaticTxPayload<SubmitParachainHeads> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoParachain",
                        "submit_parachain_heads",
                        SubmitParachainHeads {
                            at_relay_block,
                            parachains,
                            parachain_heads_proof,
                        },
                        [
                            5u8, 103u8, 254u8, 130u8, 181u8, 160u8, 201u8, 97u8, 253u8, 95u8, 9u8,
                            109u8, 68u8, 128u8, 87u8, 86u8, 21u8, 40u8, 124u8, 125u8, 23u8, 182u8,
                            240u8, 190u8, 7u8, 99u8, 188u8, 57u8, 253u8, 204u8, 177u8, 109u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoParachain",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoParachain",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_parachains::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided head of parachain that the pallet is not configured to track."]
            pub struct UntrackedParachainRejected {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
            }
            impl ::subxt::events::StaticEvent for UntrackedParachainRejected {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "UntrackedParachainRejected";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has declared that he has provided given parachain head, but it is missing"]
            #[doc = "from the storage proof."]
            pub struct MissingParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
            }
            impl ::subxt::events::StaticEvent for MissingParachainHead {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "MissingParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided parachain head hash that is not matching the hash read from the"]
            #[doc = "storage proof."]
            pub struct IncorrectParachainHeadHash {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
                pub actual_parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for IncorrectParachainHeadHash {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "IncorrectParachainHeadHash";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided obsolete parachain head, which is already known to the pallet."]
            pub struct RejectedObsoleteParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for RejectedObsoleteParachainHead {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "RejectedObsoleteParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided parachain head that exceeds the maximal configured head size."]
            pub struct RejectedLargeParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
                pub parachain_head_size: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for RejectedLargeParachainHead {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "RejectedLargeParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Parachain head has been updated."]
            pub struct UpdatedParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for UpdatedParachainHead {
                const PALLET: &'static str = "BridgeWococoParachain";
                const EVENT: &'static str = "UpdatedParachainHead";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume them. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
                #[doc = " Parachains info."]
                #[doc = ""]
                #[doc = " Contains the following info:"]
                #[doc = " - best parachain head hash"]
                #[doc = " - the head of the `ImportedParaHashes` ring buffer"]
                pub fn paras_info(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::bp_parachains::ParaInfo>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ParasInfo",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            22u8, 107u8, 60u8, 52u8, 124u8, 172u8, 17u8, 70u8, 73u8, 219u8, 67u8,
                            240u8, 79u8, 198u8, 186u8, 81u8, 128u8, 101u8, 226u8, 58u8, 160u8,
                            11u8, 162u8, 246u8, 85u8, 150u8, 137u8, 63u8, 3u8, 222u8, 87u8, 45u8,
                        ],
                    )
                }
                #[doc = " Parachains info."]
                #[doc = ""]
                #[doc = " Contains the following info:"]
                #[doc = " - best parachain head hash"]
                #[doc = " - the head of the `ImportedParaHashes` ring buffer"]
                pub fn paras_info_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::bp_parachains::ParaInfo>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ParasInfo",
                        Vec::new(),
                        [
                            22u8, 107u8, 60u8, 52u8, 124u8, 172u8, 17u8, 70u8, 73u8, 219u8, 67u8,
                            240u8, 79u8, 198u8, 186u8, 81u8, 128u8, 101u8, 226u8, 58u8, 160u8,
                            11u8, 162u8, 246u8, 85u8, 150u8, 137u8, 63u8, 3u8, 222u8, 87u8, 45u8,
                        ],
                    )
                }
                #[doc = " State roots of parachain heads which have been imported into the pallet."]
                pub fn imported_para_heads(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                    _1: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_parachains::ParaStoredHeaderData,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ImportedParaHeads",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                        ],
                        [
                            134u8, 183u8, 37u8, 49u8, 91u8, 83u8, 197u8, 0u8, 110u8, 79u8, 143u8,
                            114u8, 36u8, 164u8, 254u8, 173u8, 191u8, 69u8, 137u8, 178u8, 56u8,
                            192u8, 151u8, 201u8, 39u8, 251u8, 150u8, 144u8, 37u8, 69u8, 246u8,
                            251u8,
                        ],
                    )
                }
                #[doc = " State roots of parachain heads which have been imported into the pallet."]
                pub fn imported_para_heads_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_parachains::ParaStoredHeaderData,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ImportedParaHeads",
                        Vec::new(),
                        [
                            134u8, 183u8, 37u8, 49u8, 91u8, 83u8, 197u8, 0u8, 110u8, 79u8, 143u8,
                            114u8, 36u8, 164u8, 254u8, 173u8, 191u8, 69u8, 137u8, 178u8, 56u8,
                            192u8, 151u8, 201u8, 39u8, 251u8, 150u8, 144u8, 37u8, 69u8, 246u8,
                            251u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported parachain head hashes. Ordered by the insertion time."]
                pub fn imported_para_hashes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                    _1: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ImportedParaHashes",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                        ],
                        [
                            15u8, 216u8, 100u8, 17u8, 75u8, 61u8, 65u8, 33u8, 212u8, 227u8, 102u8,
                            48u8, 108u8, 160u8, 103u8, 60u8, 136u8, 189u8, 147u8, 103u8, 97u8,
                            181u8, 207u8, 75u8, 254u8, 157u8, 113u8, 99u8, 209u8, 103u8, 143u8,
                            110u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported parachain head hashes. Ordered by the insertion time."]
                pub fn imported_para_hashes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoParachain",
                        "ImportedParaHashes",
                        Vec::new(),
                        [
                            15u8, 216u8, 100u8, 17u8, 75u8, 61u8, 65u8, 33u8, 212u8, 227u8, 102u8,
                            48u8, 108u8, 160u8, 103u8, 60u8, 136u8, 189u8, 147u8, 103u8, 97u8,
                            181u8, 207u8, 75u8, 254u8, 157u8, 113u8, 99u8, 209u8, 103u8, 143u8,
                            110u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Name of the original `paras` pallet in the `construct_runtime!()` call at the bridged"]
                #[doc = " chain."]
                #[doc = ""]
                #[doc = " Please keep in mind that this should be the name of the `runtime_parachains::paras`"]
                #[doc = " pallet from polkadot repository, not the `pallet-bridge-parachains`."]
                pub fn paras_pallet_name(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::std::string::String>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoParachain",
                        "ParasPalletName",
                        [
                            251u8, 233u8, 211u8, 209u8, 5u8, 66u8, 94u8, 200u8, 148u8, 166u8,
                            119u8, 200u8, 59u8, 180u8, 70u8, 77u8, 182u8, 127u8, 45u8, 65u8, 28u8,
                            104u8, 253u8, 149u8, 167u8, 216u8, 2u8, 94u8, 39u8, 173u8, 198u8,
                            219u8,
                        ],
                    )
                }
                #[doc = " Maximal number of single parachain heads to keep in the storage."]
                #[doc = ""]
                #[doc = " The setting is there to prevent growing the on-chain state indefinitely. Note"]
                #[doc = " the setting does not relate to parachain block numbers - we will simply keep as much"]
                #[doc = " items in the storage, so it doesn't guarantee any fixed timeframe for heads."]
                #[doc = ""]
                #[doc = " Incautious change of this constant may lead to orphan entries in the runtime storage."]
                pub fn heads_to_keep(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoParachain",
                        "HeadsToKeep",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " Maximal size (in bytes) of the SCALE-encoded parachain head data"]
                #[doc = " (`bp_parachains::ParaStoredHeaderData`)."]
                #[doc = ""]
                #[doc = " Keep in mind that the size of any tracked parachain header data must not exceed this"]
                #[doc = " value. So if you're going to track multiple parachains, one of which is using large"]
                #[doc = " hashes, you shall choose this maximal value."]
                #[doc = ""]
                #[doc = " There's no mandatory headers in this pallet, so it can't stall if there's some header"]
                #[doc = " that exceeds this bound."]
                pub fn max_para_head_data_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoParachain",
                        "MaxParaHeadDataSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_wococo_messages {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_messages::MessagesOperatingMode,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReceiveMessagesProof { pub relayer_id_at_bridged_chain : :: subxt :: utils :: AccountId32 , pub proof : runtime_types :: bridge_runtime_common :: messages :: target :: FromBridgedChainMessagesProof < :: subxt :: utils :: H256 > , pub messages_count : :: core :: primitive :: u32 , pub dispatch_weight : runtime_types :: sp_weights :: weight_v2 :: Weight , }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReceiveMessagesDeliveryProof { pub proof : runtime_types :: bridge_runtime_common :: messages :: source :: FromBridgedChainMessagesDeliveryProof < :: subxt :: utils :: H256 > , pub relayers_state : runtime_types :: bp_messages :: UnrewardedRelayersState , }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoMessages",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_messages::MessagesOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoMessages",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            236u8, 230u8, 127u8, 17u8, 145u8, 186u8, 102u8, 200u8, 227u8, 208u8,
                            230u8, 121u8, 102u8, 199u8, 123u8, 118u8, 199u8, 160u8, 131u8, 116u8,
                            102u8, 167u8, 119u8, 144u8, 70u8, 114u8, 0u8, 223u8, 54u8, 197u8, 39u8,
                            58u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::receive_messages_proof`]."]
                pub fn receive_messages_proof(
                    &self,
                    relayer_id_at_bridged_chain: ::subxt::utils::AccountId32,
                    proof : runtime_types :: bridge_runtime_common :: messages :: target :: FromBridgedChainMessagesProof < :: subxt :: utils :: H256 >,
                    messages_count: ::core::primitive::u32,
                    dispatch_weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ReceiveMessagesProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoMessages",
                        "receive_messages_proof",
                        ReceiveMessagesProof {
                            relayer_id_at_bridged_chain,
                            proof,
                            messages_count,
                            dispatch_weight,
                        },
                        [
                            74u8, 80u8, 32u8, 12u8, 106u8, 198u8, 121u8, 131u8, 209u8, 175u8, 93u8,
                            240u8, 144u8, 63u8, 20u8, 127u8, 20u8, 227u8, 166u8, 29u8, 155u8,
                            117u8, 101u8, 109u8, 8u8, 37u8, 192u8, 144u8, 165u8, 14u8, 170u8, 65u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::receive_messages_delivery_proof`]."]
                pub fn receive_messages_delivery_proof(
                    &self,
                    proof : runtime_types :: bridge_runtime_common :: messages :: source :: FromBridgedChainMessagesDeliveryProof < :: subxt :: utils :: H256 >,
                    relayers_state: runtime_types::bp_messages::UnrewardedRelayersState,
                ) -> ::subxt::tx::StaticTxPayload<ReceiveMessagesDeliveryProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeWococoMessages",
                        "receive_messages_delivery_proof",
                        ReceiveMessagesDeliveryProof {
                            proof,
                            relayers_state,
                        },
                        [
                            3u8, 106u8, 216u8, 97u8, 151u8, 155u8, 45u8, 209u8, 104u8, 156u8, 16u8,
                            114u8, 213u8, 44u8, 77u8, 226u8, 68u8, 66u8, 178u8, 137u8, 131u8, 8u8,
                            153u8, 209u8, 146u8, 32u8, 255u8, 215u8, 130u8, 104u8, 122u8, 95u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_messages::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message has been accepted and is waiting to be delivered."]
            pub struct MessageAccepted {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub nonce: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for MessageAccepted {
                const PALLET: &'static str = "BridgeWococoMessages";
                const EVENT: &'static str = "MessageAccepted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Messages have been received from the bridged chain."]
            pub struct MessagesReceived (pub :: std :: vec :: Vec < runtime_types :: bp_messages :: ReceivedMessages < runtime_types :: bridge_runtime_common :: messages_xcm_extension :: XcmBlobMessageDispatchResult > > ,) ;
            impl ::subxt::events::StaticEvent for MessagesReceived {
                const PALLET: &'static str = "BridgeWococoMessages";
                const EVENT: &'static str = "MessagesReceived";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Messages in the inclusive range have been delivered to the bridged chain."]
            pub struct MessagesDelivered {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub messages: runtime_types::bp_messages::DeliveredMessages,
            }
            impl ::subxt::events::StaticEvent for MessagesDelivered {
                const PALLET: &'static str = "BridgeWococoMessages";
                const EVENT: &'static str = "MessagesDelivered";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume it. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, some, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::MessagesOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "PalletOperatingMode",
                        vec![],
                        [
                            215u8, 195u8, 85u8, 231u8, 158u8, 22u8, 160u8, 132u8, 69u8, 206u8,
                            238u8, 14u8, 56u8, 100u8, 134u8, 41u8, 58u8, 120u8, 225u8, 164u8,
                            173u8, 87u8, 22u8, 123u8, 102u8, 167u8, 68u8, 70u8, 184u8, 131u8,
                            232u8, 65u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => inbound lane data."]
                pub fn inbound_lanes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::LaneId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::InboundLaneData<::subxt::utils::AccountId32>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "InboundLanes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            38u8, 58u8, 110u8, 130u8, 112u8, 76u8, 231u8, 76u8, 56u8, 241u8, 183u8,
                            153u8, 112u8, 41u8, 248u8, 208u8, 217u8, 57u8, 102u8, 30u8, 107u8,
                            98u8, 59u8, 78u8, 56u8, 119u8, 186u8, 183u8, 213u8, 72u8, 199u8, 90u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => inbound lane data."]
                pub fn inbound_lanes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::InboundLaneData<::subxt::utils::AccountId32>,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "InboundLanes",
                        Vec::new(),
                        [
                            38u8, 58u8, 110u8, 130u8, 112u8, 76u8, 231u8, 76u8, 56u8, 241u8, 183u8,
                            153u8, 112u8, 41u8, 248u8, 208u8, 217u8, 57u8, 102u8, 30u8, 107u8,
                            98u8, 59u8, 78u8, 56u8, 119u8, 186u8, 183u8, 213u8, 72u8, 199u8, 90u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => outbound lane data."]
                pub fn outbound_lanes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::LaneId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::OutboundLaneData,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "OutboundLanes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            67u8, 155u8, 173u8, 244u8, 80u8, 38u8, 26u8, 71u8, 51u8, 150u8, 86u8,
                            146u8, 132u8, 122u8, 70u8, 122u8, 172u8, 246u8, 106u8, 232u8, 149u8,
                            227u8, 240u8, 146u8, 51u8, 184u8, 30u8, 182u8, 200u8, 43u8, 190u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => outbound lane data."]
                pub fn outbound_lanes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::OutboundLaneData,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "OutboundLanes",
                        Vec::new(),
                        [
                            67u8, 155u8, 173u8, 244u8, 80u8, 38u8, 26u8, 71u8, 51u8, 150u8, 86u8,
                            146u8, 132u8, 122u8, 70u8, 122u8, 172u8, 246u8, 106u8, 232u8, 149u8,
                            227u8, 240u8, 146u8, 51u8, 184u8, 30u8, 182u8, 200u8, 43u8, 190u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " All queued outbound messages."]
                pub fn outbound_messages(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::MessageKey>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "OutboundMessages",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            44u8, 35u8, 2u8, 25u8, 91u8, 101u8, 152u8, 23u8, 48u8, 250u8, 178u8,
                            15u8, 194u8, 118u8, 146u8, 1u8, 112u8, 83u8, 243u8, 166u8, 124u8,
                            153u8, 48u8, 193u8, 43u8, 31u8, 33u8, 72u8, 228u8, 113u8, 86u8, 217u8,
                        ],
                    )
                }
                #[doc = " All queued outbound messages."]
                pub fn outbound_messages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeWococoMessages",
                        "OutboundMessages",
                        Vec::new(),
                        [
                            44u8, 35u8, 2u8, 25u8, 91u8, 101u8, 152u8, 23u8, 48u8, 250u8, 178u8,
                            15u8, 194u8, 118u8, 146u8, 1u8, 112u8, 83u8, 243u8, 166u8, 124u8,
                            153u8, 48u8, 193u8, 43u8, 31u8, 33u8, 72u8, 228u8, 113u8, 86u8, 217u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Gets the chain id value from the instance."]
                pub fn bridged_chain_id(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<[::core::primitive::u8; 4usize]>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoMessages",
                        "BridgedChainId",
                        [
                            101u8, 157u8, 37u8, 163u8, 190u8, 134u8, 129u8, 212u8, 240u8, 135u8,
                            174u8, 76u8, 220u8, 179u8, 252u8, 69u8, 65u8, 253u8, 69u8, 214u8, 61u8,
                            249u8, 4u8, 38u8, 181u8, 237u8, 25u8, 131u8, 242u8, 20u8, 17u8, 152u8,
                        ],
                    )
                }
                #[doc = " Maximal encoded size of the outbound payload."]
                pub fn maximal_outbound_payload_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeWococoMessages",
                        "MaximalOutboundPayloadSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_rococo_grandpa {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SubmitFinalityProof {
                pub finality_target: ::std::boxed::Box<
                    runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                >,
                pub justification:
                    runtime_types::bp_header_chain::justification::GrandpaJustification<
                        runtime_types::sp_runtime::generic::header::Header<
                            ::core::primitive::u32,
                            runtime_types::sp_runtime::traits::BlakeTwo256,
                        >,
                    >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Initialize {
                pub init_data: runtime_types::bp_header_chain::InitializationData<
                    runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::submit_finality_proof`]."]
                pub fn submit_finality_proof(
                    &self,
                    finality_target: runtime_types::sp_runtime::generic::header::Header<
                        ::core::primitive::u32,
                        runtime_types::sp_runtime::traits::BlakeTwo256,
                    >,
                    justification : runtime_types :: bp_header_chain :: justification :: GrandpaJustification < runtime_types :: sp_runtime :: generic :: header :: Header < :: core :: primitive :: u32 , runtime_types :: sp_runtime :: traits :: BlakeTwo256 > >,
                ) -> ::subxt::tx::StaticTxPayload<SubmitFinalityProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoGrandpa",
                        "submit_finality_proof",
                        SubmitFinalityProof {
                            finality_target: ::std::boxed::Box::new(finality_target),
                            justification,
                        },
                        [
                            243u8, 151u8, 137u8, 216u8, 68u8, 155u8, 240u8, 189u8, 151u8, 109u8,
                            146u8, 206u8, 153u8, 172u8, 229u8, 34u8, 235u8, 91u8, 233u8, 74u8,
                            53u8, 0u8, 244u8, 197u8, 10u8, 245u8, 30u8, 180u8, 163u8, 147u8, 121u8,
                            252u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::initialize`]."]
                pub fn initialize(
                    &self,
                    init_data: runtime_types::bp_header_chain::InitializationData<
                        runtime_types::sp_runtime::generic::header::Header<
                            ::core::primitive::u32,
                            runtime_types::sp_runtime::traits::BlakeTwo256,
                        >,
                    >,
                ) -> ::subxt::tx::StaticTxPayload<Initialize> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoGrandpa",
                        "initialize",
                        Initialize { init_data },
                        [
                            135u8, 85u8, 211u8, 216u8, 17u8, 111u8, 188u8, 136u8, 164u8, 189u8,
                            64u8, 137u8, 228u8, 131u8, 176u8, 129u8, 27u8, 14u8, 58u8, 27u8, 72u8,
                            254u8, 101u8, 217u8, 210u8, 178u8, 195u8, 187u8, 15u8, 110u8, 78u8,
                            103u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoGrandpa",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoGrandpa",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_grandpa::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Best finalized chain header has been updated to the header with given number and hash."]
            pub struct UpdatedBestFinalizedHeader {
                pub number: ::core::primitive::u32,
                pub hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for UpdatedBestFinalizedHeader {
                const PALLET: &'static str = "BridgeRococoGrandpa";
                const EVENT: &'static str = "UpdatedBestFinalizedHeader";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Number mandatory headers that we may accept in the current block for free (returning"]
                #[doc = " `Pays::No`)."]
                #[doc = ""]
                #[doc = " If the `FreeMandatoryHeadersRemaining` hits zero, all following mandatory headers in the"]
                #[doc = " current block are accepted with fee (`Pays::Yes` is returned)."]
                #[doc = ""]
                #[doc = " The `FreeMandatoryHeadersRemaining` is an ephemeral value that is set to"]
                #[doc = " `MaxFreeMandatoryHeadersPerBlock` at each block initialization and is killed on block"]
                #[doc = " finalization. So it never ends up in the storage trie."]
                pub fn free_mandatory_headers_remaining(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "FreeMandatoryHeadersRemaining",
                        vec![],
                        [
                            0u8, 224u8, 249u8, 242u8, 233u8, 176u8, 22u8, 146u8, 231u8, 123u8,
                            181u8, 166u8, 59u8, 33u8, 68u8, 34u8, 228u8, 155u8, 69u8, 177u8, 147u8,
                            70u8, 202u8, 200u8, 188u8, 230u8, 122u8, 91u8, 83u8, 232u8, 38u8, 68u8,
                        ],
                    )
                }
                #[doc = " Hash of the header used to bootstrap the pallet."]
                pub fn initial_hash(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "InitialHash",
                        vec![],
                        [
                            174u8, 39u8, 103u8, 193u8, 12u8, 0u8, 178u8, 128u8, 1u8, 176u8, 217u8,
                            69u8, 175u8, 181u8, 65u8, 105u8, 213u8, 19u8, 146u8, 159u8, 12u8,
                            228u8, 136u8, 242u8, 26u8, 186u8, 136u8, 2u8, 235u8, 162u8, 249u8,
                            247u8,
                        ],
                    )
                }
                #[doc = " Hash of the best finalized header."]
                pub fn best_finalized(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::HeaderId<
                            ::subxt::utils::H256,
                            ::core::primitive::u32,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "BestFinalized",
                        vec![],
                        [
                            117u8, 63u8, 104u8, 89u8, 13u8, 71u8, 15u8, 76u8, 96u8, 71u8, 250u8,
                            159u8, 18u8, 199u8, 158u8, 213u8, 55u8, 96u8, 121u8, 2u8, 220u8, 244u8,
                            242u8, 127u8, 183u8, 63u8, 95u8, 175u8, 21u8, 23u8, 101u8, 194u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported hashes. Ordered by the insertion time."]
                pub fn imported_hashes(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "ImportedHashes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            28u8, 27u8, 54u8, 5u8, 190u8, 52u8, 90u8, 217u8, 92u8, 228u8, 169u8,
                            132u8, 147u8, 0u8, 78u8, 146u8, 169u8, 234u8, 47u8, 93u8, 144u8, 2u8,
                            201u8, 223u8, 161u8, 175u8, 45u8, 150u8, 45u8, 237u8, 218u8, 58u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported hashes. Ordered by the insertion time."]
                pub fn imported_hashes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "ImportedHashes",
                        Vec::new(),
                        [
                            28u8, 27u8, 54u8, 5u8, 190u8, 52u8, 90u8, 217u8, 92u8, 228u8, 169u8,
                            132u8, 147u8, 0u8, 78u8, 146u8, 169u8, 234u8, 47u8, 93u8, 144u8, 2u8,
                            201u8, 223u8, 161u8, 175u8, 45u8, 150u8, 45u8, 237u8, 218u8, 58u8,
                        ],
                    )
                }
                #[doc = " Current ring buffer position."]
                pub fn imported_hashes_pointer(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "ImportedHashesPointer",
                        vec![],
                        [
                            159u8, 83u8, 35u8, 45u8, 27u8, 249u8, 155u8, 131u8, 181u8, 196u8,
                            224u8, 26u8, 92u8, 132u8, 127u8, 237u8, 13u8, 142u8, 196u8, 147u8,
                            221u8, 216u8, 11u8, 78u8, 190u8, 241u8, 201u8, 96u8, 74u8, 185u8,
                            208u8, 42u8,
                        ],
                    )
                }
                #[doc = " Relevant fields of imported headers."]
                pub fn imported_headers(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_header_chain::StoredHeaderData<
                            ::core::primitive::u32,
                            ::subxt::utils::H256,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "ImportedHeaders",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            14u8, 120u8, 104u8, 228u8, 94u8, 125u8, 44u8, 30u8, 1u8, 32u8, 28u8,
                            65u8, 74u8, 131u8, 170u8, 109u8, 126u8, 28u8, 74u8, 125u8, 37u8, 137u8,
                            213u8, 37u8, 139u8, 224u8, 42u8, 60u8, 113u8, 141u8, 103u8, 223u8,
                        ],
                    )
                }
                #[doc = " Relevant fields of imported headers."]
                pub fn imported_headers_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_header_chain::StoredHeaderData<
                            ::core::primitive::u32,
                            ::subxt::utils::H256,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "ImportedHeaders",
                        Vec::new(),
                        [
                            14u8, 120u8, 104u8, 228u8, 94u8, 125u8, 44u8, 30u8, 1u8, 32u8, 28u8,
                            65u8, 74u8, 131u8, 170u8, 109u8, 126u8, 28u8, 74u8, 125u8, 37u8, 137u8,
                            213u8, 37u8, 139u8, 224u8, 42u8, 60u8, 113u8, 141u8, 103u8, 223u8,
                        ],
                    )
                }
                #[doc = " The current GRANDPA Authority set."]
                pub fn current_authority_set(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_bridge_grandpa::storage_types::StoredAuthoritySet,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "CurrentAuthoritySet",
                        vec![],
                        [
                            249u8, 40u8, 229u8, 120u8, 141u8, 219u8, 206u8, 5u8, 54u8, 121u8,
                            207u8, 77u8, 8u8, 80u8, 105u8, 107u8, 151u8, 111u8, 82u8, 119u8, 8u8,
                            31u8, 104u8, 82u8, 92u8, 156u8, 37u8, 160u8, 235u8, 64u8, 62u8, 94u8,
                        ],
                    )
                }
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume it. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoGrandpa",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Maximal number of \"free\" mandatory header transactions per block."]
                #[doc = ""]
                #[doc = " To be able to track the bridged chain, the pallet requires all headers that are"]
                #[doc = " changing GRANDPA authorities set at the bridged chain (we call them mandatory)."]
                #[doc = " So it is a common good deed to submit mandatory headers to the pallet. However, if the"]
                #[doc = " bridged chain gets compromised, its validators may generate as many mandatory headers"]
                #[doc = " as they want. And they may fill the whole block (at this chain) for free. This constants"]
                #[doc = " limits number of calls that we may refund in a single block. All calls above this"]
                #[doc = " limit are accepted, but are not refunded."]
                pub fn max_free_mandatory_headers_per_block(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoGrandpa",
                        "MaxFreeMandatoryHeadersPerBlock",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " Maximal number of finalized headers to keep in the storage."]
                #[doc = ""]
                #[doc = " The setting is there to prevent growing the on-chain state indefinitely. Note"]
                #[doc = " the setting does not relate to block numbers - we will simply keep as much items"]
                #[doc = " in the storage, so it doesn't guarantee any fixed timeframe for finality headers."]
                #[doc = ""]
                #[doc = " Incautious change of this constant may lead to orphan entries in the runtime storage."]
                pub fn headers_to_keep(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoGrandpa",
                        "HeadersToKeep",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_rococo_parachain {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SubmitParachainHeads {
                pub at_relay_block: (::core::primitive::u32, ::subxt::utils::H256),
                pub parachains: ::std::vec::Vec<(
                    runtime_types::bp_polkadot_core::parachains::ParaId,
                    ::subxt::utils::H256,
                )>,
                pub parachain_heads_proof:
                    runtime_types::bp_polkadot_core::parachains::ParaHeadsProof,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::submit_parachain_heads`]."]
                pub fn submit_parachain_heads(
                    &self,
                    at_relay_block: (::core::primitive::u32, ::subxt::utils::H256),
                    parachains: ::std::vec::Vec<(
                        runtime_types::bp_polkadot_core::parachains::ParaId,
                        ::subxt::utils::H256,
                    )>,
                    parachain_heads_proof : runtime_types :: bp_polkadot_core :: parachains :: ParaHeadsProof,
                ) -> ::subxt::tx::StaticTxPayload<SubmitParachainHeads> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoParachain",
                        "submit_parachain_heads",
                        SubmitParachainHeads {
                            at_relay_block,
                            parachains,
                            parachain_heads_proof,
                        },
                        [
                            5u8, 103u8, 254u8, 130u8, 181u8, 160u8, 201u8, 97u8, 253u8, 95u8, 9u8,
                            109u8, 68u8, 128u8, 87u8, 86u8, 21u8, 40u8, 124u8, 125u8, 23u8, 182u8,
                            240u8, 190u8, 7u8, 99u8, 188u8, 57u8, 253u8, 204u8, 177u8, 109u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoParachain",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoParachain",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_parachains::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided head of parachain that the pallet is not configured to track."]
            pub struct UntrackedParachainRejected {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
            }
            impl ::subxt::events::StaticEvent for UntrackedParachainRejected {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "UntrackedParachainRejected";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has declared that he has provided given parachain head, but it is missing"]
            #[doc = "from the storage proof."]
            pub struct MissingParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
            }
            impl ::subxt::events::StaticEvent for MissingParachainHead {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "MissingParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided parachain head hash that is not matching the hash read from the"]
            #[doc = "storage proof."]
            pub struct IncorrectParachainHeadHash {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
                pub actual_parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for IncorrectParachainHeadHash {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "IncorrectParachainHeadHash";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided obsolete parachain head, which is already known to the pallet."]
            pub struct RejectedObsoleteParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for RejectedObsoleteParachainHead {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "RejectedObsoleteParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "The caller has provided parachain head that exceeds the maximal configured head size."]
            pub struct RejectedLargeParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
                pub parachain_head_size: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for RejectedLargeParachainHead {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "RejectedLargeParachainHead";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Parachain head has been updated."]
            pub struct UpdatedParachainHead {
                pub parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                pub parachain_head_hash: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for UpdatedParachainHead {
                const PALLET: &'static str = "BridgeRococoParachain";
                const EVENT: &'static str = "UpdatedParachainHead";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume them. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
                #[doc = " Parachains info."]
                #[doc = ""]
                #[doc = " Contains the following info:"]
                #[doc = " - best parachain head hash"]
                #[doc = " - the head of the `ImportedParaHashes` ring buffer"]
                pub fn paras_info(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::bp_parachains::ParaInfo>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ParasInfo",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            22u8, 107u8, 60u8, 52u8, 124u8, 172u8, 17u8, 70u8, 73u8, 219u8, 67u8,
                            240u8, 79u8, 198u8, 186u8, 81u8, 128u8, 101u8, 226u8, 58u8, 160u8,
                            11u8, 162u8, 246u8, 85u8, 150u8, 137u8, 63u8, 3u8, 222u8, 87u8, 45u8,
                        ],
                    )
                }
                #[doc = " Parachains info."]
                #[doc = ""]
                #[doc = " Contains the following info:"]
                #[doc = " - best parachain head hash"]
                #[doc = " - the head of the `ImportedParaHashes` ring buffer"]
                pub fn paras_info_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<runtime_types::bp_parachains::ParaInfo>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ParasInfo",
                        Vec::new(),
                        [
                            22u8, 107u8, 60u8, 52u8, 124u8, 172u8, 17u8, 70u8, 73u8, 219u8, 67u8,
                            240u8, 79u8, 198u8, 186u8, 81u8, 128u8, 101u8, 226u8, 58u8, 160u8,
                            11u8, 162u8, 246u8, 85u8, 150u8, 137u8, 63u8, 3u8, 222u8, 87u8, 45u8,
                        ],
                    )
                }
                #[doc = " State roots of parachain heads which have been imported into the pallet."]
                pub fn imported_para_heads(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                    _1: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_parachains::ParaStoredHeaderData,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ImportedParaHeads",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                        ],
                        [
                            134u8, 183u8, 37u8, 49u8, 91u8, 83u8, 197u8, 0u8, 110u8, 79u8, 143u8,
                            114u8, 36u8, 164u8, 254u8, 173u8, 191u8, 69u8, 137u8, 178u8, 56u8,
                            192u8, 151u8, 201u8, 39u8, 251u8, 150u8, 144u8, 37u8, 69u8, 246u8,
                            251u8,
                        ],
                    )
                }
                #[doc = " State roots of parachain heads which have been imported into the pallet."]
                pub fn imported_para_heads_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_parachains::ParaStoredHeaderData,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ImportedParaHeads",
                        Vec::new(),
                        [
                            134u8, 183u8, 37u8, 49u8, 91u8, 83u8, 197u8, 0u8, 110u8, 79u8, 143u8,
                            114u8, 36u8, 164u8, 254u8, 173u8, 191u8, 69u8, 137u8, 178u8, 56u8,
                            192u8, 151u8, 201u8, 39u8, 251u8, 150u8, 144u8, 37u8, 69u8, 246u8,
                            251u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported parachain head hashes. Ordered by the insertion time."]
                pub fn imported_para_hashes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_polkadot_core::parachains::ParaId>,
                    _1: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ImportedParaHashes",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                        ],
                        [
                            15u8, 216u8, 100u8, 17u8, 75u8, 61u8, 65u8, 33u8, 212u8, 227u8, 102u8,
                            48u8, 108u8, 160u8, 103u8, 60u8, 136u8, 189u8, 147u8, 103u8, 97u8,
                            181u8, 207u8, 75u8, 254u8, 157u8, 113u8, 99u8, 209u8, 103u8, 143u8,
                            110u8,
                        ],
                    )
                }
                #[doc = " A ring buffer of imported parachain head hashes. Ordered by the insertion time."]
                pub fn imported_para_hashes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoParachain",
                        "ImportedParaHashes",
                        Vec::new(),
                        [
                            15u8, 216u8, 100u8, 17u8, 75u8, 61u8, 65u8, 33u8, 212u8, 227u8, 102u8,
                            48u8, 108u8, 160u8, 103u8, 60u8, 136u8, 189u8, 147u8, 103u8, 97u8,
                            181u8, 207u8, 75u8, 254u8, 157u8, 113u8, 99u8, 209u8, 103u8, 143u8,
                            110u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Name of the original `paras` pallet in the `construct_runtime!()` call at the bridged"]
                #[doc = " chain."]
                #[doc = ""]
                #[doc = " Please keep in mind that this should be the name of the `runtime_parachains::paras`"]
                #[doc = " pallet from polkadot repository, not the `pallet-bridge-parachains`."]
                pub fn paras_pallet_name(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::std::string::String>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoParachain",
                        "ParasPalletName",
                        [
                            251u8, 233u8, 211u8, 209u8, 5u8, 66u8, 94u8, 200u8, 148u8, 166u8,
                            119u8, 200u8, 59u8, 180u8, 70u8, 77u8, 182u8, 127u8, 45u8, 65u8, 28u8,
                            104u8, 253u8, 149u8, 167u8, 216u8, 2u8, 94u8, 39u8, 173u8, 198u8,
                            219u8,
                        ],
                    )
                }
                #[doc = " Maximal number of single parachain heads to keep in the storage."]
                #[doc = ""]
                #[doc = " The setting is there to prevent growing the on-chain state indefinitely. Note"]
                #[doc = " the setting does not relate to parachain block numbers - we will simply keep as much"]
                #[doc = " items in the storage, so it doesn't guarantee any fixed timeframe for heads."]
                #[doc = ""]
                #[doc = " Incautious change of this constant may lead to orphan entries in the runtime storage."]
                pub fn heads_to_keep(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoParachain",
                        "HeadsToKeep",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " Maximal size (in bytes) of the SCALE-encoded parachain head data"]
                #[doc = " (`bp_parachains::ParaStoredHeaderData`)."]
                #[doc = ""]
                #[doc = " Keep in mind that the size of any tracked parachain header data must not exceed this"]
                #[doc = " value. So if you're going to track multiple parachains, one of which is using large"]
                #[doc = " hashes, you shall choose this maximal value."]
                #[doc = ""]
                #[doc = " There's no mandatory headers in this pallet, so it can't stall if there's some header"]
                #[doc = " that exceeds this bound."]
                pub fn max_para_head_data_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoParachain",
                        "MaxParaHeadDataSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_rococo_messages {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_messages::MessagesOperatingMode,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReceiveMessagesProof { pub relayer_id_at_bridged_chain : :: subxt :: utils :: AccountId32 , pub proof : runtime_types :: bridge_runtime_common :: messages :: target :: FromBridgedChainMessagesProof < :: subxt :: utils :: H256 > , pub messages_count : :: core :: primitive :: u32 , pub dispatch_weight : runtime_types :: sp_weights :: weight_v2 :: Weight , }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReceiveMessagesDeliveryProof { pub proof : runtime_types :: bridge_runtime_common :: messages :: source :: FromBridgedChainMessagesDeliveryProof < :: subxt :: utils :: H256 > , pub relayers_state : runtime_types :: bp_messages :: UnrewardedRelayersState , }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoMessages",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_messages::MessagesOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoMessages",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            236u8, 230u8, 127u8, 17u8, 145u8, 186u8, 102u8, 200u8, 227u8, 208u8,
                            230u8, 121u8, 102u8, 199u8, 123u8, 118u8, 199u8, 160u8, 131u8, 116u8,
                            102u8, 167u8, 119u8, 144u8, 70u8, 114u8, 0u8, 223u8, 54u8, 197u8, 39u8,
                            58u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::receive_messages_proof`]."]
                pub fn receive_messages_proof(
                    &self,
                    relayer_id_at_bridged_chain: ::subxt::utils::AccountId32,
                    proof : runtime_types :: bridge_runtime_common :: messages :: target :: FromBridgedChainMessagesProof < :: subxt :: utils :: H256 >,
                    messages_count: ::core::primitive::u32,
                    dispatch_weight: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ReceiveMessagesProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoMessages",
                        "receive_messages_proof",
                        ReceiveMessagesProof {
                            relayer_id_at_bridged_chain,
                            proof,
                            messages_count,
                            dispatch_weight,
                        },
                        [
                            74u8, 80u8, 32u8, 12u8, 106u8, 198u8, 121u8, 131u8, 209u8, 175u8, 93u8,
                            240u8, 144u8, 63u8, 20u8, 127u8, 20u8, 227u8, 166u8, 29u8, 155u8,
                            117u8, 101u8, 109u8, 8u8, 37u8, 192u8, 144u8, 165u8, 14u8, 170u8, 65u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::receive_messages_delivery_proof`]."]
                pub fn receive_messages_delivery_proof(
                    &self,
                    proof : runtime_types :: bridge_runtime_common :: messages :: source :: FromBridgedChainMessagesDeliveryProof < :: subxt :: utils :: H256 >,
                    relayers_state: runtime_types::bp_messages::UnrewardedRelayersState,
                ) -> ::subxt::tx::StaticTxPayload<ReceiveMessagesDeliveryProof> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRococoMessages",
                        "receive_messages_delivery_proof",
                        ReceiveMessagesDeliveryProof {
                            proof,
                            relayers_state,
                        },
                        [
                            3u8, 106u8, 216u8, 97u8, 151u8, 155u8, 45u8, 209u8, 104u8, 156u8, 16u8,
                            114u8, 213u8, 44u8, 77u8, 226u8, 68u8, 66u8, 178u8, 137u8, 131u8, 8u8,
                            153u8, 209u8, 146u8, 32u8, 255u8, 215u8, 130u8, 104u8, 122u8, 95u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_messages::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message has been accepted and is waiting to be delivered."]
            pub struct MessageAccepted {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub nonce: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for MessageAccepted {
                const PALLET: &'static str = "BridgeRococoMessages";
                const EVENT: &'static str = "MessageAccepted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Messages have been received from the bridged chain."]
            pub struct MessagesReceived (pub :: std :: vec :: Vec < runtime_types :: bp_messages :: ReceivedMessages < runtime_types :: bridge_runtime_common :: messages_xcm_extension :: XcmBlobMessageDispatchResult > > ,) ;
            impl ::subxt::events::StaticEvent for MessagesReceived {
                const PALLET: &'static str = "BridgeRococoMessages";
                const EVENT: &'static str = "MessagesReceived";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Messages in the inclusive range have been delivered to the bridged chain."]
            pub struct MessagesDelivered {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub messages: runtime_types::bp_messages::DeliveredMessages,
            }
            impl ::subxt::events::StaticEvent for MessagesDelivered {
                const PALLET: &'static str = "BridgeRococoMessages";
                const EVENT: &'static str = "MessagesDelivered";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume it. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, some, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::MessagesOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "PalletOperatingMode",
                        vec![],
                        [
                            215u8, 195u8, 85u8, 231u8, 158u8, 22u8, 160u8, 132u8, 69u8, 206u8,
                            238u8, 14u8, 56u8, 100u8, 134u8, 41u8, 58u8, 120u8, 225u8, 164u8,
                            173u8, 87u8, 22u8, 123u8, 102u8, 167u8, 68u8, 70u8, 184u8, 131u8,
                            232u8, 65u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => inbound lane data."]
                pub fn inbound_lanes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::LaneId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::InboundLaneData<::subxt::utils::AccountId32>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "InboundLanes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            38u8, 58u8, 110u8, 130u8, 112u8, 76u8, 231u8, 76u8, 56u8, 241u8, 183u8,
                            153u8, 112u8, 41u8, 248u8, 208u8, 217u8, 57u8, 102u8, 30u8, 107u8,
                            98u8, 59u8, 78u8, 56u8, 119u8, 186u8, 183u8, 213u8, 72u8, 199u8, 90u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => inbound lane data."]
                pub fn inbound_lanes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::InboundLaneData<::subxt::utils::AccountId32>,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "InboundLanes",
                        Vec::new(),
                        [
                            38u8, 58u8, 110u8, 130u8, 112u8, 76u8, 231u8, 76u8, 56u8, 241u8, 183u8,
                            153u8, 112u8, 41u8, 248u8, 208u8, 217u8, 57u8, 102u8, 30u8, 107u8,
                            98u8, 59u8, 78u8, 56u8, 119u8, 186u8, 183u8, 213u8, 72u8, 199u8, 90u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => outbound lane data."]
                pub fn outbound_lanes(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::LaneId>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::OutboundLaneData,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "OutboundLanes",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            67u8, 155u8, 173u8, 244u8, 80u8, 38u8, 26u8, 71u8, 51u8, 150u8, 86u8,
                            146u8, 132u8, 122u8, 70u8, 122u8, 172u8, 246u8, 106u8, 232u8, 149u8,
                            227u8, 240u8, 146u8, 51u8, 184u8, 30u8, 182u8, 200u8, 43u8, 190u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " Map of lane id => outbound lane data."]
                pub fn outbound_lanes_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_messages::OutboundLaneData,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "OutboundLanes",
                        Vec::new(),
                        [
                            67u8, 155u8, 173u8, 244u8, 80u8, 38u8, 26u8, 71u8, 51u8, 150u8, 86u8,
                            146u8, 132u8, 122u8, 70u8, 122u8, 172u8, 246u8, 106u8, 232u8, 149u8,
                            227u8, 240u8, 146u8, 51u8, 184u8, 30u8, 182u8, 200u8, 43u8, 190u8,
                            38u8,
                        ],
                    )
                }
                #[doc = " All queued outbound messages."]
                pub fn outbound_messages(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::bp_messages::MessageKey>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "OutboundMessages",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            44u8, 35u8, 2u8, 25u8, 91u8, 101u8, 152u8, 23u8, 48u8, 250u8, 178u8,
                            15u8, 194u8, 118u8, 146u8, 1u8, 112u8, 83u8, 243u8, 166u8, 124u8,
                            153u8, 48u8, 193u8, 43u8, 31u8, 33u8, 72u8, 228u8, 113u8, 86u8, 217u8,
                        ],
                    )
                }
                #[doc = " All queued outbound messages."]
                pub fn outbound_messages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRococoMessages",
                        "OutboundMessages",
                        Vec::new(),
                        [
                            44u8, 35u8, 2u8, 25u8, 91u8, 101u8, 152u8, 23u8, 48u8, 250u8, 178u8,
                            15u8, 194u8, 118u8, 146u8, 1u8, 112u8, 83u8, 243u8, 166u8, 124u8,
                            153u8, 48u8, 193u8, 43u8, 31u8, 33u8, 72u8, 228u8, 113u8, 86u8, 217u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Gets the chain id value from the instance."]
                pub fn bridged_chain_id(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<[::core::primitive::u8; 4usize]>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoMessages",
                        "BridgedChainId",
                        [
                            101u8, 157u8, 37u8, 163u8, 190u8, 134u8, 129u8, 212u8, 240u8, 135u8,
                            174u8, 76u8, 220u8, 179u8, 252u8, 69u8, 65u8, 253u8, 69u8, 214u8, 61u8,
                            249u8, 4u8, 38u8, 181u8, 237u8, 25u8, 131u8, 242u8, 20u8, 17u8, 152u8,
                        ],
                    )
                }
                #[doc = " Maximal encoded size of the outbound payload."]
                pub fn maximal_outbound_payload_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "BridgeRococoMessages",
                        "MaximalOutboundPayloadSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod bridge_relayers {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ClaimRewards {
                pub rewards_account_params: runtime_types::bp_relayers::RewardsAccountParams,
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct Register {
                pub valid_till: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Deregister;
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::claim_rewards`]."]
                pub fn claim_rewards(
                    &self,
                    rewards_account_params: runtime_types::bp_relayers::RewardsAccountParams,
                ) -> ::subxt::tx::StaticTxPayload<ClaimRewards> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRelayers",
                        "claim_rewards",
                        ClaimRewards {
                            rewards_account_params,
                        },
                        [
                            141u8, 52u8, 193u8, 42u8, 145u8, 26u8, 147u8, 35u8, 227u8, 221u8,
                            221u8, 188u8, 104u8, 186u8, 123u8, 46u8, 190u8, 236u8, 120u8, 19u8,
                            230u8, 219u8, 238u8, 227u8, 75u8, 35u8, 36u8, 177u8, 227u8, 130u8,
                            103u8, 128u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::register`]."]
                pub fn register(
                    &self,
                    valid_till: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<Register> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRelayers",
                        "register",
                        Register { valid_till },
                        [
                            117u8, 80u8, 230u8, 23u8, 230u8, 239u8, 233u8, 149u8, 175u8, 128u8,
                            49u8, 238u8, 9u8, 182u8, 70u8, 91u8, 59u8, 61u8, 247u8, 167u8, 118u8,
                            120u8, 118u8, 138u8, 39u8, 208u8, 116u8, 33u8, 153u8, 250u8, 230u8,
                            71u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::deregister`]."]
                pub fn deregister(&self) -> ::subxt::tx::StaticTxPayload<Deregister> {
                    ::subxt::tx::StaticTxPayload::new(
                        "BridgeRelayers",
                        "deregister",
                        Deregister {},
                        [
                            198u8, 180u8, 253u8, 148u8, 124u8, 145u8, 175u8, 121u8, 42u8, 181u8,
                            41u8, 155u8, 229u8, 181u8, 66u8, 140u8, 103u8, 86u8, 242u8, 155u8,
                            192u8, 34u8, 157u8, 107u8, 211u8, 162u8, 1u8, 144u8, 35u8, 252u8, 88u8,
                            21u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_bridge_relayers::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Reward has been paid to the relayer."]
            pub struct RewardPaid {
                pub relayer: ::subxt::utils::AccountId32,
                pub rewards_account_params: runtime_types::bp_relayers::RewardsAccountParams,
                pub reward: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for RewardPaid {
                const PALLET: &'static str = "BridgeRelayers";
                const EVENT: &'static str = "RewardPaid";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Relayer registration has been added or updated."]
            pub struct RegistrationUpdated {
                pub relayer: ::subxt::utils::AccountId32,
                pub registration: runtime_types::bp_relayers::registration::Registration<
                    ::core::primitive::u32,
                    ::core::primitive::u128,
                >,
            }
            impl ::subxt::events::StaticEvent for RegistrationUpdated {
                const PALLET: &'static str = "BridgeRelayers";
                const EVENT: &'static str = "RegistrationUpdated";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Relayer has been `deregistered`."]
            pub struct Deregistered {
                pub relayer: ::subxt::utils::AccountId32,
            }
            impl ::subxt::events::StaticEvent for Deregistered {
                const PALLET: &'static str = "BridgeRelayers";
                const EVENT: &'static str = "Deregistered";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Relayer has been slashed and `deregistered`."]
            pub struct SlashedAndDeregistered {
                pub relayer: ::subxt::utils::AccountId32,
                pub registration: runtime_types::bp_relayers::registration::Registration<
                    ::core::primitive::u32,
                    ::core::primitive::u128,
                >,
            }
            impl ::subxt::events::StaticEvent for SlashedAndDeregistered {
                const PALLET: &'static str = "BridgeRelayers";
                const EVENT: &'static str = "SlashedAndDeregistered";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Map of the relayer => accumulated reward."]
                pub fn relayer_rewards(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                    _1: impl ::std::borrow::Borrow<runtime_types::bp_relayers::RewardsAccountParams>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRelayers",
                        "RelayerRewards",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Identity,
                            ),
                        ],
                        [
                            116u8, 81u8, 48u8, 55u8, 199u8, 26u8, 100u8, 7u8, 177u8, 230u8, 132u8,
                            248u8, 221u8, 90u8, 33u8, 155u8, 198u8, 216u8, 43u8, 149u8, 92u8,
                            100u8, 199u8, 183u8, 150u8, 214u8, 199u8, 222u8, 224u8, 228u8, 238u8,
                            108u8,
                        ],
                    )
                }
                #[doc = " Map of the relayer => accumulated reward."]
                pub fn relayer_rewards_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u128>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRelayers",
                        "RelayerRewards",
                        Vec::new(),
                        [
                            116u8, 81u8, 48u8, 55u8, 199u8, 26u8, 100u8, 7u8, 177u8, 230u8, 132u8,
                            248u8, 221u8, 90u8, 33u8, 155u8, 198u8, 216u8, 43u8, 149u8, 92u8,
                            100u8, 199u8, 183u8, 150u8, 214u8, 199u8, 222u8, 224u8, 228u8, 238u8,
                            108u8,
                        ],
                    )
                }
                #[doc = " Relayers that have reserved some of their balance to get free priority boost"]
                #[doc = " for their message delivery transactions."]
                #[doc = ""]
                #[doc = " Other relayers may submit transactions as well, but they will have default"]
                #[doc = " priority and will be rejected (without significant tip) in case if registered"]
                #[doc = " relayer is present."]
                pub fn registered_relayers(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::AccountId32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_relayers::registration::Registration<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRelayers",
                        "RegisteredRelayers",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Blake2_128Concat,
                        )],
                        [
                            183u8, 171u8, 239u8, 176u8, 162u8, 221u8, 154u8, 32u8, 36u8, 99u8,
                            149u8, 20u8, 237u8, 214u8, 16u8, 83u8, 0u8, 52u8, 209u8, 89u8, 9u8,
                            110u8, 206u8, 152u8, 224u8, 251u8, 114u8, 250u8, 170u8, 84u8, 3u8,
                            221u8,
                        ],
                    )
                }
                #[doc = " Relayers that have reserved some of their balance to get free priority boost"]
                #[doc = " for their message delivery transactions."]
                #[doc = ""]
                #[doc = " Other relayers may submit transactions as well, but they will have default"]
                #[doc = " priority and will be rejected (without significant tip) in case if registered"]
                #[doc = " relayer is present."]
                pub fn registered_relayers_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_relayers::registration::Registration<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                        >,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "BridgeRelayers",
                        "RegisteredRelayers",
                        Vec::new(),
                        [
                            183u8, 171u8, 239u8, 176u8, 162u8, 221u8, 154u8, 32u8, 36u8, 99u8,
                            149u8, 20u8, 237u8, 214u8, 16u8, 83u8, 0u8, 52u8, 209u8, 89u8, 9u8,
                            110u8, 206u8, 152u8, 224u8, 251u8, 114u8, 250u8, 170u8, 84u8, 3u8,
                            221u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod ethereum_inbound_queue {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Submit {
                pub message: runtime_types::snowbridge_core::inbound::Message,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetGateway {
                pub gateway: ::subxt::utils::H160,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForwardXcm {
                pub dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                pub message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::submit`]."]
                pub fn submit(
                    &self,
                    message: runtime_types::snowbridge_core::inbound::Message,
                ) -> ::subxt::tx::StaticTxPayload<Submit> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumInboundQueue",
                        "submit",
                        Submit { message },
                        [
                            67u8, 201u8, 190u8, 156u8, 26u8, 251u8, 37u8, 190u8, 47u8, 143u8,
                            130u8, 113u8, 10u8, 60u8, 158u8, 19u8, 10u8, 112u8, 70u8, 49u8, 85u8,
                            67u8, 83u8, 50u8, 10u8, 181u8, 4u8, 186u8, 195u8, 91u8, 23u8, 224u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_gateway`]."]
                pub fn set_gateway(
                    &self,
                    gateway: ::subxt::utils::H160,
                ) -> ::subxt::tx::StaticTxPayload<SetGateway> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumInboundQueue",
                        "set_gateway",
                        SetGateway { gateway },
                        [
                            65u8, 21u8, 25u8, 15u8, 201u8, 201u8, 137u8, 134u8, 214u8, 212u8, 42u8,
                            39u8, 189u8, 228u8, 123u8, 210u8, 60u8, 156u8, 156u8, 134u8, 74u8,
                            60u8, 91u8, 173u8, 121u8, 196u8, 212u8, 2u8, 164u8, 149u8, 214u8,
                            254u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumInboundQueue",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumInboundQueue",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::forward_xcm`]."]
                pub fn forward_xcm(
                    &self,
                    dest: runtime_types::xcm::VersionedMultiLocation,
                    message: runtime_types::xcm::VersionedXcm,
                ) -> ::subxt::tx::StaticTxPayload<ForwardXcm> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumInboundQueue",
                        "forward_xcm",
                        ForwardXcm {
                            dest: ::std::boxed::Box::new(dest),
                            message: ::std::boxed::Box::new(message),
                        },
                        [
                            85u8, 211u8, 73u8, 24u8, 134u8, 102u8, 37u8, 32u8, 133u8, 152u8, 28u8,
                            81u8, 143u8, 17u8, 21u8, 168u8, 72u8, 122u8, 68u8, 123u8, 4u8, 140u8,
                            125u8, 121u8, 176u8, 170u8, 26u8, 26u8, 198u8, 50u8, 36u8, 183u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::snowbridge_inbound_queue::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "A message was received from Ethereum"]
            pub struct MessageReceived {
                pub dest: runtime_types::polkadot_parachain::primitives::Id,
                pub nonce: ::core::primitive::u64,
                pub result: runtime_types::snowbridge_inbound_queue::MessageDispatchResult,
            }
            impl ::subxt::events::StaticEvent for MessageReceived {
                const PALLET: &'static str = "EthereumInboundQueue";
                const EVENT: &'static str = "MessageReceived";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The address of the Gateway contract on Ethereum"]
                pub fn gateway(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H160>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumInboundQueue",
                        "Gateway",
                        vec![],
                        [
                            1u8, 179u8, 148u8, 133u8, 91u8, 161u8, 242u8, 12u8, 107u8, 165u8,
                            147u8, 200u8, 215u8, 35u8, 160u8, 88u8, 82u8, 247u8, 194u8, 95u8,
                            206u8, 80u8, 78u8, 169u8, 35u8, 21u8, 97u8, 180u8, 209u8, 204u8, 210u8,
                            141u8,
                        ],
                    )
                }
                #[doc = " The current nonce for each parachain"]
                pub fn nonce(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumInboundQueue",
                        "Nonce",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            96u8, 225u8, 145u8, 131u8, 202u8, 135u8, 63u8, 100u8, 96u8, 33u8,
                            183u8, 172u8, 150u8, 92u8, 144u8, 116u8, 18u8, 245u8, 97u8, 235u8,
                            77u8, 43u8, 49u8, 248u8, 45u8, 42u8, 14u8, 178u8, 11u8, 152u8, 239u8,
                            51u8,
                        ],
                    )
                }
                #[doc = " The current nonce for each parachain"]
                pub fn nonce_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumInboundQueue",
                        "Nonce",
                        Vec::new(),
                        [
                            96u8, 225u8, 145u8, 131u8, 202u8, 135u8, 63u8, 100u8, 96u8, 33u8,
                            183u8, 172u8, 150u8, 92u8, 144u8, 116u8, 18u8, 245u8, 97u8, 235u8,
                            77u8, 43u8, 49u8, 248u8, 45u8, 42u8, 14u8, 178u8, 11u8, 152u8, 239u8,
                            51u8,
                        ],
                    )
                }
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume them. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumInboundQueue",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumInboundQueue",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod ethereum_outbound_queue {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumOutboundQueue",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumOutboundQueue",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::snowbridge_outbound_queue::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message has been queued and will be processed in the future"]
            pub struct MessageQueued {
                pub id: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for MessageQueued {
                const PALLET: &'static str = "EthereumOutboundQueue";
                const EVENT: &'static str = "MessageQueued";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message will be committed at the end of current block. From now on, to track the"]
            #[doc = "progress the message, use the `nonce` of `id`."]
            pub struct MessageAccepted {
                pub id: ::subxt::utils::H256,
                pub nonce: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for MessageAccepted {
                const PALLET: &'static str = "EthereumOutboundQueue";
                const EVENT: &'static str = "MessageAccepted";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Some messages have been committed"]
            pub struct MessagesCommitted {
                pub root: ::subxt::utils::H256,
                pub count: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for MessagesCommitted {
                const PALLET: &'static str = "EthereumOutboundQueue";
                const EVENT: &'static str = "MessagesCommitted";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Messages to be committed in the current block. This storage value is killed in"]
                #[doc = " `on_initialize`, so should never go into block PoV."]
                #[doc = ""]
                #[doc = " Is never read in the runtime, only by offchain message relayers."]
                #[doc = ""]
                #[doc = " Inspired by the `frame_system::Pallet::Events` storage value"]
                pub fn messages(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::std::vec::Vec<runtime_types::snowbridge_outbound_queue::PreparedMessage>,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "Messages",
                        vec![],
                        [
                            193u8, 192u8, 60u8, 233u8, 130u8, 179u8, 197u8, 188u8, 189u8, 20u8,
                            94u8, 62u8, 30u8, 134u8, 104u8, 112u8, 77u8, 45u8, 48u8, 112u8, 74u8,
                            152u8, 71u8, 186u8, 129u8, 79u8, 255u8, 191u8, 82u8, 112u8, 94u8,
                            115u8,
                        ],
                    )
                }
                #[doc = " Hashes of the ABI-encoded messages in the [`Messages`] storage value. Used to generate a"]
                #[doc = " merkle root during `on_finalize`. This storage value is killed in"]
                #[doc = " `on_initialize`, so should never go into block PoV."]
                pub fn message_leaves(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::std::vec::Vec<::subxt::utils::H256>>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "MessageLeaves",
                        vec![],
                        [
                            151u8, 221u8, 233u8, 108u8, 124u8, 124u8, 112u8, 23u8, 25u8, 96u8,
                            44u8, 241u8, 171u8, 228u8, 163u8, 170u8, 236u8, 160u8, 70u8, 197u8,
                            121u8, 244u8, 243u8, 66u8, 45u8, 202u8, 9u8, 234u8, 124u8, 221u8,
                            190u8, 200u8,
                        ],
                    )
                }
                #[doc = " The current nonce for each message origin"]
                pub fn nonce(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "Nonce",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            96u8, 225u8, 145u8, 131u8, 202u8, 135u8, 63u8, 100u8, 96u8, 33u8,
                            183u8, 172u8, 150u8, 92u8, 144u8, 116u8, 18u8, 245u8, 97u8, 235u8,
                            77u8, 43u8, 49u8, 248u8, 45u8, 42u8, 14u8, 178u8, 11u8, 152u8, 239u8,
                            51u8,
                        ],
                    )
                }
                #[doc = " The current nonce for each message origin"]
                pub fn nonce_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u64>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "Nonce",
                        Vec::new(),
                        [
                            96u8, 225u8, 145u8, 131u8, 202u8, 135u8, 63u8, 100u8, 96u8, 33u8,
                            183u8, 172u8, 150u8, 92u8, 144u8, 116u8, 18u8, 245u8, 97u8, 235u8,
                            77u8, 43u8, 49u8, 248u8, 45u8, 42u8, 14u8, 178u8, 11u8, 152u8, 239u8,
                            51u8,
                        ],
                    )
                }
                #[doc = " Optional pallet owner."]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume them. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumOutboundQueue",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " Max bytes in a message payload"]
                pub fn max_message_payload_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "EthereumOutboundQueue",
                        "MaxMessagePayloadSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " Max number of messages processed per block"]
                pub fn max_messages_per_block(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "EthereumOutboundQueue",
                        "MaxMessagesPerBlock",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod ethereum_beacon_client {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ForceCheckpoint {
                pub update: ::std::boxed::Box<
                    runtime_types::snowbridge_beacon_primitives::updates::CheckpointUpdate,
                >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Submit {
                pub update:
                    ::std::boxed::Box<runtime_types::snowbridge_beacon_primitives::updates::Update>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SubmitExecutionHeader {
                pub update: ::std::boxed::Box<
                    runtime_types::snowbridge_beacon_primitives::updates::ExecutionHeaderUpdate,
                >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOwner {
                pub new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::force_checkpoint`]."]
                pub fn force_checkpoint(
                    &self,
                    update: runtime_types::snowbridge_beacon_primitives::updates::CheckpointUpdate,
                ) -> ::subxt::tx::StaticTxPayload<ForceCheckpoint> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumBeaconClient",
                        "force_checkpoint",
                        ForceCheckpoint {
                            update: ::std::boxed::Box::new(update),
                        },
                        [
                            69u8, 230u8, 61u8, 79u8, 243u8, 3u8, 91u8, 65u8, 129u8, 148u8, 102u8,
                            145u8, 6u8, 44u8, 128u8, 46u8, 4u8, 136u8, 134u8, 214u8, 172u8, 138u8,
                            223u8, 245u8, 57u8, 207u8, 109u8, 145u8, 149u8, 113u8, 177u8, 247u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::submit`]."]
                pub fn submit(
                    &self,
                    update: runtime_types::snowbridge_beacon_primitives::updates::Update,
                ) -> ::subxt::tx::StaticTxPayload<Submit> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumBeaconClient",
                        "submit",
                        Submit {
                            update: ::std::boxed::Box::new(update),
                        },
                        [
                            78u8, 161u8, 115u8, 201u8, 133u8, 136u8, 115u8, 34u8, 55u8, 38u8,
                            109u8, 172u8, 124u8, 179u8, 198u8, 60u8, 211u8, 53u8, 49u8, 207u8,
                            238u8, 54u8, 121u8, 30u8, 232u8, 165u8, 108u8, 217u8, 15u8, 85u8, 82u8,
                            157u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::submit_execution_header`]."]
                pub fn submit_execution_header(
                    &self,
                    update : runtime_types :: snowbridge_beacon_primitives :: updates :: ExecutionHeaderUpdate,
                ) -> ::subxt::tx::StaticTxPayload<SubmitExecutionHeader> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumBeaconClient",
                        "submit_execution_header",
                        SubmitExecutionHeader {
                            update: ::std::boxed::Box::new(update),
                        },
                        [
                            38u8, 148u8, 207u8, 116u8, 149u8, 25u8, 88u8, 171u8, 185u8, 136u8,
                            250u8, 113u8, 245u8, 211u8, 31u8, 20u8, 138u8, 210u8, 58u8, 204u8, 7u8,
                            211u8, 212u8, 44u8, 237u8, 77u8, 1u8, 170u8, 210u8, 247u8, 216u8, 32u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_owner`]."]
                pub fn set_owner(
                    &self,
                    new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                ) -> ::subxt::tx::StaticTxPayload<SetOwner> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumBeaconClient",
                        "set_owner",
                        SetOwner { new_owner },
                        [
                            100u8, 221u8, 84u8, 142u8, 158u8, 5u8, 47u8, 212u8, 9u8, 35u8, 82u8,
                            135u8, 108u8, 238u8, 231u8, 197u8, 77u8, 219u8, 176u8, 222u8, 88u8,
                            167u8, 152u8, 34u8, 177u8, 244u8, 160u8, 195u8, 211u8, 3u8, 66u8,
                            253u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumBeaconClient",
                        "set_operating_mode",
                        SetOperatingMode { operating_mode },
                        [
                            128u8, 25u8, 81u8, 145u8, 111u8, 185u8, 226u8, 152u8, 18u8, 51u8, 89u8,
                            236u8, 200u8, 157u8, 157u8, 186u8, 207u8, 208u8, 152u8, 168u8, 12u8,
                            39u8, 249u8, 48u8, 195u8, 160u8, 54u8, 73u8, 30u8, 230u8, 25u8, 46u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::snowbridge_ethereum_beacon_client::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct BeaconHeaderImported {
                pub block_hash: ::subxt::utils::H256,
                pub slot: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for BeaconHeaderImported {
                const PALLET: &'static str = "EthereumBeaconClient";
                const EVENT: &'static str = "BeaconHeaderImported";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ExecutionHeaderImported {
                pub block_hash: ::subxt::utils::H256,
                pub block_number: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for ExecutionHeaderImported {
                const PALLET: &'static str = "EthereumBeaconClient";
                const EVENT: &'static str = "ExecutionHeaderImported";
            }
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct SyncCommitteeUpdated {
                pub period: ::core::primitive::u64,
            }
            impl ::subxt::events::StaticEvent for SyncCommitteeUpdated {
                const PALLET: &'static str = "EthereumBeaconClient";
                const EVENT: &'static str = "SyncCommitteeUpdated";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " Latest imported checkpoint root"]
                pub fn initial_checkpoint_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "InitialCheckpointRoot",
                        vec![],
                        [
                            10u8, 202u8, 51u8, 118u8, 72u8, 160u8, 155u8, 129u8, 106u8, 240u8,
                            50u8, 7u8, 83u8, 94u8, 237u8, 157u8, 253u8, 49u8, 216u8, 177u8, 53u8,
                            210u8, 29u8, 148u8, 169u8, 13u8, 96u8, 238u8, 207u8, 255u8, 212u8,
                            100u8,
                        ],
                    )
                }
                #[doc = " Latest imported finalized block root"]
                pub fn latest_finalized_block_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "LatestFinalizedBlockRoot",
                        vec![],
                        [
                            200u8, 47u8, 225u8, 188u8, 164u8, 22u8, 83u8, 74u8, 157u8, 16u8, 38u8,
                            57u8, 27u8, 203u8, 81u8, 60u8, 11u8, 64u8, 33u8, 15u8, 189u8, 5u8, 4u8,
                            43u8, 239u8, 1u8, 107u8, 156u8, 221u8, 189u8, 248u8, 80u8,
                        ],
                    )
                }
                #[doc = " Beacon state by finalized block root"]
                pub fn finalized_beacon_state(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::CompactBeaconState,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "FinalizedBeaconState",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            77u8, 51u8, 105u8, 202u8, 127u8, 135u8, 21u8, 199u8, 242u8, 77u8, 57u8,
                            104u8, 168u8, 82u8, 1u8, 159u8, 158u8, 84u8, 107u8, 21u8, 34u8, 90u8,
                            225u8, 44u8, 86u8, 63u8, 215u8, 114u8, 254u8, 139u8, 221u8, 68u8,
                        ],
                    )
                }
                #[doc = " Beacon state by finalized block root"]
                pub fn finalized_beacon_state_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::CompactBeaconState,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "FinalizedBeaconState",
                        Vec::new(),
                        [
                            77u8, 51u8, 105u8, 202u8, 127u8, 135u8, 21u8, 199u8, 242u8, 77u8, 57u8,
                            104u8, 168u8, 82u8, 1u8, 159u8, 158u8, 84u8, 107u8, 21u8, 34u8, 90u8,
                            225u8, 44u8, 86u8, 63u8, 215u8, 114u8, 254u8, 139u8, 221u8, 68u8,
                        ],
                    )
                }
                #[doc = " Finalized Headers: Current position in ring buffer"]
                pub fn finalized_beacon_state_index(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "FinalizedBeaconStateIndex",
                        vec![],
                        [
                            163u8, 150u8, 87u8, 219u8, 37u8, 81u8, 104u8, 28u8, 244u8, 157u8,
                            150u8, 210u8, 128u8, 233u8, 80u8, 52u8, 132u8, 231u8, 206u8, 103u8,
                            54u8, 30u8, 212u8, 176u8, 134u8, 41u8, 99u8, 218u8, 24u8, 149u8, 26u8,
                            241u8,
                        ],
                    )
                }
                #[doc = " Finalized Headers: Mapping of ring buffer index to a pruning candidate"]
                pub fn finalized_beacon_state_mapping(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "FinalizedBeaconStateMapping",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            81u8, 170u8, 241u8, 176u8, 76u8, 32u8, 17u8, 20u8, 180u8, 229u8, 237u8,
                            29u8, 68u8, 124u8, 73u8, 143u8, 211u8, 106u8, 205u8, 177u8, 131u8,
                            91u8, 255u8, 71u8, 5u8, 221u8, 140u8, 190u8, 55u8, 150u8, 200u8, 201u8,
                        ],
                    )
                }
                #[doc = " Finalized Headers: Mapping of ring buffer index to a pruning candidate"]
                pub fn finalized_beacon_state_mapping_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "FinalizedBeaconStateMapping",
                        Vec::new(),
                        [
                            81u8, 170u8, 241u8, 176u8, 76u8, 32u8, 17u8, 20u8, 180u8, 229u8, 237u8,
                            29u8, 68u8, 124u8, 73u8, 143u8, 211u8, 106u8, 205u8, 177u8, 131u8,
                            91u8, 255u8, 71u8, 5u8, 221u8, 140u8, 190u8, 55u8, 150u8, 200u8, 201u8,
                        ],
                    )
                }
                pub fn validators_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ValidatorsRoot",
                        vec![],
                        [
                            247u8, 82u8, 119u8, 174u8, 30u8, 55u8, 129u8, 27u8, 91u8, 187u8, 76u8,
                            40u8, 2u8, 159u8, 1u8, 128u8, 16u8, 231u8, 115u8, 233u8, 205u8, 24u8,
                            173u8, 148u8, 49u8, 157u8, 116u8, 3u8, 13u8, 54u8, 107u8, 97u8,
                        ],
                    )
                }
                #[doc = " Sync committee for current period"]
                pub fn current_sync_committee(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::SyncCommitteePrepared,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "CurrentSyncCommittee",
                        vec![],
                        [
                            96u8, 163u8, 248u8, 234u8, 137u8, 109u8, 85u8, 1u8, 190u8, 74u8, 242u8,
                            247u8, 229u8, 162u8, 93u8, 44u8, 82u8, 69u8, 25u8, 192u8, 31u8, 49u8,
                            50u8, 94u8, 242u8, 90u8, 183u8, 205u8, 127u8, 166u8, 28u8, 59u8,
                        ],
                    )
                }
                #[doc = " Sync committee for next period"]
                pub fn next_sync_committee(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::SyncCommitteePrepared,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "NextSyncCommittee",
                        vec![],
                        [
                            244u8, 186u8, 52u8, 44u8, 131u8, 17u8, 217u8, 5u8, 128u8, 170u8, 29u8,
                            217u8, 140u8, 245u8, 165u8, 148u8, 226u8, 162u8, 213u8, 108u8, 91u8,
                            241u8, 74u8, 134u8, 207u8, 212u8, 151u8, 8u8, 101u8, 52u8, 161u8,
                            186u8,
                        ],
                    )
                }
                #[doc = " Latest imported execution header"]
                pub fn latest_execution_state(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::ExecutionHeaderState,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "LatestExecutionState",
                        vec![],
                        [
                            164u8, 153u8, 160u8, 189u8, 90u8, 251u8, 187u8, 240u8, 250u8, 148u8,
                            78u8, 176u8, 20u8, 77u8, 215u8, 231u8, 24u8, 217u8, 169u8, 94u8, 99u8,
                            58u8, 83u8, 132u8, 89u8, 46u8, 90u8, 205u8, 37u8, 71u8, 244u8, 96u8,
                        ],
                    )
                }
                #[doc = " Execution Headers"]
                pub fn execution_headers(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::CompactExecutionHeader,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ExecutionHeaders",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            158u8, 73u8, 211u8, 224u8, 143u8, 238u8, 124u8, 46u8, 191u8, 65u8,
                            92u8, 22u8, 76u8, 47u8, 227u8, 223u8, 246u8, 68u8, 145u8, 1u8, 171u8,
                            193u8, 70u8, 28u8, 131u8, 242u8, 149u8, 172u8, 140u8, 167u8, 54u8,
                            40u8,
                        ],
                    )
                }
                #[doc = " Execution Headers"]
                pub fn execution_headers_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::CompactExecutionHeader,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ExecutionHeaders",
                        Vec::new(),
                        [
                            158u8, 73u8, 211u8, 224u8, 143u8, 238u8, 124u8, 46u8, 191u8, 65u8,
                            92u8, 22u8, 76u8, 47u8, 227u8, 223u8, 246u8, 68u8, 145u8, 1u8, 171u8,
                            193u8, 70u8, 28u8, 131u8, 242u8, 149u8, 172u8, 140u8, 167u8, 54u8,
                            40u8,
                        ],
                    )
                }
                #[doc = " Execution Headers: Current position in ring buffer"]
                pub fn execution_header_index(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ExecutionHeaderIndex",
                        vec![],
                        [
                            164u8, 204u8, 146u8, 68u8, 187u8, 179u8, 234u8, 52u8, 40u8, 141u8,
                            60u8, 145u8, 26u8, 10u8, 246u8, 52u8, 76u8, 203u8, 122u8, 68u8, 219u8,
                            176u8, 184u8, 92u8, 25u8, 38u8, 110u8, 221u8, 253u8, 163u8, 246u8,
                            211u8,
                        ],
                    )
                }
                #[doc = " Execution Headers: Mapping of ring buffer index to a pruning candidate"]
                pub fn execution_header_mapping(
                    &self,
                    _0: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ExecutionHeaderMapping",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Identity,
                        )],
                        [
                            51u8, 79u8, 28u8, 65u8, 63u8, 174u8, 220u8, 155u8, 17u8, 242u8, 197u8,
                            180u8, 244u8, 242u8, 242u8, 197u8, 168u8, 38u8, 101u8, 181u8, 10u8,
                            165u8, 13u8, 112u8, 237u8, 49u8, 207u8, 67u8, 30u8, 145u8, 6u8, 8u8,
                        ],
                    )
                }
                #[doc = " Execution Headers: Mapping of ring buffer index to a pruning candidate"]
                pub fn execution_header_mapping_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::H256>,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "ExecutionHeaderMapping",
                        Vec::new(),
                        [
                            51u8, 79u8, 28u8, 65u8, 63u8, 174u8, 220u8, 155u8, 17u8, 242u8, 197u8,
                            180u8, 244u8, 242u8, 242u8, 197u8, 168u8, 38u8, 101u8, 181u8, 10u8,
                            165u8, 13u8, 112u8, 237u8, 49u8, 207u8, 67u8, 30u8, 145u8, 6u8, 8u8,
                        ],
                    )
                }
                #[doc = " Optional pallet owner."]
                #[doc = ""]
                #[doc = " Pallet owner has a right to halt all pallet operations and then resume them. If it is"]
                #[doc = " `None`, then there are no direct ways to halt/resume pallet operations, but other"]
                #[doc = " runtime methods may still be used to do that (i.e. democracy::referendum to update halt"]
                #[doc = " flag directly or call the `halt_operations`)."]
                pub fn pallet_owner(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<::subxt::utils::AccountId32>,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "PalletOwner",
                        vec![],
                        [
                            89u8, 42u8, 74u8, 119u8, 21u8, 164u8, 30u8, 115u8, 207u8, 126u8, 98u8,
                            16u8, 162u8, 214u8, 67u8, 172u8, 178u8, 223u8, 139u8, 121u8, 174u8,
                            89u8, 215u8, 75u8, 200u8, 161u8, 61u8, 195u8, 65u8, 222u8, 246u8,
                            233u8,
                        ],
                    )
                }
                #[doc = " The current operating mode of the pallet."]
                #[doc = ""]
                #[doc = " Depending on the mode either all, or no transactions will be allowed."]
                pub fn pallet_operating_mode(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::bp_runtime::BasicOperatingMode,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumBeaconClient",
                        "PalletOperatingMode",
                        vec![],
                        [
                            218u8, 66u8, 212u8, 71u8, 38u8, 152u8, 55u8, 129u8, 125u8, 231u8, 91u8,
                            216u8, 157u8, 141u8, 173u8, 146u8, 30u8, 40u8, 132u8, 107u8, 97u8,
                            39u8, 36u8, 81u8, 231u8, 222u8, 113u8, 136u8, 233u8, 212u8, 225u8,
                            75u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                pub fn fork_versions(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_beacon_primitives::types::ForkVersions,
                    >,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "EthereumBeaconClient",
                        "ForkVersions",
                        [
                            177u8, 141u8, 98u8, 81u8, 176u8, 167u8, 48u8, 38u8, 165u8, 24u8, 159u8,
                            222u8, 42u8, 191u8, 118u8, 66u8, 190u8, 71u8, 136u8, 7u8, 251u8, 31u8,
                            179u8, 162u8, 65u8, 169u8, 46u8, 5u8, 38u8, 28u8, 35u8, 248u8,
                        ],
                    )
                }
                #[doc = " Maximum number of execution headers to keep"]
                pub fn max_execution_headers_to_keep(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "EthereumBeaconClient",
                        "MaxExecutionHeadersToKeep",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod ethereum_control {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Upgrade {
                pub impl_address: ::subxt::utils::H160,
                pub impl_code_hash: ::subxt::utils::H256,
                pub params: ::core::option::Option<::std::vec::Vec<::core::primitive::u8>>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct CreateAgent;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct CreateChannel;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UpdateChannel {
                pub mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                pub fee: ::core::primitive::u128,
                pub reward: ::core::primitive::u128,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SetOperatingMode {
                pub mode: runtime_types::snowbridge_core::outbound::OperatingMode,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct TransferNativeFromAgent {
                pub recipient: ::subxt::utils::H160,
                pub amount: ::core::primitive::u128,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::upgrade`]."]
                pub fn upgrade(
                    &self,
                    impl_address: ::subxt::utils::H160,
                    impl_code_hash: ::subxt::utils::H256,
                    params: ::core::option::Option<::std::vec::Vec<::core::primitive::u8>>,
                ) -> ::subxt::tx::StaticTxPayload<Upgrade> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "upgrade",
                        Upgrade {
                            impl_address,
                            impl_code_hash,
                            params,
                        },
                        [
                            183u8, 19u8, 145u8, 54u8, 48u8, 3u8, 80u8, 152u8, 123u8, 123u8, 216u8,
                            141u8, 234u8, 51u8, 114u8, 142u8, 53u8, 32u8, 247u8, 228u8, 129u8,
                            210u8, 164u8, 181u8, 27u8, 60u8, 46u8, 222u8, 147u8, 3u8, 189u8, 159u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::create_agent`]."]
                pub fn create_agent(&self) -> ::subxt::tx::StaticTxPayload<CreateAgent> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "create_agent",
                        CreateAgent {},
                        [
                            198u8, 237u8, 155u8, 148u8, 162u8, 187u8, 252u8, 162u8, 46u8, 166u8,
                            208u8, 74u8, 150u8, 73u8, 217u8, 154u8, 56u8, 32u8, 13u8, 211u8, 191u8,
                            68u8, 144u8, 68u8, 109u8, 214u8, 6u8, 245u8, 7u8, 112u8, 85u8, 6u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::create_channel`]."]
                pub fn create_channel(&self) -> ::subxt::tx::StaticTxPayload<CreateChannel> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "create_channel",
                        CreateChannel {},
                        [
                            123u8, 250u8, 222u8, 111u8, 221u8, 147u8, 8u8, 244u8, 142u8, 252u8,
                            110u8, 120u8, 236u8, 14u8, 225u8, 201u8, 33u8, 112u8, 229u8, 37u8,
                            194u8, 156u8, 219u8, 243u8, 191u8, 115u8, 220u8, 91u8, 28u8, 179u8,
                            69u8, 74u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::update_channel`]."]
                pub fn update_channel(
                    &self,
                    mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                    fee: ::core::primitive::u128,
                    reward: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<UpdateChannel> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "update_channel",
                        UpdateChannel { mode, fee, reward },
                        [
                            71u8, 174u8, 71u8, 223u8, 10u8, 67u8, 83u8, 60u8, 36u8, 16u8, 235u8,
                            213u8, 24u8, 206u8, 11u8, 162u8, 231u8, 203u8, 220u8, 118u8, 2u8, 22u8,
                            84u8, 163u8, 211u8, 9u8, 50u8, 105u8, 93u8, 63u8, 57u8, 26u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::set_operating_mode`]."]
                pub fn set_operating_mode(
                    &self,
                    mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                ) -> ::subxt::tx::StaticTxPayload<SetOperatingMode> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "set_operating_mode",
                        SetOperatingMode { mode },
                        [
                            198u8, 154u8, 13u8, 235u8, 244u8, 154u8, 200u8, 190u8, 78u8, 172u8,
                            143u8, 41u8, 176u8, 189u8, 49u8, 197u8, 241u8, 237u8, 168u8, 0u8,
                            201u8, 34u8, 252u8, 185u8, 131u8, 192u8, 125u8, 4u8, 211u8, 117u8,
                            169u8, 20u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::transfer_native_from_agent`]."]
                pub fn transfer_native_from_agent(
                    &self,
                    recipient: ::subxt::utils::H160,
                    amount: ::core::primitive::u128,
                ) -> ::subxt::tx::StaticTxPayload<TransferNativeFromAgent> {
                    ::subxt::tx::StaticTxPayload::new(
                        "EthereumControl",
                        "transfer_native_from_agent",
                        TransferNativeFromAgent { recipient, amount },
                        [
                            101u8, 44u8, 233u8, 190u8, 199u8, 109u8, 156u8, 14u8, 181u8, 184u8,
                            178u8, 54u8, 156u8, 134u8, 255u8, 123u8, 115u8, 174u8, 238u8, 92u8,
                            235u8, 249u8, 103u8, 64u8, 248u8, 101u8, 24u8, 91u8, 45u8, 52u8, 168u8,
                            146u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::snowbridge_control::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An Upgrade message was sent to the Gateway"]
            pub struct Upgrade {
                pub impl_address: ::subxt::utils::H160,
                pub impl_code_hash: ::subxt::utils::H256,
                pub params_hash: ::core::option::Option<::subxt::utils::H256>,
            }
            impl ::subxt::events::StaticEvent for Upgrade {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "Upgrade";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An CreateAgent message was sent to the Gateway"]
            pub struct CreateAgent {
                pub location:
                    ::std::boxed::Box<runtime_types::xcm::v3::multilocation::MultiLocation>,
                pub agent_id: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for CreateAgent {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "CreateAgent";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An CreateChannel message was sent to the Gateway"]
            pub struct CreateChannel {
                pub para_id: runtime_types::polkadot_parachain::primitives::Id,
                pub agent_id: ::subxt::utils::H256,
            }
            impl ::subxt::events::StaticEvent for CreateChannel {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "CreateChannel";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An UpdateChannel message was sent to the Gateway"]
            pub struct UpdateChannel {
                pub para_id: runtime_types::polkadot_parachain::primitives::Id,
                pub agent_id: ::subxt::utils::H256,
                pub mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                pub fee: ::core::primitive::u128,
                pub reward: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for UpdateChannel {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "UpdateChannel";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An SetOperatingMode message was sent to the Gateway"]
            pub struct SetOperatingMode {
                pub mode: runtime_types::snowbridge_core::outbound::OperatingMode,
            }
            impl ::subxt::events::StaticEvent for SetOperatingMode {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "SetOperatingMode";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "An TransferNativeFromAgent message was sent to the Gateway"]
            pub struct TransferNativeFromAgent {
                pub agent_id: ::subxt::utils::H256,
                pub recipient: ::subxt::utils::H160,
                pub amount: ::core::primitive::u128,
            }
            impl ::subxt::events::StaticEvent for TransferNativeFromAgent {
                const PALLET: &'static str = "EthereumControl";
                const EVENT: &'static str = "TransferNativeFromAgent";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                pub fn agents(
                    &self,
                    _0: impl ::std::borrow::Borrow<::subxt::utils::H256>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<()>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumControl",
                        "Agents",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            152u8, 40u8, 161u8, 42u8, 91u8, 199u8, 77u8, 233u8, 105u8, 216u8, 64u8,
                            172u8, 211u8, 181u8, 220u8, 109u8, 174u8, 198u8, 7u8, 176u8, 53u8,
                            83u8, 30u8, 225u8, 179u8, 35u8, 73u8, 115u8, 174u8, 73u8, 203u8, 58u8,
                        ],
                    )
                }
                pub fn agents_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<()>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumControl",
                        "Agents",
                        Vec::new(),
                        [
                            152u8, 40u8, 161u8, 42u8, 91u8, 199u8, 77u8, 233u8, 105u8, 216u8, 64u8,
                            172u8, 211u8, 181u8, 220u8, 109u8, 174u8, 198u8, 7u8, 176u8, 53u8,
                            83u8, 30u8, 225u8, 179u8, 35u8, 73u8, 115u8, 174u8, 73u8, 203u8, 58u8,
                        ],
                    )
                }
                pub fn channels(
                    &self,
                    _0: impl ::std::borrow::Borrow<runtime_types::polkadot_parachain::primitives::Id>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<()>,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumControl",
                        "Channels",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            53u8, 114u8, 3u8, 214u8, 248u8, 221u8, 82u8, 48u8, 116u8, 156u8, 50u8,
                            75u8, 70u8, 129u8, 87u8, 168u8, 121u8, 1u8, 132u8, 7u8, 79u8, 206u8,
                            9u8, 169u8, 251u8, 179u8, 154u8, 61u8, 70u8, 107u8, 126u8, 204u8,
                        ],
                    )
                }
                pub fn channels_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<()>,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "EthereumControl",
                        "Channels",
                        Vec::new(),
                        [
                            53u8, 114u8, 3u8, 214u8, 248u8, 221u8, 82u8, 48u8, 116u8, 156u8, 50u8,
                            75u8, 70u8, 129u8, 87u8, 168u8, 121u8, 1u8, 132u8, 7u8, 79u8, 206u8,
                            9u8, 169u8, 251u8, 179u8, 154u8, 61u8, 70u8, 107u8, 126u8, 204u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod message_queue {
        use super::root_mod;
        use super::runtime_types;
        #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
        pub mod calls {
            use super::root_mod;
            use super::runtime_types;
            type DispatchError = runtime_types::sp_runtime::DispatchError;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReapPage {
                pub message_origin:
                    runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub page_index: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ExecuteOverweight {
                pub message_origin:
                    runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub page: ::core::primitive::u32,
                pub index: ::core::primitive::u32,
                pub weight_limit: runtime_types::sp_weights::weight_v2::Weight,
            }
            pub struct TransactionApi;
            impl TransactionApi {
                #[doc = "See [`Pallet::reap_page`]."]
                pub fn reap_page(
                    &self,
                    message_origin : runtime_types :: snowbridge_outbound_queue :: AggregateMessageOrigin,
                    page_index: ::core::primitive::u32,
                ) -> ::subxt::tx::StaticTxPayload<ReapPage> {
                    ::subxt::tx::StaticTxPayload::new(
                        "MessageQueue",
                        "reap_page",
                        ReapPage {
                            message_origin,
                            page_index,
                        },
                        [
                            255u8, 233u8, 245u8, 200u8, 123u8, 65u8, 64u8, 141u8, 91u8, 103u8,
                            226u8, 25u8, 68u8, 46u8, 57u8, 16u8, 160u8, 125u8, 135u8, 102u8, 75u8,
                            173u8, 151u8, 116u8, 226u8, 55u8, 90u8, 139u8, 165u8, 134u8, 183u8,
                            218u8,
                        ],
                    )
                }
                #[doc = "See [`Pallet::execute_overweight`]."]
                pub fn execute_overweight(
                    &self,
                    message_origin : runtime_types :: snowbridge_outbound_queue :: AggregateMessageOrigin,
                    page: ::core::primitive::u32,
                    index: ::core::primitive::u32,
                    weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                ) -> ::subxt::tx::StaticTxPayload<ExecuteOverweight> {
                    ::subxt::tx::StaticTxPayload::new(
                        "MessageQueue",
                        "execute_overweight",
                        ExecuteOverweight {
                            message_origin,
                            page,
                            index,
                            weight_limit,
                        },
                        [
                            92u8, 40u8, 16u8, 99u8, 234u8, 217u8, 105u8, 237u8, 86u8, 226u8, 173u8,
                            179u8, 10u8, 185u8, 5u8, 228u8, 107u8, 89u8, 140u8, 230u8, 86u8, 126u8,
                            219u8, 145u8, 159u8, 138u8, 215u8, 104u8, 131u8, 9u8, 122u8, 168u8,
                        ],
                    )
                }
            }
        }
        #[doc = "The `Event` enum of this pallet"]
        pub type Event = runtime_types::pallet_message_queue::pallet::Event;
        pub mod events {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message discarded due to an error in the `MessageProcessor` (usually a format error)."]
            pub struct ProcessingFailed {
                pub id: [::core::primitive::u8; 32usize],
                pub origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub error: runtime_types::frame_support::traits::messages::ProcessMessageError,
            }
            impl ::subxt::events::StaticEvent for ProcessingFailed {
                const PALLET: &'static str = "MessageQueue";
                const EVENT: &'static str = "ProcessingFailed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message is processed."]
            pub struct Processed {
                pub id: [::core::primitive::u8; 32usize],
                pub origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub weight_used: runtime_types::sp_weights::weight_v2::Weight,
                pub success: ::core::primitive::bool,
            }
            impl ::subxt::events::StaticEvent for Processed {
                const PALLET: &'static str = "MessageQueue";
                const EVENT: &'static str = "Processed";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "Message placed in overweight queue."]
            pub struct OverweightEnqueued {
                pub id: [::core::primitive::u8; 32usize],
                pub origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub page_index: ::core::primitive::u32,
                pub message_index: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for OverweightEnqueued {
                const PALLET: &'static str = "MessageQueue";
                const EVENT: &'static str = "OverweightEnqueued";
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            #[doc = "This page was reaped."]
            pub struct PageReaped {
                pub origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                pub index: ::core::primitive::u32,
            }
            impl ::subxt::events::StaticEvent for PageReaped {
                const PALLET: &'static str = "MessageQueue";
                const EVENT: &'static str = "PageReaped";
            }
        }
        pub mod storage {
            use super::runtime_types;
            pub struct StorageApi;
            impl StorageApi {
                #[doc = " The index of the first and last (non-empty) pages."]
                pub fn book_state_for(
                    &self,
                    _0: impl ::std::borrow::Borrow<
                        runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                    >,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_message_queue::BookState<
                            runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        >,
                    >,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "MessageQueue",
                        "BookStateFor",
                        vec![::subxt::storage::address::StorageMapKey::new(
                            _0.borrow(),
                            ::subxt::storage::address::StorageHasher::Twox64Concat,
                        )],
                        [
                            106u8, 53u8, 87u8, 104u8, 160u8, 196u8, 214u8, 189u8, 48u8, 19u8,
                            219u8, 103u8, 19u8, 15u8, 91u8, 120u8, 169u8, 100u8, 162u8, 202u8,
                            133u8, 203u8, 252u8, 145u8, 63u8, 190u8, 214u8, 217u8, 202u8, 142u8,
                            4u8, 20u8,
                        ],
                    )
                }
                #[doc = " The index of the first and last (non-empty) pages."]
                pub fn book_state_for_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_message_queue::BookState<
                            runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        >,
                    >,
                    (),
                    ::subxt::storage::address::Yes,
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "MessageQueue",
                        "BookStateFor",
                        Vec::new(),
                        [
                            106u8, 53u8, 87u8, 104u8, 160u8, 196u8, 214u8, 189u8, 48u8, 19u8,
                            219u8, 103u8, 19u8, 15u8, 91u8, 120u8, 169u8, 100u8, 162u8, 202u8,
                            133u8, 203u8, 252u8, 145u8, 63u8, 190u8, 214u8, 217u8, 202u8, 142u8,
                            4u8, 20u8,
                        ],
                    )
                }
                #[doc = " The origin at which we should begin servicing."]
                pub fn service_head(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    (),
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "MessageQueue",
                        "ServiceHead",
                        vec![],
                        [
                            13u8, 208u8, 53u8, 30u8, 120u8, 14u8, 129u8, 237u8, 114u8, 95u8, 158u8,
                            84u8, 67u8, 205u8, 210u8, 24u8, 49u8, 167u8, 250u8, 193u8, 209u8, 78u8,
                            58u8, 69u8, 203u8, 38u8, 159u8, 175u8, 184u8, 3u8, 198u8, 151u8,
                        ],
                    )
                }
                #[doc = " The map of page indices to pages."]
                pub fn pages(
                    &self,
                    _0: impl ::std::borrow::Borrow<
                        runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                    >,
                    _1: impl ::std::borrow::Borrow<::core::primitive::u32>,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_message_queue::Page<::core::primitive::u32>,
                    >,
                    ::subxt::storage::address::Yes,
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "MessageQueue",
                        "Pages",
                        vec![
                            ::subxt::storage::address::StorageMapKey::new(
                                _0.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                            ::subxt::storage::address::StorageMapKey::new(
                                _1.borrow(),
                                ::subxt::storage::address::StorageHasher::Twox64Concat,
                            ),
                        ],
                        [
                            201u8, 12u8, 210u8, 226u8, 174u8, 190u8, 122u8, 246u8, 195u8, 207u8,
                            123u8, 61u8, 86u8, 178u8, 60u8, 83u8, 67u8, 110u8, 149u8, 96u8, 46u8,
                            106u8, 107u8, 41u8, 111u8, 152u8, 244u8, 53u8, 141u8, 181u8, 236u8,
                            241u8,
                        ],
                    )
                }
                #[doc = " The map of page indices to pages."]
                pub fn pages_root(
                    &self,
                ) -> ::subxt::storage::address::StaticStorageAddress<
                    ::subxt::metadata::DecodeStaticType<
                        runtime_types::pallet_message_queue::Page<::core::primitive::u32>,
                    >,
                    (),
                    (),
                    ::subxt::storage::address::Yes,
                > {
                    ::subxt::storage::address::StaticStorageAddress::new(
                        "MessageQueue",
                        "Pages",
                        Vec::new(),
                        [
                            201u8, 12u8, 210u8, 226u8, 174u8, 190u8, 122u8, 246u8, 195u8, 207u8,
                            123u8, 61u8, 86u8, 178u8, 60u8, 83u8, 67u8, 110u8, 149u8, 96u8, 46u8,
                            106u8, 107u8, 41u8, 111u8, 152u8, 244u8, 53u8, 141u8, 181u8, 236u8,
                            241u8,
                        ],
                    )
                }
            }
        }
        pub mod constants {
            use super::runtime_types;
            pub struct ConstantsApi;
            impl ConstantsApi {
                #[doc = " The size of the page; this implies the maximum message size which can be sent."]
                #[doc = ""]
                #[doc = " A good value depends on the expected message sizes, their weights, the weight that is"]
                #[doc = " available for processing them and the maximal needed message size. The maximal message"]
                #[doc = " size is slightly lower than this as defined by [`MaxMessageLenOf`]."]
                pub fn heap_size(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "MessageQueue",
                        "HeapSize",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The maximum number of stale pages (i.e. of overweight messages) allowed before culling"]
                #[doc = " can happen. Once there are more stale pages than this, then historical pages may be"]
                #[doc = " dropped, even if they contain unprocessed overweight messages."]
                pub fn max_stale(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<::core::primitive::u32>,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "MessageQueue",
                        "MaxStale",
                        [
                            98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
                            125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
                            178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
                            145u8,
                        ],
                    )
                }
                #[doc = " The amount of weight (if any) which should be provided to the message queue for"]
                #[doc = " servicing enqueued items."]
                #[doc = ""]
                #[doc = " This may be legitimately `None` in the case that you will call"]
                #[doc = " `ServiceQueues::service_queues` manually."]
                pub fn service_weight(
                    &self,
                ) -> ::subxt::constants::StaticConstantAddress<
                    ::subxt::metadata::DecodeStaticType<
                        ::core::option::Option<runtime_types::sp_weights::weight_v2::Weight>,
                    >,
                > {
                    ::subxt::constants::StaticConstantAddress::new(
                        "MessageQueue",
                        "ServiceWeight",
                        [
                            199u8, 205u8, 164u8, 188u8, 101u8, 240u8, 98u8, 54u8, 0u8, 78u8, 218u8,
                            77u8, 164u8, 196u8, 0u8, 194u8, 181u8, 251u8, 195u8, 24u8, 98u8, 147u8,
                            169u8, 53u8, 22u8, 202u8, 66u8, 236u8, 163u8, 36u8, 162u8, 46u8,
                        ],
                    )
                }
            }
        }
    }
    pub mod runtime_types {
        use super::runtime_types;
        pub mod amcl {
            use super::runtime_types;
            pub mod bls381 {
                use super::runtime_types;
                pub mod big {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct Big {
                        pub w: [::core::primitive::i32; 14usize],
                    }
                }
                pub mod ecp {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct ECP {
                        pub x: runtime_types::amcl::bls381::fp::FP,
                        pub y: runtime_types::amcl::bls381::fp::FP,
                        pub z: runtime_types::amcl::bls381::fp::FP,
                    }
                }
                pub mod fp {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct FP {
                        pub x: runtime_types::amcl::bls381::big::Big,
                        pub xes: ::core::primitive::i32,
                    }
                }
            }
        }
        pub mod bounded_collections {
            use super::runtime_types;
            pub mod bounded_vec {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BoundedVec<_0>(pub ::std::vec::Vec<_0>);
            }
            pub mod weak_bounded_vec {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct WeakBoundedVec<_0>(pub ::std::vec::Vec<_0>);
            }
        }
        pub mod bp_header_chain {
            use super::runtime_types;
            pub mod justification {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct GrandpaJustification<_0> {
                    pub round: ::core::primitive::u64,
                    pub commit: runtime_types::finality_grandpa::Commit<
                        ::subxt::utils::H256,
                        ::core::primitive::u32,
                        runtime_types::sp_consensus_grandpa::app::Signature,
                        runtime_types::sp_consensus_grandpa::app::Public,
                    >,
                    pub votes_ancestries: ::std::vec::Vec<_0>,
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum HeaderChainError {
                #[codec(index = 0)]
                UnknownHeader,
                #[codec(index = 1)]
                StorageProof(runtime_types::bp_runtime::storage_proof::Error),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct InitializationData<_0> {
                pub header: ::std::boxed::Box<_0>,
                pub authority_list: ::std::vec::Vec<(
                    runtime_types::sp_consensus_grandpa::app::Public,
                    ::core::primitive::u64,
                )>,
                pub set_id: ::core::primitive::u64,
                pub operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct StoredHeaderData<_0, _1> {
                pub number: _0,
                pub state_root: _1,
            }
        }
        pub mod bp_messages {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct DeliveredMessages {
                pub begin: ::core::primitive::u64,
                pub end: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct InboundLaneData<_0> {
                pub relayers: ::std::vec::Vec<runtime_types::bp_messages::UnrewardedRelayer<_0>>,
                pub last_confirmed_nonce: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct LaneId(pub [::core::primitive::u8; 4usize]);
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct MessageKey {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub nonce: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum MessagesOperatingMode {
                #[codec(index = 0)]
                Basic(runtime_types::bp_runtime::BasicOperatingMode),
                #[codec(index = 1)]
                RejectingOutboundMessages,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct OutboundLaneData {
                pub oldest_unpruned_nonce: ::core::primitive::u64,
                pub latest_received_nonce: ::core::primitive::u64,
                pub latest_generated_nonce: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum ReceivalResult<_0> {
                #[codec(index = 0)]
                Dispatched(runtime_types::bp_runtime::messages::MessageDispatchResult<_0>),
                #[codec(index = 1)]
                InvalidNonce,
                #[codec(index = 2)]
                TooManyUnrewardedRelayers,
                #[codec(index = 3)]
                TooManyUnconfirmedMessages,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ReceivedMessages<_0> {
                pub lane: runtime_types::bp_messages::LaneId,
                pub receive_results: ::std::vec::Vec<(
                    ::core::primitive::u64,
                    runtime_types::bp_messages::ReceivalResult<_0>,
                )>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UnrewardedRelayer<_0> {
                pub relayer: _0,
                pub messages: runtime_types::bp_messages::DeliveredMessages,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct UnrewardedRelayersState {
                pub unrewarded_relayer_entries: ::core::primitive::u64,
                pub messages_in_oldest_entry: ::core::primitive::u64,
                pub total_messages: ::core::primitive::u64,
                pub last_delivered_nonce: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum VerificationError {
                #[codec(index = 0)]
                EmptyMessageProof,
                #[codec(index = 1)]
                HeaderChain(runtime_types::bp_header_chain::HeaderChainError),
                #[codec(index = 2)]
                InboundLaneStorage(runtime_types::bp_runtime::storage_proof::Error),
                #[codec(index = 3)]
                InvalidMessageWeight,
                #[codec(index = 4)]
                MessagesCountMismatch,
                #[codec(index = 5)]
                MessageStorage(runtime_types::bp_runtime::storage_proof::Error),
                #[codec(index = 6)]
                MessageTooLarge,
                #[codec(index = 7)]
                OutboundLaneStorage(runtime_types::bp_runtime::storage_proof::Error),
                #[codec(index = 8)]
                StorageProof(runtime_types::bp_runtime::storage_proof::Error),
                #[codec(index = 9)]
                Other,
            }
        }
        pub mod bp_parachains {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct BestParaHeadHash {
                pub at_relay_block_number: ::core::primitive::u32,
                pub head_hash: ::subxt::utils::H256,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ParaInfo {
                pub best_head_hash: runtime_types::bp_parachains::BestParaHeadHash,
                pub next_imported_hash_position: ::core::primitive::u32,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ParaStoredHeaderData(pub ::std::vec::Vec<::core::primitive::u8>);
        }
        pub mod bp_polkadot_core {
            use super::runtime_types;
            pub mod parachains {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ParaHeadsProof(
                    pub ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                );
                #[derive(
                    :: subxt :: ext :: codec :: CompactAs,
                    :: subxt :: ext :: codec :: Decode,
                    :: subxt :: ext :: codec :: Encode,
                    Debug,
                )]
                pub struct ParaId(pub ::core::primitive::u32);
            }
        }
        pub mod bp_relayers {
            use super::runtime_types;
            pub mod registration {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Registration<_0, _1> {
                    pub valid_till: _0,
                    pub stake: _1,
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum RewardsAccountOwner {
                #[codec(index = 0)]
                ThisChain,
                #[codec(index = 1)]
                BridgedChain,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RewardsAccountParams {
                pub lane_id: runtime_types::bp_messages::LaneId,
                pub bridged_chain_id: [::core::primitive::u8; 4usize],
                pub owner: runtime_types::bp_relayers::RewardsAccountOwner,
            }
        }
        pub mod bp_runtime {
            use super::runtime_types;
            pub mod messages {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct MessageDispatchResult<_0> {
                    pub unspent_weight: runtime_types::sp_weights::weight_v2::Weight,
                    pub dispatch_level_result: _0,
                }
            }
            pub mod storage_proof {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Error {
                    #[codec(index = 0)]
                    DuplicateNodesInProof,
                    #[codec(index = 1)]
                    UnusedNodesInTheProof,
                    #[codec(index = 2)]
                    StorageRootMismatch,
                    #[codec(index = 3)]
                    StorageValueUnavailable,
                    #[codec(index = 4)]
                    StorageValueEmpty,
                    #[codec(index = 5)]
                    StorageValueDecodeFailed(runtime_types::bp_runtime::StrippableError),
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum BasicOperatingMode {
                #[codec(index = 0)]
                Normal,
                #[codec(index = 1)]
                Halted,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct HeaderId<_0, _1>(pub _1, pub _0);
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum OwnedBridgeModuleError {
                #[codec(index = 0)]
                Halted,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct StrippableError;
        }
        pub mod bridge_hub_rococo_runtime {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct BridgeRejectObsoleteHeadersAndMessages;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum OriginCaller {
                #[codec(index = 0)]
                system(
                    runtime_types::frame_support::dispatch::RawOrigin<::subxt::utils::AccountId32>,
                ),
                #[codec(index = 31)]
                PolkadotXcm(runtime_types::pallet_xcm::pallet::Origin),
                #[codec(index = 32)]
                CumulusXcm(runtime_types::cumulus_pallet_xcm::pallet::Origin),
                #[codec(index = 3)]
                Void(runtime_types::sp_core::Void),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Runtime;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum RuntimeCall {
                #[codec(index = 0)]
                System(runtime_types::frame_system::pallet::Call),
                #[codec(index = 1)]
                ParachainSystem(runtime_types::cumulus_pallet_parachain_system::pallet::Call),
                #[codec(index = 2)]
                Timestamp(runtime_types::pallet_timestamp::pallet::Call),
                #[codec(index = 10)]
                Balances(runtime_types::pallet_balances::pallet::Call),
                #[codec(index = 21)]
                CollatorSelection(runtime_types::pallet_collator_selection::pallet::Call),
                #[codec(index = 22)]
                Session(runtime_types::pallet_session::pallet::Call),
                #[codec(index = 30)]
                XcmpQueue(runtime_types::cumulus_pallet_xcmp_queue::pallet::Call),
                #[codec(index = 31)]
                PolkadotXcm(runtime_types::pallet_xcm::pallet::Call),
                #[codec(index = 33)]
                DmpQueue(runtime_types::cumulus_pallet_dmp_queue::pallet::Call),
                #[codec(index = 40)]
                Utility(runtime_types::pallet_utility::pallet::Call),
                #[codec(index = 36)]
                Multisig(runtime_types::pallet_multisig::pallet::Call),
                #[codec(index = 41)]
                BridgeWococoGrandpa(runtime_types::pallet_bridge_grandpa::pallet::Call),
                #[codec(index = 42)]
                BridgeWococoParachain(runtime_types::pallet_bridge_parachains::pallet::Call),
                #[codec(index = 46)]
                BridgeWococoMessages(runtime_types::pallet_bridge_messages::pallet::Call),
                #[codec(index = 43)]
                BridgeRococoGrandpa(runtime_types::pallet_bridge_grandpa::pallet::Call),
                #[codec(index = 44)]
                BridgeRococoParachain(runtime_types::pallet_bridge_parachains::pallet::Call),
                #[codec(index = 45)]
                BridgeRococoMessages(runtime_types::pallet_bridge_messages::pallet::Call),
                #[codec(index = 47)]
                BridgeRelayers(runtime_types::pallet_bridge_relayers::pallet::Call),
                #[codec(index = 48)]
                EthereumInboundQueue(runtime_types::snowbridge_inbound_queue::pallet::Call),
                #[codec(index = 49)]
                EthereumOutboundQueue(runtime_types::snowbridge_outbound_queue::pallet::Call),
                #[codec(index = 50)]
                EthereumBeaconClient(
                    runtime_types::snowbridge_ethereum_beacon_client::pallet::Call,
                ),
                #[codec(index = 51)]
                EthereumControl(runtime_types::snowbridge_control::pallet::Call),
                #[codec(index = 60)]
                MessageQueue(runtime_types::pallet_message_queue::pallet::Call),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum RuntimeEvent {
                #[codec(index = 0)]
                System(runtime_types::frame_system::pallet::Event),
                #[codec(index = 1)]
                ParachainSystem(runtime_types::cumulus_pallet_parachain_system::pallet::Event),
                #[codec(index = 10)]
                Balances(runtime_types::pallet_balances::pallet::Event),
                #[codec(index = 11)]
                TransactionPayment(runtime_types::pallet_transaction_payment::pallet::Event),
                #[codec(index = 21)]
                CollatorSelection(runtime_types::pallet_collator_selection::pallet::Event),
                #[codec(index = 22)]
                Session(runtime_types::pallet_session::pallet::Event),
                #[codec(index = 30)]
                XcmpQueue(runtime_types::cumulus_pallet_xcmp_queue::pallet::Event),
                #[codec(index = 31)]
                PolkadotXcm(runtime_types::pallet_xcm::pallet::Event),
                #[codec(index = 32)]
                CumulusXcm(runtime_types::cumulus_pallet_xcm::pallet::Event),
                #[codec(index = 33)]
                DmpQueue(runtime_types::cumulus_pallet_dmp_queue::pallet::Event),
                #[codec(index = 40)]
                Utility(runtime_types::pallet_utility::pallet::Event),
                #[codec(index = 36)]
                Multisig(runtime_types::pallet_multisig::pallet::Event),
                #[codec(index = 41)]
                BridgeWococoGrandpa(runtime_types::pallet_bridge_grandpa::pallet::Event),
                #[codec(index = 42)]
                BridgeWococoParachain(runtime_types::pallet_bridge_parachains::pallet::Event),
                #[codec(index = 46)]
                BridgeWococoMessages(runtime_types::pallet_bridge_messages::pallet::Event),
                #[codec(index = 43)]
                BridgeRococoGrandpa(runtime_types::pallet_bridge_grandpa::pallet::Event),
                #[codec(index = 44)]
                BridgeRococoParachain(runtime_types::pallet_bridge_parachains::pallet::Event),
                #[codec(index = 45)]
                BridgeRococoMessages(runtime_types::pallet_bridge_messages::pallet::Event),
                #[codec(index = 47)]
                BridgeRelayers(runtime_types::pallet_bridge_relayers::pallet::Event),
                #[codec(index = 48)]
                EthereumInboundQueue(runtime_types::snowbridge_inbound_queue::pallet::Event),
                #[codec(index = 49)]
                EthereumOutboundQueue(runtime_types::snowbridge_outbound_queue::pallet::Event),
                #[codec(index = 50)]
                EthereumBeaconClient(
                    runtime_types::snowbridge_ethereum_beacon_client::pallet::Event,
                ),
                #[codec(index = 51)]
                EthereumControl(runtime_types::snowbridge_control::pallet::Event),
                #[codec(index = 60)]
                MessageQueue(runtime_types::pallet_message_queue::pallet::Event),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum RuntimeHoldReason {}
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SessionKeys {
                pub aura: runtime_types::sp_consensus_aura::sr25519::app_sr25519::Public,
            }
        }
        pub mod bridge_runtime_common {
            use super::runtime_types;
            pub mod messages {
                use super::runtime_types;
                pub mod source {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct FromBridgedChainMessagesDeliveryProof<_0> {
                        pub bridged_header_hash: _0,
                        pub storage_proof: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                        pub lane: runtime_types::bp_messages::LaneId,
                    }
                }
                pub mod target {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct FromBridgedChainMessagesProof<_0> {
                        pub bridged_header_hash: _0,
                        pub storage_proof: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                        pub lane: runtime_types::bp_messages::LaneId,
                        pub nonces_start: ::core::primitive::u64,
                        pub nonces_end: ::core::primitive::u64,
                    }
                }
            }
            pub mod messages_xcm_extension {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum XcmBlobMessageDispatchResult {
                    #[codec(index = 0)]
                    InvalidPayload,
                    #[codec(index = 1)]
                    Dispatched,
                    #[codec(index = 2)]
                    NotDispatched,
                }
            }
            pub mod refund_relayer_extension {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct RefundBridgedParachainMessages;
            }
        }
        pub mod cumulus_pallet_dmp_queue {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::service_overweight`]."]
                    service_overweight {
                        index: ::core::primitive::u64,
                        weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The message index given is unknown."]
                    Unknown,
                    #[codec(index = 1)]
                    #[doc = "The amount of weight given is possibly not enough for executing the message."]
                    OverLimit,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Downward message is invalid XCM."]
                    InvalidFormat {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 1)]
                    #[doc = "Downward message is unsupported version of XCM."]
                    UnsupportedVersion {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 2)]
                    #[doc = "Downward message executed with the given outcome."]
                    ExecutedDownward {
                        message_hash: [::core::primitive::u8; 32usize],
                        message_id: [::core::primitive::u8; 32usize],
                        outcome: runtime_types::xcm::v3::traits::Outcome,
                    },
                    #[codec(index = 3)]
                    #[doc = "The weight limit for handling downward messages was reached."]
                    WeightExhausted {
                        message_hash: [::core::primitive::u8; 32usize],
                        message_id: [::core::primitive::u8; 32usize],
                        remaining_weight: runtime_types::sp_weights::weight_v2::Weight,
                        required_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 4)]
                    #[doc = "Downward message is overweight and was placed in the overweight queue."]
                    OverweightEnqueued {
                        message_hash: [::core::primitive::u8; 32usize],
                        message_id: [::core::primitive::u8; 32usize],
                        overweight_index: ::core::primitive::u64,
                        required_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 5)]
                    #[doc = "Downward message from the overweight queue was executed."]
                    OverweightServiced {
                        overweight_index: ::core::primitive::u64,
                        weight_used: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 6)]
                    #[doc = "The maximum number of downward messages was reached."]
                    MaxMessagesExhausted {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ConfigData {
                pub max_individual: runtime_types::sp_weights::weight_v2::Weight,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct PageIndexData {
                pub begin_used: ::core::primitive::u32,
                pub end_used: ::core::primitive::u32,
                pub overweight_count: ::core::primitive::u64,
            }
        }
        pub mod cumulus_pallet_parachain_system {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    # [codec (index = 0)] # [doc = "See [`Pallet::set_validation_data`]."] set_validation_data { data : runtime_types :: cumulus_primitives_parachain_inherent :: ParachainInherentData , } , # [codec (index = 1)] # [doc = "See [`Pallet::sudo_send_upward_message`]."] sudo_send_upward_message { message : :: std :: vec :: Vec < :: core :: primitive :: u8 > , } , # [codec (index = 2)] # [doc = "See [`Pallet::authorize_upgrade`]."] authorize_upgrade { code_hash : :: subxt :: utils :: H256 , check_version : :: core :: primitive :: bool , } , # [codec (index = 3)] # [doc = "See [`Pallet::enact_authorized_upgrade`]."] enact_authorized_upgrade { code : :: std :: vec :: Vec < :: core :: primitive :: u8 > , } , }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Attempt to upgrade validation function while existing upgrade pending."]
                    OverlappingUpgrades,
                    #[codec(index = 1)]
                    #[doc = "Polkadot currently prohibits this parachain from upgrading its validation function."]
                    ProhibitedByPolkadot,
                    #[codec(index = 2)]
                    #[doc = "The supplied validation function has compiled into a blob larger than Polkadot is"]
                    #[doc = "willing to run."]
                    TooBig,
                    #[codec(index = 3)]
                    #[doc = "The inherent which supplies the validation data did not run this block."]
                    ValidationDataNotAvailable,
                    #[codec(index = 4)]
                    #[doc = "The inherent which supplies the host configuration did not run this block."]
                    HostConfigurationNotAvailable,
                    #[codec(index = 5)]
                    #[doc = "No validation function upgrade is currently scheduled."]
                    NotScheduled,
                    #[codec(index = 6)]
                    #[doc = "No code upgrade has been authorized."]
                    NothingAuthorized,
                    #[codec(index = 7)]
                    #[doc = "The given code upgrade has not been authorized."]
                    Unauthorized,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "The validation function has been scheduled to apply."]
                    ValidationFunctionStored,
                    #[codec(index = 1)]
                    #[doc = "The validation function was applied as of the contained relay chain block number."]
                    ValidationFunctionApplied {
                        relay_chain_block_num: ::core::primitive::u32,
                    },
                    #[codec(index = 2)]
                    #[doc = "The relay-chain aborted the upgrade process."]
                    ValidationFunctionDiscarded,
                    #[codec(index = 3)]
                    #[doc = "An upgrade has been authorized."]
                    UpgradeAuthorized { code_hash: ::subxt::utils::H256 },
                    #[codec(index = 4)]
                    #[doc = "Some downward messages have been received and will be processed."]
                    DownwardMessagesReceived { count: ::core::primitive::u32 },
                    #[codec(index = 5)]
                    #[doc = "Downward messages were processed using the given weight."]
                    DownwardMessagesProcessed {
                        weight_used: runtime_types::sp_weights::weight_v2::Weight,
                        dmq_head: ::subxt::utils::H256,
                    },
                    #[codec(index = 6)]
                    #[doc = "An upward message was sent to the relay chain."]
                    UpwardMessageSent {
                        message_hash: ::core::option::Option<[::core::primitive::u8; 32usize]>,
                    },
                }
            }
            pub mod relay_state_snapshot {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct MessagingStateSnapshot { pub dmq_mqc_head : :: subxt :: utils :: H256 , pub relay_dispatch_queue_size : runtime_types :: cumulus_pallet_parachain_system :: relay_state_snapshot :: RelayDispachQueueSize , pub ingress_channels : :: std :: vec :: Vec < (runtime_types :: polkadot_parachain :: primitives :: Id , runtime_types :: polkadot_primitives :: v5 :: AbridgedHrmpChannel ,) > , pub egress_channels : :: std :: vec :: Vec < (runtime_types :: polkadot_parachain :: primitives :: Id , runtime_types :: polkadot_primitives :: v5 :: AbridgedHrmpChannel ,) > , }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct RelayDispachQueueSize {
                    pub remaining_count: ::core::primitive::u32,
                    pub remaining_size: ::core::primitive::u32,
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct CodeUpgradeAuthorization {
                pub code_hash: ::subxt::utils::H256,
                pub check_version: ::core::primitive::bool,
            }
        }
        pub mod cumulus_pallet_xcm {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {}
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Downward message is invalid XCM."]
                    #[doc = "\\[ id \\]"]
                    InvalidFormat([::core::primitive::u8; 32usize]),
                    #[codec(index = 1)]
                    #[doc = "Downward message is unsupported version of XCM."]
                    #[doc = "\\[ id \\]"]
                    UnsupportedVersion([::core::primitive::u8; 32usize]),
                    #[codec(index = 2)]
                    #[doc = "Downward message executed with the given outcome."]
                    #[doc = "\\[ id, outcome \\]"]
                    ExecutedDownward(
                        [::core::primitive::u8; 32usize],
                        runtime_types::xcm::v3::traits::Outcome,
                    ),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Origin {
                    #[codec(index = 0)]
                    Relay,
                    #[codec(index = 1)]
                    SiblingParachain(runtime_types::polkadot_parachain::primitives::Id),
                }
            }
        }
        pub mod cumulus_pallet_xcmp_queue {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::service_overweight`]."]
                    service_overweight {
                        index: ::core::primitive::u64,
                        weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::suspend_xcm_execution`]."]
                    suspend_xcm_execution,
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::resume_xcm_execution`]."]
                    resume_xcm_execution,
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::update_suspend_threshold`]."]
                    update_suspend_threshold { new: ::core::primitive::u32 },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::update_drop_threshold`]."]
                    update_drop_threshold { new: ::core::primitive::u32 },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::update_resume_threshold`]."]
                    update_resume_threshold { new: ::core::primitive::u32 },
                    #[codec(index = 6)]
                    #[doc = "See [`Pallet::update_threshold_weight`]."]
                    update_threshold_weight {
                        new: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 7)]
                    #[doc = "See [`Pallet::update_weight_restrict_decay`]."]
                    update_weight_restrict_decay {
                        new: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 8)]
                    #[doc = "See [`Pallet::update_xcmp_max_individual_weight`]."]
                    update_xcmp_max_individual_weight {
                        new: runtime_types::sp_weights::weight_v2::Weight,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Failed to send XCM message."]
                    FailedToSend,
                    #[codec(index = 1)]
                    #[doc = "Bad XCM origin."]
                    BadXcmOrigin,
                    #[codec(index = 2)]
                    #[doc = "Bad XCM data."]
                    BadXcm,
                    #[codec(index = 3)]
                    #[doc = "Bad overweight index."]
                    BadOverweightIndex,
                    #[codec(index = 4)]
                    #[doc = "Provided weight is possibly not enough to execute the message."]
                    WeightOverLimit,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Some XCM was executed ok."]
                    Success {
                        message_hash: [::core::primitive::u8; 32usize],
                        message_id: [::core::primitive::u8; 32usize],
                        weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 1)]
                    #[doc = "Some XCM failed."]
                    Fail {
                        message_hash: [::core::primitive::u8; 32usize],
                        message_id: [::core::primitive::u8; 32usize],
                        error: runtime_types::xcm::v3::traits::Error,
                        weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 2)]
                    #[doc = "Bad XCM version used."]
                    BadVersion {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 3)]
                    #[doc = "Bad XCM format used."]
                    BadFormat {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 4)]
                    #[doc = "An HRMP message was sent to a sibling parachain."]
                    XcmpMessageSent {
                        message_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 5)]
                    #[doc = "An XCM exceeded the individual message weight budget."]
                    OverweightEnqueued {
                        sender: runtime_types::polkadot_parachain::primitives::Id,
                        sent_at: ::core::primitive::u32,
                        index: ::core::primitive::u64,
                        required: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 6)]
                    #[doc = "An XCM from the overweight queue was executed with the given actual weight used."]
                    OverweightServiced {
                        index: ::core::primitive::u64,
                        used: runtime_types::sp_weights::weight_v2::Weight,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct InboundChannelDetails {
                pub sender: runtime_types::polkadot_parachain::primitives::Id,
                pub state: runtime_types::cumulus_pallet_xcmp_queue::InboundState,
                pub message_metadata: ::std::vec::Vec<(
                    ::core::primitive::u32,
                    runtime_types::polkadot_parachain::primitives::XcmpMessageFormat,
                )>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum InboundState {
                #[codec(index = 0)]
                Ok,
                #[codec(index = 1)]
                Suspended,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct OutboundChannelDetails {
                pub recipient: runtime_types::polkadot_parachain::primitives::Id,
                pub state: runtime_types::cumulus_pallet_xcmp_queue::OutboundState,
                pub signals_exist: ::core::primitive::bool,
                pub first_index: ::core::primitive::u16,
                pub last_index: ::core::primitive::u16,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum OutboundState {
                #[codec(index = 0)]
                Ok,
                #[codec(index = 1)]
                Suspended,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct QueueConfigData {
                pub suspend_threshold: ::core::primitive::u32,
                pub drop_threshold: ::core::primitive::u32,
                pub resume_threshold: ::core::primitive::u32,
                pub threshold_weight: runtime_types::sp_weights::weight_v2::Weight,
                pub weight_restrict_decay: runtime_types::sp_weights::weight_v2::Weight,
                pub xcmp_max_individual_weight: runtime_types::sp_weights::weight_v2::Weight,
            }
        }
        pub mod cumulus_primitives_parachain_inherent {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct MessageQueueChain(pub ::subxt::utils::H256);
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ParachainInherentData {
                pub validation_data:
                    runtime_types::polkadot_primitives::v5::PersistedValidationData<
                        ::subxt::utils::H256,
                        ::core::primitive::u32,
                    >,
                pub relay_chain_state: runtime_types::sp_trie::storage_proof::StorageProof,
                pub downward_messages: ::std::vec::Vec<
                    runtime_types::polkadot_core_primitives::InboundDownwardMessage<
                        ::core::primitive::u32,
                    >,
                >,
                pub horizontal_messages: ::subxt::utils::KeyedVec<
                    runtime_types::polkadot_parachain::primitives::Id,
                    ::std::vec::Vec<
                        runtime_types::polkadot_core_primitives::InboundHrmpMessage<
                            ::core::primitive::u32,
                        >,
                    >,
                >,
            }
        }
        pub mod finality_grandpa {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Commit<_0, _1, _2, _3> {
                pub target_hash: _0,
                pub target_number: _1,
                pub precommits: ::std::vec::Vec<
                    runtime_types::finality_grandpa::SignedPrecommit<_0, _1, _2, _3>,
                >,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Precommit<_0, _1> {
                pub target_hash: _0,
                pub target_number: _1,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct SignedPrecommit<_0, _1, _2, _3> {
                pub precommit: runtime_types::finality_grandpa::Precommit<_0, _1>,
                pub signature: _2,
                pub id: _3,
            }
        }
        pub mod frame_support {
            use super::runtime_types;
            pub mod dispatch {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum DispatchClass {
                    #[codec(index = 0)]
                    Normal,
                    #[codec(index = 1)]
                    Operational,
                    #[codec(index = 2)]
                    Mandatory,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct DispatchInfo {
                    pub weight: runtime_types::sp_weights::weight_v2::Weight,
                    pub class: runtime_types::frame_support::dispatch::DispatchClass,
                    pub pays_fee: runtime_types::frame_support::dispatch::Pays,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Pays {
                    #[codec(index = 0)]
                    Yes,
                    #[codec(index = 1)]
                    No,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct PerDispatchClass<_0> {
                    pub normal: _0,
                    pub operational: _0,
                    pub mandatory: _0,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum RawOrigin<_0> {
                    #[codec(index = 0)]
                    Root,
                    #[codec(index = 1)]
                    Signed(_0),
                    #[codec(index = 2)]
                    None,
                }
            }
            pub mod traits {
                use super::runtime_types;
                pub mod messages {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum ProcessMessageError {
                        #[codec(index = 0)]
                        BadFormat,
                        #[codec(index = 1)]
                        Corrupt,
                        #[codec(index = 2)]
                        Unsupported,
                        #[codec(index = 3)]
                        Overweight(runtime_types::sp_weights::weight_v2::Weight),
                        #[codec(index = 4)]
                        Yield,
                    }
                }
                pub mod tokens {
                    use super::runtime_types;
                    pub mod misc {
                        use super::runtime_types;
                        #[derive(
                            :: subxt :: ext :: codec :: Decode,
                            :: subxt :: ext :: codec :: Encode,
                            Debug,
                        )]
                        pub enum BalanceStatus {
                            #[codec(index = 0)]
                            Free,
                            #[codec(index = 1)]
                            Reserved,
                        }
                    }
                }
            }
        }
        pub mod frame_system {
            use super::runtime_types;
            pub mod extensions {
                use super::runtime_types;
                pub mod check_genesis {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckGenesis;
                }
                pub mod check_mortality {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckMortality(pub runtime_types::sp_runtime::generic::era::Era);
                }
                pub mod check_non_zero_sender {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckNonZeroSender;
                }
                pub mod check_nonce {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckNonce(#[codec(compact)] pub ::core::primitive::u32);
                }
                pub mod check_spec_version {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckSpecVersion;
                }
                pub mod check_tx_version {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckTxVersion;
                }
                pub mod check_weight {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct CheckWeight;
                }
            }
            pub mod limits {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BlockLength {
                    pub max: runtime_types::frame_support::dispatch::PerDispatchClass<
                        ::core::primitive::u32,
                    >,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BlockWeights {
                    pub base_block: runtime_types::sp_weights::weight_v2::Weight,
                    pub max_block: runtime_types::sp_weights::weight_v2::Weight,
                    pub per_class: runtime_types::frame_support::dispatch::PerDispatchClass<
                        runtime_types::frame_system::limits::WeightsPerClass,
                    >,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct WeightsPerClass {
                    pub base_extrinsic: runtime_types::sp_weights::weight_v2::Weight,
                    pub max_extrinsic:
                        ::core::option::Option<runtime_types::sp_weights::weight_v2::Weight>,
                    pub max_total:
                        ::core::option::Option<runtime_types::sp_weights::weight_v2::Weight>,
                    pub reserved:
                        ::core::option::Option<runtime_types::sp_weights::weight_v2::Weight>,
                }
            }
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::remark`]."]
                    remark {
                        remark: ::std::vec::Vec<::core::primitive::u8>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_heap_pages`]."]
                    set_heap_pages { pages: ::core::primitive::u64 },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::set_code`]."]
                    set_code {
                        code: ::std::vec::Vec<::core::primitive::u8>,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::set_code_without_checks`]."]
                    set_code_without_checks {
                        code: ::std::vec::Vec<::core::primitive::u8>,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::set_storage`]."]
                    set_storage {
                        items: ::std::vec::Vec<(
                            ::std::vec::Vec<::core::primitive::u8>,
                            ::std::vec::Vec<::core::primitive::u8>,
                        )>,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::kill_storage`]."]
                    kill_storage {
                        keys: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                    },
                    #[codec(index = 6)]
                    #[doc = "See [`Pallet::kill_prefix`]."]
                    kill_prefix {
                        prefix: ::std::vec::Vec<::core::primitive::u8>,
                        subkeys: ::core::primitive::u32,
                    },
                    #[codec(index = 7)]
                    #[doc = "See [`Pallet::remark_with_event`]."]
                    remark_with_event {
                        remark: ::std::vec::Vec<::core::primitive::u8>,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Error for the System pallet"]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The name of specification does not match between the current runtime"]
                    #[doc = "and the new runtime."]
                    InvalidSpecName,
                    #[codec(index = 1)]
                    #[doc = "The specification version is not allowed to decrease between the current runtime"]
                    #[doc = "and the new runtime."]
                    SpecVersionNeedsToIncrease,
                    #[codec(index = 2)]
                    #[doc = "Failed to extract the runtime version from the new runtime."]
                    #[doc = ""]
                    #[doc = "Either calling `Core_version` or decoding `RuntimeVersion` failed."]
                    FailedToExtractRuntimeVersion,
                    #[codec(index = 3)]
                    #[doc = "Suicide called when the account has non-default composite data."]
                    NonDefaultComposite,
                    #[codec(index = 4)]
                    #[doc = "There is a non-zero reference count preventing the account from being purged."]
                    NonZeroRefCount,
                    #[codec(index = 5)]
                    #[doc = "The origin filter prevent the call to be dispatched."]
                    CallFiltered,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Event for the System pallet."]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "An extrinsic completed successfully."]
                    ExtrinsicSuccess {
                        dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
                    },
                    #[codec(index = 1)]
                    #[doc = "An extrinsic failed."]
                    ExtrinsicFailed {
                        dispatch_error: runtime_types::sp_runtime::DispatchError,
                        dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
                    },
                    #[codec(index = 2)]
                    #[doc = "`:code` was updated."]
                    CodeUpdated,
                    #[codec(index = 3)]
                    #[doc = "A new account was created."]
                    NewAccount {
                        account: ::subxt::utils::AccountId32,
                    },
                    #[codec(index = 4)]
                    #[doc = "An account was reaped."]
                    KilledAccount {
                        account: ::subxt::utils::AccountId32,
                    },
                    #[codec(index = 5)]
                    #[doc = "On on-chain remark happened."]
                    Remarked {
                        sender: ::subxt::utils::AccountId32,
                        hash: ::subxt::utils::H256,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct AccountInfo<_0, _1> {
                pub nonce: _0,
                pub consumers: _0,
                pub providers: _0,
                pub sufficients: _0,
                pub data: _1,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct EventRecord<_0, _1> {
                pub phase: runtime_types::frame_system::Phase,
                pub event: _0,
                pub topics: ::std::vec::Vec<_1>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct LastRuntimeUpgradeInfo {
                #[codec(compact)]
                pub spec_version: ::core::primitive::u32,
                pub spec_name: ::std::string::String,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum Phase {
                #[codec(index = 0)]
                ApplyExtrinsic(::core::primitive::u32),
                #[codec(index = 1)]
                Finalization,
                #[codec(index = 2)]
                Initialization,
            }
        }
        pub mod milagro_bls {
            use super::runtime_types;
            pub mod keys {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct PublicKey {
                    pub point: runtime_types::amcl::bls381::ecp::ECP,
                }
            }
        }
        pub mod pallet_balances {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::transfer_allow_death`]."]
                    transfer_allow_death {
                        dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        value: ::core::primitive::u128,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_balance_deprecated`]."]
                    set_balance_deprecated {
                        who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        new_free: ::core::primitive::u128,
                        #[codec(compact)]
                        old_reserved: ::core::primitive::u128,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::force_transfer`]."]
                    force_transfer {
                        source: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        value: ::core::primitive::u128,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::transfer_keep_alive`]."]
                    transfer_keep_alive {
                        dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        value: ::core::primitive::u128,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::transfer_all`]."]
                    transfer_all {
                        dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        keep_alive: ::core::primitive::bool,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::force_unreserve`]."]
                    force_unreserve {
                        who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 6)]
                    #[doc = "See [`Pallet::upgrade_accounts`]."]
                    upgrade_accounts {
                        who: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 7)]
                    #[doc = "See [`Pallet::transfer`]."]
                    transfer {
                        dest: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        value: ::core::primitive::u128,
                    },
                    #[codec(index = 8)]
                    #[doc = "See [`Pallet::force_set_balance`]."]
                    force_set_balance {
                        who: ::subxt::utils::MultiAddress<::subxt::utils::AccountId32, ()>,
                        #[codec(compact)]
                        new_free: ::core::primitive::u128,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Vesting balance too high to send value."]
                    VestingBalance,
                    #[codec(index = 1)]
                    #[doc = "Account liquidity restrictions prevent withdrawal."]
                    LiquidityRestrictions,
                    #[codec(index = 2)]
                    #[doc = "Balance too low to send value."]
                    InsufficientBalance,
                    #[codec(index = 3)]
                    #[doc = "Value too low to create account due to existential deposit."]
                    ExistentialDeposit,
                    #[codec(index = 4)]
                    #[doc = "Transfer/payment would kill account."]
                    Expendability,
                    #[codec(index = 5)]
                    #[doc = "A vesting schedule already exists for this account."]
                    ExistingVestingSchedule,
                    #[codec(index = 6)]
                    #[doc = "Beneficiary account must pre-exist."]
                    DeadAccount,
                    #[codec(index = 7)]
                    #[doc = "Number of named reserves exceed `MaxReserves`."]
                    TooManyReserves,
                    #[codec(index = 8)]
                    #[doc = "Number of holds exceed `MaxHolds`."]
                    TooManyHolds,
                    #[codec(index = 9)]
                    #[doc = "Number of freezes exceed `MaxFreezes`."]
                    TooManyFreezes,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "An account was created with some free balance."]
                    Endowed {
                        account: ::subxt::utils::AccountId32,
                        free_balance: ::core::primitive::u128,
                    },
                    #[codec(index = 1)]
                    #[doc = "An account was removed whose balance was non-zero but below ExistentialDeposit,"]
                    #[doc = "resulting in an outright loss."]
                    DustLost {
                        account: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 2)]
                    #[doc = "Transfer succeeded."]
                    Transfer {
                        from: ::subxt::utils::AccountId32,
                        to: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 3)]
                    #[doc = "A balance was set by root."]
                    BalanceSet {
                        who: ::subxt::utils::AccountId32,
                        free: ::core::primitive::u128,
                    },
                    #[codec(index = 4)]
                    #[doc = "Some balance was reserved (moved from free to reserved)."]
                    Reserved {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 5)]
                    #[doc = "Some balance was unreserved (moved from reserved to free)."]
                    Unreserved {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 6)]
                    #[doc = "Some balance was moved from the reserve of the first account to the second account."]
                    #[doc = "Final argument indicates the destination balance type."]
                    ReserveRepatriated {
                        from: ::subxt::utils::AccountId32,
                        to: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                        destination_status:
                            runtime_types::frame_support::traits::tokens::misc::BalanceStatus,
                    },
                    #[codec(index = 7)]
                    #[doc = "Some amount was deposited (e.g. for transaction fees)."]
                    Deposit {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 8)]
                    #[doc = "Some amount was withdrawn from the account (e.g. for transaction fees)."]
                    Withdraw {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 9)]
                    #[doc = "Some amount was removed from the account (e.g. for misbehavior)."]
                    Slashed {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 10)]
                    #[doc = "Some amount was minted into an account."]
                    Minted {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 11)]
                    #[doc = "Some amount was burned from an account."]
                    Burned {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 12)]
                    #[doc = "Some amount was suspended from an account (it can be restored later)."]
                    Suspended {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 13)]
                    #[doc = "Some amount was restored into an account."]
                    Restored {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 14)]
                    #[doc = "An account was upgraded."]
                    Upgraded { who: ::subxt::utils::AccountId32 },
                    #[codec(index = 15)]
                    #[doc = "Total issuance was increased by `amount`, creating a credit to be balanced."]
                    Issued { amount: ::core::primitive::u128 },
                    #[codec(index = 16)]
                    #[doc = "Total issuance was decreased by `amount`, creating a debt to be balanced."]
                    Rescinded { amount: ::core::primitive::u128 },
                    #[codec(index = 17)]
                    #[doc = "Some balance was locked."]
                    Locked {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 18)]
                    #[doc = "Some balance was unlocked."]
                    Unlocked {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 19)]
                    #[doc = "Some balance was frozen."]
                    Frozen {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                    #[codec(index = 20)]
                    #[doc = "Some balance was thawed."]
                    Thawed {
                        who: ::subxt::utils::AccountId32,
                        amount: ::core::primitive::u128,
                    },
                }
            }
            pub mod types {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct AccountData<_0> {
                    pub free: _0,
                    pub reserved: _0,
                    pub frozen: _0,
                    pub flags: runtime_types::pallet_balances::types::ExtraFlags,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BalanceLock<_0> {
                    pub id: [::core::primitive::u8; 8usize],
                    pub amount: _0,
                    pub reasons: runtime_types::pallet_balances::types::Reasons,
                }
                #[derive(
                    :: subxt :: ext :: codec :: CompactAs,
                    :: subxt :: ext :: codec :: Decode,
                    :: subxt :: ext :: codec :: Encode,
                    Debug,
                )]
                pub struct ExtraFlags(pub ::core::primitive::u128);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct IdAmount<_0, _1> {
                    pub id: _0,
                    pub amount: _1,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Reasons {
                    #[codec(index = 0)]
                    Fee,
                    #[codec(index = 1)]
                    Misc,
                    #[codec(index = 2)]
                    All,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ReserveData<_0, _1> {
                    pub id: _0,
                    pub amount: _1,
                }
            }
        }
        pub mod pallet_bridge_grandpa {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::submit_finality_proof`]."]
                    submit_finality_proof {
                        finality_target: ::std::boxed::Box<
                            runtime_types::sp_runtime::generic::header::Header<
                                ::core::primitive::u32,
                                runtime_types::sp_runtime::traits::BlakeTwo256,
                            >,
                        >,
                        justification:
                            runtime_types::bp_header_chain::justification::GrandpaJustification<
                                runtime_types::sp_runtime::generic::header::Header<
                                    ::core::primitive::u32,
                                    runtime_types::sp_runtime::traits::BlakeTwo256,
                                >,
                            >,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::initialize`]."]
                    initialize {
                        init_data: runtime_types::bp_header_chain::InitializationData<
                            runtime_types::sp_runtime::generic::header::Header<
                                ::core::primitive::u32,
                                runtime_types::sp_runtime::traits::BlakeTwo256,
                            >,
                        >,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::set_owner`]."]
                    set_owner {
                        new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::set_operating_mode`]."]
                    set_operating_mode {
                        operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The given justification is invalid for the given header."]
                    InvalidJustification,
                    #[codec(index = 1)]
                    #[doc = "The authority set from the underlying header chain is invalid."]
                    InvalidAuthoritySet,
                    #[codec(index = 2)]
                    #[doc = "The header being imported is older than the best finalized header known to the pallet."]
                    OldHeader,
                    #[codec(index = 3)]
                    #[doc = "The scheduled authority set change found in the header is unsupported by the pallet."]
                    #[doc = ""]
                    #[doc = "This is the case for non-standard (e.g forced) authority set changes."]
                    UnsupportedScheduledChange,
                    #[codec(index = 4)]
                    #[doc = "The pallet is not yet initialized."]
                    NotInitialized,
                    #[codec(index = 5)]
                    #[doc = "The pallet has already been initialized."]
                    AlreadyInitialized,
                    #[codec(index = 6)]
                    #[doc = "Too many authorities in the set."]
                    TooManyAuthoritiesInSet,
                    #[codec(index = 7)]
                    #[doc = "Error generated by the `OwnedBridgeModule` trait."]
                    BridgeModule(runtime_types::bp_runtime::OwnedBridgeModuleError),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Best finalized chain header has been updated to the header with given number and hash."]
                    UpdatedBestFinalizedHeader {
                        number: ::core::primitive::u32,
                        hash: ::subxt::utils::H256,
                    },
                }
            }
            pub mod storage_types {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct StoredAuthoritySet {
                    pub authorities: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
                        runtime_types::sp_consensus_grandpa::app::Public,
                        ::core::primitive::u64,
                    )>,
                    pub set_id: ::core::primitive::u64,
                }
            }
        }
        pub mod pallet_bridge_messages {
            use super::runtime_types;
            pub mod outbound_lane {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum ReceivalConfirmationError {
                    #[codec(index = 0)]
                    FailedToConfirmFutureMessages,
                    #[codec(index = 1)]
                    EmptyUnrewardedRelayerEntry,
                    #[codec(index = 2)]
                    NonConsecutiveUnrewardedRelayerEntries,
                    #[codec(index = 3)]
                    TryingToConfirmMoreMessagesThanExpected,
                }
            }
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    # [codec (index = 0)] # [doc = "See [`Pallet::set_owner`]."] set_owner { new_owner : :: core :: option :: Option < :: subxt :: utils :: AccountId32 > , } , # [codec (index = 1)] # [doc = "See [`Pallet::set_operating_mode`]."] set_operating_mode { operating_mode : runtime_types :: bp_messages :: MessagesOperatingMode , } , # [codec (index = 2)] # [doc = "See [`Pallet::receive_messages_proof`]."] receive_messages_proof { relayer_id_at_bridged_chain : :: subxt :: utils :: AccountId32 , proof : runtime_types :: bridge_runtime_common :: messages :: target :: FromBridgedChainMessagesProof < :: subxt :: utils :: H256 > , messages_count : :: core :: primitive :: u32 , dispatch_weight : runtime_types :: sp_weights :: weight_v2 :: Weight , } , # [codec (index = 3)] # [doc = "See [`Pallet::receive_messages_delivery_proof`]."] receive_messages_delivery_proof { proof : runtime_types :: bridge_runtime_common :: messages :: source :: FromBridgedChainMessagesDeliveryProof < :: subxt :: utils :: H256 > , relayers_state : runtime_types :: bp_messages :: UnrewardedRelayersState , } , }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    # [codec (index = 0)] # [doc = "Pallet is not in Normal operating mode."] NotOperatingNormally , # [codec (index = 1)] # [doc = "The outbound lane is inactive."] InactiveOutboundLane , # [codec (index = 2)] # [doc = "Message has been treated as invalid by chain verifier."] MessageRejectedByChainVerifier (runtime_types :: bp_messages :: VerificationError ,) , # [codec (index = 3)] # [doc = "Message has been treated as invalid by lane verifier."] MessageRejectedByLaneVerifier (runtime_types :: bp_messages :: VerificationError ,) , # [codec (index = 4)] # [doc = "Message has been treated as invalid by the pallet logic."] MessageRejectedByPallet (runtime_types :: bp_messages :: VerificationError ,) , # [codec (index = 5)] # [doc = "Submitter has failed to pay fee for delivering and dispatching messages."] FailedToWithdrawMessageFee , # [codec (index = 6)] # [doc = "The transaction brings too many messages."] TooManyMessagesInTheProof , # [codec (index = 7)] # [doc = "Invalid messages has been submitted."] InvalidMessagesProof , # [codec (index = 8)] # [doc = "Invalid messages delivery proof has been submitted."] InvalidMessagesDeliveryProof , # [codec (index = 9)] # [doc = "The relayer has declared invalid unrewarded relayers state in the"] # [doc = "`receive_messages_delivery_proof` call."] InvalidUnrewardedRelayersState , # [codec (index = 10)] # [doc = "The cumulative dispatch weight, passed by relayer is not enough to cover dispatch"] # [doc = "of all bundled messages."] InsufficientDispatchWeight , # [codec (index = 11)] # [doc = "The message someone is trying to work with (i.e. increase fee) is not yet sent."] MessageIsNotYetSent , # [codec (index = 12)] # [doc = "Error confirming messages receival."] ReceivalConfirmation (runtime_types :: pallet_bridge_messages :: outbound_lane :: ReceivalConfirmationError ,) , # [codec (index = 13)] # [doc = "Error generated by the `OwnedBridgeModule` trait."] BridgeModule (runtime_types :: bp_runtime :: OwnedBridgeModuleError ,) , }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    # [codec (index = 0)] # [doc = "Message has been accepted and is waiting to be delivered."] MessageAccepted { lane_id : runtime_types :: bp_messages :: LaneId , nonce : :: core :: primitive :: u64 , } , # [codec (index = 1)] # [doc = "Messages have been received from the bridged chain."] MessagesReceived (:: std :: vec :: Vec < runtime_types :: bp_messages :: ReceivedMessages < runtime_types :: bridge_runtime_common :: messages_xcm_extension :: XcmBlobMessageDispatchResult > > ,) , # [codec (index = 2)] # [doc = "Messages in the inclusive range have been delivered to the bridged chain."] MessagesDelivered { lane_id : runtime_types :: bp_messages :: LaneId , messages : runtime_types :: bp_messages :: DeliveredMessages , } , }
            }
        }
        pub mod pallet_bridge_parachains {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::submit_parachain_heads`]."]
                    submit_parachain_heads {
                        at_relay_block: (::core::primitive::u32, ::subxt::utils::H256),
                        parachains: ::std::vec::Vec<(
                            runtime_types::bp_polkadot_core::parachains::ParaId,
                            ::subxt::utils::H256,
                        )>,
                        parachain_heads_proof:
                            runtime_types::bp_polkadot_core::parachains::ParaHeadsProof,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_owner`]."]
                    set_owner {
                        new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::set_operating_mode`]."]
                    set_operating_mode {
                        operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Relay chain block hash is unknown to us."]
                    UnknownRelayChainBlock,
                    #[codec(index = 1)]
                    #[doc = "The number of stored relay block is different from what the relayer has provided."]
                    InvalidRelayChainBlockNumber,
                    #[codec(index = 2)]
                    #[doc = "Parachain heads storage proof is invalid."]
                    HeaderChainStorageProof(runtime_types::bp_header_chain::HeaderChainError),
                    #[codec(index = 3)]
                    #[doc = "Error generated by the `OwnedBridgeModule` trait."]
                    BridgeModule(runtime_types::bp_runtime::OwnedBridgeModuleError),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "The caller has provided head of parachain that the pallet is not configured to track."]
                    UntrackedParachainRejected {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                    },
                    #[codec(index = 1)]
                    #[doc = "The caller has declared that he has provided given parachain head, but it is missing"]
                    #[doc = "from the storage proof."]
                    MissingParachainHead {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                    },
                    #[codec(index = 2)]
                    #[doc = "The caller has provided parachain head hash that is not matching the hash read from the"]
                    #[doc = "storage proof."]
                    IncorrectParachainHeadHash {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                        parachain_head_hash: ::subxt::utils::H256,
                        actual_parachain_head_hash: ::subxt::utils::H256,
                    },
                    #[codec(index = 3)]
                    #[doc = "The caller has provided obsolete parachain head, which is already known to the pallet."]
                    RejectedObsoleteParachainHead {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                        parachain_head_hash: ::subxt::utils::H256,
                    },
                    #[codec(index = 4)]
                    #[doc = "The caller has provided parachain head that exceeds the maximal configured head size."]
                    RejectedLargeParachainHead {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                        parachain_head_hash: ::subxt::utils::H256,
                        parachain_head_size: ::core::primitive::u32,
                    },
                    #[codec(index = 5)]
                    #[doc = "Parachain head has been updated."]
                    UpdatedParachainHead {
                        parachain: runtime_types::bp_polkadot_core::parachains::ParaId,
                        parachain_head_hash: ::subxt::utils::H256,
                    },
                }
            }
        }
        pub mod pallet_bridge_relayers {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::claim_rewards`]."]
                    claim_rewards {
                        rewards_account_params: runtime_types::bp_relayers::RewardsAccountParams,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::register`]."]
                    register { valid_till: ::core::primitive::u32 },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::deregister`]."]
                    deregister,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "No reward can be claimed by given relayer."]
                    NoRewardForRelayer,
                    #[codec(index = 1)]
                    #[doc = "Reward payment procedure has failed."]
                    FailedToPayReward,
                    #[codec(index = 2)]
                    #[doc = "The relayer has tried to register for past block or registration lease"]
                    #[doc = "is too short."]
                    InvalidRegistrationLease,
                    #[codec(index = 3)]
                    #[doc = "New registration lease is less than the previous one."]
                    CannotReduceRegistrationLease,
                    #[codec(index = 4)]
                    #[doc = "Failed to reserve enough funds on relayer account."]
                    FailedToReserve,
                    #[codec(index = 5)]
                    #[doc = "Failed to `unreserve` enough funds on relayer account."]
                    FailedToUnreserve,
                    #[codec(index = 6)]
                    #[doc = "Cannot `deregister` if not registered."]
                    NotRegistered,
                    #[codec(index = 7)]
                    #[doc = "Failed to `deregister` relayer, because lease is still active."]
                    RegistrationIsStillActive,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Reward has been paid to the relayer."]
                    RewardPaid {
                        relayer: ::subxt::utils::AccountId32,
                        rewards_account_params: runtime_types::bp_relayers::RewardsAccountParams,
                        reward: ::core::primitive::u128,
                    },
                    #[codec(index = 1)]
                    #[doc = "Relayer registration has been added or updated."]
                    RegistrationUpdated {
                        relayer: ::subxt::utils::AccountId32,
                        registration: runtime_types::bp_relayers::registration::Registration<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                        >,
                    },
                    #[codec(index = 2)]
                    #[doc = "Relayer has been `deregistered`."]
                    Deregistered {
                        relayer: ::subxt::utils::AccountId32,
                    },
                    #[codec(index = 3)]
                    #[doc = "Relayer has been slashed and `deregistered`."]
                    SlashedAndDeregistered {
                        relayer: ::subxt::utils::AccountId32,
                        registration: runtime_types::bp_relayers::registration::Registration<
                            ::core::primitive::u32,
                            ::core::primitive::u128,
                        >,
                    },
                }
            }
        }
        pub mod pallet_collator_selection {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::set_invulnerables`]."]
                    set_invulnerables {
                        new: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_desired_candidates`]."]
                    set_desired_candidates { max: ::core::primitive::u32 },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::set_candidacy_bond`]."]
                    set_candidacy_bond { bond: ::core::primitive::u128 },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::register_as_candidate`]."]
                    register_as_candidate,
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::leave_intent`]."]
                    leave_intent,
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::add_invulnerable`]."]
                    add_invulnerable { who: ::subxt::utils::AccountId32 },
                    #[codec(index = 6)]
                    #[doc = "See [`Pallet::remove_invulnerable`]."]
                    remove_invulnerable { who: ::subxt::utils::AccountId32 },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct CandidateInfo<_0, _1> {
                    pub who: _0,
                    pub deposit: _1,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The pallet has too many candidates."]
                    TooManyCandidates,
                    #[codec(index = 1)]
                    #[doc = "Leaving would result in too few candidates."]
                    TooFewCandidates,
                    #[codec(index = 2)]
                    #[doc = "Account is already a candidate."]
                    AlreadyCandidate,
                    #[codec(index = 3)]
                    #[doc = "Account is not a candidate."]
                    NotCandidate,
                    #[codec(index = 4)]
                    #[doc = "There are too many Invulnerables."]
                    TooManyInvulnerables,
                    #[codec(index = 5)]
                    #[doc = "Account is already an Invulnerable."]
                    AlreadyInvulnerable,
                    #[codec(index = 6)]
                    #[doc = "Account is not an Invulnerable."]
                    NotInvulnerable,
                    #[codec(index = 7)]
                    #[doc = "Account has no associated validator ID."]
                    NoAssociatedValidatorId,
                    #[codec(index = 8)]
                    #[doc = "Validator ID is not yet registered."]
                    ValidatorNotRegistered,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "New Invulnerables were set."]
                    NewInvulnerables {
                        invulnerables: ::std::vec::Vec<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 1)]
                    #[doc = "A new Invulnerable was added."]
                    InvulnerableAdded {
                        account_id: ::subxt::utils::AccountId32,
                    },
                    #[codec(index = 2)]
                    #[doc = "An Invulnerable was removed."]
                    InvulnerableRemoved {
                        account_id: ::subxt::utils::AccountId32,
                    },
                    #[codec(index = 3)]
                    #[doc = "The number of desired candidates was set."]
                    NewDesiredCandidates {
                        desired_candidates: ::core::primitive::u32,
                    },
                    #[codec(index = 4)]
                    #[doc = "The candidacy bond was set."]
                    NewCandidacyBond {
                        bond_amount: ::core::primitive::u128,
                    },
                    #[codec(index = 5)]
                    #[doc = "A new candidate joined."]
                    CandidateAdded {
                        account_id: ::subxt::utils::AccountId32,
                        deposit: ::core::primitive::u128,
                    },
                    #[codec(index = 6)]
                    #[doc = "A candidate was removed."]
                    CandidateRemoved {
                        account_id: ::subxt::utils::AccountId32,
                    },
                }
            }
        }
        pub mod pallet_message_queue {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::reap_page`]."]
                    reap_page {
                        message_origin:
                            runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        page_index: ::core::primitive::u32,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::execute_overweight`]."]
                    execute_overweight {
                        message_origin:
                            runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        page: ::core::primitive::u32,
                        index: ::core::primitive::u32,
                        weight_limit: runtime_types::sp_weights::weight_v2::Weight,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Page is not reapable because it has items remaining to be processed and is not old"]
                    #[doc = "enough."]
                    NotReapable,
                    #[codec(index = 1)]
                    #[doc = "Page to be reaped does not exist."]
                    NoPage,
                    #[codec(index = 2)]
                    #[doc = "The referenced message could not be found."]
                    NoMessage,
                    #[codec(index = 3)]
                    #[doc = "The message was already processed and cannot be processed again."]
                    AlreadyProcessed,
                    #[codec(index = 4)]
                    #[doc = "The message is queued for future execution."]
                    Queued,
                    #[codec(index = 5)]
                    #[doc = "There is temporarily not enough weight to continue servicing messages."]
                    InsufficientWeight,
                    #[codec(index = 6)]
                    #[doc = "This message is temporarily unprocessable."]
                    #[doc = ""]
                    #[doc = "Such errors are expected, but not guaranteed, to resolve themselves eventually through"]
                    #[doc = "retrying."]
                    TemporarilyUnprocessable,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Message discarded due to an error in the `MessageProcessor` (usually a format error)."]
                    ProcessingFailed {
                        id: [::core::primitive::u8; 32usize],
                        origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        error: runtime_types::frame_support::traits::messages::ProcessMessageError,
                    },
                    #[codec(index = 1)]
                    #[doc = "Message is processed."]
                    Processed {
                        id: [::core::primitive::u8; 32usize],
                        origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        weight_used: runtime_types::sp_weights::weight_v2::Weight,
                        success: ::core::primitive::bool,
                    },
                    #[codec(index = 2)]
                    #[doc = "Message placed in overweight queue."]
                    OverweightEnqueued {
                        id: [::core::primitive::u8; 32usize],
                        origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        page_index: ::core::primitive::u32,
                        message_index: ::core::primitive::u32,
                    },
                    #[codec(index = 3)]
                    #[doc = "This page was reaped."]
                    PageReaped {
                        origin: runtime_types::snowbridge_outbound_queue::AggregateMessageOrigin,
                        index: ::core::primitive::u32,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct BookState<_0> {
                pub begin: ::core::primitive::u32,
                pub end: ::core::primitive::u32,
                pub count: ::core::primitive::u32,
                pub ready_neighbours:
                    ::core::option::Option<runtime_types::pallet_message_queue::Neighbours<_0>>,
                pub message_count: ::core::primitive::u64,
                pub size: ::core::primitive::u64,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Neighbours<_0> {
                pub prev: _0,
                pub next: _0,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Page<_0> {
                pub remaining: _0,
                pub remaining_size: _0,
                pub first_index: _0,
                pub first: _0,
                pub last: _0,
                pub heap: runtime_types::bounded_collections::bounded_vec::BoundedVec<
                    ::core::primitive::u8,
                >,
            }
        }
        pub mod pallet_multisig {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::as_multi_threshold_1`]."]
                    as_multi_threshold_1 {
                        other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                        call: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                        >,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::as_multi`]."]
                    as_multi {
                        threshold: ::core::primitive::u16,
                        other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                        maybe_timepoint: ::core::option::Option<
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        >,
                        call: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                        >,
                        max_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::approve_as_multi`]."]
                    approve_as_multi {
                        threshold: ::core::primitive::u16,
                        other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                        maybe_timepoint: ::core::option::Option<
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        >,
                        call_hash: [::core::primitive::u8; 32usize],
                        max_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::cancel_as_multi`]."]
                    cancel_as_multi {
                        threshold: ::core::primitive::u16,
                        other_signatories: ::std::vec::Vec<::subxt::utils::AccountId32>,
                        timepoint:
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        call_hash: [::core::primitive::u8; 32usize],
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Threshold must be 2 or greater."]
                    MinimumThreshold,
                    #[codec(index = 1)]
                    #[doc = "Call is already approved by this signatory."]
                    AlreadyApproved,
                    #[codec(index = 2)]
                    #[doc = "Call doesn't need any (more) approvals."]
                    NoApprovalsNeeded,
                    #[codec(index = 3)]
                    #[doc = "There are too few signatories in the list."]
                    TooFewSignatories,
                    #[codec(index = 4)]
                    #[doc = "There are too many signatories in the list."]
                    TooManySignatories,
                    #[codec(index = 5)]
                    #[doc = "The signatories were provided out of order; they should be ordered."]
                    SignatoriesOutOfOrder,
                    #[codec(index = 6)]
                    #[doc = "The sender was contained in the other signatories; it shouldn't be."]
                    SenderInSignatories,
                    #[codec(index = 7)]
                    #[doc = "Multisig operation not found when attempting to cancel."]
                    NotFound,
                    #[codec(index = 8)]
                    #[doc = "Only the account that originally created the multisig is able to cancel it."]
                    NotOwner,
                    #[codec(index = 9)]
                    #[doc = "No timepoint was given, yet the multisig operation is already underway."]
                    NoTimepoint,
                    #[codec(index = 10)]
                    #[doc = "A different timepoint was given to the multisig operation that is underway."]
                    WrongTimepoint,
                    #[codec(index = 11)]
                    #[doc = "A timepoint was given, yet no multisig operation is underway."]
                    UnexpectedTimepoint,
                    #[codec(index = 12)]
                    #[doc = "The maximum weight information provided was too low."]
                    MaxWeightTooLow,
                    #[codec(index = 13)]
                    #[doc = "The data to be stored is already stored."]
                    AlreadyStored,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "A new multisig operation has begun."]
                    NewMultisig {
                        approving: ::subxt::utils::AccountId32,
                        multisig: ::subxt::utils::AccountId32,
                        call_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 1)]
                    #[doc = "A multisig operation has been approved by someone."]
                    MultisigApproval {
                        approving: ::subxt::utils::AccountId32,
                        timepoint:
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        multisig: ::subxt::utils::AccountId32,
                        call_hash: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 2)]
                    #[doc = "A multisig operation has been executed."]
                    MultisigExecuted {
                        approving: ::subxt::utils::AccountId32,
                        timepoint:
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        multisig: ::subxt::utils::AccountId32,
                        call_hash: [::core::primitive::u8; 32usize],
                        result:
                            ::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
                    },
                    #[codec(index = 3)]
                    #[doc = "A multisig operation has been cancelled."]
                    MultisigCancelled {
                        cancelling: ::subxt::utils::AccountId32,
                        timepoint:
                            runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
                        multisig: ::subxt::utils::AccountId32,
                        call_hash: [::core::primitive::u8; 32usize],
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Multisig<_0, _1, _2> {
                pub when: runtime_types::pallet_multisig::Timepoint<_0>,
                pub deposit: _1,
                pub depositor: _2,
                pub approvals: runtime_types::bounded_collections::bounded_vec::BoundedVec<_2>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct Timepoint<_0> {
                pub height: _0,
                pub index: _0,
            }
        }
        pub mod pallet_session {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::set_keys`]."]
                    set_keys {
                        keys: runtime_types::bridge_hub_rococo_runtime::SessionKeys,
                        proof: ::std::vec::Vec<::core::primitive::u8>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::purge_keys`]."]
                    purge_keys,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Error for the session pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Invalid ownership proof."]
                    InvalidProof,
                    #[codec(index = 1)]
                    #[doc = "No associated validator ID for account."]
                    NoAssociatedValidatorId,
                    #[codec(index = 2)]
                    #[doc = "Registered duplicate key."]
                    DuplicatedKey,
                    #[codec(index = 3)]
                    #[doc = "No keys are associated with this account."]
                    NoKeys,
                    #[codec(index = 4)]
                    #[doc = "Key setting account is not live, so it's impossible to associate keys."]
                    NoAccount,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "New session has happened. Note that the argument is the session index, not the"]
                    #[doc = "block number as the type might suggest."]
                    NewSession {
                        session_index: ::core::primitive::u32,
                    },
                }
            }
        }
        pub mod pallet_timestamp {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::set`]."]
                    set {
                        #[codec(compact)]
                        now: ::core::primitive::u64,
                    },
                }
            }
        }
        pub mod pallet_transaction_payment {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,"]
                    #[doc = "has been paid by `who`."]
                    TransactionFeePaid {
                        who: ::subxt::utils::AccountId32,
                        actual_fee: ::core::primitive::u128,
                        tip: ::core::primitive::u128,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ChargeTransactionPayment(#[codec(compact)] pub ::core::primitive::u128);
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum Releases {
                #[codec(index = 0)]
                V1Ancient,
                #[codec(index = 1)]
                V2,
            }
        }
        pub mod pallet_utility {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::batch`]."]
                    batch {
                        calls:
                            ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::as_derivative`]."]
                    as_derivative {
                        index: ::core::primitive::u16,
                        call: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                        >,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::batch_all`]."]
                    batch_all {
                        calls:
                            ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::dispatch_as`]."]
                    dispatch_as {
                        as_origin: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::OriginCaller,
                        >,
                        call: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                        >,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::force_batch`]."]
                    force_batch {
                        calls:
                            ::std::vec::Vec<runtime_types::bridge_hub_rococo_runtime::RuntimeCall>,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::with_weight`]."]
                    with_weight {
                        call: ::std::boxed::Box<
                            runtime_types::bridge_hub_rococo_runtime::RuntimeCall,
                        >,
                        weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Too many calls batched."]
                    TooManyCalls,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Batch of dispatches did not complete fully. Index of first failing dispatch given, as"]
                    #[doc = "well as the error."]
                    BatchInterrupted {
                        index: ::core::primitive::u32,
                        error: runtime_types::sp_runtime::DispatchError,
                    },
                    #[codec(index = 1)]
                    #[doc = "Batch of dispatches completed fully with no error."]
                    BatchCompleted,
                    #[codec(index = 2)]
                    #[doc = "Batch of dispatches completed but has errors."]
                    BatchCompletedWithErrors,
                    #[codec(index = 3)]
                    #[doc = "A single item within a Batch of dispatches has completed with no error."]
                    ItemCompleted,
                    #[codec(index = 4)]
                    #[doc = "A single item within a Batch of dispatches has completed with error."]
                    ItemFailed {
                        error: runtime_types::sp_runtime::DispatchError,
                    },
                    #[codec(index = 5)]
                    #[doc = "A call was dispatched."]
                    DispatchedAs {
                        result:
                            ::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
                    },
                }
            }
        }
        pub mod pallet_xcm {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::send`]."]
                    send {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::teleport_assets`]."]
                    teleport_assets {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                        fee_asset_item: ::core::primitive::u32,
                    },
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::reserve_transfer_assets`]."]
                    reserve_transfer_assets {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                        fee_asset_item: ::core::primitive::u32,
                    },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::execute`]."]
                    execute {
                        message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
                        max_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::force_xcm_version`]."]
                    force_xcm_version {
                        location:
                            ::std::boxed::Box<runtime_types::xcm::v3::multilocation::MultiLocation>,
                        version: ::core::primitive::u32,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::force_default_xcm_version`]."]
                    force_default_xcm_version {
                        maybe_xcm_version: ::core::option::Option<::core::primitive::u32>,
                    },
                    #[codec(index = 6)]
                    #[doc = "See [`Pallet::force_subscribe_version_notify`]."]
                    force_subscribe_version_notify {
                        location: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                    },
                    #[codec(index = 7)]
                    #[doc = "See [`Pallet::force_unsubscribe_version_notify`]."]
                    force_unsubscribe_version_notify {
                        location: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                    },
                    #[codec(index = 8)]
                    #[doc = "See [`Pallet::limited_reserve_transfer_assets`]."]
                    limited_reserve_transfer_assets {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                        fee_asset_item: ::core::primitive::u32,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 9)]
                    #[doc = "See [`Pallet::limited_teleport_assets`]."]
                    limited_teleport_assets {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        beneficiary: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        assets: ::std::boxed::Box<runtime_types::xcm::VersionedMultiAssets>,
                        fee_asset_item: ::core::primitive::u32,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 10)]
                    #[doc = "See [`Pallet::force_suspension`]."]
                    force_suspension { suspended: ::core::primitive::bool },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The desired destination was unreachable, generally because there is a no way of routing"]
                    #[doc = "to it."]
                    Unreachable,
                    #[codec(index = 1)]
                    #[doc = "There was some other issue (i.e. not to do with routing) in sending the message. Perhaps"]
                    #[doc = "a lack of space for buffering the message."]
                    SendFailure,
                    #[codec(index = 2)]
                    #[doc = "The message execution fails the filter."]
                    Filtered,
                    #[codec(index = 3)]
                    #[doc = "The message's weight could not be determined."]
                    UnweighableMessage,
                    #[codec(index = 4)]
                    #[doc = "The destination `MultiLocation` provided cannot be inverted."]
                    DestinationNotInvertible,
                    #[codec(index = 5)]
                    #[doc = "The assets to be sent are empty."]
                    Empty,
                    #[codec(index = 6)]
                    #[doc = "Could not re-anchor the assets to declare the fees for the destination chain."]
                    CannotReanchor,
                    #[codec(index = 7)]
                    #[doc = "Too many assets have been attempted for transfer."]
                    TooManyAssets,
                    #[codec(index = 8)]
                    #[doc = "Origin is invalid for sending."]
                    InvalidOrigin,
                    #[codec(index = 9)]
                    #[doc = "The version of the `Versioned` value used is not able to be interpreted."]
                    BadVersion,
                    #[codec(index = 10)]
                    #[doc = "The given location could not be used (e.g. because it cannot be expressed in the"]
                    #[doc = "desired version of XCM)."]
                    BadLocation,
                    #[codec(index = 11)]
                    #[doc = "The referenced subscription could not be found."]
                    NoSubscription,
                    #[codec(index = 12)]
                    #[doc = "The location is invalid since it already has a subscription from us."]
                    AlreadySubscribed,
                    #[codec(index = 13)]
                    #[doc = "Invalid asset for the operation."]
                    InvalidAsset,
                    #[codec(index = 14)]
                    #[doc = "The owner does not own (all) of the asset that they wish to do the operation on."]
                    LowBalance,
                    #[codec(index = 15)]
                    #[doc = "The asset owner has too many locks on the asset."]
                    TooManyLocks,
                    #[codec(index = 16)]
                    #[doc = "The given account is not an identifiable sovereign account for any location."]
                    AccountNotSovereign,
                    #[codec(index = 17)]
                    #[doc = "The operation required fees to be paid which the initiator could not meet."]
                    FeesNotMet,
                    #[codec(index = 18)]
                    #[doc = "A remote lock with the corresponding data could not be found."]
                    LockNotFound,
                    #[codec(index = 19)]
                    #[doc = "The unlock operation cannot succeed because there are still consumers of the lock."]
                    InUse,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Execution of an XCM message was attempted."]
                    Attempted {
                        outcome: runtime_types::xcm::v3::traits::Outcome,
                    },
                    #[codec(index = 1)]
                    #[doc = "A XCM message was sent."]
                    Sent {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                        message: runtime_types::xcm::v3::Xcm,
                        message_id: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 2)]
                    #[doc = "Query response received which does not match a registered query. This may be because a"]
                    #[doc = "matching query was never registered, it may be because it is a duplicate response, or"]
                    #[doc = "because the query timed out."]
                    UnexpectedResponse {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                    },
                    #[codec(index = 3)]
                    #[doc = "Query response has been received and is ready for taking with `take_response`. There is"]
                    #[doc = "no registered notification call."]
                    ResponseReady {
                        query_id: ::core::primitive::u64,
                        response: runtime_types::xcm::v3::Response,
                    },
                    #[codec(index = 4)]
                    #[doc = "Query response has been received and query is removed. The registered notification has"]
                    #[doc = "been dispatched and executed successfully."]
                    Notified {
                        query_id: ::core::primitive::u64,
                        pallet_index: ::core::primitive::u8,
                        call_index: ::core::primitive::u8,
                    },
                    #[codec(index = 5)]
                    #[doc = "Query response has been received and query is removed. The registered notification could"]
                    #[doc = "not be dispatched because the dispatch weight is greater than the maximum weight"]
                    #[doc = "originally budgeted by this runtime for the query result."]
                    NotifyOverweight {
                        query_id: ::core::primitive::u64,
                        pallet_index: ::core::primitive::u8,
                        call_index: ::core::primitive::u8,
                        actual_weight: runtime_types::sp_weights::weight_v2::Weight,
                        max_budgeted_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 6)]
                    #[doc = "Query response has been received and query is removed. There was a general error with"]
                    #[doc = "dispatching the notification call."]
                    NotifyDispatchError {
                        query_id: ::core::primitive::u64,
                        pallet_index: ::core::primitive::u8,
                        call_index: ::core::primitive::u8,
                    },
                    #[codec(index = 7)]
                    #[doc = "Query response has been received and query is removed. The dispatch was unable to be"]
                    #[doc = "decoded into a `Call`; this might be due to dispatch function having a signature which"]
                    #[doc = "is not `(origin, QueryId, Response)`."]
                    NotifyDecodeFailed {
                        query_id: ::core::primitive::u64,
                        pallet_index: ::core::primitive::u8,
                        call_index: ::core::primitive::u8,
                    },
                    #[codec(index = 8)]
                    #[doc = "Expected query response has been received but the origin location of the response does"]
                    #[doc = "not match that expected. The query remains registered for a later, valid, response to"]
                    #[doc = "be received and acted upon."]
                    InvalidResponder {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                        expected_location: ::core::option::Option<
                            runtime_types::xcm::v3::multilocation::MultiLocation,
                        >,
                    },
                    #[codec(index = 9)]
                    #[doc = "Expected query response has been received but the expected origin location placed in"]
                    #[doc = "storage by this runtime previously cannot be decoded. The query remains registered."]
                    #[doc = ""]
                    #[doc = "This is unexpected (since a location placed in storage in a previously executing"]
                    #[doc = "runtime should be readable prior to query timeout) and dangerous since the possibly"]
                    #[doc = "valid response will be dropped. Manual governance intervention is probably going to be"]
                    #[doc = "needed."]
                    InvalidResponderVersion {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                    },
                    #[codec(index = 10)]
                    #[doc = "Received query response has been read and removed."]
                    ResponseTaken { query_id: ::core::primitive::u64 },
                    #[codec(index = 11)]
                    #[doc = "Some assets have been placed in an asset trap."]
                    AssetsTrapped {
                        hash: ::subxt::utils::H256,
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        assets: runtime_types::xcm::VersionedMultiAssets,
                    },
                    #[codec(index = 12)]
                    #[doc = "An XCM version change notification message has been attempted to be sent."]
                    #[doc = ""]
                    #[doc = "The cost of sending it (borne by the chain) is included."]
                    VersionChangeNotified {
                        destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                        result: ::core::primitive::u32,
                        cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                        message_id: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 13)]
                    #[doc = "The supported version of a location has been changed. This might be through an"]
                    #[doc = "automatic notification or a manual intervention."]
                    SupportedVersionChanged {
                        location: runtime_types::xcm::v3::multilocation::MultiLocation,
                        version: ::core::primitive::u32,
                    },
                    #[codec(index = 14)]
                    #[doc = "A given location which had a version change subscription was dropped owing to an error"]
                    #[doc = "sending the notification to it."]
                    NotifyTargetSendFail {
                        location: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                        error: runtime_types::xcm::v3::traits::Error,
                    },
                    #[codec(index = 15)]
                    #[doc = "A given location which had a version change subscription was dropped owing to an error"]
                    #[doc = "migrating the location to our new XCM format."]
                    NotifyTargetMigrationFail {
                        location: runtime_types::xcm::VersionedMultiLocation,
                        query_id: ::core::primitive::u64,
                    },
                    #[codec(index = 16)]
                    #[doc = "Expected query response has been received but the expected querier location placed in"]
                    #[doc = "storage by this runtime previously cannot be decoded. The query remains registered."]
                    #[doc = ""]
                    #[doc = "This is unexpected (since a location placed in storage in a previously executing"]
                    #[doc = "runtime should be readable prior to query timeout) and dangerous since the possibly"]
                    #[doc = "valid response will be dropped. Manual governance intervention is probably going to be"]
                    #[doc = "needed."]
                    InvalidQuerierVersion {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                    },
                    #[codec(index = 17)]
                    #[doc = "Expected query response has been received but the querier location of the response does"]
                    #[doc = "not match the expected. The query remains registered for a later, valid, response to"]
                    #[doc = "be received and acted upon."]
                    InvalidQuerier {
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        query_id: ::core::primitive::u64,
                        expected_querier: runtime_types::xcm::v3::multilocation::MultiLocation,
                        maybe_actual_querier: ::core::option::Option<
                            runtime_types::xcm::v3::multilocation::MultiLocation,
                        >,
                    },
                    #[codec(index = 18)]
                    #[doc = "A remote has requested XCM version change notification from us and we have honored it."]
                    #[doc = "A version information message is sent to them and its cost is included."]
                    VersionNotifyStarted {
                        destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                        cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                        message_id: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 19)]
                    #[doc = "We have requested that a remote chain send us XCM version change notifications."]
                    VersionNotifyRequested {
                        destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                        cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                        message_id: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 20)]
                    #[doc = "We have requested that a remote chain stops sending us XCM version change notifications."]
                    VersionNotifyUnrequested {
                        destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                        cost: runtime_types::xcm::v3::multiasset::MultiAssets,
                        message_id: [::core::primitive::u8; 32usize],
                    },
                    #[codec(index = 21)]
                    #[doc = "Fees were paid from a location for an operation (often for using `SendXcm`)."]
                    FeesPaid {
                        paying: runtime_types::xcm::v3::multilocation::MultiLocation,
                        fees: runtime_types::xcm::v3::multiasset::MultiAssets,
                    },
                    #[codec(index = 22)]
                    #[doc = "Some assets have been claimed from an asset trap"]
                    AssetsClaimed {
                        hash: ::subxt::utils::H256,
                        origin: runtime_types::xcm::v3::multilocation::MultiLocation,
                        assets: runtime_types::xcm::VersionedMultiAssets,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Origin {
                    #[codec(index = 0)]
                    Xcm(runtime_types::xcm::v3::multilocation::MultiLocation),
                    #[codec(index = 1)]
                    Response(runtime_types::xcm::v3::multilocation::MultiLocation),
                }
            }
        }
        pub mod polkadot_core_primitives {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct InboundDownwardMessage<_0> {
                pub sent_at: _0,
                pub msg: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct InboundHrmpMessage<_0> {
                pub sent_at: _0,
                pub data: ::std::vec::Vec<::core::primitive::u8>,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct OutboundHrmpMessage<_0> {
                pub recipient: _0,
                pub data: ::std::vec::Vec<::core::primitive::u8>,
            }
        }
        pub mod polkadot_parachain {
            use super::runtime_types;
            pub mod primitives {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct HeadData(pub ::std::vec::Vec<::core::primitive::u8>);
                #[derive(
                    :: subxt :: ext :: codec :: CompactAs,
                    :: subxt :: ext :: codec :: Decode,
                    :: subxt :: ext :: codec :: Encode,
                    Debug,
                )]
                pub struct Id(pub ::core::primitive::u32);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum XcmpMessageFormat {
                    #[codec(index = 0)]
                    ConcatenatedVersionedXcm,
                    #[codec(index = 1)]
                    ConcatenatedEncodedBlob,
                    #[codec(index = 2)]
                    Signals,
                }
            }
        }
        pub mod polkadot_primitives {
            use super::runtime_types;
            pub mod v5 {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct AbridgedHostConfiguration {
                    pub max_code_size: ::core::primitive::u32,
                    pub max_head_data_size: ::core::primitive::u32,
                    pub max_upward_queue_count: ::core::primitive::u32,
                    pub max_upward_queue_size: ::core::primitive::u32,
                    pub max_upward_message_size: ::core::primitive::u32,
                    pub max_upward_message_num_per_candidate: ::core::primitive::u32,
                    pub hrmp_max_message_num_per_candidate: ::core::primitive::u32,
                    pub validation_upgrade_cooldown: ::core::primitive::u32,
                    pub validation_upgrade_delay: ::core::primitive::u32,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct AbridgedHrmpChannel {
                    pub max_capacity: ::core::primitive::u32,
                    pub max_total_size: ::core::primitive::u32,
                    pub max_message_size: ::core::primitive::u32,
                    pub msg_count: ::core::primitive::u32,
                    pub total_size: ::core::primitive::u32,
                    pub mqc_head: ::core::option::Option<::subxt::utils::H256>,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct PersistedValidationData<_0, _1> {
                    pub parent_head: runtime_types::polkadot_parachain::primitives::HeadData,
                    pub relay_parent_number: _1,
                    pub relay_parent_storage_root: _0,
                    pub max_pov_size: _1,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum UpgradeRestriction {
                    #[codec(index = 0)]
                    Present,
                }
            }
        }
        pub mod primitive_types {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct U256(pub [::core::primitive::u64; 4usize]);
        }
        pub mod snowbridge_beacon_primitives {
            use super::runtime_types;
            pub mod bls {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum BlsError {
                    #[codec(index = 0)]
                    InvalidSignature,
                    #[codec(index = 1)]
                    InvalidPublicKey,
                    #[codec(index = 2)]
                    InvalidAggregatePublicKeys,
                    #[codec(index = 3)]
                    SignatureVerificationFailed,
                }
            }
            pub mod types {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BeaconHeader {
                    pub slot: ::core::primitive::u64,
                    pub proposer_index: ::core::primitive::u64,
                    pub parent_root: ::subxt::utils::H256,
                    pub state_root: ::subxt::utils::H256,
                    pub body_root: ::subxt::utils::H256,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct CompactBeaconState {
                    #[codec(compact)]
                    pub slot: ::core::primitive::u64,
                    pub block_roots_root: ::subxt::utils::H256,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct CompactExecutionHeader {
                    pub parent_hash: ::subxt::utils::H256,
                    #[codec(compact)]
                    pub block_number: ::core::primitive::u64,
                    pub state_root: ::subxt::utils::H256,
                    pub receipts_root: ::subxt::utils::H256,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ExecutionHeaderState {
                    pub beacon_block_root: ::subxt::utils::H256,
                    pub beacon_slot: ::core::primitive::u64,
                    pub block_hash: ::subxt::utils::H256,
                    pub block_number: ::core::primitive::u64,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ExecutionPayloadHeader {
                    pub parent_hash: ::subxt::utils::H256,
                    pub fee_recipient: ::subxt::utils::H160,
                    pub state_root: ::subxt::utils::H256,
                    pub receipts_root: ::subxt::utils::H256,
                    pub logs_bloom: ::std::vec::Vec<::core::primitive::u8>,
                    pub prev_randao: ::subxt::utils::H256,
                    pub block_number: ::core::primitive::u64,
                    pub gas_limit: ::core::primitive::u64,
                    pub gas_used: ::core::primitive::u64,
                    pub timestamp: ::core::primitive::u64,
                    pub extra_data: ::std::vec::Vec<::core::primitive::u8>,
                    pub base_fee_per_gas: runtime_types::primitive_types::U256,
                    pub block_hash: ::subxt::utils::H256,
                    pub transactions_root: ::subxt::utils::H256,
                    pub withdrawals_root: ::subxt::utils::H256,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Fork {
                    pub version: [::core::primitive::u8; 4usize],
                    pub epoch: ::core::primitive::u64,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ForkVersions {
                    pub genesis: runtime_types::snowbridge_beacon_primitives::types::Fork,
                    pub altair: runtime_types::snowbridge_beacon_primitives::types::Fork,
                    pub bellatrix: runtime_types::snowbridge_beacon_primitives::types::Fork,
                    pub capella: runtime_types::snowbridge_beacon_primitives::types::Fork,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct PublicKey(pub [::core::primitive::u8; 48usize]);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Signature(pub [::core::primitive::u8; 96usize]);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct SyncAggregate {
                    pub sync_committee_bits: [::core::primitive::u8; 4usize],
                    pub sync_committee_signature:
                        runtime_types::snowbridge_beacon_primitives::types::Signature,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct SyncCommittee {
                    pub pubkeys:
                        [runtime_types::snowbridge_beacon_primitives::types::PublicKey; 32usize],
                    pub aggregate_pubkey:
                        runtime_types::snowbridge_beacon_primitives::types::PublicKey,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct SyncCommitteePrepared {
                    pub root: ::subxt::utils::H256,
                    pub pubkeys:
                        ::std::boxed::Box<[runtime_types::milagro_bls::keys::PublicKey; 32usize]>,
                    pub aggregate_pubkey: runtime_types::milagro_bls::keys::PublicKey,
                }
            }
            pub mod updates {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct AncestryProof {
                    pub header_branch: ::std::vec::Vec<::subxt::utils::H256>,
                    pub finalized_block_root: ::subxt::utils::H256,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct CheckpointUpdate {
                    pub header: runtime_types::snowbridge_beacon_primitives::types::BeaconHeader,
                    pub current_sync_committee:
                        runtime_types::snowbridge_beacon_primitives::types::SyncCommittee,
                    pub current_sync_committee_branch: ::std::vec::Vec<::subxt::utils::H256>,
                    pub validators_root: ::subxt::utils::H256,
                    pub block_roots_root: ::subxt::utils::H256,
                    pub block_roots_branch: ::std::vec::Vec<::subxt::utils::H256>,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct ExecutionHeaderUpdate {
                    pub header: runtime_types::snowbridge_beacon_primitives::types::BeaconHeader,
                    pub ancestry_proof: ::core::option::Option<
                        runtime_types::snowbridge_beacon_primitives::updates::AncestryProof,
                    >,
                    pub execution_header:
                        runtime_types::snowbridge_beacon_primitives::types::ExecutionPayloadHeader,
                    pub execution_branch: ::std::vec::Vec<::subxt::utils::H256>,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct NextSyncCommitteeUpdate {
                    pub next_sync_committee:
                        runtime_types::snowbridge_beacon_primitives::types::SyncCommittee,
                    pub next_sync_committee_branch: ::std::vec::Vec<::subxt::utils::H256>,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Update { pub attested_header : runtime_types :: snowbridge_beacon_primitives :: types :: BeaconHeader , pub sync_aggregate : runtime_types :: snowbridge_beacon_primitives :: types :: SyncAggregate , pub signature_slot : :: core :: primitive :: u64 , pub next_sync_committee_update : :: core :: option :: Option < runtime_types :: snowbridge_beacon_primitives :: updates :: NextSyncCommitteeUpdate > , pub finalized_header : runtime_types :: snowbridge_beacon_primitives :: types :: BeaconHeader , pub finality_branch : :: std :: vec :: Vec < :: subxt :: utils :: H256 > , pub block_roots_root : :: subxt :: utils :: H256 , pub block_roots_branch : :: std :: vec :: Vec < :: subxt :: utils :: H256 > , }
            }
        }
        pub mod snowbridge_control {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::upgrade`]."]
                    upgrade {
                        impl_address: ::subxt::utils::H160,
                        impl_code_hash: ::subxt::utils::H256,
                        params: ::core::option::Option<::std::vec::Vec<::core::primitive::u8>>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::create_agent`]."]
                    create_agent,
                    #[codec(index = 2)]
                    #[doc = "See [`Pallet::create_channel`]."]
                    create_channel,
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::update_channel`]."]
                    update_channel {
                        mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                        fee: ::core::primitive::u128,
                        reward: ::core::primitive::u128,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::set_operating_mode`]."]
                    set_operating_mode {
                        mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::transfer_native_from_agent`]."]
                    transfer_native_from_agent {
                        recipient: ::subxt::utils::H160,
                        amount: ::core::primitive::u128,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    UpgradeDataTooLarge,
                    #[codec(index = 1)]
                    SubmissionFailed,
                    #[codec(index = 2)]
                    LocationReanchorFailed,
                    #[codec(index = 3)]
                    LocationToParaIdConversionFailed,
                    #[codec(index = 4)]
                    LocationToAgentIdConversionFailed,
                    #[codec(index = 5)]
                    AgentAlreadyCreated,
                    #[codec(index = 6)]
                    AgentNotExist,
                    #[codec(index = 7)]
                    ChannelAlreadyCreated,
                    #[codec(index = 8)]
                    ChannelNotExist,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "An Upgrade message was sent to the Gateway"]
                    Upgrade {
                        impl_address: ::subxt::utils::H160,
                        impl_code_hash: ::subxt::utils::H256,
                        params_hash: ::core::option::Option<::subxt::utils::H256>,
                    },
                    #[codec(index = 1)]
                    #[doc = "An CreateAgent message was sent to the Gateway"]
                    CreateAgent {
                        location:
                            ::std::boxed::Box<runtime_types::xcm::v3::multilocation::MultiLocation>,
                        agent_id: ::subxt::utils::H256,
                    },
                    #[codec(index = 2)]
                    #[doc = "An CreateChannel message was sent to the Gateway"]
                    CreateChannel {
                        para_id: runtime_types::polkadot_parachain::primitives::Id,
                        agent_id: ::subxt::utils::H256,
                    },
                    #[codec(index = 3)]
                    #[doc = "An UpdateChannel message was sent to the Gateway"]
                    UpdateChannel {
                        para_id: runtime_types::polkadot_parachain::primitives::Id,
                        agent_id: ::subxt::utils::H256,
                        mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                        fee: ::core::primitive::u128,
                        reward: ::core::primitive::u128,
                    },
                    #[codec(index = 4)]
                    #[doc = "An SetOperatingMode message was sent to the Gateway"]
                    SetOperatingMode {
                        mode: runtime_types::snowbridge_core::outbound::OperatingMode,
                    },
                    #[codec(index = 5)]
                    #[doc = "An TransferNativeFromAgent message was sent to the Gateway"]
                    TransferNativeFromAgent {
                        agent_id: ::subxt::utils::H256,
                        recipient: ::subxt::utils::H160,
                        amount: ::core::primitive::u128,
                    },
                }
            }
        }
        pub mod snowbridge_core {
            use super::runtime_types;
            pub mod inbound {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Message {
                    pub data: ::std::vec::Vec<::core::primitive::u8>,
                    pub proof: runtime_types::snowbridge_core::inbound::Proof,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Proof {
                    pub block_hash: ::subxt::utils::H256,
                    pub tx_index: ::core::primitive::u32,
                    pub data: (
                        ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                        ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                    ),
                }
            }
            pub mod outbound {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum OperatingMode {
                    #[codec(index = 0)]
                    Normal,
                    #[codec(index = 1)]
                    RejectingOutboundMessages,
                }
            }
        }
        pub mod snowbridge_ethereum_beacon_client {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    # [codec (index = 0)] # [doc = "See [`Pallet::force_checkpoint`]."] force_checkpoint { update : :: std :: boxed :: Box < runtime_types :: snowbridge_beacon_primitives :: updates :: CheckpointUpdate > , } , # [codec (index = 1)] # [doc = "See [`Pallet::submit`]."] submit { update : :: std :: boxed :: Box < runtime_types :: snowbridge_beacon_primitives :: updates :: Update > , } , # [codec (index = 2)] # [doc = "See [`Pallet::submit_execution_header`]."] submit_execution_header { update : :: std :: boxed :: Box < runtime_types :: snowbridge_beacon_primitives :: updates :: ExecutionHeaderUpdate > , } , # [codec (index = 3)] # [doc = "See [`Pallet::set_owner`]."] set_owner { new_owner : :: core :: option :: Option < :: subxt :: utils :: AccountId32 > , } , # [codec (index = 4)] # [doc = "See [`Pallet::set_operating_mode`]."] set_operating_mode { operating_mode : runtime_types :: bp_runtime :: BasicOperatingMode , } , }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    SkippedSyncCommitteePeriod,
                    #[codec(index = 1)]
                    #[doc = "Attested header is older than latest finalized header."]
                    IrrelevantUpdate,
                    #[codec(index = 2)]
                    NotBootstrapped,
                    #[codec(index = 3)]
                    SyncCommitteeParticipantsNotSupermajority,
                    #[codec(index = 4)]
                    InvalidHeaderMerkleProof,
                    #[codec(index = 5)]
                    InvalidSyncCommitteeMerkleProof,
                    #[codec(index = 6)]
                    InvalidExecutionHeaderProof,
                    #[codec(index = 7)]
                    InvalidAncestryMerkleProof,
                    #[codec(index = 8)]
                    InvalidBlockRootsRootMerkleProof,
                    #[codec(index = 9)]
                    HeaderNotFinalized,
                    #[codec(index = 10)]
                    MissingHeader,
                    #[codec(index = 11)]
                    InvalidProof,
                    #[codec(index = 12)]
                    DecodeFailed,
                    #[codec(index = 13)]
                    BlockBodyHashTreeRootFailed,
                    #[codec(index = 14)]
                    HeaderHashTreeRootFailed,
                    #[codec(index = 15)]
                    SyncCommitteeHashTreeRootFailed,
                    #[codec(index = 16)]
                    SigningRootHashTreeRootFailed,
                    #[codec(index = 17)]
                    ForkDataHashTreeRootFailed,
                    #[codec(index = 18)]
                    ExpectedFinalizedHeaderNotStored,
                    #[codec(index = 19)]
                    BLSPreparePublicKeysFailed,
                    #[codec(index = 20)]
                    BLSVerificationFailed(
                        runtime_types::snowbridge_beacon_primitives::bls::BlsError,
                    ),
                    #[codec(index = 21)]
                    InvalidUpdateSlot,
                    #[codec(index = 22)]
                    #[doc = "The given update is not in the expected period, or the given next sync committee does"]
                    #[doc = "not match the next sync committee in storage."]
                    InvalidSyncCommitteeUpdate,
                    #[codec(index = 23)]
                    ExecutionHeaderTooFarBehind,
                    #[codec(index = 24)]
                    ExecutionHeaderSkippedBlock,
                    #[codec(index = 25)]
                    BridgeModule(runtime_types::bp_runtime::OwnedBridgeModuleError),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    BeaconHeaderImported {
                        block_hash: ::subxt::utils::H256,
                        slot: ::core::primitive::u64,
                    },
                    #[codec(index = 1)]
                    ExecutionHeaderImported {
                        block_hash: ::subxt::utils::H256,
                        block_number: ::core::primitive::u64,
                    },
                    #[codec(index = 2)]
                    SyncCommitteeUpdated { period: ::core::primitive::u64 },
                }
            }
        }
        pub mod snowbridge_inbound_queue {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::submit`]."]
                    submit {
                        message: runtime_types::snowbridge_core::inbound::Message,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_gateway`]."]
                    set_gateway { gateway: ::subxt::utils::H160 },
                    #[codec(index = 3)]
                    #[doc = "See [`Pallet::set_owner`]."]
                    set_owner {
                        new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 4)]
                    #[doc = "See [`Pallet::set_operating_mode`]."]
                    set_operating_mode {
                        operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                    },
                    #[codec(index = 5)]
                    #[doc = "See [`Pallet::forward_xcm`]."]
                    forward_xcm {
                        dest: ::std::boxed::Box<runtime_types::xcm::VersionedMultiLocation>,
                        message: ::std::boxed::Box<runtime_types::xcm::VersionedXcm>,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "Message came from an invalid outbound channel on the Ethereum side."]
                    InvalidGateway,
                    #[codec(index = 1)]
                    #[doc = "Message has an invalid envelope."]
                    InvalidEnvelope,
                    #[codec(index = 2)]
                    #[doc = "Message has an unexpected nonce."]
                    InvalidNonce,
                    #[codec(index = 3)]
                    #[doc = "Cannot convert location"]
                    InvalidAccountConversion,
                    #[codec(index = 4)]
                    #[doc = "Error generated by the `OwnedBridgeModule` trait."]
                    BridgeModule(runtime_types::bp_runtime::OwnedBridgeModuleError),
                    #[codec(index = 5)]
                    #[doc = "Forward Xcm failed"]
                    XcmForwardFailed,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "A message was received from Ethereum"]
                    MessageReceived {
                        dest: runtime_types::polkadot_parachain::primitives::Id,
                        nonce: ::core::primitive::u64,
                        result: runtime_types::snowbridge_inbound_queue::MessageDispatchResult,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum MessageDispatchResult {
                #[codec(index = 0)]
                InvalidPayload,
                #[codec(index = 1)]
                Dispatched,
                #[codec(index = 2)]
                NotDispatched(runtime_types::xcm::v3::traits::SendError),
            }
        }
        pub mod snowbridge_outbound_queue {
            use super::runtime_types;
            pub mod pallet {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
                pub enum Call {
                    #[codec(index = 0)]
                    #[doc = "See [`Pallet::set_owner`]."]
                    set_owner {
                        new_owner: ::core::option::Option<::subxt::utils::AccountId32>,
                    },
                    #[codec(index = 1)]
                    #[doc = "See [`Pallet::set_operating_mode`]."]
                    set_operating_mode {
                        operating_mode: runtime_types::bp_runtime::BasicOperatingMode,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Error` enum of this pallet."]
                pub enum Error {
                    #[codec(index = 0)]
                    #[doc = "The message is too large"]
                    MessageTooLarge,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                #[doc = "The `Event` enum of this pallet"]
                pub enum Event {
                    #[codec(index = 0)]
                    #[doc = "Message has been queued and will be processed in the future"]
                    MessageQueued { id: ::subxt::utils::H256 },
                    #[codec(index = 1)]
                    #[doc = "Message will be committed at the end of current block. From now on, to track the"]
                    #[doc = "progress the message, use the `nonce` of `id`."]
                    MessageAccepted {
                        id: ::subxt::utils::H256,
                        nonce: ::core::primitive::u64,
                    },
                    #[codec(index = 2)]
                    #[doc = "Some messages have been committed"]
                    MessagesCommitted {
                        root: ::subxt::utils::H256,
                        count: ::core::primitive::u64,
                    },
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum AggregateMessageOrigin {
                #[codec(index = 0)]
                Parachain(runtime_types::polkadot_parachain::primitives::Id),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct PreparedMessage {
                pub origin: runtime_types::polkadot_parachain::primitives::Id,
                pub nonce: ::core::primitive::u64,
                pub command: ::core::primitive::u8,
                pub params: ::std::vec::Vec<::core::primitive::u8>,
            }
        }
        pub mod sp_arithmetic {
            use super::runtime_types;
            pub mod fixed_point {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: CompactAs,
                    :: subxt :: ext :: codec :: Decode,
                    :: subxt :: ext :: codec :: Encode,
                    Debug,
                )]
                pub struct FixedU128(pub ::core::primitive::u128);
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum ArithmeticError {
                #[codec(index = 0)]
                Underflow,
                #[codec(index = 1)]
                Overflow,
                #[codec(index = 2)]
                DivisionByZero,
            }
        }
        pub mod sp_consensus_aura {
            use super::runtime_types;
            pub mod sr25519 {
                use super::runtime_types;
                pub mod app_sr25519 {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct Public(pub runtime_types::sp_core::sr25519::Public);
                }
            }
        }
        pub mod sp_consensus_grandpa {
            use super::runtime_types;
            pub mod app {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Public(pub runtime_types::sp_core::ed25519::Public);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Signature(pub runtime_types::sp_core::ed25519::Signature);
            }
        }
        pub mod sp_consensus_slots {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: CompactAs,
                :: subxt :: ext :: codec :: Decode,
                :: subxt :: ext :: codec :: Encode,
                Debug,
            )]
            pub struct Slot(pub ::core::primitive::u64);
        }
        pub mod sp_core {
            use super::runtime_types;
            pub mod crypto {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct KeyTypeId(pub [::core::primitive::u8; 4usize]);
            }
            pub mod ecdsa {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Signature(pub [::core::primitive::u8; 65usize]);
            }
            pub mod ed25519 {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Public(pub [::core::primitive::u8; 32usize]);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Signature(pub [::core::primitive::u8; 64usize]);
            }
            pub mod sr25519 {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Public(pub [::core::primitive::u8; 32usize]);
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Signature(pub [::core::primitive::u8; 64usize]);
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum Void {}
        }
        pub mod sp_runtime {
            use super::runtime_types;
            pub mod generic {
                use super::runtime_types;
                pub mod digest {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct Digest {
                        pub logs:
                            ::std::vec::Vec<runtime_types::sp_runtime::generic::digest::DigestItem>,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum DigestItem {
                        #[codec(index = 6)]
                        PreRuntime(
                            [::core::primitive::u8; 4usize],
                            ::std::vec::Vec<::core::primitive::u8>,
                        ),
                        #[codec(index = 4)]
                        Consensus(
                            [::core::primitive::u8; 4usize],
                            ::std::vec::Vec<::core::primitive::u8>,
                        ),
                        #[codec(index = 5)]
                        Seal(
                            [::core::primitive::u8; 4usize],
                            ::std::vec::Vec<::core::primitive::u8>,
                        ),
                        #[codec(index = 0)]
                        Other(::std::vec::Vec<::core::primitive::u8>),
                        #[codec(index = 8)]
                        RuntimeEnvironmentUpdated,
                    }
                }
                pub mod era {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Era {
                        #[codec(index = 0)]
                        Immortal,
                        #[codec(index = 1)]
                        Mortal1(::core::primitive::u8),
                        #[codec(index = 2)]
                        Mortal2(::core::primitive::u8),
                        #[codec(index = 3)]
                        Mortal3(::core::primitive::u8),
                        #[codec(index = 4)]
                        Mortal4(::core::primitive::u8),
                        #[codec(index = 5)]
                        Mortal5(::core::primitive::u8),
                        #[codec(index = 6)]
                        Mortal6(::core::primitive::u8),
                        #[codec(index = 7)]
                        Mortal7(::core::primitive::u8),
                        #[codec(index = 8)]
                        Mortal8(::core::primitive::u8),
                        #[codec(index = 9)]
                        Mortal9(::core::primitive::u8),
                        #[codec(index = 10)]
                        Mortal10(::core::primitive::u8),
                        #[codec(index = 11)]
                        Mortal11(::core::primitive::u8),
                        #[codec(index = 12)]
                        Mortal12(::core::primitive::u8),
                        #[codec(index = 13)]
                        Mortal13(::core::primitive::u8),
                        #[codec(index = 14)]
                        Mortal14(::core::primitive::u8),
                        #[codec(index = 15)]
                        Mortal15(::core::primitive::u8),
                        #[codec(index = 16)]
                        Mortal16(::core::primitive::u8),
                        #[codec(index = 17)]
                        Mortal17(::core::primitive::u8),
                        #[codec(index = 18)]
                        Mortal18(::core::primitive::u8),
                        #[codec(index = 19)]
                        Mortal19(::core::primitive::u8),
                        #[codec(index = 20)]
                        Mortal20(::core::primitive::u8),
                        #[codec(index = 21)]
                        Mortal21(::core::primitive::u8),
                        #[codec(index = 22)]
                        Mortal22(::core::primitive::u8),
                        #[codec(index = 23)]
                        Mortal23(::core::primitive::u8),
                        #[codec(index = 24)]
                        Mortal24(::core::primitive::u8),
                        #[codec(index = 25)]
                        Mortal25(::core::primitive::u8),
                        #[codec(index = 26)]
                        Mortal26(::core::primitive::u8),
                        #[codec(index = 27)]
                        Mortal27(::core::primitive::u8),
                        #[codec(index = 28)]
                        Mortal28(::core::primitive::u8),
                        #[codec(index = 29)]
                        Mortal29(::core::primitive::u8),
                        #[codec(index = 30)]
                        Mortal30(::core::primitive::u8),
                        #[codec(index = 31)]
                        Mortal31(::core::primitive::u8),
                        #[codec(index = 32)]
                        Mortal32(::core::primitive::u8),
                        #[codec(index = 33)]
                        Mortal33(::core::primitive::u8),
                        #[codec(index = 34)]
                        Mortal34(::core::primitive::u8),
                        #[codec(index = 35)]
                        Mortal35(::core::primitive::u8),
                        #[codec(index = 36)]
                        Mortal36(::core::primitive::u8),
                        #[codec(index = 37)]
                        Mortal37(::core::primitive::u8),
                        #[codec(index = 38)]
                        Mortal38(::core::primitive::u8),
                        #[codec(index = 39)]
                        Mortal39(::core::primitive::u8),
                        #[codec(index = 40)]
                        Mortal40(::core::primitive::u8),
                        #[codec(index = 41)]
                        Mortal41(::core::primitive::u8),
                        #[codec(index = 42)]
                        Mortal42(::core::primitive::u8),
                        #[codec(index = 43)]
                        Mortal43(::core::primitive::u8),
                        #[codec(index = 44)]
                        Mortal44(::core::primitive::u8),
                        #[codec(index = 45)]
                        Mortal45(::core::primitive::u8),
                        #[codec(index = 46)]
                        Mortal46(::core::primitive::u8),
                        #[codec(index = 47)]
                        Mortal47(::core::primitive::u8),
                        #[codec(index = 48)]
                        Mortal48(::core::primitive::u8),
                        #[codec(index = 49)]
                        Mortal49(::core::primitive::u8),
                        #[codec(index = 50)]
                        Mortal50(::core::primitive::u8),
                        #[codec(index = 51)]
                        Mortal51(::core::primitive::u8),
                        #[codec(index = 52)]
                        Mortal52(::core::primitive::u8),
                        #[codec(index = 53)]
                        Mortal53(::core::primitive::u8),
                        #[codec(index = 54)]
                        Mortal54(::core::primitive::u8),
                        #[codec(index = 55)]
                        Mortal55(::core::primitive::u8),
                        #[codec(index = 56)]
                        Mortal56(::core::primitive::u8),
                        #[codec(index = 57)]
                        Mortal57(::core::primitive::u8),
                        #[codec(index = 58)]
                        Mortal58(::core::primitive::u8),
                        #[codec(index = 59)]
                        Mortal59(::core::primitive::u8),
                        #[codec(index = 60)]
                        Mortal60(::core::primitive::u8),
                        #[codec(index = 61)]
                        Mortal61(::core::primitive::u8),
                        #[codec(index = 62)]
                        Mortal62(::core::primitive::u8),
                        #[codec(index = 63)]
                        Mortal63(::core::primitive::u8),
                        #[codec(index = 64)]
                        Mortal64(::core::primitive::u8),
                        #[codec(index = 65)]
                        Mortal65(::core::primitive::u8),
                        #[codec(index = 66)]
                        Mortal66(::core::primitive::u8),
                        #[codec(index = 67)]
                        Mortal67(::core::primitive::u8),
                        #[codec(index = 68)]
                        Mortal68(::core::primitive::u8),
                        #[codec(index = 69)]
                        Mortal69(::core::primitive::u8),
                        #[codec(index = 70)]
                        Mortal70(::core::primitive::u8),
                        #[codec(index = 71)]
                        Mortal71(::core::primitive::u8),
                        #[codec(index = 72)]
                        Mortal72(::core::primitive::u8),
                        #[codec(index = 73)]
                        Mortal73(::core::primitive::u8),
                        #[codec(index = 74)]
                        Mortal74(::core::primitive::u8),
                        #[codec(index = 75)]
                        Mortal75(::core::primitive::u8),
                        #[codec(index = 76)]
                        Mortal76(::core::primitive::u8),
                        #[codec(index = 77)]
                        Mortal77(::core::primitive::u8),
                        #[codec(index = 78)]
                        Mortal78(::core::primitive::u8),
                        #[codec(index = 79)]
                        Mortal79(::core::primitive::u8),
                        #[codec(index = 80)]
                        Mortal80(::core::primitive::u8),
                        #[codec(index = 81)]
                        Mortal81(::core::primitive::u8),
                        #[codec(index = 82)]
                        Mortal82(::core::primitive::u8),
                        #[codec(index = 83)]
                        Mortal83(::core::primitive::u8),
                        #[codec(index = 84)]
                        Mortal84(::core::primitive::u8),
                        #[codec(index = 85)]
                        Mortal85(::core::primitive::u8),
                        #[codec(index = 86)]
                        Mortal86(::core::primitive::u8),
                        #[codec(index = 87)]
                        Mortal87(::core::primitive::u8),
                        #[codec(index = 88)]
                        Mortal88(::core::primitive::u8),
                        #[codec(index = 89)]
                        Mortal89(::core::primitive::u8),
                        #[codec(index = 90)]
                        Mortal90(::core::primitive::u8),
                        #[codec(index = 91)]
                        Mortal91(::core::primitive::u8),
                        #[codec(index = 92)]
                        Mortal92(::core::primitive::u8),
                        #[codec(index = 93)]
                        Mortal93(::core::primitive::u8),
                        #[codec(index = 94)]
                        Mortal94(::core::primitive::u8),
                        #[codec(index = 95)]
                        Mortal95(::core::primitive::u8),
                        #[codec(index = 96)]
                        Mortal96(::core::primitive::u8),
                        #[codec(index = 97)]
                        Mortal97(::core::primitive::u8),
                        #[codec(index = 98)]
                        Mortal98(::core::primitive::u8),
                        #[codec(index = 99)]
                        Mortal99(::core::primitive::u8),
                        #[codec(index = 100)]
                        Mortal100(::core::primitive::u8),
                        #[codec(index = 101)]
                        Mortal101(::core::primitive::u8),
                        #[codec(index = 102)]
                        Mortal102(::core::primitive::u8),
                        #[codec(index = 103)]
                        Mortal103(::core::primitive::u8),
                        #[codec(index = 104)]
                        Mortal104(::core::primitive::u8),
                        #[codec(index = 105)]
                        Mortal105(::core::primitive::u8),
                        #[codec(index = 106)]
                        Mortal106(::core::primitive::u8),
                        #[codec(index = 107)]
                        Mortal107(::core::primitive::u8),
                        #[codec(index = 108)]
                        Mortal108(::core::primitive::u8),
                        #[codec(index = 109)]
                        Mortal109(::core::primitive::u8),
                        #[codec(index = 110)]
                        Mortal110(::core::primitive::u8),
                        #[codec(index = 111)]
                        Mortal111(::core::primitive::u8),
                        #[codec(index = 112)]
                        Mortal112(::core::primitive::u8),
                        #[codec(index = 113)]
                        Mortal113(::core::primitive::u8),
                        #[codec(index = 114)]
                        Mortal114(::core::primitive::u8),
                        #[codec(index = 115)]
                        Mortal115(::core::primitive::u8),
                        #[codec(index = 116)]
                        Mortal116(::core::primitive::u8),
                        #[codec(index = 117)]
                        Mortal117(::core::primitive::u8),
                        #[codec(index = 118)]
                        Mortal118(::core::primitive::u8),
                        #[codec(index = 119)]
                        Mortal119(::core::primitive::u8),
                        #[codec(index = 120)]
                        Mortal120(::core::primitive::u8),
                        #[codec(index = 121)]
                        Mortal121(::core::primitive::u8),
                        #[codec(index = 122)]
                        Mortal122(::core::primitive::u8),
                        #[codec(index = 123)]
                        Mortal123(::core::primitive::u8),
                        #[codec(index = 124)]
                        Mortal124(::core::primitive::u8),
                        #[codec(index = 125)]
                        Mortal125(::core::primitive::u8),
                        #[codec(index = 126)]
                        Mortal126(::core::primitive::u8),
                        #[codec(index = 127)]
                        Mortal127(::core::primitive::u8),
                        #[codec(index = 128)]
                        Mortal128(::core::primitive::u8),
                        #[codec(index = 129)]
                        Mortal129(::core::primitive::u8),
                        #[codec(index = 130)]
                        Mortal130(::core::primitive::u8),
                        #[codec(index = 131)]
                        Mortal131(::core::primitive::u8),
                        #[codec(index = 132)]
                        Mortal132(::core::primitive::u8),
                        #[codec(index = 133)]
                        Mortal133(::core::primitive::u8),
                        #[codec(index = 134)]
                        Mortal134(::core::primitive::u8),
                        #[codec(index = 135)]
                        Mortal135(::core::primitive::u8),
                        #[codec(index = 136)]
                        Mortal136(::core::primitive::u8),
                        #[codec(index = 137)]
                        Mortal137(::core::primitive::u8),
                        #[codec(index = 138)]
                        Mortal138(::core::primitive::u8),
                        #[codec(index = 139)]
                        Mortal139(::core::primitive::u8),
                        #[codec(index = 140)]
                        Mortal140(::core::primitive::u8),
                        #[codec(index = 141)]
                        Mortal141(::core::primitive::u8),
                        #[codec(index = 142)]
                        Mortal142(::core::primitive::u8),
                        #[codec(index = 143)]
                        Mortal143(::core::primitive::u8),
                        #[codec(index = 144)]
                        Mortal144(::core::primitive::u8),
                        #[codec(index = 145)]
                        Mortal145(::core::primitive::u8),
                        #[codec(index = 146)]
                        Mortal146(::core::primitive::u8),
                        #[codec(index = 147)]
                        Mortal147(::core::primitive::u8),
                        #[codec(index = 148)]
                        Mortal148(::core::primitive::u8),
                        #[codec(index = 149)]
                        Mortal149(::core::primitive::u8),
                        #[codec(index = 150)]
                        Mortal150(::core::primitive::u8),
                        #[codec(index = 151)]
                        Mortal151(::core::primitive::u8),
                        #[codec(index = 152)]
                        Mortal152(::core::primitive::u8),
                        #[codec(index = 153)]
                        Mortal153(::core::primitive::u8),
                        #[codec(index = 154)]
                        Mortal154(::core::primitive::u8),
                        #[codec(index = 155)]
                        Mortal155(::core::primitive::u8),
                        #[codec(index = 156)]
                        Mortal156(::core::primitive::u8),
                        #[codec(index = 157)]
                        Mortal157(::core::primitive::u8),
                        #[codec(index = 158)]
                        Mortal158(::core::primitive::u8),
                        #[codec(index = 159)]
                        Mortal159(::core::primitive::u8),
                        #[codec(index = 160)]
                        Mortal160(::core::primitive::u8),
                        #[codec(index = 161)]
                        Mortal161(::core::primitive::u8),
                        #[codec(index = 162)]
                        Mortal162(::core::primitive::u8),
                        #[codec(index = 163)]
                        Mortal163(::core::primitive::u8),
                        #[codec(index = 164)]
                        Mortal164(::core::primitive::u8),
                        #[codec(index = 165)]
                        Mortal165(::core::primitive::u8),
                        #[codec(index = 166)]
                        Mortal166(::core::primitive::u8),
                        #[codec(index = 167)]
                        Mortal167(::core::primitive::u8),
                        #[codec(index = 168)]
                        Mortal168(::core::primitive::u8),
                        #[codec(index = 169)]
                        Mortal169(::core::primitive::u8),
                        #[codec(index = 170)]
                        Mortal170(::core::primitive::u8),
                        #[codec(index = 171)]
                        Mortal171(::core::primitive::u8),
                        #[codec(index = 172)]
                        Mortal172(::core::primitive::u8),
                        #[codec(index = 173)]
                        Mortal173(::core::primitive::u8),
                        #[codec(index = 174)]
                        Mortal174(::core::primitive::u8),
                        #[codec(index = 175)]
                        Mortal175(::core::primitive::u8),
                        #[codec(index = 176)]
                        Mortal176(::core::primitive::u8),
                        #[codec(index = 177)]
                        Mortal177(::core::primitive::u8),
                        #[codec(index = 178)]
                        Mortal178(::core::primitive::u8),
                        #[codec(index = 179)]
                        Mortal179(::core::primitive::u8),
                        #[codec(index = 180)]
                        Mortal180(::core::primitive::u8),
                        #[codec(index = 181)]
                        Mortal181(::core::primitive::u8),
                        #[codec(index = 182)]
                        Mortal182(::core::primitive::u8),
                        #[codec(index = 183)]
                        Mortal183(::core::primitive::u8),
                        #[codec(index = 184)]
                        Mortal184(::core::primitive::u8),
                        #[codec(index = 185)]
                        Mortal185(::core::primitive::u8),
                        #[codec(index = 186)]
                        Mortal186(::core::primitive::u8),
                        #[codec(index = 187)]
                        Mortal187(::core::primitive::u8),
                        #[codec(index = 188)]
                        Mortal188(::core::primitive::u8),
                        #[codec(index = 189)]
                        Mortal189(::core::primitive::u8),
                        #[codec(index = 190)]
                        Mortal190(::core::primitive::u8),
                        #[codec(index = 191)]
                        Mortal191(::core::primitive::u8),
                        #[codec(index = 192)]
                        Mortal192(::core::primitive::u8),
                        #[codec(index = 193)]
                        Mortal193(::core::primitive::u8),
                        #[codec(index = 194)]
                        Mortal194(::core::primitive::u8),
                        #[codec(index = 195)]
                        Mortal195(::core::primitive::u8),
                        #[codec(index = 196)]
                        Mortal196(::core::primitive::u8),
                        #[codec(index = 197)]
                        Mortal197(::core::primitive::u8),
                        #[codec(index = 198)]
                        Mortal198(::core::primitive::u8),
                        #[codec(index = 199)]
                        Mortal199(::core::primitive::u8),
                        #[codec(index = 200)]
                        Mortal200(::core::primitive::u8),
                        #[codec(index = 201)]
                        Mortal201(::core::primitive::u8),
                        #[codec(index = 202)]
                        Mortal202(::core::primitive::u8),
                        #[codec(index = 203)]
                        Mortal203(::core::primitive::u8),
                        #[codec(index = 204)]
                        Mortal204(::core::primitive::u8),
                        #[codec(index = 205)]
                        Mortal205(::core::primitive::u8),
                        #[codec(index = 206)]
                        Mortal206(::core::primitive::u8),
                        #[codec(index = 207)]
                        Mortal207(::core::primitive::u8),
                        #[codec(index = 208)]
                        Mortal208(::core::primitive::u8),
                        #[codec(index = 209)]
                        Mortal209(::core::primitive::u8),
                        #[codec(index = 210)]
                        Mortal210(::core::primitive::u8),
                        #[codec(index = 211)]
                        Mortal211(::core::primitive::u8),
                        #[codec(index = 212)]
                        Mortal212(::core::primitive::u8),
                        #[codec(index = 213)]
                        Mortal213(::core::primitive::u8),
                        #[codec(index = 214)]
                        Mortal214(::core::primitive::u8),
                        #[codec(index = 215)]
                        Mortal215(::core::primitive::u8),
                        #[codec(index = 216)]
                        Mortal216(::core::primitive::u8),
                        #[codec(index = 217)]
                        Mortal217(::core::primitive::u8),
                        #[codec(index = 218)]
                        Mortal218(::core::primitive::u8),
                        #[codec(index = 219)]
                        Mortal219(::core::primitive::u8),
                        #[codec(index = 220)]
                        Mortal220(::core::primitive::u8),
                        #[codec(index = 221)]
                        Mortal221(::core::primitive::u8),
                        #[codec(index = 222)]
                        Mortal222(::core::primitive::u8),
                        #[codec(index = 223)]
                        Mortal223(::core::primitive::u8),
                        #[codec(index = 224)]
                        Mortal224(::core::primitive::u8),
                        #[codec(index = 225)]
                        Mortal225(::core::primitive::u8),
                        #[codec(index = 226)]
                        Mortal226(::core::primitive::u8),
                        #[codec(index = 227)]
                        Mortal227(::core::primitive::u8),
                        #[codec(index = 228)]
                        Mortal228(::core::primitive::u8),
                        #[codec(index = 229)]
                        Mortal229(::core::primitive::u8),
                        #[codec(index = 230)]
                        Mortal230(::core::primitive::u8),
                        #[codec(index = 231)]
                        Mortal231(::core::primitive::u8),
                        #[codec(index = 232)]
                        Mortal232(::core::primitive::u8),
                        #[codec(index = 233)]
                        Mortal233(::core::primitive::u8),
                        #[codec(index = 234)]
                        Mortal234(::core::primitive::u8),
                        #[codec(index = 235)]
                        Mortal235(::core::primitive::u8),
                        #[codec(index = 236)]
                        Mortal236(::core::primitive::u8),
                        #[codec(index = 237)]
                        Mortal237(::core::primitive::u8),
                        #[codec(index = 238)]
                        Mortal238(::core::primitive::u8),
                        #[codec(index = 239)]
                        Mortal239(::core::primitive::u8),
                        #[codec(index = 240)]
                        Mortal240(::core::primitive::u8),
                        #[codec(index = 241)]
                        Mortal241(::core::primitive::u8),
                        #[codec(index = 242)]
                        Mortal242(::core::primitive::u8),
                        #[codec(index = 243)]
                        Mortal243(::core::primitive::u8),
                        #[codec(index = 244)]
                        Mortal244(::core::primitive::u8),
                        #[codec(index = 245)]
                        Mortal245(::core::primitive::u8),
                        #[codec(index = 246)]
                        Mortal246(::core::primitive::u8),
                        #[codec(index = 247)]
                        Mortal247(::core::primitive::u8),
                        #[codec(index = 248)]
                        Mortal248(::core::primitive::u8),
                        #[codec(index = 249)]
                        Mortal249(::core::primitive::u8),
                        #[codec(index = 250)]
                        Mortal250(::core::primitive::u8),
                        #[codec(index = 251)]
                        Mortal251(::core::primitive::u8),
                        #[codec(index = 252)]
                        Mortal252(::core::primitive::u8),
                        #[codec(index = 253)]
                        Mortal253(::core::primitive::u8),
                        #[codec(index = 254)]
                        Mortal254(::core::primitive::u8),
                        #[codec(index = 255)]
                        Mortal255(::core::primitive::u8),
                    }
                }
                pub mod header {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct Header<_0, _1> {
                        pub parent_hash: ::subxt::utils::H256,
                        #[codec(compact)]
                        pub number: _0,
                        pub state_root: ::subxt::utils::H256,
                        pub extrinsics_root: ::subxt::utils::H256,
                        pub digest: runtime_types::sp_runtime::generic::digest::Digest,
                        #[codec(skip)]
                        pub __subxt_unused_type_params: ::core::marker::PhantomData<_1>,
                    }
                }
                pub mod unchecked_extrinsic {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct UncheckedExtrinsic<_0, _1, _2, _3>(
                        pub ::std::vec::Vec<::core::primitive::u8>,
                        #[codec(skip)] pub ::core::marker::PhantomData<(_0, _1, _2, _3)>,
                    );
                }
            }
            pub mod traits {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct BlakeTwo256;
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum DispatchError {
                #[codec(index = 0)]
                Other,
                #[codec(index = 1)]
                CannotLookup,
                #[codec(index = 2)]
                BadOrigin,
                #[codec(index = 3)]
                Module(runtime_types::sp_runtime::ModuleError),
                #[codec(index = 4)]
                ConsumerRemaining,
                #[codec(index = 5)]
                NoProviders,
                #[codec(index = 6)]
                TooManyConsumers,
                #[codec(index = 7)]
                Token(runtime_types::sp_runtime::TokenError),
                #[codec(index = 8)]
                Arithmetic(runtime_types::sp_arithmetic::ArithmeticError),
                #[codec(index = 9)]
                Transactional(runtime_types::sp_runtime::TransactionalError),
                #[codec(index = 10)]
                Exhausted,
                #[codec(index = 11)]
                Corruption,
                #[codec(index = 12)]
                Unavailable,
                #[codec(index = 13)]
                RootNotAllowed,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct ModuleError {
                pub index: ::core::primitive::u8,
                pub error: [::core::primitive::u8; 4usize],
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum MultiSignature {
                #[codec(index = 0)]
                Ed25519(runtime_types::sp_core::ed25519::Signature),
                #[codec(index = 1)]
                Sr25519(runtime_types::sp_core::sr25519::Signature),
                #[codec(index = 2)]
                Ecdsa(runtime_types::sp_core::ecdsa::Signature),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum TokenError {
                #[codec(index = 0)]
                FundsUnavailable,
                #[codec(index = 1)]
                OnlyProvider,
                #[codec(index = 2)]
                BelowMinimum,
                #[codec(index = 3)]
                CannotCreate,
                #[codec(index = 4)]
                UnknownAsset,
                #[codec(index = 5)]
                Frozen,
                #[codec(index = 6)]
                Unsupported,
                #[codec(index = 7)]
                CannotCreateHold,
                #[codec(index = 8)]
                NotExpendable,
                #[codec(index = 9)]
                Blocked,
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum TransactionalError {
                #[codec(index = 0)]
                LimitReached,
                #[codec(index = 1)]
                NoLayer,
            }
        }
        pub mod sp_trie {
            use super::runtime_types;
            pub mod storage_proof {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct StorageProof {
                    pub trie_nodes: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
                }
            }
        }
        pub mod sp_version {
            use super::runtime_types;
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RuntimeVersion {
                pub spec_name: ::std::string::String,
                pub impl_name: ::std::string::String,
                pub authoring_version: ::core::primitive::u32,
                pub spec_version: ::core::primitive::u32,
                pub impl_version: ::core::primitive::u32,
                pub apis:
                    ::std::vec::Vec<([::core::primitive::u8; 8usize], ::core::primitive::u32)>,
                pub transaction_version: ::core::primitive::u32,
                pub state_version: ::core::primitive::u8,
            }
        }
        pub mod sp_weights {
            use super::runtime_types;
            pub mod weight_v2 {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Weight {
                    #[codec(compact)]
                    pub ref_time: ::core::primitive::u64,
                    #[codec(compact)]
                    pub proof_size: ::core::primitive::u64,
                }
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub struct RuntimeDbWeight {
                pub read: ::core::primitive::u64,
                pub write: ::core::primitive::u64,
            }
        }
        pub mod xcm {
            use super::runtime_types;
            pub mod double_encoded {
                use super::runtime_types;
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct DoubleEncoded {
                    pub encoded: ::std::vec::Vec<::core::primitive::u8>,
                }
            }
            pub mod v2 {
                use super::runtime_types;
                pub mod junction {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Junction {
                        #[codec(index = 0)]
                        Parachain(#[codec(compact)] ::core::primitive::u32),
                        #[codec(index = 1)]
                        AccountId32 {
                            network: runtime_types::xcm::v2::NetworkId,
                            id: [::core::primitive::u8; 32usize],
                        },
                        #[codec(index = 2)]
                        AccountIndex64 {
                            network: runtime_types::xcm::v2::NetworkId,
                            #[codec(compact)]
                            index: ::core::primitive::u64,
                        },
                        #[codec(index = 3)]
                        AccountKey20 {
                            network: runtime_types::xcm::v2::NetworkId,
                            key: [::core::primitive::u8; 20usize],
                        },
                        #[codec(index = 4)]
                        PalletInstance(::core::primitive::u8),
                        #[codec(index = 5)]
                        GeneralIndex(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 6)]
                        GeneralKey(
                            runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
                                ::core::primitive::u8,
                            >,
                        ),
                        #[codec(index = 7)]
                        OnlyChild,
                        #[codec(index = 8)]
                        Plurality {
                            id: runtime_types::xcm::v2::BodyId,
                            part: runtime_types::xcm::v2::BodyPart,
                        },
                    }
                }
                pub mod multiasset {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum AssetId {
                        #[codec(index = 0)]
                        Concrete(runtime_types::xcm::v2::multilocation::MultiLocation),
                        #[codec(index = 1)]
                        Abstract(::std::vec::Vec<::core::primitive::u8>),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum AssetInstance {
                        #[codec(index = 0)]
                        Undefined,
                        #[codec(index = 1)]
                        Index(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 2)]
                        Array4([::core::primitive::u8; 4usize]),
                        #[codec(index = 3)]
                        Array8([::core::primitive::u8; 8usize]),
                        #[codec(index = 4)]
                        Array16([::core::primitive::u8; 16usize]),
                        #[codec(index = 5)]
                        Array32([::core::primitive::u8; 32usize]),
                        #[codec(index = 6)]
                        Blob(::std::vec::Vec<::core::primitive::u8>),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Fungibility {
                        #[codec(index = 0)]
                        Fungible(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 1)]
                        NonFungible(runtime_types::xcm::v2::multiasset::AssetInstance),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiAsset {
                        pub id: runtime_types::xcm::v2::multiasset::AssetId,
                        pub fun: runtime_types::xcm::v2::multiasset::Fungibility,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum MultiAssetFilter {
                        #[codec(index = 0)]
                        Definite(runtime_types::xcm::v2::multiasset::MultiAssets),
                        #[codec(index = 1)]
                        Wild(runtime_types::xcm::v2::multiasset::WildMultiAsset),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiAssets(
                        pub ::std::vec::Vec<runtime_types::xcm::v2::multiasset::MultiAsset>,
                    );
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum WildFungibility {
                        #[codec(index = 0)]
                        Fungible,
                        #[codec(index = 1)]
                        NonFungible,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum WildMultiAsset {
                        #[codec(index = 0)]
                        All,
                        #[codec(index = 1)]
                        AllOf {
                            id: runtime_types::xcm::v2::multiasset::AssetId,
                            fun: runtime_types::xcm::v2::multiasset::WildFungibility,
                        },
                    }
                }
                pub mod multilocation {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Junctions {
                        #[codec(index = 0)]
                        Here,
                        #[codec(index = 1)]
                        X1(runtime_types::xcm::v2::junction::Junction),
                        #[codec(index = 2)]
                        X2(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 3)]
                        X3(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 4)]
                        X4(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 5)]
                        X5(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 6)]
                        X6(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 7)]
                        X7(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                        #[codec(index = 8)]
                        X8(
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                            runtime_types::xcm::v2::junction::Junction,
                        ),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiLocation {
                        pub parents: ::core::primitive::u8,
                        pub interior: runtime_types::xcm::v2::multilocation::Junctions,
                    }
                }
                pub mod traits {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Error {
                        #[codec(index = 0)]
                        Overflow,
                        #[codec(index = 1)]
                        Unimplemented,
                        #[codec(index = 2)]
                        UntrustedReserveLocation,
                        #[codec(index = 3)]
                        UntrustedTeleportLocation,
                        #[codec(index = 4)]
                        MultiLocationFull,
                        #[codec(index = 5)]
                        MultiLocationNotInvertible,
                        #[codec(index = 6)]
                        BadOrigin,
                        #[codec(index = 7)]
                        InvalidLocation,
                        #[codec(index = 8)]
                        AssetNotFound,
                        #[codec(index = 9)]
                        FailedToTransactAsset,
                        #[codec(index = 10)]
                        NotWithdrawable,
                        #[codec(index = 11)]
                        LocationCannotHold,
                        #[codec(index = 12)]
                        ExceedsMaxMessageSize,
                        #[codec(index = 13)]
                        DestinationUnsupported,
                        #[codec(index = 14)]
                        Transport,
                        #[codec(index = 15)]
                        Unroutable,
                        #[codec(index = 16)]
                        UnknownClaim,
                        #[codec(index = 17)]
                        FailedToDecode,
                        #[codec(index = 18)]
                        MaxWeightInvalid,
                        #[codec(index = 19)]
                        NotHoldingFees,
                        #[codec(index = 20)]
                        TooExpensive,
                        #[codec(index = 21)]
                        Trap(::core::primitive::u64),
                        #[codec(index = 22)]
                        UnhandledXcmVersion,
                        #[codec(index = 23)]
                        WeightLimitReached(::core::primitive::u64),
                        #[codec(index = 24)]
                        Barrier,
                        #[codec(index = 25)]
                        WeightNotComputable,
                    }
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum BodyId {
                    #[codec(index = 0)]
                    Unit,
                    #[codec(index = 1)]
                    Named(
                        runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
                            ::core::primitive::u8,
                        >,
                    ),
                    #[codec(index = 2)]
                    Index(#[codec(compact)] ::core::primitive::u32),
                    #[codec(index = 3)]
                    Executive,
                    #[codec(index = 4)]
                    Technical,
                    #[codec(index = 5)]
                    Legislative,
                    #[codec(index = 6)]
                    Judicial,
                    #[codec(index = 7)]
                    Defense,
                    #[codec(index = 8)]
                    Administration,
                    #[codec(index = 9)]
                    Treasury,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum BodyPart {
                    #[codec(index = 0)]
                    Voice,
                    #[codec(index = 1)]
                    Members {
                        #[codec(compact)]
                        count: ::core::primitive::u32,
                    },
                    #[codec(index = 2)]
                    Fraction {
                        #[codec(compact)]
                        nom: ::core::primitive::u32,
                        #[codec(compact)]
                        denom: ::core::primitive::u32,
                    },
                    #[codec(index = 3)]
                    AtLeastProportion {
                        #[codec(compact)]
                        nom: ::core::primitive::u32,
                        #[codec(compact)]
                        denom: ::core::primitive::u32,
                    },
                    #[codec(index = 4)]
                    MoreThanProportion {
                        #[codec(compact)]
                        nom: ::core::primitive::u32,
                        #[codec(compact)]
                        denom: ::core::primitive::u32,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Instruction {
                    #[codec(index = 0)]
                    WithdrawAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
                    #[codec(index = 1)]
                    ReserveAssetDeposited(runtime_types::xcm::v2::multiasset::MultiAssets),
                    #[codec(index = 2)]
                    ReceiveTeleportedAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
                    #[codec(index = 3)]
                    QueryResponse {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        response: runtime_types::xcm::v2::Response,
                        #[codec(compact)]
                        max_weight: ::core::primitive::u64,
                    },
                    #[codec(index = 4)]
                    TransferAsset {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssets,
                        beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
                    },
                    #[codec(index = 5)]
                    TransferReserveAsset {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssets,
                        dest: runtime_types::xcm::v2::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v2::Xcm,
                    },
                    #[codec(index = 6)]
                    Transact {
                        origin_type: runtime_types::xcm::v2::OriginKind,
                        #[codec(compact)]
                        require_weight_at_most: ::core::primitive::u64,
                        call: runtime_types::xcm::double_encoded::DoubleEncoded,
                    },
                    #[codec(index = 7)]
                    HrmpNewChannelOpenRequest {
                        #[codec(compact)]
                        sender: ::core::primitive::u32,
                        #[codec(compact)]
                        max_message_size: ::core::primitive::u32,
                        #[codec(compact)]
                        max_capacity: ::core::primitive::u32,
                    },
                    #[codec(index = 8)]
                    HrmpChannelAccepted {
                        #[codec(compact)]
                        recipient: ::core::primitive::u32,
                    },
                    #[codec(index = 9)]
                    HrmpChannelClosing {
                        #[codec(compact)]
                        initiator: ::core::primitive::u32,
                        #[codec(compact)]
                        sender: ::core::primitive::u32,
                        #[codec(compact)]
                        recipient: ::core::primitive::u32,
                    },
                    #[codec(index = 10)]
                    ClearOrigin,
                    #[codec(index = 11)]
                    DescendOrigin(runtime_types::xcm::v2::multilocation::Junctions),
                    #[codec(index = 12)]
                    ReportError {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        dest: runtime_types::xcm::v2::multilocation::MultiLocation,
                        #[codec(compact)]
                        max_response_weight: ::core::primitive::u64,
                    },
                    #[codec(index = 13)]
                    DepositAsset {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        #[codec(compact)]
                        max_assets: ::core::primitive::u32,
                        beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
                    },
                    #[codec(index = 14)]
                    DepositReserveAsset {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        #[codec(compact)]
                        max_assets: ::core::primitive::u32,
                        dest: runtime_types::xcm::v2::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v2::Xcm,
                    },
                    #[codec(index = 15)]
                    ExchangeAsset {
                        give: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        receive: runtime_types::xcm::v2::multiasset::MultiAssets,
                    },
                    #[codec(index = 16)]
                    InitiateReserveWithdraw {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        reserve: runtime_types::xcm::v2::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v2::Xcm,
                    },
                    #[codec(index = 17)]
                    InitiateTeleport {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        dest: runtime_types::xcm::v2::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v2::Xcm,
                    },
                    #[codec(index = 18)]
                    QueryHolding {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        dest: runtime_types::xcm::v2::multilocation::MultiLocation,
                        assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
                        #[codec(compact)]
                        max_response_weight: ::core::primitive::u64,
                    },
                    #[codec(index = 19)]
                    BuyExecution {
                        fees: runtime_types::xcm::v2::multiasset::MultiAsset,
                        weight_limit: runtime_types::xcm::v2::WeightLimit,
                    },
                    #[codec(index = 20)]
                    RefundSurplus,
                    #[codec(index = 21)]
                    SetErrorHandler(runtime_types::xcm::v2::Xcm),
                    #[codec(index = 22)]
                    SetAppendix(runtime_types::xcm::v2::Xcm),
                    #[codec(index = 23)]
                    ClearError,
                    #[codec(index = 24)]
                    ClaimAsset {
                        assets: runtime_types::xcm::v2::multiasset::MultiAssets,
                        ticket: runtime_types::xcm::v2::multilocation::MultiLocation,
                    },
                    #[codec(index = 25)]
                    Trap(#[codec(compact)] ::core::primitive::u64),
                    #[codec(index = 26)]
                    SubscribeVersion {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        #[codec(compact)]
                        max_response_weight: ::core::primitive::u64,
                    },
                    #[codec(index = 27)]
                    UnsubscribeVersion,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum NetworkId {
                    #[codec(index = 0)]
                    Any,
                    #[codec(index = 1)]
                    Named(
                        runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
                            ::core::primitive::u8,
                        >,
                    ),
                    #[codec(index = 2)]
                    Polkadot,
                    #[codec(index = 3)]
                    Kusama,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum OriginKind {
                    #[codec(index = 0)]
                    Native,
                    #[codec(index = 1)]
                    SovereignAccount,
                    #[codec(index = 2)]
                    Superuser,
                    #[codec(index = 3)]
                    Xcm,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Response {
                    #[codec(index = 0)]
                    Null,
                    #[codec(index = 1)]
                    Assets(runtime_types::xcm::v2::multiasset::MultiAssets),
                    #[codec(index = 2)]
                    ExecutionResult(
                        ::core::option::Option<(
                            ::core::primitive::u32,
                            runtime_types::xcm::v2::traits::Error,
                        )>,
                    ),
                    #[codec(index = 3)]
                    Version(::core::primitive::u32),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum WeightLimit {
                    #[codec(index = 0)]
                    Unlimited,
                    #[codec(index = 1)]
                    Limited(#[codec(compact)] ::core::primitive::u64),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Xcm(pub ::std::vec::Vec<runtime_types::xcm::v2::Instruction>);
            }
            pub mod v3 {
                use super::runtime_types;
                pub mod junction {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum BodyId {
                        #[codec(index = 0)]
                        Unit,
                        #[codec(index = 1)]
                        Moniker([::core::primitive::u8; 4usize]),
                        #[codec(index = 2)]
                        Index(#[codec(compact)] ::core::primitive::u32),
                        #[codec(index = 3)]
                        Executive,
                        #[codec(index = 4)]
                        Technical,
                        #[codec(index = 5)]
                        Legislative,
                        #[codec(index = 6)]
                        Judicial,
                        #[codec(index = 7)]
                        Defense,
                        #[codec(index = 8)]
                        Administration,
                        #[codec(index = 9)]
                        Treasury,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum BodyPart {
                        #[codec(index = 0)]
                        Voice,
                        #[codec(index = 1)]
                        Members {
                            #[codec(compact)]
                            count: ::core::primitive::u32,
                        },
                        #[codec(index = 2)]
                        Fraction {
                            #[codec(compact)]
                            nom: ::core::primitive::u32,
                            #[codec(compact)]
                            denom: ::core::primitive::u32,
                        },
                        #[codec(index = 3)]
                        AtLeastProportion {
                            #[codec(compact)]
                            nom: ::core::primitive::u32,
                            #[codec(compact)]
                            denom: ::core::primitive::u32,
                        },
                        #[codec(index = 4)]
                        MoreThanProportion {
                            #[codec(compact)]
                            nom: ::core::primitive::u32,
                            #[codec(compact)]
                            denom: ::core::primitive::u32,
                        },
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Junction {
                        #[codec(index = 0)]
                        Parachain(#[codec(compact)] ::core::primitive::u32),
                        #[codec(index = 1)]
                        AccountId32 {
                            network:
                                ::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
                            id: [::core::primitive::u8; 32usize],
                        },
                        #[codec(index = 2)]
                        AccountIndex64 {
                            network:
                                ::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
                            #[codec(compact)]
                            index: ::core::primitive::u64,
                        },
                        #[codec(index = 3)]
                        AccountKey20 {
                            network:
                                ::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
                            key: [::core::primitive::u8; 20usize],
                        },
                        #[codec(index = 4)]
                        PalletInstance(::core::primitive::u8),
                        #[codec(index = 5)]
                        GeneralIndex(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 6)]
                        GeneralKey {
                            length: ::core::primitive::u8,
                            data: [::core::primitive::u8; 32usize],
                        },
                        #[codec(index = 7)]
                        OnlyChild,
                        #[codec(index = 8)]
                        Plurality {
                            id: runtime_types::xcm::v3::junction::BodyId,
                            part: runtime_types::xcm::v3::junction::BodyPart,
                        },
                        #[codec(index = 9)]
                        GlobalConsensus(runtime_types::xcm::v3::junction::NetworkId),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum NetworkId {
                        #[codec(index = 0)]
                        ByGenesis([::core::primitive::u8; 32usize]),
                        #[codec(index = 1)]
                        ByFork {
                            block_number: ::core::primitive::u64,
                            block_hash: [::core::primitive::u8; 32usize],
                        },
                        #[codec(index = 2)]
                        Polkadot,
                        #[codec(index = 3)]
                        Kusama,
                        #[codec(index = 4)]
                        Westend,
                        #[codec(index = 5)]
                        Rococo,
                        #[codec(index = 6)]
                        Wococo,
                        #[codec(index = 7)]
                        Ethereum {
                            #[codec(compact)]
                            chain_id: ::core::primitive::u64,
                        },
                        #[codec(index = 8)]
                        BitcoinCore,
                        #[codec(index = 9)]
                        BitcoinCash,
                    }
                }
                pub mod junctions {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Junctions {
                        #[codec(index = 0)]
                        Here,
                        #[codec(index = 1)]
                        X1(runtime_types::xcm::v3::junction::Junction),
                        #[codec(index = 2)]
                        X2(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 3)]
                        X3(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 4)]
                        X4(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 5)]
                        X5(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 6)]
                        X6(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 7)]
                        X7(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                        #[codec(index = 8)]
                        X8(
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                            runtime_types::xcm::v3::junction::Junction,
                        ),
                    }
                }
                pub mod multiasset {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum AssetId {
                        #[codec(index = 0)]
                        Concrete(runtime_types::xcm::v3::multilocation::MultiLocation),
                        #[codec(index = 1)]
                        Abstract([::core::primitive::u8; 32usize]),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum AssetInstance {
                        #[codec(index = 0)]
                        Undefined,
                        #[codec(index = 1)]
                        Index(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 2)]
                        Array4([::core::primitive::u8; 4usize]),
                        #[codec(index = 3)]
                        Array8([::core::primitive::u8; 8usize]),
                        #[codec(index = 4)]
                        Array16([::core::primitive::u8; 16usize]),
                        #[codec(index = 5)]
                        Array32([::core::primitive::u8; 32usize]),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Fungibility {
                        #[codec(index = 0)]
                        Fungible(#[codec(compact)] ::core::primitive::u128),
                        #[codec(index = 1)]
                        NonFungible(runtime_types::xcm::v3::multiasset::AssetInstance),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiAsset {
                        pub id: runtime_types::xcm::v3::multiasset::AssetId,
                        pub fun: runtime_types::xcm::v3::multiasset::Fungibility,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum MultiAssetFilter {
                        #[codec(index = 0)]
                        Definite(runtime_types::xcm::v3::multiasset::MultiAssets),
                        #[codec(index = 1)]
                        Wild(runtime_types::xcm::v3::multiasset::WildMultiAsset),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiAssets(
                        pub ::std::vec::Vec<runtime_types::xcm::v3::multiasset::MultiAsset>,
                    );
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum WildFungibility {
                        #[codec(index = 0)]
                        Fungible,
                        #[codec(index = 1)]
                        NonFungible,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum WildMultiAsset {
                        #[codec(index = 0)]
                        All,
                        #[codec(index = 1)]
                        AllOf {
                            id: runtime_types::xcm::v3::multiasset::AssetId,
                            fun: runtime_types::xcm::v3::multiasset::WildFungibility,
                        },
                        #[codec(index = 2)]
                        AllCounted(#[codec(compact)] ::core::primitive::u32),
                        #[codec(index = 3)]
                        AllOfCounted {
                            id: runtime_types::xcm::v3::multiasset::AssetId,
                            fun: runtime_types::xcm::v3::multiasset::WildFungibility,
                            #[codec(compact)]
                            count: ::core::primitive::u32,
                        },
                    }
                }
                pub mod multilocation {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub struct MultiLocation {
                        pub parents: ::core::primitive::u8,
                        pub interior: runtime_types::xcm::v3::junctions::Junctions,
                    }
                }
                pub mod traits {
                    use super::runtime_types;
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Error {
                        #[codec(index = 0)]
                        Overflow,
                        #[codec(index = 1)]
                        Unimplemented,
                        #[codec(index = 2)]
                        UntrustedReserveLocation,
                        #[codec(index = 3)]
                        UntrustedTeleportLocation,
                        #[codec(index = 4)]
                        LocationFull,
                        #[codec(index = 5)]
                        LocationNotInvertible,
                        #[codec(index = 6)]
                        BadOrigin,
                        #[codec(index = 7)]
                        InvalidLocation,
                        #[codec(index = 8)]
                        AssetNotFound,
                        #[codec(index = 9)]
                        FailedToTransactAsset,
                        #[codec(index = 10)]
                        NotWithdrawable,
                        #[codec(index = 11)]
                        LocationCannotHold,
                        #[codec(index = 12)]
                        ExceedsMaxMessageSize,
                        #[codec(index = 13)]
                        DestinationUnsupported,
                        #[codec(index = 14)]
                        Transport,
                        #[codec(index = 15)]
                        Unroutable,
                        #[codec(index = 16)]
                        UnknownClaim,
                        #[codec(index = 17)]
                        FailedToDecode,
                        #[codec(index = 18)]
                        MaxWeightInvalid,
                        #[codec(index = 19)]
                        NotHoldingFees,
                        #[codec(index = 20)]
                        TooExpensive,
                        #[codec(index = 21)]
                        Trap(::core::primitive::u64),
                        #[codec(index = 22)]
                        ExpectationFalse,
                        #[codec(index = 23)]
                        PalletNotFound,
                        #[codec(index = 24)]
                        NameMismatch,
                        #[codec(index = 25)]
                        VersionIncompatible,
                        #[codec(index = 26)]
                        HoldingWouldOverflow,
                        #[codec(index = 27)]
                        ExportError,
                        #[codec(index = 28)]
                        ReanchorFailed,
                        #[codec(index = 29)]
                        NoDeal,
                        #[codec(index = 30)]
                        FeesNotMet,
                        #[codec(index = 31)]
                        LockError,
                        #[codec(index = 32)]
                        NoPermission,
                        #[codec(index = 33)]
                        Unanchored,
                        #[codec(index = 34)]
                        NotDepositable,
                        #[codec(index = 35)]
                        UnhandledXcmVersion,
                        #[codec(index = 36)]
                        WeightLimitReached(runtime_types::sp_weights::weight_v2::Weight),
                        #[codec(index = 37)]
                        Barrier,
                        #[codec(index = 38)]
                        WeightNotComputable,
                        #[codec(index = 39)]
                        ExceedsStackLimit,
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum Outcome {
                        #[codec(index = 0)]
                        Complete(runtime_types::sp_weights::weight_v2::Weight),
                        #[codec(index = 1)]
                        Incomplete(
                            runtime_types::sp_weights::weight_v2::Weight,
                            runtime_types::xcm::v3::traits::Error,
                        ),
                        #[codec(index = 2)]
                        Error(runtime_types::xcm::v3::traits::Error),
                    }
                    #[derive(
                        :: subxt :: ext :: codec :: Decode,
                        :: subxt :: ext :: codec :: Encode,
                        Debug,
                    )]
                    pub enum SendError {
                        #[codec(index = 0)]
                        NotApplicable,
                        #[codec(index = 1)]
                        Transport,
                        #[codec(index = 2)]
                        Unroutable,
                        #[codec(index = 3)]
                        DestinationUnsupported,
                        #[codec(index = 4)]
                        ExceedsMaxMessageSize,
                        #[codec(index = 5)]
                        MissingArgument,
                        #[codec(index = 6)]
                        Fees,
                    }
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Instruction {
                    #[codec(index = 0)]
                    WithdrawAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 1)]
                    ReserveAssetDeposited(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 2)]
                    ReceiveTeleportedAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 3)]
                    QueryResponse {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        response: runtime_types::xcm::v3::Response,
                        max_weight: runtime_types::sp_weights::weight_v2::Weight,
                        querier: ::core::option::Option<
                            runtime_types::xcm::v3::multilocation::MultiLocation,
                        >,
                    },
                    #[codec(index = 4)]
                    TransferAsset {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssets,
                        beneficiary: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 5)]
                    TransferReserveAsset {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssets,
                        dest: runtime_types::xcm::v3::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v3::Xcm,
                    },
                    #[codec(index = 6)]
                    Transact {
                        origin_kind: runtime_types::xcm::v2::OriginKind,
                        require_weight_at_most: runtime_types::sp_weights::weight_v2::Weight,
                        call: runtime_types::xcm::double_encoded::DoubleEncoded,
                    },
                    #[codec(index = 7)]
                    HrmpNewChannelOpenRequest {
                        #[codec(compact)]
                        sender: ::core::primitive::u32,
                        #[codec(compact)]
                        max_message_size: ::core::primitive::u32,
                        #[codec(compact)]
                        max_capacity: ::core::primitive::u32,
                    },
                    #[codec(index = 8)]
                    HrmpChannelAccepted {
                        #[codec(compact)]
                        recipient: ::core::primitive::u32,
                    },
                    #[codec(index = 9)]
                    HrmpChannelClosing {
                        #[codec(compact)]
                        initiator: ::core::primitive::u32,
                        #[codec(compact)]
                        sender: ::core::primitive::u32,
                        #[codec(compact)]
                        recipient: ::core::primitive::u32,
                    },
                    #[codec(index = 10)]
                    ClearOrigin,
                    #[codec(index = 11)]
                    DescendOrigin(runtime_types::xcm::v3::junctions::Junctions),
                    #[codec(index = 12)]
                    ReportError(runtime_types::xcm::v3::QueryResponseInfo),
                    #[codec(index = 13)]
                    DepositAsset {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                        beneficiary: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 14)]
                    DepositReserveAsset {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                        dest: runtime_types::xcm::v3::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v3::Xcm,
                    },
                    #[codec(index = 15)]
                    ExchangeAsset {
                        give: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                        want: runtime_types::xcm::v3::multiasset::MultiAssets,
                        maximal: ::core::primitive::bool,
                    },
                    #[codec(index = 16)]
                    InitiateReserveWithdraw {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                        reserve: runtime_types::xcm::v3::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v3::Xcm,
                    },
                    #[codec(index = 17)]
                    InitiateTeleport {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                        dest: runtime_types::xcm::v3::multilocation::MultiLocation,
                        xcm: runtime_types::xcm::v3::Xcm,
                    },
                    #[codec(index = 18)]
                    ReportHolding {
                        response_info: runtime_types::xcm::v3::QueryResponseInfo,
                        assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
                    },
                    #[codec(index = 19)]
                    BuyExecution {
                        fees: runtime_types::xcm::v3::multiasset::MultiAsset,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 20)]
                    RefundSurplus,
                    #[codec(index = 21)]
                    SetErrorHandler(runtime_types::xcm::v3::Xcm),
                    #[codec(index = 22)]
                    SetAppendix(runtime_types::xcm::v3::Xcm),
                    #[codec(index = 23)]
                    ClearError,
                    #[codec(index = 24)]
                    ClaimAsset {
                        assets: runtime_types::xcm::v3::multiasset::MultiAssets,
                        ticket: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 25)]
                    Trap(#[codec(compact)] ::core::primitive::u64),
                    #[codec(index = 26)]
                    SubscribeVersion {
                        #[codec(compact)]
                        query_id: ::core::primitive::u64,
                        max_response_weight: runtime_types::sp_weights::weight_v2::Weight,
                    },
                    #[codec(index = 27)]
                    UnsubscribeVersion,
                    #[codec(index = 28)]
                    BurnAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 29)]
                    ExpectAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 30)]
                    ExpectOrigin(
                        ::core::option::Option<
                            runtime_types::xcm::v3::multilocation::MultiLocation,
                        >,
                    ),
                    #[codec(index = 31)]
                    ExpectError(
                        ::core::option::Option<(
                            ::core::primitive::u32,
                            runtime_types::xcm::v3::traits::Error,
                        )>,
                    ),
                    #[codec(index = 32)]
                    ExpectTransactStatus(runtime_types::xcm::v3::MaybeErrorCode),
                    #[codec(index = 33)]
                    QueryPallet {
                        module_name: ::std::vec::Vec<::core::primitive::u8>,
                        response_info: runtime_types::xcm::v3::QueryResponseInfo,
                    },
                    #[codec(index = 34)]
                    ExpectPallet {
                        #[codec(compact)]
                        index: ::core::primitive::u32,
                        name: ::std::vec::Vec<::core::primitive::u8>,
                        module_name: ::std::vec::Vec<::core::primitive::u8>,
                        #[codec(compact)]
                        crate_major: ::core::primitive::u32,
                        #[codec(compact)]
                        min_crate_minor: ::core::primitive::u32,
                    },
                    #[codec(index = 35)]
                    ReportTransactStatus(runtime_types::xcm::v3::QueryResponseInfo),
                    #[codec(index = 36)]
                    ClearTransactStatus,
                    #[codec(index = 37)]
                    UniversalOrigin(runtime_types::xcm::v3::junction::Junction),
                    #[codec(index = 38)]
                    ExportMessage {
                        network: runtime_types::xcm::v3::junction::NetworkId,
                        destination: runtime_types::xcm::v3::junctions::Junctions,
                        xcm: runtime_types::xcm::v3::Xcm,
                    },
                    #[codec(index = 39)]
                    LockAsset {
                        asset: runtime_types::xcm::v3::multiasset::MultiAsset,
                        unlocker: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 40)]
                    UnlockAsset {
                        asset: runtime_types::xcm::v3::multiasset::MultiAsset,
                        target: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 41)]
                    NoteUnlockable {
                        asset: runtime_types::xcm::v3::multiasset::MultiAsset,
                        owner: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 42)]
                    RequestUnlock {
                        asset: runtime_types::xcm::v3::multiasset::MultiAsset,
                        locker: runtime_types::xcm::v3::multilocation::MultiLocation,
                    },
                    #[codec(index = 43)]
                    SetFeesMode {
                        jit_withdraw: ::core::primitive::bool,
                    },
                    #[codec(index = 44)]
                    SetTopic([::core::primitive::u8; 32usize]),
                    #[codec(index = 45)]
                    ClearTopic,
                    #[codec(index = 46)]
                    AliasOrigin(runtime_types::xcm::v3::multilocation::MultiLocation),
                    #[codec(index = 47)]
                    UnpaidExecution {
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                        check_origin: ::core::option::Option<
                            runtime_types::xcm::v3::multilocation::MultiLocation,
                        >,
                    },
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum MaybeErrorCode {
                    #[codec(index = 0)]
                    Success,
                    #[codec(index = 1)]
                    Error(
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    ),
                    #[codec(index = 2)]
                    TruncatedError(
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            ::core::primitive::u8,
                        >,
                    ),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct PalletInfo {
                    #[codec(compact)]
                    pub index: ::core::primitive::u32,
                    pub name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
                        ::core::primitive::u8,
                    >,
                    pub module_name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
                        ::core::primitive::u8,
                    >,
                    #[codec(compact)]
                    pub major: ::core::primitive::u32,
                    #[codec(compact)]
                    pub minor: ::core::primitive::u32,
                    #[codec(compact)]
                    pub patch: ::core::primitive::u32,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct QueryResponseInfo {
                    pub destination: runtime_types::xcm::v3::multilocation::MultiLocation,
                    #[codec(compact)]
                    pub query_id: ::core::primitive::u64,
                    pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum Response {
                    #[codec(index = 0)]
                    Null,
                    #[codec(index = 1)]
                    Assets(runtime_types::xcm::v3::multiasset::MultiAssets),
                    #[codec(index = 2)]
                    ExecutionResult(
                        ::core::option::Option<(
                            ::core::primitive::u32,
                            runtime_types::xcm::v3::traits::Error,
                        )>,
                    ),
                    #[codec(index = 3)]
                    Version(::core::primitive::u32),
                    #[codec(index = 4)]
                    PalletsInfo(
                        runtime_types::bounded_collections::bounded_vec::BoundedVec<
                            runtime_types::xcm::v3::PalletInfo,
                        >,
                    ),
                    #[codec(index = 5)]
                    DispatchResult(runtime_types::xcm::v3::MaybeErrorCode),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub enum WeightLimit {
                    #[codec(index = 0)]
                    Unlimited,
                    #[codec(index = 1)]
                    Limited(runtime_types::sp_weights::weight_v2::Weight),
                }
                #[derive(
                    :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
                )]
                pub struct Xcm(pub ::std::vec::Vec<runtime_types::xcm::v3::Instruction>);
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum VersionedMultiAssets {
                #[codec(index = 1)]
                V2(runtime_types::xcm::v2::multiasset::MultiAssets),
                #[codec(index = 3)]
                V3(runtime_types::xcm::v3::multiasset::MultiAssets),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum VersionedMultiLocation {
                #[codec(index = 1)]
                V2(runtime_types::xcm::v2::multilocation::MultiLocation),
                #[codec(index = 3)]
                V3(runtime_types::xcm::v3::multilocation::MultiLocation),
            }
            #[derive(
                :: subxt :: ext :: codec :: Decode, :: subxt :: ext :: codec :: Encode, Debug,
            )]
            pub enum VersionedXcm {
                #[codec(index = 2)]
                V2(runtime_types::xcm::v2::Xcm),
                #[codec(index = 3)]
                V3(runtime_types::xcm::v3::Xcm),
            }
        }
    }
    #[doc = r" The default error type returned when there is a runtime issue,"]
    #[doc = r" exposed here for ease of use."]
    pub type DispatchError = runtime_types::sp_runtime::DispatchError;
    pub fn constants() -> ConstantsApi {
        ConstantsApi
    }
    pub fn storage() -> StorageApi {
        StorageApi
    }
    pub fn tx() -> TransactionApi {
        TransactionApi
    }
    pub struct ConstantsApi;
    impl ConstantsApi {
        pub fn system(&self) -> system::constants::ConstantsApi {
            system::constants::ConstantsApi
        }
        pub fn timestamp(&self) -> timestamp::constants::ConstantsApi {
            timestamp::constants::ConstantsApi
        }
        pub fn balances(&self) -> balances::constants::ConstantsApi {
            balances::constants::ConstantsApi
        }
        pub fn transaction_payment(&self) -> transaction_payment::constants::ConstantsApi {
            transaction_payment::constants::ConstantsApi
        }
        pub fn utility(&self) -> utility::constants::ConstantsApi {
            utility::constants::ConstantsApi
        }
        pub fn multisig(&self) -> multisig::constants::ConstantsApi {
            multisig::constants::ConstantsApi
        }
        pub fn bridge_wococo_grandpa(&self) -> bridge_wococo_grandpa::constants::ConstantsApi {
            bridge_wococo_grandpa::constants::ConstantsApi
        }
        pub fn bridge_wococo_parachain(&self) -> bridge_wococo_parachain::constants::ConstantsApi {
            bridge_wococo_parachain::constants::ConstantsApi
        }
        pub fn bridge_wococo_messages(&self) -> bridge_wococo_messages::constants::ConstantsApi {
            bridge_wococo_messages::constants::ConstantsApi
        }
        pub fn bridge_rococo_grandpa(&self) -> bridge_rococo_grandpa::constants::ConstantsApi {
            bridge_rococo_grandpa::constants::ConstantsApi
        }
        pub fn bridge_rococo_parachain(&self) -> bridge_rococo_parachain::constants::ConstantsApi {
            bridge_rococo_parachain::constants::ConstantsApi
        }
        pub fn bridge_rococo_messages(&self) -> bridge_rococo_messages::constants::ConstantsApi {
            bridge_rococo_messages::constants::ConstantsApi
        }
        pub fn ethereum_outbound_queue(&self) -> ethereum_outbound_queue::constants::ConstantsApi {
            ethereum_outbound_queue::constants::ConstantsApi
        }
        pub fn ethereum_beacon_client(&self) -> ethereum_beacon_client::constants::ConstantsApi {
            ethereum_beacon_client::constants::ConstantsApi
        }
        pub fn message_queue(&self) -> message_queue::constants::ConstantsApi {
            message_queue::constants::ConstantsApi
        }
    }
    pub struct StorageApi;
    impl StorageApi {
        pub fn system(&self) -> system::storage::StorageApi {
            system::storage::StorageApi
        }
        pub fn parachain_system(&self) -> parachain_system::storage::StorageApi {
            parachain_system::storage::StorageApi
        }
        pub fn timestamp(&self) -> timestamp::storage::StorageApi {
            timestamp::storage::StorageApi
        }
        pub fn parachain_info(&self) -> parachain_info::storage::StorageApi {
            parachain_info::storage::StorageApi
        }
        pub fn balances(&self) -> balances::storage::StorageApi {
            balances::storage::StorageApi
        }
        pub fn transaction_payment(&self) -> transaction_payment::storage::StorageApi {
            transaction_payment::storage::StorageApi
        }
        pub fn authorship(&self) -> authorship::storage::StorageApi {
            authorship::storage::StorageApi
        }
        pub fn collator_selection(&self) -> collator_selection::storage::StorageApi {
            collator_selection::storage::StorageApi
        }
        pub fn session(&self) -> session::storage::StorageApi {
            session::storage::StorageApi
        }
        pub fn aura(&self) -> aura::storage::StorageApi {
            aura::storage::StorageApi
        }
        pub fn aura_ext(&self) -> aura_ext::storage::StorageApi {
            aura_ext::storage::StorageApi
        }
        pub fn xcmp_queue(&self) -> xcmp_queue::storage::StorageApi {
            xcmp_queue::storage::StorageApi
        }
        pub fn dmp_queue(&self) -> dmp_queue::storage::StorageApi {
            dmp_queue::storage::StorageApi
        }
        pub fn multisig(&self) -> multisig::storage::StorageApi {
            multisig::storage::StorageApi
        }
        pub fn bridge_wococo_grandpa(&self) -> bridge_wococo_grandpa::storage::StorageApi {
            bridge_wococo_grandpa::storage::StorageApi
        }
        pub fn bridge_wococo_parachain(&self) -> bridge_wococo_parachain::storage::StorageApi {
            bridge_wococo_parachain::storage::StorageApi
        }
        pub fn bridge_wococo_messages(&self) -> bridge_wococo_messages::storage::StorageApi {
            bridge_wococo_messages::storage::StorageApi
        }
        pub fn bridge_rococo_grandpa(&self) -> bridge_rococo_grandpa::storage::StorageApi {
            bridge_rococo_grandpa::storage::StorageApi
        }
        pub fn bridge_rococo_parachain(&self) -> bridge_rococo_parachain::storage::StorageApi {
            bridge_rococo_parachain::storage::StorageApi
        }
        pub fn bridge_rococo_messages(&self) -> bridge_rococo_messages::storage::StorageApi {
            bridge_rococo_messages::storage::StorageApi
        }
        pub fn bridge_relayers(&self) -> bridge_relayers::storage::StorageApi {
            bridge_relayers::storage::StorageApi
        }
        pub fn ethereum_inbound_queue(&self) -> ethereum_inbound_queue::storage::StorageApi {
            ethereum_inbound_queue::storage::StorageApi
        }
        pub fn ethereum_outbound_queue(&self) -> ethereum_outbound_queue::storage::StorageApi {
            ethereum_outbound_queue::storage::StorageApi
        }
        pub fn ethereum_beacon_client(&self) -> ethereum_beacon_client::storage::StorageApi {
            ethereum_beacon_client::storage::StorageApi
        }
        pub fn ethereum_control(&self) -> ethereum_control::storage::StorageApi {
            ethereum_control::storage::StorageApi
        }
        pub fn message_queue(&self) -> message_queue::storage::StorageApi {
            message_queue::storage::StorageApi
        }
    }
    pub struct TransactionApi;
    impl TransactionApi {
        pub fn system(&self) -> system::calls::TransactionApi {
            system::calls::TransactionApi
        }
        pub fn parachain_system(&self) -> parachain_system::calls::TransactionApi {
            parachain_system::calls::TransactionApi
        }
        pub fn timestamp(&self) -> timestamp::calls::TransactionApi {
            timestamp::calls::TransactionApi
        }
        pub fn balances(&self) -> balances::calls::TransactionApi {
            balances::calls::TransactionApi
        }
        pub fn collator_selection(&self) -> collator_selection::calls::TransactionApi {
            collator_selection::calls::TransactionApi
        }
        pub fn session(&self) -> session::calls::TransactionApi {
            session::calls::TransactionApi
        }
        pub fn xcmp_queue(&self) -> xcmp_queue::calls::TransactionApi {
            xcmp_queue::calls::TransactionApi
        }
        pub fn polkadot_xcm(&self) -> polkadot_xcm::calls::TransactionApi {
            polkadot_xcm::calls::TransactionApi
        }
        pub fn dmp_queue(&self) -> dmp_queue::calls::TransactionApi {
            dmp_queue::calls::TransactionApi
        }
        pub fn utility(&self) -> utility::calls::TransactionApi {
            utility::calls::TransactionApi
        }
        pub fn multisig(&self) -> multisig::calls::TransactionApi {
            multisig::calls::TransactionApi
        }
        pub fn bridge_wococo_grandpa(&self) -> bridge_wococo_grandpa::calls::TransactionApi {
            bridge_wococo_grandpa::calls::TransactionApi
        }
        pub fn bridge_wococo_parachain(&self) -> bridge_wococo_parachain::calls::TransactionApi {
            bridge_wococo_parachain::calls::TransactionApi
        }
        pub fn bridge_wococo_messages(&self) -> bridge_wococo_messages::calls::TransactionApi {
            bridge_wococo_messages::calls::TransactionApi
        }
        pub fn bridge_rococo_grandpa(&self) -> bridge_rococo_grandpa::calls::TransactionApi {
            bridge_rococo_grandpa::calls::TransactionApi
        }
        pub fn bridge_rococo_parachain(&self) -> bridge_rococo_parachain::calls::TransactionApi {
            bridge_rococo_parachain::calls::TransactionApi
        }
        pub fn bridge_rococo_messages(&self) -> bridge_rococo_messages::calls::TransactionApi {
            bridge_rococo_messages::calls::TransactionApi
        }
        pub fn bridge_relayers(&self) -> bridge_relayers::calls::TransactionApi {
            bridge_relayers::calls::TransactionApi
        }
        pub fn ethereum_inbound_queue(&self) -> ethereum_inbound_queue::calls::TransactionApi {
            ethereum_inbound_queue::calls::TransactionApi
        }
        pub fn ethereum_outbound_queue(&self) -> ethereum_outbound_queue::calls::TransactionApi {
            ethereum_outbound_queue::calls::TransactionApi
        }
        pub fn ethereum_beacon_client(&self) -> ethereum_beacon_client::calls::TransactionApi {
            ethereum_beacon_client::calls::TransactionApi
        }
        pub fn ethereum_control(&self) -> ethereum_control::calls::TransactionApi {
            ethereum_control::calls::TransactionApi
        }
        pub fn message_queue(&self) -> message_queue::calls::TransactionApi {
            message_queue::calls::TransactionApi
        }
    }
    #[doc = r" check whether the Client you are using is aligned with the statically generated codegen."]
    pub fn validate_codegen<T: ::subxt::Config, C: ::subxt::client::OfflineClientT<T>>(
        client: &C,
    ) -> Result<(), ::subxt::error::MetadataError> {
        let runtime_metadata_hash = client.metadata().metadata_hash(&PALLETS);
        if runtime_metadata_hash
            != [
                164u8, 51u8, 44u8, 207u8, 3u8, 152u8, 6u8, 188u8, 153u8, 4u8, 195u8, 79u8, 122u8,
                161u8, 92u8, 249u8, 30u8, 8u8, 246u8, 158u8, 61u8, 235u8, 57u8, 91u8, 11u8, 7u8,
                156u8, 101u8, 89u8, 157u8, 23u8, 250u8,
            ]
        {
            Err(::subxt::error::MetadataError::IncompatibleMetadata)
        } else {
            Ok(())
        }
    }
}
