#![allow(clippy::all)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use snowbridge_beacon_primitives::{Fork, ForkVersions};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, ConstU32, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, Keccak256},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};

use sp_std::prelude::*;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchResult,
	match_types, parameter_types,
	traits::{
		tokens::fungible::ItemOf, AsEnsureOriginWithArg, Contains, EitherOfDiverse,
		EqualPrivilegeOnly, Everything, IsInVec, KeyOwnerProofSystem, Nothing, PalletInfoAccess,
		Randomness,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		ConstantMultiplier, IdentityFee, Weight,
	},
	PalletId, StorageValue,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{traits::AccountIdConversion, Perbill, Permill};

pub use snowbridge_core::MessageId;

use polkadot_parachain::primitives::Sibling;

use pallet_xcm::XcmPassthrough;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
	AsPrefixedGeneralIndex, ConvertedConcreteId, CurrencyAdapter, EnsureXcmOrigin,
	FixedWeightBounds, FungiblesAdapter, IsConcrete, NativeAsset, NoChecking, ParentAsSuperuser,
	ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	UsingComponents,
};

use xcm_executor::{traits::JustTry, Config, XcmExecutor};

use runtime_common::{fee::WeightToFee, MaxMessagePayloadSize, MaxMessagesPerCommit};

pub use runtime_primitives::{AccountId, Address, Balance, BlockNumber, Hash, Index, Signature};

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
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
	state_version: 1,
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
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 0.5 seconds of compute with a 12 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
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
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
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
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Self>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Self>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Self>;
	type PalletsOrigin = OriginCaller;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Self>;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

// Cumulus and XCMP

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Rococo;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Junction::Parachain(ParachainInfo::parachain_id().into()).into();
	pub Local: MultiLocation = PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	ConvertedConcreteId<u32, Balance, AsPrefixedGeneralIndex<Local, u32, JustTry>, JustTry>,
	// Convert MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	NoChecking,
	CheckingAccount,
>;

type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = NativeAsset;
	type IsTeleporter = NativeAsset; // <- should be enough to allow teleportation of ROC
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ()>;
	type ResponseHandler = ();
	type AssetTrap = ();
	// Don't handle asset claim.
	type AssetLocker = ();
	type AssetExchanger = ();
	// Don't handle asset trap.
	type AssetClaims = ();
	// Don't handle responses for now.
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
}

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(10);
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = (SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>,);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type PriceForSiblingDelivery = ();
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// Governance

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

type EnsureRootOrHalfLocalCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, LocalCouncilInstance, 1, 2>,
>;

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 10;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRootOrHalfLocalCouncil;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
	type Preimages = Preimage;
}

parameter_types! {
	pub PreimageBaseDeposit: Balance = 1_000_000;
	pub PreimageByteDeposit: Balance = 1_000;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
	pub const LocalCouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const LocalCouncilMaxProposals: u32 = 100;
	pub const LocalCouncilMaxMembers: u32 = 8;
}

type LocalCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Config<LocalCouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = LocalCouncilMotionDuration;
	type MaxProposals = LocalCouncilMaxProposals;
	type MaxMembers = LocalCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Self>;
}

type LocalCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<LocalCouncilMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrHalfLocalCouncil;
	type RemoveOrigin = EnsureRootOrHalfLocalCouncil;
	type SwapOrigin = EnsureRootOrHalfLocalCouncil;
	type ResetOrigin = EnsureRootOrHalfLocalCouncil;
	type PrimeOrigin = EnsureRootOrHalfLocalCouncil;
	type MembershipInitialized = LocalCouncil;
	type MembershipChanged = LocalCouncil;
	type MaxMembers = LocalCouncilMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Self>;
}

// Assets

parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
}

pub type AssetsForceOrigin = EitherOfDiverse<EnsureRoot<AccountId>, EnsureRootOrHalfLocalCouncil>;

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = u32;
	type AssetIdParameter = codec::Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Our pallets

impl dispatch::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type MessageId = MessageId;
	type RuntimeCall = RuntimeCall;
	type CallFilter = Everything;
}

use snowbridge_basic_channel::{
	inbound as basic_channel_inbound, outbound as basic_channel_outbound,
};

impl basic_channel_inbound::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Verifier = ethereum_beacon_client::Pallet<Runtime>;
	type MessageDispatch = dispatch::Pallet<Runtime>;
	type WeightInfo = ();
}

impl basic_channel_outbound::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type SourceId = <Self as frame_system::Config>::AccountId;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type WeightInfo = basic_channel_outbound::weights::SnowbridgeWeight<Self>;
}

parameter_types! {
	pub const MaxSyncCommitteeSize: u32 = 512;
	pub const MaxProofBranchSize: u32 = 20;
	pub const MaxExtraDataSize: u32 = 32;
	pub const MaxLogsBloomSize: u32 = 256;
	pub const MaxFeeRecipientSize: u32 = 20;
	pub const MaxDepositDataSize: u32 = 16;
	pub const MaxPublicKeySize: u32 = 48;
	pub const MaxSignatureSize: u32 = 96;
	pub const MaxProposerSlashingSize: u32 = 16;
	pub const MaxAttesterSlashingSize: u32 = 2;
	pub const MaxVoluntaryExitSize: u32 = 16;
	pub const MaxAttestationSize: u32 = 128;
	pub const MaxValidatorsPerCommittee: u32 = 2048;
	pub const MaxSlotsPerHistoricalRoot: u64 = 8192;
	pub const MaxFinalizedHeaderSlotArray: u32 = 1000;
	pub const WeakSubjectivityPeriodSeconds: u32 = 97200;
	pub const ChainForkVersions: ForkVersions = ForkVersions{
		genesis: Fork {
			version: [0, 0, 0, 0], // 0x00000000
			epoch: 0,
		},
		altair: Fork {
			version: [1, 0, 0, 0], // 0x01000000
			epoch: 74240,
		},
		bellatrix: Fork {
			version: [2, 0, 0, 0], // 0x02000000
			epoch: 144896,
		},
	};
}

impl ethereum_beacon_client::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TimeProvider = pallet_timestamp::Pallet<Runtime>;
	type MaxSyncCommitteeSize = MaxSyncCommitteeSize;
	type MaxProofBranchSize = MaxProofBranchSize;
	type MaxExtraDataSize = MaxExtraDataSize;
	type MaxLogsBloomSize = MaxLogsBloomSize;
	type MaxFeeRecipientSize = MaxFeeRecipientSize;
	type MaxDepositDataSize = MaxDepositDataSize;
	type MaxPublicKeySize = MaxPublicKeySize;
	type MaxSignatureSize = MaxSignatureSize;
	type MaxProposerSlashingSize = MaxProposerSlashingSize;
	type MaxAttesterSlashingSize = MaxAttesterSlashingSize;
	type MaxVoluntaryExitSize = MaxVoluntaryExitSize;
	type MaxAttestationSize = MaxAttestationSize;
	type MaxValidatorsPerCommittee = MaxValidatorsPerCommittee;
	type MaxSlotsPerHistoricalRoot = MaxSlotsPerHistoricalRoot;
	type MaxFinalizedHeaderSlotArray = MaxFinalizedHeaderSlotArray;
	type ForkVersions = ChainForkVersions;
	type WeakSubjectivityPeriodSeconds = WeakSubjectivityPeriodSeconds;
	type WeightInfo = ethereum_beacon_client::weights::SnowbridgeWeight<Self>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = 32;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = pallet_collator_selection::weights::SubstrateWeight<Runtime>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = runtime_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 3,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 4,
		Utility: pallet_utility::{Pallet, Call, Storage, Event} = 5,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 6,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 7,

		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 8,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Config, Event<T>} = 9,

		LocalCouncil: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 10,
		LocalCouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 11,

		// Bridge Infrastructure
		BasicInboundChannel: basic_channel_inbound::{Pallet, Call, Config, Storage, Event<T>} = 12,
		BasicOutboundChannel: basic_channel_outbound::{Pallet, Config<T>, Storage, Event<T>} = 13,
		Dispatch: dispatch::{Pallet, Call, Storage, Event<T>, Origin} = 16,
		EthereumBeaconClient: ethereum_beacon_client::{Pallet, Call, Config<T>, Storage, Event<T>} = 18,
		Assets: pallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>} = 19,

		// XCM
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 22,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 23,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 24,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 25,

		Authorship: pallet_authorship::{Pallet, Storage} = 26,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 27,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 28,
		Aura: pallet_aura::{Pallet, Config<T>} = 29,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config} = 30,

		// For dev only, will be removed in production
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 31,
	}
);

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
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
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
			OpaqueMetadata::new(Runtime::metadata().into())
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
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
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
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_collective, LocalCouncil);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_utility, Utility);
			list_benchmark!(list, extra, pallet_scheduler, Scheduler);
			list_benchmark!(list, extra, assets, Assets);
			list_benchmark!(list, extra, basic_channel_outbound, BasicOutboundChannel);
			list_benchmark!(list, extra, ethereum_beacon_client, EthereumBeaconClient);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

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
			add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			add_benchmark!(params, batches, assets, Assets);
			add_benchmark!(params, batches, basic_channel_outbound, BasicOutboundChannel);
			add_benchmark!(params, batches, ethereum_beacon_client, EthereumBeaconClient);

			if batches.is_empty() {
				return Err("Benchmark not found for this pallet.".into())
			}

			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(&block)
	}
}

cumulus_pallet_parachain_system::register_validate_block!(
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
);
