package fisherman

import (
	"context"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/client"
	"github.com/snowfork/go-substrate-rpc-client/v4/rpc/author"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	ancestryTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
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

	// encodedVID, err := types.EncodeToBytes(types.NewOption(commitment.ValidatorSetID))
	encodedVID, err := types.EncodeToBytes(validatorSetID)
	if err != nil {
		return nil, err
	}
	setIdSessionKey, err := types.CreateStorageKey(meta, "Beefy", "SetIdSession", encodedVID)
	if err != nil {
		return nil, err
	}
	encodedSessionKey, err := types.EncodeToBytes(setIdSessionKey)
	var offenderSession uint32
	ok, err := li.relaychainConn.API().RPC.State.GetStorage(setIdSessionKey, &offenderSession, latestHash)

	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("DEBUG: No value for SetIdSession key: %x", encodedSessionKey)
	}

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

	// if offenderSession != currentSession {
	// epochDurationKey, err := types.CreateStorageKey(meta, "Babe", "EpochDuration")
	// if err != nil {
	// 	return err
	// }
	// var epochDuration uint64

	epochDurationRaw, err := meta.FindConstantValue("Babe", "EpochDuration")
	if err != nil {
		return nil, fmt.Errorf("couldn't find const: %w", err)
	}
	epochDuration := binary.LittleEndian.Uint64(epochDurationRaw)
	log.Debug("epochDuration: ", epochDuration)

	// TODO: handle if offender claims to be in nextSession. Also, check whether slot skips are an issue.
	blockInOffenderSession := latestBlockNumber - epochDuration*uint64(currentSession-offenderSession)

	// a block in offender's session - only used for getting key ownership proof
	offenderSessionBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(blockInOffenderSession)
	if err != nil {
		return nil, err
	}
	// }

	err = li.relaychainConn.API().Client.Call(&keyOwnershipProofRaw, "state_call", callName, keyOwnershipProofPayload, offenderSessionBlockHash.Hex())

	if err != nil || !ok {
		return nil, fmt.Errorf("generate key owner proof: %w", err)
	}

	keyOwnershipProof, err := hex.DecodeString(keyOwnershipProofRaw[2:])
	if err != nil || !ok {
		return nil, fmt.Errorf("decode proof: %w", err)
	}
	return keyOwnershipProof, nil
}

func (li *BeefyListener) getSignerInfo(meta *types.Metadata) (*signature.KeyringPair, types.UCompact, error) {

	signer := li.relaychainConn.Keypair()
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

	commitmentHash := (&keccak.Keccak256{}).Hash(commitmentBytes)
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

// construct vote payload
func constructVotePayload(commitment contracts.BeefyClientCommitment, offenderPubKeyCompressed []byte, offenderSig []byte) []byte {
	payload := append([]byte{0x04}, commitment.Payload[0].PayloadID[:]...)
	// commitment
	payload = append(payload, 0x80)
	payload = append(payload, commitment.Payload[0].Data...)
	// block number
	blockNumberBytes := make([]byte, 4)
	binary.LittleEndian.PutUint32(blockNumberBytes, commitment.BlockNumber)
	payload = append(payload, blockNumberBytes...)
	// validator set id
	validatorSetBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(validatorSetBytes, commitment.ValidatorSetID)
	payload = append(payload, validatorSetBytes...)
	// id

	payload = append(payload, offenderPubKeyCompressed...)
	// signature
	payload = append(payload, offenderSig[:]...)

	return payload
}

// construct ancestry proof payload
func (li *BeefyListener) constructAncestryProofPayload(commitment contracts.BeefyClientCommitment, latestHash types.Hash) ([]byte, error) {

	var ancestryProof ancestryTypes.GenerateAncestryProofResponse
	err := client.CallWithBlockHash(li.relaychainConn.API().Client, &ancestryProof, "mmr_generateAncestryProof", &latestHash, commitment.BlockNumber, nil)
	if err != nil {
		return nil, fmt.Errorf("generate MMR ancestry proof: %w", err)
	}

	prevPeaksBytes, err := types.EncodeToBytes(ancestryProof.PrevPeaks)
	if err != nil {
		return nil, fmt.Errorf("encode ancestry proof: %w", err)
	}

	payload := prevPeaksBytes

	prevLeafCountBytes, err := types.EncodeToBytes(ancestryProof.PrevLeafCount)
	if err != nil {
		return nil, fmt.Errorf("encode prev leaf count: %w", err)
	}
	payload = append(payload, prevLeafCountBytes...)

	leafCountBytes, err := types.EncodeToBytes(ancestryProof.LeafCount)
	if err != nil {
		return nil, fmt.Errorf("encode leaf count: %w", err)
	}
	payload = append(payload, leafCountBytes...)

	itemsBytes, err := types.EncodeToBytes(ancestryProof.Items)
	if err != nil {
		return nil, fmt.Errorf("encode ancestry proof items: %w", err)
	}
	payload = append(payload, itemsBytes...)

	return payload, nil
}

func (li *BeefyListener) getLatestBeefyBlock() (types.Hash, *types.SignedBlock, error) {
	latestHash, err := li.relaychainConn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return types.Hash{}, nil, fmt.Errorf("get finalized head: %w", err)
	}

	latestBlock, err := li.relaychainConn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return types.Hash{}, nil, fmt.Errorf("get block: %w", err)
	}

	return latestHash, latestBlock, nil
}

func (li *BeefyListener) signedExtrinsicFromCall(meta *types.Metadata, call types.Call) (types.Extrinsic, error) {
	ext := types.NewExtrinsic(call)
	signer, nonce, err := li.getSignerInfo(meta)
	if err != nil {
		return ext, fmt.Errorf("get signer info: %w", err)
	}

	latestHash, latestBlock, err := li.getLatestBeefyBlock()
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

	err = ext.Sign(*signer, o)
	if err != nil {
		return ext, fmt.Errorf("sign extrinsic: %w", err)
	}

	return ext, nil
}

func (li *BeefyListener) watchExtrinsicSubscription(sub *author.ExtrinsicStatusSubscription) error {
	for {
		status := <-sub.Chan()

		if status.IsDropped || status.IsInvalid || status.IsUsurped || status.IsFinalityTimeout {
			sub.Unsubscribe()
			log.WithFields(log.Fields{
				// "nonce":  ext.Signature.Nonce,
				"status": status,
			}).Error("Extrinsic removed from the transaction pool")
			return fmt.Errorf("extrinsic removed from the transaction pool")
		}

		if status.IsFinalized {
			log.Info("Finalized at block hash ", status.AsFinalized.Hex())
			sub.Unsubscribe()
			break
		}
	}
	return nil
}

func decodeTransactionCallData(callData []byte) (string, map[string]interface{}, error) {
	// Parse the ABI
	parsedABI, err := abi.JSON(strings.NewReader(contracts.BeefyClientMetaData.ABI))
	if err != nil {
		return "", nil, fmt.Errorf("parse ABI: %w", err)
	}

	// Get the method signature from the first 4 bytes
	methodSig := callData[:4]
	method, err := parsedABI.MethodById(methodSig)
	if err != nil {
		return "", nil, fmt.Errorf("get method from signature: %w", err)
	}

	// Decode the parameters
	params, err := method.Inputs.Unpack(callData[4:])
	if err != nil {
		return "", nil, fmt.Errorf("unpack parameters: %w", err)
	}

	// Convert to map for handling
	decoded := make(map[string]interface{})
	for i, param := range params {
		decoded[method.Inputs[i].Name] = param
	}

	return method.Name, decoded, nil
}

func (li *BeefyListener) getTransactionCallData(ctx context.Context, txHash common.Hash) ([]byte, error) {
	// Get the transaction
	tx, _, err := li.ethereumConn.Client().TransactionByHash(ctx, txHash)
	if err != nil {
		return nil, fmt.Errorf("get transaction: %w", err)
	}

	// Get the input data
	return tx.Data(), nil
}
