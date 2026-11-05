//go:build mage

package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/magefile/mage/mg"
	"github.com/magefile/mage/sh"
)

func Build() {
	mg.Deps(BuildMain)
}

func BuildMain() error {
	err := sh.Run("sszgen", "--path", "relays/beacon/state", "--objs", "BlockRootsContainerMainnet,TransactionsRootContainer,WithdrawalsRootContainerMainnet,BeaconStateDenebMainnet,BeaconBlockDenebMainnet,SignedBeaconBlockDeneb,SignedBeaconBlockElectra,BeaconStateElectra,BeaconBlockElectra,BeaconStateFulu")
	if err != nil {
		return err
	}

	// abigen must match the go-ethereum module version. A newer abigen on PATH
	// (e.g. from a local geth checkout) can emit bindings that don't compile.
	if err := ensureModuleAbigen(); err != nil {
		return err
	}

	err = sh.RunWith(abigenPathEnv(), "go", "generate", "./...")
	if err != nil {
		return err
	}
	return sh.Run("go", "build", "-o", "build/snowbridge-relay", "main.go")
}

func Test() error {
	return sh.RunV("go", "test", "./...")
}

func Lint() error {
	return sh.Run("revive", "-config", "revive.toml", "./...")
}

func Install() error {
	return sh.Run("go", "build", "-o", "$GOPATH/bin/snowbridge-relay", "main.go")
}

func ensureModuleAbigen() error {
	gethVer, err := sh.Output("go", "list", "-m", "-f", "{{.Version}}", "github.com/ethereum/go-ethereum")
	if err != nil {
		return err
	}
	gethVer = strings.TrimSpace(gethVer)

	abigenBin := filepath.Join(gopathBin(), "abigen")
	if current, err := sh.Output(abigenBin, "--version"); err == nil {
		if abigenMatches(current, gethVer) {
			fmt.Printf("Using existing abigen@%s\n", gethVer)
			return nil
		}
	}

	fmt.Printf("Installing abigen@%s to match go.mod\n", gethVer)
	return sh.Run("go", "install", "github.com/ethereum/go-ethereum/cmd/abigen@"+gethVer)
}

func abigenMatches(versionOutput, gethVer string) bool {
	// abigen prints e.g. "abigen version 1.17.5-stable"
	// go.mod versions look like "v1.17.5"
	want := strings.TrimPrefix(gethVer, "v")
	re := regexp.MustCompile(`abigen version ([0-9]+\.[0-9]+\.[0-9]+)`)
	m := re.FindStringSubmatch(versionOutput)
	if len(m) < 2 {
		return false
	}
	return m[1] == want
}

func gopathBin() string {
	gopath, err := sh.Output("go", "env", "GOPATH")
	if err != nil {
		return filepath.Join(os.Getenv("HOME"), "go", "bin")
	}
	return filepath.Join(strings.TrimSpace(gopath), "bin")
}

func abigenPathEnv() map[string]string {
	return map[string]string{
		"PATH": gopathBin() + string(os.PathListSeparator) + os.Getenv("PATH"),
	}
}
