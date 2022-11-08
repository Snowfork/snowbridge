// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/submittable';

import type { ApiTypes, AugmentedSubmittable, SubmittableExtrinsic, SubmittableExtrinsicFunction } from '@polkadot/api-base/types';
import type { Bytes, Compact, Option, U256, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AnyNumber, IMethod, ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H160, H256, MultiAddress, Perbill, Weight } from '@polkadot/types/interfaces/runtime';

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> = SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> = SubmittableExtrinsicFunction<ApiType>;

declare module '@polkadot/api-base/types/submittable' {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    assets: {
      /**
       * Approve an amount of asset for transfer by a delegated third-party account.
       * 
       * Origin must be Signed.
       * 
       * Ensures that `ApprovalDeposit` worth of `Currency` is reserved from signing account
       * for the purpose of holding the approval. If some non-zero amount of assets is already
       * approved from signing account to `delegate`, then it is topped up or unreserved to
       * meet the right value.
       * 
       * NOTE: The signing account does not need to own `amount` of assets at the point of
       * making this call.
       * 
       * - `id`: The identifier of the asset.
       * - `delegate`: The account to delegate permission to transfer asset.
       * - `amount`: The amount of asset that may be transferred by `delegate`. If there is
       * already an approval in place, then this acts additively.
       * 
       * Emits `ApprovedTransfer` on success.
       * 
       * Weight: `O(1)`
       **/
      approveTransfer: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, delegate: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, Compact<u128>]>;
      /**
       * Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
       * 
       * Origin must be Signed and the sender should be the Manager of the asset `id`.
       * 
       * Bails with `NoAccount` if the `who` is already dead.
       * 
       * - `id`: The identifier of the asset to have some amount burned.
       * - `who`: The account to be debited from.
       * - `amount`: The maximum amount by which `who`'s balance should be reduced.
       * 
       * Emits `Burned` with the actual amount burned. If this takes the balance to below the
       * minimum for the asset, then the amount burned is increased to take it to zero.
       * 
       * Weight: `O(1)`
       * Modes: Post-existence of `who`; Pre & post Zombie-status of `who`.
       **/
      burn: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, Compact<u128>]>;
      /**
       * Cancel all of some asset approved for delegated transfer by a third-party account.
       * 
       * Origin must be Signed and there must be an approval in place between signer and
       * `delegate`.
       * 
       * Unreserves any deposit previously reserved by `approve_transfer` for the approval.
       * 
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       * 
       * Emits `ApprovalCancelled` on success.
       * 
       * Weight: `O(1)`
       **/
      cancelApproval: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, delegate: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress]>;
      /**
       * Clear the metadata for an asset.
       * 
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       * 
       * Any deposit is freed for the asset owner.
       * 
       * - `id`: The identifier of the asset to clear.
       * 
       * Emits `MetadataCleared`.
       * 
       * Weight: `O(1)`
       **/
      clearMetadata: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Issue a new class of fungible assets from a public origin.
       * 
       * This new asset class has no assets initially and its owner is the origin.
       * 
       * The origin must be Signed and the sender must have sufficient funds free.
       * 
       * Funds of sender are reserved by `AssetDeposit`.
       * 
       * Parameters:
       * - `id`: The identifier of the new asset. This must not be currently in use to identify
       * an existing asset.
       * - `admin`: The admin of this class of assets. The admin is the initial address of each
       * member of the asset class's admin team.
       * - `min_balance`: The minimum balance of this new asset that any single account must
       * have. If an account's balance is reduced below this, then it collapses to zero.
       * 
       * Emits `Created` event when successful.
       * 
       * Weight: `O(1)`
       **/
      create: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, admin: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, minBalance: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, u128]>;
      /**
       * Destroy a class of fungible assets.
       * 
       * The origin must conform to `ForceOrigin` or must be Signed and the sender must be the
       * owner of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be destroyed. This must identify an existing
       * asset.
       * 
       * Emits `Destroyed` event when successful.
       * 
       * NOTE: It can be helpful to first freeze an asset before destroying it so that you
       * can provide accurate witness information and prevent users from manipulating state
       * in a way that can make it harder to destroy.
       * 
       * Weight: `O(c + p + a)` where:
       * - `c = (witness.accounts - witness.sufficients)`
       * - `s = witness.sufficients`
       * - `a = witness.approvals`
       **/
      destroy: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, witness: PalletAssetsDestroyWitness | { accounts?: any; sufficients?: any; approvals?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, PalletAssetsDestroyWitness]>;
      /**
       * Alter the attributes of a given asset.
       * 
       * Origin must be `ForceOrigin`.
       * 
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       * - `min_balance`: The minimum balance of this new asset that any single account must
       * have. If an account's balance is reduced below this, then it collapses to zero.
       * - `is_sufficient`: Whether a non-zero balance of this asset is deposit of sufficient
       * value to account for the state bloat associated with its balance storage. If set to
       * `true`, then non-zero balances may be stored without a `consumer` reference (and thus
       * an ED in the Balances pallet or whatever else is used to control user-account state
       * growth).
       * - `is_frozen`: Whether this asset class is frozen except for permissioned/admin
       * instructions.
       * 
       * Emits `AssetStatusChanged` with the identity of the asset.
       * 
       * Weight: `O(1)`
       **/
      forceAssetStatus: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, owner: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, issuer: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, admin: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, freezer: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, minBalance: Compact<u128> | AnyNumber | Uint8Array, isSufficient: bool | boolean | Uint8Array, isFrozen: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, MultiAddress, MultiAddress, MultiAddress, Compact<u128>, bool, bool]>;
      /**
       * Cancel all of some asset approved for delegated transfer by a third-party account.
       * 
       * Origin must be either ForceOrigin or Signed origin with the signer being the Admin
       * account of the asset `id`.
       * 
       * Unreserves any deposit previously reserved by `approve_transfer` for the approval.
       * 
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       * 
       * Emits `ApprovalCancelled` on success.
       * 
       * Weight: `O(1)`
       **/
      forceCancelApproval: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, owner: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, delegate: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, MultiAddress]>;
      /**
       * Clear the metadata for an asset.
       * 
       * Origin must be ForceOrigin.
       * 
       * Any deposit is returned.
       * 
       * - `id`: The identifier of the asset to clear.
       * 
       * Emits `MetadataCleared`.
       * 
       * Weight: `O(1)`
       **/
      forceClearMetadata: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Issue a new class of fungible assets from a privileged origin.
       * 
       * This new asset class has no assets initially.
       * 
       * The origin must conform to `ForceOrigin`.
       * 
       * Unlike `create`, no funds are reserved.
       * 
       * - `id`: The identifier of the new asset. This must not be currently in use to identify
       * an existing asset.
       * - `owner`: The owner of this class of assets. The owner has full superuser permissions
       * over this asset, but may later change and configure the permissions using
       * `transfer_ownership` and `set_team`.
       * - `min_balance`: The minimum balance of this new asset that any single account must
       * have. If an account's balance is reduced below this, then it collapses to zero.
       * 
       * Emits `ForceCreated` event when successful.
       * 
       * Weight: `O(1)`
       **/
      forceCreate: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, owner: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, isSufficient: bool | boolean | Uint8Array, minBalance: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, bool, Compact<u128>]>;
      /**
       * Force the metadata for an asset to some value.
       * 
       * Origin must be ForceOrigin.
       * 
       * Any deposit is left alone.
       * 
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       * 
       * Emits `MetadataSet`.
       * 
       * Weight: `O(N + S)` where N and S are the length of the name and symbol respectively.
       **/
      forceSetMetadata: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, name: Bytes | string | Uint8Array, symbol: Bytes | string | Uint8Array, decimals: u8 | AnyNumber | Uint8Array, isFrozen: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Bytes, Bytes, u8, bool]>;
      /**
       * Move some assets from one account to another.
       * 
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       * 
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `source`: The account to be debited.
       * - `dest`: The account to be credited.
       * - `amount`: The amount by which the `source`'s balance of assets should be reduced and
       * `dest`'s balance increased. The amount actually transferred may be slightly greater in
       * the case that the transfer would otherwise take the `source` balance above zero but
       * below the minimum balance. Must be greater than zero.
       * 
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance
       * to below the minimum for the asset, then the amount transferred is increased to take it
       * to zero.
       * 
       * Weight: `O(1)`
       * Modes: Pre-existence of `dest`; Post-existence of `source`; Account pre-existence of
       * `dest`.
       **/
      forceTransfer: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, source: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * Disallow further unprivileged transfers from an account.
       * 
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be frozen.
       * 
       * Emits `Frozen`.
       * 
       * Weight: `O(1)`
       **/
      freeze: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress]>;
      /**
       * Disallow further unprivileged transfers for the asset class.
       * 
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be frozen.
       * 
       * Emits `Frozen`.
       * 
       * Weight: `O(1)`
       **/
      freezeAsset: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Mint assets of a particular class.
       * 
       * The origin must be Signed and the sender must be the Issuer of the asset `id`.
       * 
       * - `id`: The identifier of the asset to have some amount minted.
       * - `beneficiary`: The account to be credited with the minted assets.
       * - `amount`: The amount of the asset to be minted.
       * 
       * Emits `Issued` event when successful.
       * 
       * Weight: `O(1)`
       * Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`.
       **/
      mint: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, beneficiary: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, Compact<u128>]>;
      /**
       * Return the deposit (if any) of an asset account.
       * 
       * The origin must be Signed.
       * 
       * - `id`: The identifier of the asset for the account to be created.
       * - `allow_burn`: If `true` then assets may be destroyed in order to complete the refund.
       * 
       * Emits `Refunded` event when successful.
       **/
      refund: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, allowBurn: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, bool]>;
      /**
       * Set the metadata for an asset.
       * 
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       * 
       * Funds of sender are reserved according to the formula:
       * `MetadataDepositBase + MetadataDepositPerByte * (name.len + symbol.len)` taking into
       * account any already reserved funds.
       * 
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       * 
       * Emits `MetadataSet`.
       * 
       * Weight: `O(1)`
       **/
      setMetadata: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, name: Bytes | string | Uint8Array, symbol: Bytes | string | Uint8Array, decimals: u8 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Bytes, Bytes, u8]>;
      /**
       * Change the Issuer, Admin and Freezer of an asset.
       * 
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be frozen.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       * 
       * Emits `TeamChanged`.
       * 
       * Weight: `O(1)`
       **/
      setTeam: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, issuer: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, admin: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, freezer: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, MultiAddress, MultiAddress]>;
      /**
       * Allow unprivileged transfers from an account again.
       * 
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be unfrozen.
       * 
       * Emits `Thawed`.
       * 
       * Weight: `O(1)`
       **/
      thaw: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress]>;
      /**
       * Allow unprivileged transfers for the asset again.
       * 
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       * 
       * - `id`: The identifier of the asset to be thawed.
       * 
       * Emits `Thawed`.
       * 
       * Weight: `O(1)`
       **/
      thawAsset: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Create an asset account for non-provider assets.
       * 
       * A deposit will be taken from the signer account.
       * 
       * - `origin`: Must be Signed; the signer account must have sufficient funds for a deposit
       * to be taken.
       * - `id`: The identifier of the asset for the account to be created.
       * 
       * Emits `Touched` event when successful.
       **/
      touch: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Move some assets from the sender account to another.
       * 
       * Origin must be Signed.
       * 
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be reduced and
       * `target`'s balance increased. The amount actually transferred may be slightly greater in
       * the case that the transfer would otherwise take the sender balance above zero but below
       * the minimum balance. Must be greater than zero.
       * 
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance
       * to below the minimum for the asset, then the amount transferred is increased to take it
       * to zero.
       * 
       * Weight: `O(1)`
       * Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of
       * `target`.
       **/
      transfer: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, target: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, Compact<u128>]>;
      /**
       * Transfer some asset balance from a previously delegated account to some third-party
       * account.
       * 
       * Origin must be Signed and there must be an approval in place by the `owner` to the
       * signer.
       * 
       * If the entire amount approved for transfer is transferred, then any deposit previously
       * reserved by `approve_transfer` is unreserved.
       * 
       * - `id`: The identifier of the asset.
       * - `owner`: The account which previously approved for a transfer of at least `amount` and
       * from which the asset balance will be withdrawn.
       * - `destination`: The account to which the asset balance of `amount` will be transferred.
       * - `amount`: The amount of assets to transfer.
       * 
       * Emits `TransferredApproved` on success.
       * 
       * Weight: `O(1)`
       **/
      transferApproved: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, owner: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, destination: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * Move some assets from the sender account to another, keeping the sender account alive.
       * 
       * Origin must be Signed.
       * 
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be reduced and
       * `target`'s balance increased. The amount actually transferred may be slightly greater in
       * the case that the transfer would otherwise take the sender balance above zero but below
       * the minimum balance. Must be greater than zero.
       * 
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance
       * to below the minimum for the asset, then the amount transferred is increased to take it
       * to zero.
       * 
       * Weight: `O(1)`
       * Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of
       * `target`.
       **/
      transferKeepAlive: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, target: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress, Compact<u128>]>;
      /**
       * Change the Owner of an asset.
       * 
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       * 
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       * 
       * Emits `OwnerChanged`.
       * 
       * Weight: `O(1)`
       **/
      transferOwnership: AugmentedSubmittable<(id: Compact<u128> | AnyNumber | Uint8Array, owner: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, MultiAddress]>;
    };
    authorship: {
      /**
       * Provide a set of uncles.
       **/
      setUncles: AugmentedSubmittable<(newUncles: Vec<SpRuntimeHeader> | (SpRuntimeHeader | { parentHash?: any; number?: any; stateRoot?: any; extrinsicsRoot?: any; digest?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<SpRuntimeHeader>]>;
    };
    balances: {
      /**
       * Exactly as `transfer`, except the origin must be root and the source account may be
       * specified.
       * # <weight>
       * - Same as transfer, but additional read and write because the source account is not
       * assumed to be in the overlay.
       * # </weight>
       **/
      forceTransfer: AugmentedSubmittable<(source: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * Unreserve some balance from a user by force.
       * 
       * Can only be called by ROOT.
       **/
      forceUnreserve: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u128]>;
      /**
       * Set the balances of a given account.
       * 
       * This will alter `FreeBalance` and `ReservedBalance` in storage. it will
       * also alter the total issuance of the system (`TotalIssuance`) appropriately.
       * If the new free or reserved balance is below the existential deposit,
       * it will reset the account nonce (`frame_system::AccountNonce`).
       * 
       * The dispatch origin for this call is `root`.
       **/
      setBalance: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, newFree: Compact<u128> | AnyNumber | Uint8Array, newReserved: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Compact<u128>]>;
      /**
       * Transfer some liquid free balance to another account.
       * 
       * `transfer` will set the `FreeBalance` of the sender and receiver.
       * If the sender's account is below the existential deposit as a result
       * of the transfer, the account will be reaped.
       * 
       * The dispatch origin for this call must be `Signed` by the transactor.
       * 
       * # <weight>
       * - Dependent on arguments but not critical, given proper implementations for input config
       * types. See related functions below.
       * - It contains a limited number of reads and writes internally and no complex
       * computation.
       * 
       * Related functions:
       * 
       * - `ensure_can_withdraw` is always called internally but has a bounded complexity.
       * - Transferring balances to accounts that did not exist before will cause
       * `T::OnNewAccount::on_new_account` to be called.
       * - Removing enough funds from an account will trigger `T::DustRemoval::on_unbalanced`.
       * - `transfer_keep_alive` works the same way as `transfer`, but has an additional check
       * that the transfer will not kill the origin account.
       * ---------------------------------
       * - Origin account is already in memory, so no DB operations for them.
       * # </weight>
       **/
      transfer: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * Transfer the entire transferable balance from the caller account.
       * 
       * NOTE: This function only attempts to transfer _transferable_ balances. This means that
       * any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
       * transferred by this function. To ensure that this function results in a killed account,
       * you might need to prepare the account by removing any reference counters, storage
       * deposits, etc...
       * 
       * The dispatch origin of this call must be Signed.
       * 
       * - `dest`: The recipient of the transfer.
       * - `keep_alive`: A boolean to determine if the `transfer_all` operation should send all
       * of the funds the account has, causing the sender account to be killed (false), or
       * transfer everything except at least the existential deposit, which will guarantee to
       * keep the sender account alive (true). # <weight>
       * - O(1). Just like transfer, but reading the user's transferable balance first.
       * #</weight>
       **/
      transferAll: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, keepAlive: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, bool]>;
      /**
       * Same as the [`transfer`] call, but with a check that the transfer will not kill the
       * origin account.
       * 
       * 99% of the time you want [`transfer`] instead.
       * 
       * [`transfer`]: struct.Pallet.html#method.transfer
       **/
      transferKeepAlive: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
    };
    basicInboundChannel: {
      submit: AugmentedSubmittable<(message: SnowbridgeCoreMessage | { data?: any; proof?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeCoreMessage]>;
    };
    collatorSelection: {
      /**
       * Deregister `origin` as a collator candidate. Note that the collator can only leave on
       * session change. The `CandidacyBond` will be unreserved immediately.
       * 
       * This call will fail if the total number of candidates would drop below `MinCandidates`.
       * 
       * This call is not available to `Invulnerable` collators.
       **/
      leaveIntent: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Register this account as a collator candidate. The account must (a) already have
       * registered session keys and (b) be able to reserve the `CandidacyBond`.
       * 
       * This call is not available to `Invulnerable` collators.
       **/
      registerAsCandidate: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set the candidacy bond amount.
       **/
      setCandidacyBond: AugmentedSubmittable<(bond: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Set the ideal number of collators (not including the invulnerables).
       * If lowering this number, then the number of running collators could be higher than this figure.
       * Aside from that edge case, there should be no other way to have more collators than the desired number.
       **/
      setDesiredCandidates: AugmentedSubmittable<(max: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Set the list of invulnerable (fixed) collators.
       **/
      setInvulnerables: AugmentedSubmittable<(updated: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
    };
    dispatch: {
    };
    dmpQueue: {
      /**
       * Service a single overweight message.
       * 
       * - `origin`: Must pass `ExecuteOverweightOrigin`.
       * - `index`: The index of the overweight message to service.
       * - `weight_limit`: The amount of weight that message execution may take.
       * 
       * Errors:
       * - `Unknown`: Message of `index` is unknown.
       * - `OverLimit`: Message execution may use greater than `weight_limit`.
       * 
       * Events:
       * - `OverweightServiced`: On success.
       **/
      serviceOverweight: AugmentedSubmittable<(index: u64 | AnyNumber | Uint8Array, weightLimit: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Weight]>;
    };
    dotApp: {
      lock: AugmentedSubmittable<(channelId: SnowbridgeCoreChannelId | 'Basic' | 'Incentivized' | number | Uint8Array, recipient: H160 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeCoreChannelId, H160, u128]>;
      unlock: AugmentedSubmittable<(sender: H160 | string | Uint8Array, recipient: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: U256 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [H160, MultiAddress, U256]>;
    };
    erc20App: {
      burn: AugmentedSubmittable<(channelId: SnowbridgeCoreChannelId | 'Basic' | 'Incentivized' | number | Uint8Array, token: H160 | string | Uint8Array, recipient: H160 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeCoreChannelId, H160, H160, u128]>;
      create: AugmentedSubmittable<(token: H160 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H160]>;
      mint: AugmentedSubmittable<(token: H160 | string | Uint8Array, sender: H160 | string | Uint8Array, recipient: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array, destination: Option<SnowbridgeXcmSupportPrimitivesRemoteParachain> | null | Uint8Array | SnowbridgeXcmSupportPrimitivesRemoteParachain | { paraId?: any; fee?: any } | string) => SubmittableExtrinsic<ApiType>, [H160, H160, MultiAddress, u128, Option<SnowbridgeXcmSupportPrimitivesRemoteParachain>]>;
    };
    ethApp: {
      /**
       * Users can burn their holdings to release funds on the Ethereum side
       **/
      burn: AugmentedSubmittable<(channelId: SnowbridgeCoreChannelId | 'Basic' | 'Incentivized' | number | Uint8Array, recipient: H160 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeCoreChannelId, H160, u128]>;
      mint: AugmentedSubmittable<(sender: H160 | string | Uint8Array, recipient: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array, destination: Option<SnowbridgeXcmSupportPrimitivesRemoteParachain> | null | Uint8Array | SnowbridgeXcmSupportPrimitivesRemoteParachain | { paraId?: any; fee?: any } | string) => SubmittableExtrinsic<ApiType>, [H160, MultiAddress, u128, Option<SnowbridgeXcmSupportPrimitivesRemoteParachain>]>;
    };
    ethereumBeaconClient: {
      importExecutionHeader: AugmentedSubmittable<(update: SnowbridgeBeaconPrimitivesBlockUpdate | { block?: any; syncAggregate?: any; forkVersion?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeBeaconPrimitivesBlockUpdate]>;
      importFinalizedHeader: AugmentedSubmittable<(finalizedHeaderUpdate: SnowbridgeBeaconPrimitivesFinalizedHeaderUpdate | { attestedHeader?: any; finalizedHeader?: any; finalityBranch?: any; syncAggregate?: any; forkVersion?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeBeaconPrimitivesFinalizedHeaderUpdate]>;
      syncCommitteePeriodUpdate: AugmentedSubmittable<(syncCommitteePeriodUpdate: SnowbridgeBeaconPrimitivesSyncCommitteePeriodUpdate | { attestedHeader?: any; nextSyncCommittee?: any; nextSyncCommitteeBranch?: any; finalizedHeader?: any; finalityBranch?: any; syncAggregate?: any; forkVersion?: any; syncCommitteePeriod?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeBeaconPrimitivesSyncCommitteePeriodUpdate]>;
    };
    incentivizedInboundChannel: {
      setRewardFraction: AugmentedSubmittable<(fraction: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      submit: AugmentedSubmittable<(message: SnowbridgeCoreMessage | { data?: any; proof?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbridgeCoreMessage]>;
    };
    incentivizedOutboundChannel: {
      setFee: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
    };
    localCouncil: {
      /**
       * Close a vote that is either approved, disapproved or whose voting period has ended.
       * 
       * May be called by any signed account in order to finish voting and close the proposal.
       * 
       * If called before the end of the voting period it will only close the vote if it is
       * has enough votes to be approved or disapproved.
       * 
       * If called after the end of the voting period abstentions are counted as rejections
       * unless there is a prime member set and the prime member cast an approval.
       * 
       * If the close operation completes successfully with disapproval, the transaction fee will
       * be waived. Otherwise execution of the approved operation will be charged to the caller.
       * 
       * + `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed
       * proposal.
       * + `length_bound`: The upper bound for the length of the proposal in storage. Checked via
       * `storage::read` so it is `size_of::<u32>() == 4` larger than the pure length.
       * 
       * # <weight>
       * ## Weight
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       * - DB:
       * - 2 storage reads (`Members`: codec `O(M)`, `Prime`: codec `O(1)`)
       * - 3 mutations (`Voting`: codec `O(M)`, `ProposalOf`: codec `O(B)`, `Proposals`: codec
       * `O(P2)`)
       * - any mutations done while executing `proposal` (`P1`)
       * - up to 3 events
       * # </weight>
       **/
      close: AugmentedSubmittable<(proposalHash: H256 | string | Uint8Array, index: Compact<u32> | AnyNumber | Uint8Array, proposalWeightBound: Compact<Weight> | AnyNumber | Uint8Array, lengthBound: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, Compact<u32>, Compact<Weight>, Compact<u32>]>;
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless of its current
       * state.
       * 
       * Must be called by the Root origin.
       * 
       * Parameters:
       * * `proposal_hash`: The hash of the proposal that should be disapproved.
       * 
       * # <weight>
       * Complexity: O(P) where P is the number of max proposals
       * DB Weight:
       * * Reads: Proposals
       * * Writes: Voting, Proposals, ProposalOf
       * # </weight>
       **/
      disapproveProposal: AugmentedSubmittable<(proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Dispatch a proposal from a member using the `Member` origin.
       * 
       * Origin must be a member of the collective.
       * 
       * # <weight>
       * ## Weight
       * - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of dispatching
       * `proposal`
       * - DB: 1 read (codec `O(M)`) + DB access of `proposal`
       * - 1 event
       * # </weight>
       **/
      execute: AugmentedSubmittable<(proposal: Call | IMethod | string | Uint8Array, lengthBound: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, Compact<u32>]>;
      /**
       * Add a new proposal to either be voted on or executed directly.
       * 
       * Requires the sender to be member.
       * 
       * `threshold` determines whether `proposal` is executed directly (`threshold < 2`)
       * or put up for voting.
       * 
       * # <weight>
       * ## Weight
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
       * - DB:
       * - 1 storage read `is_member` (codec `O(M)`)
       * - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
       * - DB accesses influenced by `threshold`:
       * - EITHER storage accesses done by `proposal` (`threshold < 2`)
       * - OR proposal insertion (`threshold <= 2`)
       * - 1 storage mutation `Proposals` (codec `O(P2)`)
       * - 1 storage mutation `ProposalCount` (codec `O(1)`)
       * - 1 storage write `ProposalOf` (codec `O(B)`)
       * - 1 storage write `Voting` (codec `O(M)`)
       * - 1 event
       * # </weight>
       **/
      propose: AugmentedSubmittable<(threshold: Compact<u32> | AnyNumber | Uint8Array, proposal: Call | IMethod | string | Uint8Array, lengthBound: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>, Call, Compact<u32>]>;
      /**
       * Set the collective's membership.
       * 
       * - `new_members`: The new member list. Be nice to the chain and provide it sorted.
       * - `prime`: The prime member whose vote sets the default.
       * - `old_count`: The upper bound for the previous number of members in storage. Used for
       * weight estimation.
       * 
       * Requires root origin.
       * 
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
       * the weight estimations rely on it to estimate dispatchable weight.
       * 
       * # WARNING:
       * 
       * The `pallet-collective` can also be managed by logic outside of the pallet through the
       * implementation of the trait [`ChangeMembers`].
       * Any call to `set_members` must be careful that the member set doesn't get out of sync
       * with other logic managing the member set.
       * 
       * # <weight>
       * ## Weight
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       * - DB:
       * - 1 storage mutation (codec `O(M)` read, `O(N)` write) for reading and writing the
       * members
       * - 1 storage read (codec `O(P)`) for reading the proposals
       * - `P` storage mutations (codec `O(M)`) for updating the votes for each proposal
       * - 1 storage write (codec `O(1)`) for deleting the old `prime` and setting the new one
       * # </weight>
       **/
      setMembers: AugmentedSubmittable<(newMembers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], prime: Option<AccountId32> | null | Uint8Array | AccountId32 | string, oldCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>, Option<AccountId32>, u32]>;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       * 
       * Requires the sender to be a member.
       * 
       * Transaction fees will be waived if the member is voting on any particular proposal
       * for the first time and the call is successful. Subsequent vote changes will charge a
       * fee.
       * # <weight>
       * ## Weight
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       * - DB:
       * - 1 storage read `Members` (codec `O(M)`)
       * - 1 storage mutation `Voting` (codec `O(M)`)
       * - 1 event
       * # </weight>
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: Compact<u32> | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, Compact<u32>, bool]>;
    };
    localCouncilMembership: {
      /**
       * Add a member `who` to the set.
       * 
       * May only be called from `T::AddOrigin`.
       **/
      addMember: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Swap out the sending member for some other key `new`.
       * 
       * May only be called from `Signed` origin of a current member.
       * 
       * Prime membership is passed from the origin account to `new`, if extant.
       **/
      changeKey: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Remove the prime member if it exists.
       * 
       * May only be called from `T::PrimeOrigin`.
       **/
      clearPrime: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Remove a member `who` from the set.
       * 
       * May only be called from `T::RemoveOrigin`.
       **/
      removeMember: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Change the membership to a new set, disregarding the existing membership. Be nice and
       * pass `members` pre-sorted.
       * 
       * May only be called from `T::ResetOrigin`.
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Set the prime member. Must be a current member.
       * 
       * May only be called from `T::PrimeOrigin`.
       **/
      setPrime: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Swap out one member `remove` for another `add`.
       * 
       * May only be called from `T::SwapOrigin`.
       * 
       * Prime membership is *not* passed from `remove` to `add`, if extant.
       **/
      swapMember: AugmentedSubmittable<(remove: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, add: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, MultiAddress]>;
    };
    parachainSystem: {
      authorizeUpgrade: AugmentedSubmittable<(codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      enactAuthorizedUpgrade: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the current validation data.
       * 
       * This should be invoked exactly once per block. It will panic at the finalization
       * phase if the call was not invoked.
       * 
       * The dispatch origin for this call must be `Inherent`
       * 
       * As a side effect, this function upgrades the current validation function
       * if the appropriate time has come.
       **/
      setValidationData: AugmentedSubmittable<(data: CumulusPrimitivesParachainInherentParachainInherentData | { validationData?: any; relayChainState?: any; downwardMessages?: any; horizontalMessages?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [CumulusPrimitivesParachainInherentParachainInherentData]>;
      sudoSendUpwardMessage: AugmentedSubmittable<(message: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
    };
    polkadotXcm: {
      /**
       * Execute an XCM message from a local, signed, origin.
       * 
       * An event is deposited indicating whether `msg` could be executed completely or only
       * partially.
       * 
       * No more than `max_weight` will be used in its attempted execution. If this is less than the
       * maximum amount of weight that the message could take to be executed, then no execution
       * attempt will be made.
       * 
       * NOTE: A successful return to this does *not* imply that the `msg` was executed successfully
       * to completion; only that *some* of it was executed.
       **/
      execute: AugmentedSubmittable<(message: XcmVersionedXcm | { V0: any } | { V1: any } | { V2: any } | string | Uint8Array, maxWeight: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedXcm, Weight]>;
      /**
       * Set a safe XCM version (the version that XCM should be encoded with if the most recent
       * version a destination can accept is unknown).
       * 
       * - `origin`: Must be Root.
       * - `maybe_xcm_version`: The default XCM encoding version, or `None` to disable.
       **/
      forceDefaultXcmVersion: AugmentedSubmittable<(maybeXcmVersion: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [Option<u32>]>;
      /**
       * Ask a location to notify us regarding their XCM version and any changes to it.
       * 
       * - `origin`: Must be Root.
       * - `location`: The location to which we should subscribe for XCM version notifications.
       **/
      forceSubscribeVersionNotify: AugmentedSubmittable<(location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation]>;
      /**
       * Require that a particular destination should no longer notify us regarding any XCM
       * version changes.
       * 
       * - `origin`: Must be Root.
       * - `location`: The location to which we are currently subscribed for XCM version
       * notifications which we no longer desire.
       **/
      forceUnsubscribeVersionNotify: AugmentedSubmittable<(location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation]>;
      /**
       * Extoll that a particular destination can be communicated with through a particular
       * version of XCM.
       * 
       * - `origin`: Must be Root.
       * - `location`: The destination that is being described.
       * - `xcm_version`: The latest version of XCM that `location` supports.
       **/
      forceXcmVersion: AugmentedSubmittable<(location: XcmV1MultiLocation | { parents?: any; interior?: any } | string | Uint8Array, xcmVersion: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmV1MultiLocation, u32]>;
      /**
       * Transfer some assets from the local chain to the sovereign account of a destination
       * chain and forward a notification XCM.
       * 
       * Fee payment on the destination side is made from the asset in the `assets` vector of
       * index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight
       * is needed than `weight_limit`, then the operation will fail and the assets send may be
       * at risk.
       * 
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
       * from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
       * an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on the
       * `dest` side.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay
       * fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       **/
      limitedReserveTransferAssets: AugmentedSubmittable<(dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array, feeAssetItem: u32 | AnyNumber | Uint8Array, weightLimit: XcmV2WeightLimit | { Unlimited: any } | { Limited: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32, XcmV2WeightLimit]>;
      /**
       * Teleport some assets from the local chain to some destination chain.
       * 
       * Fee payment on the destination side is made from the asset in the `assets` vector of
       * index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight
       * is needed than `weight_limit`, then the operation will fail and the assets send may be
       * at risk.
       * 
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
       * from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
       * an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. The first item should be the currency used to to pay the fee on the
       * `dest` side. May not be empty.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay
       * fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       **/
      limitedTeleportAssets: AugmentedSubmittable<(dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array, feeAssetItem: u32 | AnyNumber | Uint8Array, weightLimit: XcmV2WeightLimit | { Unlimited: any } | { Limited: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32, XcmV2WeightLimit]>;
      /**
       * Transfer some assets from the local chain to the sovereign account of a destination
       * chain and forward a notification XCM.
       * 
       * Fee payment on the destination side is made from the asset in the `assets` vector of
       * index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,
       * with all fees taken as needed from the asset.
       * 
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
       * from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
       * an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on the
       * `dest` side.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay
       * fees.
       **/
      reserveTransferAssets: AugmentedSubmittable<(dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array, feeAssetItem: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]>;
      send: AugmentedSubmittable<(dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, message: XcmVersionedXcm | { V0: any } | { V1: any } | { V2: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation, XcmVersionedXcm]>;
      /**
       * Teleport some assets from the local chain to some destination chain.
       * 
       * Fee payment on the destination side is made from the asset in the `assets` vector of
       * index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,
       * with all fees taken as needed from the asset.
       * 
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
       * from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
       * an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. The first item should be the currency used to to pay the fee on the
       * `dest` side. May not be empty.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay
       * fees.
       **/
      teleportAssets: AugmentedSubmittable<(dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array, assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array, feeAssetItem: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]>;
    };
    preimage: {
      /**
       * Register a preimage on-chain.
       * 
       * If the preimage was previously requested, no fees or deposits are taken for providing
       * the preimage. Otherwise, a deposit is taken proportional to the size of the preimage.
       **/
      notePreimage: AugmentedSubmittable<(bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Request a preimage be uploaded to the chain without paying any fees or deposits.
       * 
       * If the preimage requests has already been provided on-chain, we unreserve any deposit
       * a user may have paid, and take the control of the preimage out of their hands.
       **/
      requestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Clear an unrequested preimage from the runtime storage.
       **/
      unnotePreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Clear a previously made request for a preimage.
       * 
       * NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`.
       **/
      unrequestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
    };
    scheduler: {
      /**
       * Cancel an anonymously scheduled task.
       **/
      cancel: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Cancel a named scheduled task.
       **/
      cancelNamed: AugmentedSubmittable<(id: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Anonymously schedule a task.
       **/
      schedule: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Anonymously schedule a task after a delay.
       * 
       * # <weight>
       * Same as [`schedule`].
       * # </weight>
       **/
      scheduleAfter: AugmentedSubmittable<(after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Schedule a named task.
       **/
      scheduleNamed: AugmentedSubmittable<(id: Bytes | string | Uint8Array, when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Schedule a named task after a delay.
       * 
       * # <weight>
       * Same as [`schedule_named`](Self::schedule_named).
       * # </weight>
       **/
      scheduleNamedAfter: AugmentedSubmittable<(id: Bytes | string | Uint8Array, after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
    };
    session: {
      /**
       * Removes any session key(s) of the function caller.
       * 
       * This doesn't take effect until the next session.
       * 
       * The dispatch origin of this function must be Signed and the account must be either be
       * convertible to a validator ID using the chain's typical addressing system (this usually
       * means being a controller account) or directly convertible into a validator ID (which
       * usually means being a stash account).
       * 
       * # <weight>
       * - Complexity: `O(1)` in number of key types. Actual cost depends on the number of length
       * of `T::Keys::key_ids()` which is fixed.
       * - DbReads: `T::ValidatorIdOf`, `NextKeys`, `origin account`
       * - DbWrites: `NextKeys`, `origin account`
       * - DbWrites per key id: `KeyOwner`
       * # </weight>
       **/
      purgeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Sets the session key(s) of the function caller to `keys`.
       * Allows an account to set its session key prior to becoming a validator.
       * This doesn't take effect until the next session.
       * 
       * The dispatch origin of this function must be signed.
       * 
       * # <weight>
       * - Complexity: `O(1)`. Actual cost depends on the number of length of
       * `T::Keys::key_ids()` which is fixed.
       * - DbReads: `origin account`, `T::ValidatorIdOf`, `NextKeys`
       * - DbWrites: `origin account`, `NextKeys`
       * - DbReads per key id: `KeyOwner`
       * - DbWrites per key id: `KeyOwner`
       * # </weight>
       **/
      setKeys: AugmentedSubmittable<(keys: SnowbaseRuntimeSessionKeys | { aura?: any } | string | Uint8Array, proof: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbaseRuntimeSessionKeys, Bytes]>;
    };
    sudo: {
      /**
       * Authenticates the current sudo key and sets the given AccountId (`new`) as the new sudo
       * key.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB change.
       * # </weight>
       **/
      setKey: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + 10,000.
       * # </weight>
       **/
      sudo: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Signed` origin from
       * a given account.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + 10,000.
       * # </weight>
       **/
      sudoAs: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       * This function does not check the weight of the call, and instead allows the
       * Sudo user to specify the weight of the call.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - The weight of this call is defined by the caller.
       * # </weight>
       **/
      sudoUncheckedWeight: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array, weight: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, Weight]>;
    };
    system: {
      /**
       * A dispatch that will fill the block weight up to the given ratio.
       **/
      fillBlock: AugmentedSubmittable<(ratio: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * Kill all storage items with a key that starts with the given prefix.
       * 
       * **NOTE:** We rely on the Root origin to provide us the number of subkeys under
       * the prefix we are removing to accurately calculate the weight of this function.
       **/
      killPrefix: AugmentedSubmittable<(prefix: Bytes | string | Uint8Array, subkeys: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32]>;
      /**
       * Kill some items from storage.
       **/
      killStorage: AugmentedSubmittable<(keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Bytes>]>;
      /**
       * Make some on-chain remark.
       * 
       * # <weight>
       * - `O(1)`
       * # </weight>
       **/
      remark: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Make some on-chain remark and emit event.
       **/
      remarkWithEvent: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code.
       * 
       * # <weight>
       * - `O(C + S)` where `C` length of `code` and `S` complexity of `can_set_code`
       * - 1 call to `can_set_code`: `O(S)` (calls `sp_io::misc::runtime_version` which is
       * expensive).
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event.
       * The weight of this function is dependent on the runtime, but generally this is very
       * expensive. We will treat this as a full block.
       * # </weight>
       **/
      setCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       * 
       * # <weight>
       * - `O(C)` where `C` length of `code`
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event.
       * The weight of this function is dependent on the runtime. We will treat this as a full
       * block. # </weight>
       **/
      setCodeWithoutChecks: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the number of pages in the WebAssembly environment's heap.
       **/
      setHeapPages: AugmentedSubmittable<(pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Set some items of storage.
       **/
      setStorage: AugmentedSubmittable<(items: Vec<ITuple<[Bytes, Bytes]>> | ([Bytes | string | Uint8Array, Bytes | string | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[Bytes, Bytes]>>]>;
    };
    timestamp: {
      /**
       * Set the current time.
       * 
       * This call should be invoked exactly once per block. It will panic at the finalization
       * phase, if this call hasn't been invoked by that time.
       * 
       * The timestamp should be greater than the previous one by the amount specified by
       * `MinimumPeriod`.
       * 
       * The dispatch origin for this call must be `Inherent`.
       * 
       * # <weight>
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)`). (because of `DidUpdate::take` in
       * `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
       * # </weight>
       **/
      set: AugmentedSubmittable<(now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u64>]>;
    };
    utility: {
      /**
       * Send a call through an indexed pseudonym of the sender.
       * 
       * Filter from origin are passed along. The call will be dispatched with an origin which
       * use the same filter as the origin of this call.
       * 
       * NOTE: If you need to ensure that any account-based filtering is not honored (i.e.
       * because you expect `proxy` to have been used prior in the call stack and you do not want
       * the call restrictions to apply to any sub-accounts), then use `as_multi_threshold_1`
       * in the Multisig pallet instead.
       * 
       * NOTE: Prior to version *12, this was called `as_limited_sub`.
       * 
       * The dispatch origin for this call must be _Signed_.
       **/
      asDerivative: AugmentedSubmittable<(index: u16 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u16, Call]>;
      /**
       * Send a batch of dispatch calls.
       * 
       * May be called from any origin.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then call are dispatch without checking origin filter. (This includes
       * bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * # <weight>
       * - Complexity: O(C) where C is the number of calls to be batched.
       * # </weight>
       * 
       * This will return `Ok` in all circumstances. To determine the success of the batch, an
       * event is deposited. If a call failed and the batch was interrupted, then the
       * `BatchInterrupted` event is deposited, along with the number of successful calls made
       * and the error of the failed call. If all were successful, then the `BatchCompleted`
       * event is deposited.
       **/
      batch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Send a batch of dispatch calls and atomically execute them.
       * The whole transaction will rollback and fail if any of the calls failed.
       * 
       * May be called from any origin.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then call are dispatch without checking origin filter. (This includes
       * bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * # <weight>
       * - Complexity: O(C) where C is the number of calls to be batched.
       * # </weight>
       **/
      batchAll: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Dispatches a function call with a provided origin.
       * 
       * The dispatch origin for this call must be _Root_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + T::WeightInfo::dispatch_as().
       * # </weight>
       **/
      dispatchAs: AugmentedSubmittable<(asOrigin: SnowbaseRuntimeOriginCaller | { system: any } | { Void: any } | { LocalCouncil: any } | { Dispatch: any } | { PolkadotXcm: any } | { CumulusXcm: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SnowbaseRuntimeOriginCaller, Call]>;
      /**
       * Send a batch of dispatch calls.
       * Unlike `batch`, it allows errors and won't interrupt.
       * 
       * May be called from any origin.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then call are dispatch without checking origin filter. (This includes
       * bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * # <weight>
       * - Complexity: O(C) where C is the number of calls to be batched.
       * # </weight>
       **/
      forceBatch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
    };
    xcmpQueue: {
      /**
       * Resumes all XCM executions for the XCMP queue.
       * 
       * Note that this function doesn't change the status of the in/out bound channels.
       * 
       * - `origin`: Must pass `ControllerOrigin`.
       **/
      resumeXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Services a single overweight XCM.
       * 
       * - `origin`: Must pass `ExecuteOverweightOrigin`.
       * - `index`: The index of the overweight XCM to service
       * - `weight_limit`: The amount of weight that XCM execution may take.
       * 
       * Errors:
       * - `BadOverweightIndex`: XCM under `index` is not found in the `Overweight` storage map.
       * - `BadXcm`: XCM under `index` cannot be properly decoded into a valid XCM format.
       * - `WeightOverLimit`: XCM execution may use greater `weight_limit`.
       * 
       * Events:
       * - `OverweightServiced`: On success.
       **/
      serviceOverweight: AugmentedSubmittable<(index: u64 | AnyNumber | Uint8Array, weightLimit: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Weight]>;
      /**
       * Suspends all XCM executions for the XCMP queue, regardless of the sender's origin.
       * 
       * - `origin`: Must pass `ControllerOrigin`.
       **/
      suspendXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Overwrites the number of pages of messages which must be in the queue after which we drop any further
       * messages from the channel.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.drop_threshold`
       **/
      updateDropThreshold: AugmentedSubmittable<(updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Overwrites the number of pages of messages which the queue must be reduced to before it signals that
       * message sending may recommence after it has been suspended.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.resume_threshold`
       **/
      updateResumeThreshold: AugmentedSubmittable<(updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Overwrites the number of pages of messages which must be in the queue for the other side to be told to
       * suspend their sending.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.suspend_value`
       **/
      updateSuspendThreshold: AugmentedSubmittable<(updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Overwrites the amount of remaining weight under which we stop processing messages.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.threshold_weight`
       **/
      updateThresholdWeight: AugmentedSubmittable<(updated: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Weight]>;
      /**
       * Overwrites the speed to which the available weight approaches the maximum weight.
       * A lower number results in a faster progression. A value of 1 makes the entire weight available initially.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.weight_restrict_decay`.
       **/
      updateWeightRestrictDecay: AugmentedSubmittable<(updated: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Weight]>;
      /**
       * Overwrite the maximum amount of weight any individual message may consume.
       * Messages above this weight go into the overweight queue and may only be serviced explicitly.
       * 
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.xcmp_max_individual_weight`.
       **/
      updateXcmpMaxIndividualWeight: AugmentedSubmittable<(updated: Weight | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Weight]>;
    };
  } // AugmentedSubmittables
} // declare module
