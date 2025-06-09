package parachain

import (
	"encoding/hex"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

func (li *BeefyListener) getKeyOwnershipProof(meta *types.Metadata, latestHash types.Hash, latestBlockNumber uint64, offenderPubKeyCompressed []byte, validatorSetID uint64) ([]byte, error) {
	var keyOwnershipProofRaw string
	callName := "BeefyApi_generate_key_ownership_proof"
	// TODO: not used in `BeefyApi_generate_key_ownership_proof`, but nonetheless should get session number that validator set was last active for with `beefy_set_id_session`
	sessionDummy, err := types.EncodeToBytes(uint64(0))
	if err != nil {
		return nil, err
	}
	keyOwnershipProofPayload := "0x" + fmt.Sprintf("%x", sessionDummy) + fmt.Sprintf("%x", offenderPubKeyCompressed)
	log.Info("DEBUG: kopPayload: ", keyOwnershipProofPayload)

	// encodedVID, err := types.EncodeToBytes(types.NewOption(commitment.ValidatorSetID))
	encodedVID, err := types.EncodeToBytes(validatorSetID)
	if err != nil {
		return nil, err
	}
	log.Info("DEBUG encoded: ", encodedVID)
	setIdSessionKey, err := types.CreateStorageKey(meta, "Beefy", "SetIdSession", encodedVID)
	if err != nil {
		return nil, err
	}
	log.Info("DEBUG storage key:", setIdSessionKey)
	encodedSessionKey, err := types.EncodeToBytes(setIdSessionKey)
	log.Info("DEBUG storage key:", setIdSessionKey.Hex())
	var offenderSession uint32
	ok, err := li.relaychainConn.API().RPC.State.GetStorage(setIdSessionKey, &offenderSession, latestHash)

	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("DEBUG: No value for SetIdSession key: %x", encodedSessionKey)
	}
	log.Info("DEBUG setIdSession: ", offenderSession)

	currentEpochIndexKey, err := types.CreateStorageKey(meta, "Babe", "EpochIndex", nil)
	if err != nil {
		return nil, err
	}
	var currentSession uint32
	ok, err = li.relaychainConn.API().RPC.State.GetStorage(currentEpochIndexKey, &currentSession, latestHash)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("DEBUG: No value for SetIdSession key: %x", currentEpochIndexKey.Hex())
	}
	log.Info("DEBUG currentSession: ", currentSession)

	// if offenderSession != currentSession {
	// epochDurationKey, err := types.CreateStorageKey(meta, "Babe", "EpochDuration")
	// if err != nil {
	// 	return err
	// }
	// var epochDuration uint64

	// ok, err = li.relaychainConn.API().RPC.State.GetStorage(epochDurationKey, &epochDuration, latestHash)
	// if err != nil {
	// 	return err
	// }
	// if !ok {
	// 	return fmt.Errorf("DEBUG: No value for Epoch key: %x", epochDurationKey.Hex())
	// }
	// log.Info("DEBUG epochDuration: ", epochDuration)
	// TODO: hardcoded atm, and also fragile since slots can be skipped
	epochDuration := uint64(20)
	// TODO: handle if offender claims to be in nextSession
	blockInOffenderSession := latestBlockNumber - epochDuration*uint64(currentSession-offenderSession)

	// a block in offender's session - only used for getting key ownership proof
	offenderSessionBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(blockInOffenderSession)
	if err != nil {
		return nil, err
	}
	log.Info("DEBUG offender session block: ", offenderSessionBlockHash.Hex())
	// }

	err = li.relaychainConn.API().Client.Call(&keyOwnershipProofRaw, "state_call", callName, keyOwnershipProofPayload, offenderSessionBlockHash.Hex())

	if err != nil || !ok {
		return nil, fmt.Errorf("generate key owner proof: %w", err)
	}
	log.Info("return: ", keyOwnershipProofRaw)

	keyOwnershipProof, err := hex.DecodeString(keyOwnershipProofRaw[2:])
	if err != nil || !ok {
		return nil, fmt.Errorf("decode proof: %w", err)
	}
	return keyOwnershipProof, nil
}
