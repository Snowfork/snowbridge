package ethereum

import (
	"fmt"
	"bytes"
	"context"
	"math/big"

	"github.com/spf13/viper"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"

	keybase "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/substrate"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
	"github.com/snowfork/polkadot-ethereum/prover"

	"github.com/ethereum/go-ethereum/accounts/abi"
)


