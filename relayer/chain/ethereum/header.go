// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
)

func MakeHeaderFromEthHeader(_ *etypes.Header, _ *logrus.Entry) (*chain.Header, error) {
	return &chain.Header{}, nil
}
