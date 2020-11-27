module github.com/snowfork/polkadot-ethereum/relayer

go 1.13

require (
	github.com/aristanetworks/goarista v0.0.0-20201012165903-2cb20defcd66 // indirect
	github.com/btcsuite/btcd v0.20.1-beta // indirect
	github.com/centrifuge/go-substrate-rpc-client v2.0.0-rc6+incompatible
	github.com/deckarep/golang-set v1.7.1 // indirect
	github.com/ethereum/go-ethereum v1.8.23
	github.com/fatih/color v1.10.0 // indirect
	github.com/magefile/mage v1.10.0
	github.com/mattn/go-runewidth v0.0.9 // indirect
	github.com/mgechev/revive v1.0.2 // indirect
	github.com/mitchellh/go-homedir v1.1.0
	github.com/pierrec/xxHash v0.1.5 // indirect
	github.com/rs/cors v1.7.0 // indirect
	github.com/sirupsen/logrus v1.6.0
	github.com/spf13/cobra v1.0.0
	github.com/spf13/viper v1.7.0
	github.com/stretchr/testify v1.4.0
	github.com/tranvictor/ethashproof v0.0.0-00010101000000-000000000000
	golang.org/x/crypto v0.0.0-20201012173705-84dcc777aaee // indirect
	golang.org/x/sync v0.0.0-20201020160332-67f06af15bc9
	golang.org/x/sys v0.0.0-20201112073958-5cba982894dd // indirect
	golang.org/x/tools v0.0.0-20201112185108-eeaa07dd7696 // indirect
)

replace github.com/tranvictor/ethashproof => ./ethashproof
