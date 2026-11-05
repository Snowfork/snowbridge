// Copyright 2026 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

// Package xcmv5 provides a focused SCALE decoder for XCM VersionedXcm V5 payloads.
// It extracts the dApp service fee (Definite DepositAsset of bridged ether to a
// configured AccountId32 recipient) without requiring the gas-estimator binary.
package xcmv5

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"math/big"
	"strings"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
)

const (
	versionedXcmV5 = 5

	instrDepositAsset = 13

	junctionAccountId32    = 1
	junctionGlobalConsensus = 9

	networkEthereum = 7 // NetworkId::Ethereum has #[codec(index = 7)]

	assetFilterDefinite = 0
	fungible            = 0
)

// ParseRecipients normalizes 0x-prefixed or bare 32-byte hex account ids.
func ParseRecipients(hexIDs []string) ([][32]byte, error) {
	out := make([][32]byte, 0, len(hexIDs))
	for _, s := range hexIDs {
		s = strings.TrimPrefix(strings.TrimSpace(strings.ToLower(s)), "0x")
		if s == "" {
			continue
		}
		b, err := hex.DecodeString(s)
		if err != nil {
			return nil, fmt.Errorf("service fee recipient %q: %w", s, err)
		}
		if len(b) != 32 {
			return nil, fmt.Errorf("service fee recipient %q: want 32 bytes, got %d", s, len(b))
		}
		var id [32]byte
		copy(id[:], b)
		out = append(out, id)
	}
	return out, nil
}

// ExtractServiceFee sums bridged-ether Definite DepositAsset amounts to any of
// recipients in a VersionedXcm V5 blob. Only top-level instructions are walked
// (matching gas-estimator extract_service_fee). Returns 0 on empty input,
// non-V5, decode errors after partial progress, or no matching deposits.
func ExtractServiceFee(xcmData []byte, recipients [][32]byte) *big.Int {
	total := new(big.Int)
	if len(xcmData) == 0 || len(recipients) == 0 {
		return total
	}
	recipientSet := make(map[[32]byte]struct{}, len(recipients))
	for _, r := range recipients {
		recipientSet[r] = struct{}{}
	}

	dec := scale.NewDecoder(bytes.NewReader(xcmData))
	version, err := dec.ReadOneByte()
	if err != nil || version != versionedXcmV5 {
		return total
	}
	n, err := decodeCompactUint(dec)
	if err != nil {
		return total
	}
	for i := uint64(0); i < n; i++ {
		idx, err := dec.ReadOneByte()
		if err != nil {
			return total
		}
		if idx == instrDepositAsset {
			amount, ok, err := decodeDepositAssetServiceFee(dec, recipientSet)
			if err != nil {
				return total
			}
			if ok {
				total.Add(total, amount)
			}
			continue
		}
		if err := skipInstructionPayload(dec, idx, 0); err != nil {
			// Soft-fail: keep fees already found (SDK places service fee early).
			return total
		}
	}
	return total
}

func decodeDepositAssetServiceFee(dec *scale.Decoder, recipients map[[32]byte]struct{}) (*big.Int, bool, error) {
	filterKind, err := dec.ReadOneByte()
	if err != nil {
		return nil, false, err
	}
	if filterKind != assetFilterDefinite {
		// AssetFilter::Wild — already consumed discriminant 1.
		if filterKind != 1 {
			return nil, false, fmt.Errorf("unknown AssetFilter %d", filterKind)
		}
		if err := skipWildAsset(dec); err != nil {
			return nil, false, err
		}
		if err := skipLocation(dec); err != nil {
			return nil, false, err
		}
		return nil, false, nil
	}

	assets, err := decodeAssets(dec)
	if err != nil {
		return nil, false, err
	}
	account, err := decodeLocationAccountId32(dec)
	if err != nil {
		return nil, false, err
	}
	if account == nil {
		return nil, false, nil
	}
	if _, ok := recipients[*account]; !ok {
		return nil, false, nil
	}

	sum := new(big.Int)
	for _, a := range assets {
		if !a.isBridgedEther || a.amount == nil {
			continue
		}
		sum.Add(sum, a.amount)
	}
	if sum.Sign() == 0 {
		return nil, false, nil
	}
	return sum, true, nil
}

type decodedAsset struct {
	isBridgedEther bool
	amount         *big.Int
}

func decodeAssets(dec *scale.Decoder) ([]decodedAsset, error) {
	n, err := decodeCompactUint(dec)
	if err != nil {
		return nil, err
	}
	out := make([]decodedAsset, 0, n)
	for i := uint64(0); i < n; i++ {
		a, err := decodeAsset(dec)
		if err != nil {
			return nil, err
		}
		out = append(out, a)
	}
	return out, nil
}

func decodeAsset(dec *scale.Decoder) (decodedAsset, error) {
	isEther, err := peekBridgedEtherLocation(dec)
	if err != nil {
		return decodedAsset{}, err
	}
	funKind, err := dec.ReadOneByte()
	if err != nil {
		return decodedAsset{}, err
	}
	var amount *big.Int
	switch funKind {
	case fungible:
		amount, err = decodeCompactUintBig(dec)
		if err != nil {
			return decodedAsset{}, err
		}
	case 1: // NonFungible
		if err := skipAssetInstance(dec); err != nil {
			return decodedAsset{}, err
		}
	default:
		return decodedAsset{}, fmt.Errorf("unknown fungibility %d", funKind)
	}
	return decodedAsset{isBridgedEther: isEther, amount: amount}, nil
}

// peekBridgedEtherLocation decodes a Location and reports whether it is
// parents=2 / X1(GlobalConsensus(Ethereum{any chain_id})).
func peekBridgedEtherLocation(dec *scale.Decoder) (bool, error) {
	parents, err := dec.ReadOneByte()
	if err != nil {
		return false, err
	}
	junctionsKind, err := dec.ReadOneByte()
	if err != nil {
		return false, err
	}
	isEther := false
	switch junctionsKind {
	case 0: // Here
	case 1: // X1
		j, err := decodeJunction(dec)
		if err != nil {
			return false, err
		}
		isEther = parents == 2 && j.isEthereumConsensus
	default:
		count := int(junctionsKind)
		if count < 1 || count > 8 {
			return false, fmt.Errorf("invalid junctions kind %d", junctionsKind)
		}
		for i := 0; i < count; i++ {
			if _, err := decodeJunction(dec); err != nil {
				return false, err
			}
		}
	}
	return isEther, nil
}

type junctionInfo struct {
	accountId32         *[32]byte
	isEthereumConsensus bool
}

func decodeJunction(dec *scale.Decoder) (junctionInfo, error) {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return junctionInfo{}, err
	}
	switch kind {
	case 0: // Parachain(compact u32)
		_, err := decodeCompactUint(dec)
		return junctionInfo{}, err
	case junctionAccountId32:
		hasNet, err := dec.ReadOneByte()
		if err != nil {
			return junctionInfo{}, err
		}
		if hasNet == 1 {
			if err := skipNetworkId(dec); err != nil {
				return junctionInfo{}, err
			}
		} else if hasNet != 0 {
			return junctionInfo{}, fmt.Errorf("bad Option network %d", hasNet)
		}
		var id [32]byte
		if err := dec.Read(id[:]); err != nil {
			return junctionInfo{}, err
		}
		return junctionInfo{accountId32: &id}, nil
	case 2: // AccountIndex64
		hasNet, err := dec.ReadOneByte()
		if err != nil {
			return junctionInfo{}, err
		}
		if hasNet == 1 {
			if err := skipNetworkId(dec); err != nil {
				return junctionInfo{}, err
			}
		}
		_, err = decodeCompactUint(dec)
		return junctionInfo{}, err
	case 3: // AccountKey20
		hasNet, err := dec.ReadOneByte()
		if err != nil {
			return junctionInfo{}, err
		}
		if hasNet == 1 {
			if err := skipNetworkId(dec); err != nil {
				return junctionInfo{}, err
			}
		}
		var key [20]byte
		return junctionInfo{}, dec.Read(key[:])
	case 4: // PalletInstance(u8)
		_, err := dec.ReadOneByte()
		return junctionInfo{}, err
	case 5: // GeneralIndex(compact u128)
		_, err := decodeCompactUintBig(dec)
		return junctionInfo{}, err
	case 6: // GeneralKey { length u8, data [32]byte }
		if _, err := dec.ReadOneByte(); err != nil {
			return junctionInfo{}, err
		}
		var data [32]byte
		return junctionInfo{}, dec.Read(data[:])
	case 7: // OnlyChild
		return junctionInfo{}, nil
	case 8: // Plurality { BodyId, BodyPart }
		if err := skipBodyId(dec); err != nil {
			return junctionInfo{}, err
		}
		return junctionInfo{}, skipBodyPart(dec)
	case junctionGlobalConsensus:
		net, err := decodeNetworkId(dec)
		if err != nil {
			return junctionInfo{}, err
		}
		return junctionInfo{isEthereumConsensus: net == networkEthereum}, nil
	default:
		return junctionInfo{}, fmt.Errorf("unknown junction %d", kind)
	}
}

func decodeNetworkId(dec *scale.Decoder) (byte, error) {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return 0, err
	}
	switch kind {
	case 0: // ByGenesis
		var h [32]byte
		return kind, dec.Read(h[:])
	case 1: // ByFork
		var n [8]byte
		if err := dec.Read(n[:]); err != nil {
			return 0, err
		}
		var h [32]byte
		return kind, dec.Read(h[:])
	case 2, 3: // Polkadot, Kusama
		return kind, nil
	case networkEthereum:
		_, err := decodeCompactUint(dec)
		return kind, err
	case 8, 9, 10: // BitcoinCore, BitcoinCash, PolkadotBulletin
		return kind, nil
	default:
		return 0, fmt.Errorf("unknown NetworkId %d", kind)
	}
}

func skipNetworkId(dec *scale.Decoder) error {
	_, err := decodeNetworkId(dec)
	return err
}

func decodeLocationAccountId32(dec *scale.Decoder) (account *[32]byte, err error) {
	parents, err := dec.ReadOneByte()
	if err != nil {
		return nil, err
	}
	junctionsKind, err := dec.ReadOneByte()
	if err != nil {
		return nil, err
	}
	switch junctionsKind {
	case 0:
		return nil, nil
	case 1:
		j, err := decodeJunction(dec)
		if err != nil {
			return nil, err
		}
		if parents == 0 && j.accountId32 != nil {
			return j.accountId32, nil
		}
		return nil, nil
	default:
		count := int(junctionsKind)
		if count < 1 || count > 8 {
			return nil, fmt.Errorf("invalid junctions kind %d", junctionsKind)
		}
		for i := 0; i < count; i++ {
			if _, err := decodeJunction(dec); err != nil {
				return nil, err
			}
		}
		return nil, nil
	}
}

func skipLocation(dec *scale.Decoder) error {
	_, err := decodeLocationAccountId32(dec)
	return err
}

func skipWildAsset(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0: // All
		return nil
	case 1: // AllOf { id, fun }
		if _, err := peekBridgedEtherLocation(dec); err != nil {
			return err
		}
		_, err := dec.ReadOneByte() // WildFungibility
		return err
	case 2: // AllCounted(compact u32)
		_, err := decodeCompactUint(dec)
		return err
	case 3: // AllOfCounted
		if _, err := peekBridgedEtherLocation(dec); err != nil {
			return err
		}
		if _, err := dec.ReadOneByte(); err != nil {
			return err
		}
		_, err := decodeCompactUint(dec)
		return err
	default:
		return fmt.Errorf("unknown WildAsset %d", kind)
	}
}

func skipAssetFilter(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case assetFilterDefinite:
		_, err := decodeAssets(dec)
		return err
	case 1:
		return skipWildAsset(dec)
	default:
		return fmt.Errorf("unknown AssetFilter %d", kind)
	}
}

func skipAsset(dec *scale.Decoder) error {
	_, err := decodeAsset(dec)
	return err
}

func skipAssets(dec *scale.Decoder) error {
	_, err := decodeAssets(dec)
	return err
}

func skipAssetInstance(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	case 1:
		_, err := decodeCompactUintBig(dec)
		return err
	case 2:
		var b [4]byte
		return dec.Read(b[:])
	case 3:
		var b [8]byte
		return dec.Read(b[:])
	case 4:
		var b [16]byte
		return dec.Read(b[:])
	case 5:
		var b [32]byte
		return dec.Read(b[:])
	default:
		return fmt.Errorf("unknown AssetInstance %d", kind)
	}
}

func skipAssetTransferFilter(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	if kind > 2 {
		return fmt.Errorf("unknown AssetTransferFilter %d", kind)
	}
	return skipAssetFilter(dec)
}

func skipWeight(dec *scale.Decoder) error {
	if _, err := decodeCompactUint(dec); err != nil {
		return err
	}
	_, err := decodeCompactUint(dec)
	return err
}

func skipWeightLimit(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	case 1:
		return skipWeight(dec)
	default:
		return fmt.Errorf("unknown WeightLimit %d", kind)
	}
}

func skipQueryResponseInfo(dec *scale.Decoder) error {
	if err := skipLocation(dec); err != nil {
		return err
	}
	if _, err := decodeCompactUint(dec); err != nil {
		return err
	}
	return skipWeight(dec)
}

func skipMaybeErrorCode(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	case 1, 2:
		_, err := decodeBytes(dec)
		return err
	default:
		return fmt.Errorf("unknown MaybeErrorCode %d", kind)
	}
}

func skipXCMError(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 9, 14: // FailedToTransactAsset / Transport — #[codec(skip)] str
		return nil
	case 21: // Trap(u64)
		var b [8]byte
		return dec.Read(b[:])
	case 37: // WeightLimitReached(Weight) — after TooManyAssets=35, UnhandledXcmVersion=36
		return skipWeight(dec)
	default:
		// Remaining variants are unit-like (including explicit indices 0–35 and 36,38,39).
		return nil
	}
}

func skipResponse(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	case 1:
		return skipAssets(dec)
	case 2: // Option<(u32, Error)>
		has, err := dec.ReadOneByte()
		if err != nil {
			return err
		}
		if has == 0 {
			return nil
		}
		var idx [4]byte
		if err := dec.Read(idx[:]); err != nil {
			return err
		}
		return skipXCMError(dec)
	case 3: // Version u32
		var v [4]byte
		return dec.Read(v[:])
	case 4: // PalletsInfo
		n, err := decodeCompactUint(dec)
		if err != nil {
			return err
		}
		for i := uint64(0); i < n; i++ {
			if _, err := decodeCompactUint(dec); err != nil {
				return err
			}
			if _, err := decodeBytes(dec); err != nil {
				return err
			}
			if _, err := decodeBytes(dec); err != nil {
				return err
			}
			for j := 0; j < 3; j++ {
				if _, err := decodeCompactUint(dec); err != nil {
					return err
				}
			}
		}
		return nil
	case 5:
		return skipMaybeErrorCode(dec)
	default:
		return fmt.Errorf("unknown Response %d", kind)
	}
}

func skipBodyId(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 1: // Moniker
		var b [4]byte
		return dec.Read(b[:])
	case 2: // Index
		_, err := decodeCompactUint(dec)
		return err
	default:
		return nil
	}
}

func skipBodyPart(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	case 1:
		_, err := decodeCompactUint(dec)
		return err
	case 2, 3, 4:
		if _, err := decodeCompactUint(dec); err != nil {
			return err
		}
		_, err := decodeCompactUint(dec)
		return err
	default:
		return fmt.Errorf("unknown BodyPart %d", kind)
	}
}

func skipInterior(dec *scale.Decoder) error {
	kind, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	switch kind {
	case 0:
		return nil
	default:
		count := int(kind)
		if count < 1 || count > 8 {
			return fmt.Errorf("invalid interior kind %d", kind)
		}
		for i := 0; i < count; i++ {
			if _, err := decodeJunction(dec); err != nil {
				return err
			}
		}
		return nil
	}
}

func skipOptionLocation(dec *scale.Decoder) error {
	has, err := dec.ReadOneByte()
	if err != nil {
		return err
	}
	if has == 0 {
		return nil
	}
	return skipLocation(dec)
}

func skipBytes(dec *scale.Decoder) error {
	_, err := decodeBytes(dec)
	return err
}

func decodeBytes(dec *scale.Decoder) ([]byte, error) {
	n, err := decodeCompactUint(dec)
	if err != nil {
		return nil, err
	}
	b := make([]byte, n)
	if n > 0 {
		if err := dec.Read(b); err != nil {
			return nil, err
		}
	}
	return b, nil
}

// skipInstructionPayload skips the payload of instruction idx. Nested XCMs are
// skipped without extracting fees (top-level only, matching Rust).
func skipInstructionPayload(dec *scale.Decoder, idx byte, depth int) error {
	if depth > 8 {
		return fmt.Errorf("xcm decode depth exceeded")
	}
	switch idx {
	case 0, 1, 2: // Withdraw/Reserve/Teleport Assets
		return skipAssets(dec)
	case 3: // QueryResponse
		if _, err := decodeCompactUint(dec); err != nil {
			return err
		}
		if err := skipResponse(dec); err != nil {
			return err
		}
		if err := skipWeight(dec); err != nil {
			return err
		}
		return skipOptionLocation(dec)
	case 4: // TransferAsset
		if err := skipAssets(dec); err != nil {
			return err
		}
		return skipLocation(dec)
	case 5: // TransferReserveAsset
		if err := skipAssets(dec); err != nil {
			return err
		}
		if err := skipLocation(dec); err != nil {
			return err
		}
		return skipXcmVec(dec, depth+1)
	case 6: // Transact
		if _, err := dec.ReadOneByte(); err != nil { // OriginKind
			return err
		}
		has, err := dec.ReadOneByte() // Option<Weight>
		if err != nil {
			return err
		}
		if has == 1 {
			if err := skipWeight(dec); err != nil {
				return err
			}
		} else if has != 0 {
			return fmt.Errorf("bad Option weight %d", has)
		}
		return skipBytes(dec) // DoubleEncoded = Vec<u8>
	case 7: // HrmpNewChannelOpenRequest
		for i := 0; i < 3; i++ {
			if _, err := decodeCompactUint(dec); err != nil {
				return err
			}
		}
		return nil
	case 8: // HrmpChannelAccepted
		_, err := decodeCompactUint(dec)
		return err
	case 9: // HrmpChannelClosing
		for i := 0; i < 3; i++ {
			if _, err := decodeCompactUint(dec); err != nil {
				return err
			}
		}
		return nil
	case 10: // ClearOrigin
		return nil
	case 11: // DescendOrigin
		return skipInterior(dec)
	case 12: // ReportError
		return skipQueryResponseInfo(dec)
	case instrDepositAsset:
		if err := skipAssetFilter(dec); err != nil {
			return err
		}
		return skipLocation(dec)
	case 14: // DepositReserveAsset
		if err := skipAssetFilter(dec); err != nil {
			return err
		}
		if err := skipLocation(dec); err != nil {
			return err
		}
		return skipXcmVec(dec, depth+1)
	case 15: // ExchangeAsset
		if err := skipAssetFilter(dec); err != nil {
			return err
		}
		if err := skipAssets(dec); err != nil {
			return err
		}
		_, err := dec.ReadOneByte() // bool
		return err
	case 16, 17: // InitiateReserveWithdraw / InitiateTeleport
		if err := skipAssetFilter(dec); err != nil {
			return err
		}
		if err := skipLocation(dec); err != nil {
			return err
		}
		return skipXcmVec(dec, depth+1)
	case 18: // ReportHolding
		if err := skipQueryResponseInfo(dec); err != nil {
			return err
		}
		return skipAssetFilter(dec)
	case 19: // BuyExecution
		if err := skipAsset(dec); err != nil {
			return err
		}
		return skipWeightLimit(dec)
	case 20: // RefundSurplus
		return nil
	case 21, 22: // SetErrorHandler / SetAppendix
		return skipXcmVec(dec, depth+1)
	case 23: // ClearError
		return nil
	case 24: // ClaimAsset
		if err := skipAssets(dec); err != nil {
			return err
		}
		return skipLocation(dec)
	case 25: // Trap(compact u64)
		_, err := decodeCompactUint(dec)
		return err
	case 26: // SubscribeVersion
		if _, err := decodeCompactUint(dec); err != nil {
			return err
		}
		return skipWeight(dec)
	case 27: // UnsubscribeVersion
		return nil
	case 28, 29: // BurnAsset / ExpectAsset
		return skipAssets(dec)
	case 30: // ExpectOrigin
		return skipOptionLocation(dec)
	case 31: // ExpectError Option<(u32, Error)>
		has, err := dec.ReadOneByte()
		if err != nil {
			return err
		}
		if has == 0 {
			return nil
		}
		var idx [4]byte
		if err := dec.Read(idx[:]); err != nil {
			return err
		}
		return skipXCMError(dec)
	case 32: // ExpectTransactStatus
		return skipMaybeErrorCode(dec)
	case 33: // QueryPallet
		if err := skipBytes(dec); err != nil {
			return err
		}
		return skipQueryResponseInfo(dec)
	case 34: // ExpectPallet
		if _, err := decodeCompactUint(dec); err != nil {
			return err
		}
		if err := skipBytes(dec); err != nil {
			return err
		}
		if err := skipBytes(dec); err != nil {
			return err
		}
		if _, err := decodeCompactUint(dec); err != nil {
			return err
		}
		_, err := decodeCompactUint(dec)
		return err
	case 35: // ReportTransactStatus
		return skipQueryResponseInfo(dec)
	case 36: // ClearTransactStatus
		return nil
	case 37: // UniversalOrigin
		_, err := decodeJunction(dec)
		return err
	case 38: // ExportMessage
		if err := skipNetworkId(dec); err != nil {
			return err
		}
		if err := skipInterior(dec); err != nil {
			return err
		}
		return skipXcmVec(dec, depth+1)
	case 39, 40, 41, 42: // Lock/Unlock/NoteUnlockable/RequestUnlock
		if err := skipAsset(dec); err != nil {
			return err
		}
		return skipLocation(dec)
	case 43: // SetFeesMode
		_, err := dec.ReadOneByte()
		return err
	case 44: // SetTopic
		var t [32]byte
		return dec.Read(t[:])
	case 45: // ClearTopic
		return nil
	case 46: // AliasOrigin
		return skipLocation(dec)
	case 47: // UnpaidExecution
		if err := skipWeightLimit(dec); err != nil {
			return err
		}
		return skipOptionLocation(dec)
	case 48: // PayFees
		return skipAsset(dec)
	case 49: // InitiateTransfer
		if err := skipLocation(dec); err != nil {
			return err
		}
		has, err := dec.ReadOneByte()
		if err != nil {
			return err
		}
		if has == 1 {
			if err := skipAssetTransferFilter(dec); err != nil {
				return err
			}
		} else if has != 0 {
			return fmt.Errorf("bad Option remote_fees %d", has)
		}
		if _, err := dec.ReadOneByte(); err != nil { // preserve_origin bool
			return err
		}
		n, err := decodeCompactUint(dec)
		if err != nil {
			return err
		}
		for i := uint64(0); i < n; i++ {
			if err := skipAssetTransferFilter(dec); err != nil {
				return err
			}
		}
		return skipXcmVec(dec, depth+1)
	case 50: // ExecuteWithOrigin
		has, err := dec.ReadOneByte()
		if err != nil {
			return err
		}
		if has == 1 {
			if err := skipInterior(dec); err != nil {
				return err
			}
		} else if has != 0 {
			return fmt.Errorf("bad Option descendant_origin %d", has)
		}
		return skipXcmVec(dec, depth+1)
	case 51: // SetHints
		n, err := decodeCompactUint(dec)
		if err != nil {
			return err
		}
		for i := uint64(0); i < n; i++ {
			hint, err := dec.ReadOneByte()
			if err != nil {
				return err
			}
			if hint != 0 { // only AssetClaimer
				return fmt.Errorf("unknown Hint %d", hint)
			}
			if err := skipLocation(dec); err != nil {
				return err
			}
		}
		return nil
	default:
		return fmt.Errorf("unknown instruction %d", idx)
	}
}

func skipXcmVec(dec *scale.Decoder, depth int) error {
	n, err := decodeCompactUint(dec)
	if err != nil {
		return err
	}
	for i := uint64(0); i < n; i++ {
		idx, err := dec.ReadOneByte()
		if err != nil {
			return err
		}
		if err := skipInstructionPayload(dec, idx, depth); err != nil {
			return err
		}
	}
	return nil
}

func decodeCompactUint(dec *scale.Decoder) (uint64, error) {
	v, err := dec.DecodeUintCompact()
	if err != nil {
		return 0, err
	}
	if !v.IsUint64() {
		return 0, fmt.Errorf("compact value overflows uint64")
	}
	return v.Uint64(), nil
}

func decodeCompactUintBig(dec *scale.Decoder) (*big.Int, error) {
	v, err := dec.DecodeUintCompact()
	if err != nil {
		return nil, err
	}
	return new(big.Int).Set(v), nil
}
