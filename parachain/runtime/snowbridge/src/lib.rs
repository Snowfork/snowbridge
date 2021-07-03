#![allow(clippy::all)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_api::impl_runtime_apis;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2},
	OpaqueMetadata, U256,
};
use sp_runtime::traits::{
	AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, IdentifyAccount, Keccak256, Verify,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};

use sp_std::prelude::*;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchResult,
	parameter_types,
	traits::{All, Filter, IsInVec, KeyOwnerProofSystem, Randomness},
	weights::{constants::WEIGHT_PER_SECOND, IdentityFee, Weight},
	PalletId, StorageValue,
	match_type,
};
use frame_system::{EnsureOneOf, EnsureRoot};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{traits::AccountIdConversion, Perbill, Permill};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

pub use artemis_core::{AssetId, ChannelId, MessageId};
use dispatch::EnsureEthereumAccount;

pub use verifier_lightclient::{EthereumDifficultyConfig, EthereumHeader};

use polkadot_parachain::primitives::Sibling;
use xcm::v0::{MultiAsset, Junction, MultiLocation, NetworkId, Xcm, BodyId};
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
	EnsureXcmOrigin, UsingComponents, FixedWeightBounds, IsConcrete, LocationInverter,
	NativeAsset, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SovereignSignedViaLocation, TakeWeightCredit, SignedToAccountId32,
};
use xcm_executor::{Config, XcmExecutor};
use pallet_xcm::XcmPassthrough;

use artemis_xcm_support::AssetsTransactor;
use assets::SingleAssetAdaptor;

use runtime_common::{
	INDEXING_PREFIX,
	OutboundRouter,
	Ether,
	MaxMessagePayloadSize,
	MaxMessagesPerCommit,
	DotPalletId,
	TreasuryPalletId,
};

mod weights;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("snowbridge"),
	impl_name: create_runtime_str!("snowbridge"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 12000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = runtime_common::build_block_weights(
		weights::constants::BlockExecutionWeight::get(),
		weights::constants::ExtrinsicBaseWeight::get(),
		2 * WEIGHT_PER_SECOND,
		NORMAL_DISPATCH_RATIO,
	);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = weights::constants::RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system_weights::WeightInfo<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp_weights::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = weights::pallet_utility_weights::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances_weights::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

// Cumulus and XCMP

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
	pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain(ParachainInfo::parachain_id().into()));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RococoNetwork, AccountId>,
);

type LocalAssetTransactor1 = AssetsTransactor<Assets, LocationToAccountId, AccountId>;
type LocalAssetTransactor2 = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RococoLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	()
>;

type LocalAssetTransactor = (LocalAssetTransactor1, LocalAssetTransactor2);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RococoNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	pub UnitWeightCost: Weight = 1_000_000;
}

match_type! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation::X1(Junction::Parent) |
		MultiLocation::X2(Junction::Parent, Junction::Plurality { id: BodyId::Executive, .. })
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = NativeAsset;
	type IsTeleporter = NativeAsset; // <- should be enough to allow teleportation of ROC
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type Trader = UsingComponents<IdentityFee<Balance>, RococoLocation, AccountId, Balances, ()>;
	type ResponseHandler = (); // Don't handle responses for now.
}

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = (
	SignedToAccountId32<Origin, AccountId, RococoNetwork>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type XcmReserveTransferFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// Governance

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

type EnsureRootOrHalfLocalCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, LocalCouncilInstance>,
>;

parameter_types! {
	pub const LocalCouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const LocalCouncilMaxProposals: u32 = 100;
	pub const LocalCouncilMaxMembers: u32 = 8;
}

type LocalCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Config<LocalCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = LocalCouncilMotionDuration;
	type MaxProposals = LocalCouncilMaxProposals;
	type MaxMembers = LocalCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective_weights::WeightInfo<Runtime>;
}

type LocalCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<LocalCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrHalfLocalCouncil;
	type RemoveOrigin = EnsureRootOrHalfLocalCouncil;
	type SwapOrigin = EnsureRootOrHalfLocalCouncil;
	type ResetOrigin = EnsureRootOrHalfLocalCouncil;
	type PrimeOrigin = EnsureRootOrHalfLocalCouncil;
	type MembershipInitialized = LocalCouncil;
	type MembershipChanged = LocalCouncil;
	type MaxMembers = LocalCouncilMaxMembers;
	type WeightInfo = ();
}

// Our pallets

pub struct CallFilter;
impl Filter<Call> for CallFilter {
	fn filter(call: &Call) -> bool {
		match call {
			Call::ETH(_) | Call::ERC20(_) | Call::DOT(_) => true,
			_ => false,
		}
	}
}

impl dispatch::Config for Runtime {
	type Origin = Origin;
	type Event = Event;
	type MessageId = MessageId;
	type Call = Call;
	type CallFilter = CallFilter;
}

use basic_channel::inbound as basic_channel_inbound;
use basic_channel::outbound as basic_channel_outbound;
use incentivized_channel::inbound as incentivized_channel_inbound;
use incentivized_channel::outbound as incentivized_channel_outbound;

impl basic_channel_inbound::Config for Runtime {
	type Event = Event;
	type Verifier = verifier_lightclient::Module<Runtime>;
	type MessageDispatch = dispatch::Module<Runtime>;
	type WeightInfo = weights::basic_channel_inbound_weights::WeightInfo<Runtime>;
}

impl basic_channel_outbound::Config for Runtime {
	const INDEXING_PREFIX: &'static [u8] = INDEXING_PREFIX;
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type SetPrincipalOrigin = EnsureRootOrHalfLocalCouncil;
	type WeightInfo = weights::basic_channel_outbound_weights::WeightInfo<Runtime>;
}

parameter_types! {
	pub SourceAccount: AccountId = DotPalletId::get().into_account();
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account();
}

pub struct FeeConverter;

impl Convert<U256, Balance> for FeeConverter {
	fn convert(amount: U256) -> Balance {
		dot_app::primitives::unwrap::<Runtime>(amount, Decimals::get())
			.expect("Should not panic unless runtime is misconfigured")
	}
}

impl incentivized_channel_inbound::Config for Runtime {
	type Event = Event;
	type Verifier = verifier_lightclient::Module<Runtime>;
	type MessageDispatch = dispatch::Module<Runtime>;
	type Currency = Balances;
	type SourceAccount = SourceAccount;
	type TreasuryAccount = TreasuryAccount;
	type FeeConverter = FeeConverter;
	type UpdateOrigin = EnsureRootOrHalfLocalCouncil;
	type WeightInfo = weights::incentivized_channel_inbound_weights::WeightInfo<Runtime>;
}

impl incentivized_channel_outbound::Config for Runtime {
	const INDEXING_PREFIX: &'static [u8] = INDEXING_PREFIX;
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type FeeCurrency = SingleAssetAdaptor<Runtime, Ether>;
	type SetFeeOrigin = EnsureRootOrHalfLocalCouncil;
	type WeightInfo = weights::incentivized_channel_outbound_weights::WeightInfo<Runtime>;
}

pub const ROPSTEN_DIFFICULTY_CONFIG: EthereumDifficultyConfig = EthereumDifficultyConfig {
	byzantium_fork_block: 1700000,
	constantinople_fork_block: 4230000,
	muir_glacier_fork_block: 7117117,
};

parameter_types! {
	pub const DescendantsUntilFinalized: u8 = 3;
	pub const DifficultyConfig: EthereumDifficultyConfig = ROPSTEN_DIFFICULTY_CONFIG;
	pub const VerifyPoW: bool = true;
}

impl verifier_lightclient::Config for Runtime {
	type Event = Event;
	type DescendantsUntilFinalized = DescendantsUntilFinalized;
	type DifficultyConfig = DifficultyConfig;
	type VerifyPoW = VerifyPoW;
	type WeightInfo = weights::verifier_lightclient_weights::WeightInfo<Runtime>;
}

impl assets::Config for Runtime {
	type Event = Event;
	type WeightInfo = weights::assets_weights::WeightInfo<Runtime>;
}

parameter_types! {
	pub const EthAssetId: AssetId = AssetId::ETH;
}

impl eth_app::Config for Runtime {
	type Event = Event;
	type Asset = assets::SingleAssetAdaptor<Runtime, EthAssetId>;
	type OutboundRouter = OutboundRouter<Runtime>;
	type CallOrigin = EnsureEthereumAccount;
	type WeightInfo = weights::eth_app_weights::WeightInfo<Runtime>;
}

impl erc20_app::Config for Runtime {
	type Event = Event;
	type Assets = assets::Module<Runtime>;
	type OutboundRouter = OutboundRouter<Runtime>;
	type CallOrigin = EnsureEthereumAccount;
	type WeightInfo = weights::erc20_app_weights::WeightInfo<Runtime>;
}

parameter_types! {
	pub const Decimals: u32 = 10;
}

impl dot_app::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type OutboundRouter = OutboundRouter<Runtime>;
	type CallOrigin = EnsureEthereumAccount;
	type PalletId = DotPalletId;
	type Decimals = Decimals;
	type WeightInfo = weights::dot_app_weights::WeightInfo<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 3,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage} = 4,
		Utility: pallet_utility::{Pallet, Call, Event, Storage} = 5,

		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 6,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 7,

		LocalCouncil: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 8,
		LocalCouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 9,

		// Bridge Infrastructure
		BasicInboundChannel: basic_channel_inbound::{Pallet, Call, Config, Storage, Event} = 10,
		BasicOutboundChannel: basic_channel_outbound::{Pallet, Config<T>, Storage, Event} = 11,
		IncentivizedInboundChannel: incentivized_channel_inbound::{Pallet, Call, Config, Storage, Event} = 12,
		IncentivizedOutboundChannel: incentivized_channel_outbound::{Pallet, Config<T>, Storage, Event} = 13,
		Dispatch: dispatch::{Pallet, Call, Storage, Event<T>, Origin} = 14,
		VerifierLightclient: verifier_lightclient::{Pallet, Call, Storage, Event, Config} = 15,
		Assets: assets::{Pallet, Call, Config<T>, Storage, Event<T>} = 16,

		// XCM
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 17,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 18,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 19,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 20,

		Aura: pallet_aura::{Pallet, Config<T>} = 21,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config} = 22,

		// For dev only, will be removed in production
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 23,

		// Bridge applications
		// NOTE: Do not change the following pallet indices without updating
		//   the peer apps (smart contracts) on the Ethereum side.
		DOT: dot_app::{Pallet, Call, Config<T>, Storage, Event<T>} = 64,
		ETH: eth_app::{Pallet, Call, Config, Storage, Event<T>} = 65,
		ERC20: erc20_app::{Pallet, Call, Config, Storage, Event<T>} = 66,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_collective, LocalCouncil);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_utility, Utility);
			add_benchmark!(params, batches, verifier_lightclient, VerifierLightclient);
			add_benchmark!(params, batches, assets, Assets);
			add_benchmark!(params, batches, basic_channel::inbound, BasicInboundChannel);
			add_benchmark!(params, batches, basic_channel::outbound, BasicOutboundChannel);
			add_benchmark!(params, batches, incentivized_channel::inbound, IncentivizedInboundChannel);
			add_benchmark!(params, batches, incentivized_channel::outbound, IncentivizedOutboundChannel);
			add_benchmark!(params, batches, dot_app, DOT);
			add_benchmark!(params, batches, erc20_app, ERC20);
			add_benchmark!(params, batches, eth_app, ETH);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block!(
	Runtime,
	cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
);
