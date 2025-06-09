package parachain

import (
	"encoding/binary"
	"encoding/hex"
	"fmt"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/rpc/author"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"

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

func (li *BeefyListener) getSignerInfo(meta *types.Metadata) (signature.KeyringPair, types.UCompact, error) {

	signer := signature.KeyringPair{
		URI:       "//Bob",
		PublicKey: []byte{0x8e, 0xaf, 0x04, 0x15, 0x16, 0x87, 0x73, 0x63, 0x26, 0xc9, 0xfe, 0xa1, 0x7e, 0x25, 0xfc, 0x52, 0x87, 0x61, 0x36, 0x93, 0xc9, 0x12, 0x90, 0x9c, 0xb2, 0x26, 0xaa, 0x47, 0x94, 0xf2, 0x6a, 0x48}, //nolint:lll
		Address:   "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
	}
	key, err := types.CreateStorageKey(meta, "System", "Account", signer.PublicKey)
	if err != nil {
		return signer, types.NewUCompactFromUInt(0), fmt.Errorf("create storage key: %w", err)
	}

	var accountInfo types.AccountInfo
	ok, err := li.relaychainConn.API().RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil || !ok {
		return signer, types.NewUCompactFromUInt(0), fmt.Errorf("get storage key latest: %w", err)
	}

	nonce := uint64(accountInfo.Nonce)
	log.Info("Nonce: ", nonce)

	return signer, types.NewUCompactFromUInt(nonce), nil
}

func getOffenderPubKeyAndSig(commitment contracts.BeefyClientCommitment, validatorProof contracts.BeefyClientValidatorProof) ([]byte, []byte, error) {
	// encode the commitment, but in the canonical sequence: Payload, BlockNumber, ValidatorSetID
	commitmentPayloadBytes, err := types.EncodeToBytes(commitment.Payload)
	if err != nil {
		return nil, nil, fmt.Errorf("Errored encode commitment: %w", err)
	}
	commitmentBlockNumberBytes, err := types.EncodeToBytes(commitment.BlockNumber)
	if err != nil {
		return nil, nil, fmt.Errorf("Errored encode commitment: %w", err)
	}
	commitmentValidatorSetIdBytes, err := types.EncodeToBytes(commitment.ValidatorSetID)
	if err != nil {
		return nil, nil, fmt.Errorf("Errored encode commitment: %w", err)
	}
	commitmentBytes := append(commitmentPayloadBytes, commitmentBlockNumberBytes...)
	commitmentBytes = append(commitmentBytes, commitmentValidatorSetIdBytes...)
	log.Info("DEBUG encoded commitment: ", commitmentBytes)

	commitmentHash := (&keccak.Keccak256{}).Hash(commitmentBytes)
	log.Info("payload1: commitmentHash: ", commitmentHash)
	log.Info("payload1: commitmentHash: ", fmt.Sprintf("%x", commitmentHash))
	var offenderSig []byte
	offenderSig = append(validatorProof.R[:], validatorProof.S[:]...)

	if validatorProof.V == 27 || validatorProof.V == 28 {
		offenderSig = append(offenderSig, validatorProof.V-27)
	} else {
		return nil, nil, fmt.Errorf("Invalid V value")
	}

	offenderPubKey, err := crypto.SigToPub(commitmentHash[:], offenderSig[:])
	if err != nil {
		return nil, nil, fmt.Errorf("Errored recover pubkey: %w", err)
	}
	return crypto.CompressPubkey(offenderPubKey), offenderSig, nil

}

// build vote payload
func buildVotePayload(commitment contracts.BeefyClientCommitment, offenderPubKeyCompressed []byte, offenderSig []byte) []byte {
	payload1 := append([]byte{0x04}, commitment.Payload[0].PayloadID[:]...)
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	// commitment
	payload1 = append(payload1, 0x80)
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	payload1 = append(payload1, commitment.Payload[0].Data...)
	log.Info("payload1: data ", fmt.Sprintf("%x", commitment.Payload[0].Data))
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	// block number
	blockNumberBytes := make([]byte, 4)
	binary.LittleEndian.PutUint32(blockNumberBytes, commitment.BlockNumber)
	log.Info("payload1: block ", commitment.BlockNumber)
	payload1 = append(payload1, blockNumberBytes...)
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	// validator set id
	validatorSetBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(validatorSetBytes, commitment.ValidatorSetID)
	payload1 = append(payload1, validatorSetBytes...)
	log.Info("payload1: vset ", commitment.ValidatorSetID)
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	// id
	log.Info("DEBUG commitment: ", commitment)

	payload1 = append(payload1, offenderPubKeyCompressed...)
	log.Info("payload1: offenderPubKey ", fmt.Sprintf("%x", offenderPubKeyCompressed))
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))
	// signature
	payload1 = append(payload1, offenderSig[:]...)
	log.Info("payload1: signature ", offenderSig)
	log.Info("payload1: signature hex ", fmt.Sprintf("%x", offenderSig))
	log.Info("payload1: ", fmt.Sprintf("%x", payload1))

	return payload1
}

func (li *BeefyListener) getLatestBlockInfo() (types.Hash, *types.SignedBlock, error) {
	latestHash, err := li.relaychainConn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return types.Hash{}, nil, fmt.Errorf("get finalized head: %w", err)
	}

	latestBlock, err := li.relaychainConn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return types.Hash{}, nil, fmt.Errorf("get block: %w", err)
	}

	log.Info("Latest block number: ", latestBlock.Block.Header.Number)

	return latestHash, latestBlock, nil
}

func (li *BeefyListener) signedExtrinsicFromCall(meta *types.Metadata, call types.Call) (types.Extrinsic, error) {
	ext := types.NewExtrinsic(call)
	signer, nonce, err := li.getSignerInfo(meta)
	if err != nil {
		return ext, fmt.Errorf("get signer info: %w", err)
	}

	latestHash, latestBlock, err := li.getLatestBlockInfo()
	if err != nil {
		return ext, fmt.Errorf("get latest block info: %w", err)
	}

	// TODO: check if applicable here
	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	genesisHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return ext, fmt.Errorf("get block hash: %w", err)
	}

	rv, err := li.relaychainConn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return ext, fmt.Errorf("get runtime version: %w", err)
	}

	o := types.SignatureOptions{
		BlockHash:          latestHash,
		Era:                era,
		GenesisHash:        genesisHash,
		Nonce:              nonce,
		SpecVersion:        rv.SpecVersion,
		Tip:                types.NewUCompactFromUInt(0),
		TransactionVersion: rv.TransactionVersion,
	}

	callHex, err := types.EncodeToHexString(call)
	log.Info("Extrinsic unsigned hex: ", callHex)
	log.Info("Extrinsic unsigned: ", ext)
	extHex, err := types.EncodeToHexString(ext)
	log.Info("Extrinsic unsigned hex: ", extHex)
	err = ext.Sign(signer, o)
	if err != nil {
		return ext, fmt.Errorf("sign extrinsic: %w", err)
	}
	log.Info("Extrinsic: ", ext)
	extHex, err = types.EncodeToHexString(ext)
	log.Info("Extrinsic signed hex: ", extHex)

	return ext, nil

	// ext.Sign(signature.TestKeyringPairAlice, o)
}

func (li *BeefyListener) watchExtrinsicSubscription(sub *author.ExtrinsicStatusSubscription) error {
	for {
		status := <-sub.Chan()
		fmt.Printf("Transaction status: %#v\n", status)

		if status.IsDropped || status.IsInvalid || status.IsUsurped || status.IsFinalityTimeout {
			sub.Unsubscribe()
			log.WithFields(log.Fields{
				// "nonce":  ext.Signature.Nonce,
				"status": status,
			}).Error("Extrinsic removed from the transaction pool")
			return fmt.Errorf("extrinsic removed from the transaction pool")
		}

		if status.IsInBlock {
			log.Info("Completed at block hash ", status.AsInBlock.Hex())
		}
		if status.IsFinalized {
			log.Info("Finalized at block hash ", status.AsFinalized.Hex())
			sub.Unsubscribe()
			break
		}
	}
	return nil
}
