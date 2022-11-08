// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/consts';

import type { ApiTypes, AugmentedConst } from '@polkadot/api-base/types';
import type { u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AccountId32, Weight } from '@polkadot/types/interfaces/runtime';

export type __AugmentedConst<ApiType extends ApiTypes> = AugmentedConst<ApiType>;

declare module '@polkadot/api-base/types/consts' {
  interface AugmentedConsts<ApiType extends ApiTypes> {
    assets: {
      /**
       * The amount of funds that must be reserved when creating a new approval.
       **/
      approvalDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of funds that must be reserved for a non-provider asset account to be
       * maintained.
       **/
      assetAccountDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The basic amount of funds that must be reserved for an asset.
       **/
      assetDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The basic amount of funds that must be reserved when adding metadata to your asset.
       **/
      metadataDepositBase: u128 & AugmentedConst<ApiType>;
      /**
       * The additional funds that must be reserved for the number of bytes you store in your
       * metadata.
       **/
      metadataDepositPerByte: u128 & AugmentedConst<ApiType>;
      /**
       * The maximum length of a name or symbol stored on-chain.
       **/
      stringLimit: u32 & AugmentedConst<ApiType>;
    };
    authorship: {
      /**
       * The number of blocks back we should accept uncles.
       * This means that we will deal with uncle-parents that are
       * `UncleGenerations + 1` before `now`.
       **/
      uncleGenerations: u32 & AugmentedConst<ApiType>;
    };
    balances: {
      /**
       * The minimum amount required to keep an account open.
       **/
      existentialDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The maximum number of locks that should exist on an account.
       * Not strictly enforced, but used for weight estimation.
       **/
      maxLocks: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of named reserves that can exist on an account.
       **/
      maxReserves: u32 & AugmentedConst<ApiType>;
    };
    basicOutboundChannel: {
      /**
       * Max bytes in a message payload
       **/
      maxMessagePayloadSize: u32 & AugmentedConst<ApiType>;
      /**
       * Max number of messages per commitment
       **/
      maxMessagesPerCommit: u32 & AugmentedConst<ApiType>;
    };
    dotApp: {
      decimals: u32 & AugmentedConst<ApiType>;
    };
    ethereumBeaconClient: {
      maxAttestationSize: u32 & AugmentedConst<ApiType>;
      maxAttesterSlashingSize: u32 & AugmentedConst<ApiType>;
      maxDepositDataSize: u32 & AugmentedConst<ApiType>;
      maxExtraDataSize: u32 & AugmentedConst<ApiType>;
      maxFeeRecipientSize: u32 & AugmentedConst<ApiType>;
      maxLogsBloomSize: u32 & AugmentedConst<ApiType>;
      maxProofBranchSize: u32 & AugmentedConst<ApiType>;
      maxProposerSlashingSize: u32 & AugmentedConst<ApiType>;
      maxPublicKeySize: u32 & AugmentedConst<ApiType>;
      maxSignatureSize: u32 & AugmentedConst<ApiType>;
      maxSyncCommitteeSize: u32 & AugmentedConst<ApiType>;
      maxValidatorsPerCommittee: u32 & AugmentedConst<ApiType>;
      maxVoluntaryExitSize: u32 & AugmentedConst<ApiType>;
    };
    incentivizedInboundChannel: {
      /**
       * Source of funds to pay relayers
       **/
      sourceAccount: AccountId32 & AugmentedConst<ApiType>;
      /**
       * Treasury Account
       **/
      treasuryAccount: AccountId32 & AugmentedConst<ApiType>;
    };
    incentivizedOutboundChannel: {
      /**
       * Max number of messages per commitment
       **/
      maxMessagePayloadSize: u32 & AugmentedConst<ApiType>;
      /**
       * Max number of messages per commitment
       **/
      maxMessagesPerCommit: u32 & AugmentedConst<ApiType>;
    };
    scheduler: {
      /**
       * The maximum weight that may be scheduled per block for any dispatchables of less
       * priority than `schedule::HARD_DEADLINE`.
       **/
      maximumWeight: Weight & AugmentedConst<ApiType>;
      /**
       * The maximum number of scheduled calls in the queue for a single block.
       * Not strictly enforced, but used for weight estimation.
       **/
      maxScheduledPerBlock: u32 & AugmentedConst<ApiType>;
    };
    system: {
      /**
       * Maximum number of block number to block hash mappings to keep (oldest pruned first).
       **/
      blockHashCount: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum length of a block (in bytes).
       **/
      blockLength: FrameSystemLimitsBlockLength & AugmentedConst<ApiType>;
      /**
       * Block & extrinsics weights: base values and limits.
       **/
      blockWeights: FrameSystemLimitsBlockWeights & AugmentedConst<ApiType>;
      /**
       * The weight of runtime database operations the runtime can invoke.
       **/
      dbWeight: FrameSupportWeightsRuntimeDbWeight & AugmentedConst<ApiType>;
      /**
       * The designated SS58 prefix of this chain.
       * 
       * This replaces the "ss58Format" property declared in the chain spec. Reason is
       * that the runtime should know about the prefix in order to make use of it as
       * an identifier of the chain.
       **/
      ss58Prefix: u16 & AugmentedConst<ApiType>;
      /**
       * Get the chain's current version.
       **/
      version: SpVersionRuntimeVersion & AugmentedConst<ApiType>;
    };
    timestamp: {
      /**
       * The minimum period between blocks. Beware that this is different to the *expected*
       * period that the block production apparatus provides. Your chosen consensus system will
       * generally work with this to determine a sensible block time. e.g. For Aura, it will be
       * double this period on default settings.
       **/
      minimumPeriod: u64 & AugmentedConst<ApiType>;
    };
    transactionPayment: {
      /**
       * A fee mulitplier for `Operational` extrinsics to compute "virtual tip" to boost their
       * `priority`
       * 
       * This value is multipled by the `final_fee` to obtain a "virtual tip" that is later
       * added to a tip component in regular `priority` calculations.
       * It means that a `Normal` transaction can front-run a similarly-sized `Operational`
       * extrinsic (with no tip), by including a tip value greater than the virtual tip.
       * 
       * ```rust,ignore
       * // For `Normal`
       * let priority = priority_calc(tip);
       * 
       * // For `Operational`
       * let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
       * let priority = priority_calc(tip + virtual_tip);
       * ```
       * 
       * Note that since we use `final_fee` the multiplier applies also to the regular `tip`
       * sent with the transaction. So, not only does the transaction get a priority bump based
       * on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`
       * transactions.
       **/
      operationalFeeMultiplier: u8 & AugmentedConst<ApiType>;
    };
    utility: {
      /**
       * The limit on the number of batched calls.
       **/
      batchedCallsLimit: u32 & AugmentedConst<ApiType>;
    };
  } // AugmentedConsts
} // declare module
