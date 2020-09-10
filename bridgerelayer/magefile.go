//+build mage

package main

import (
	"github.com/magefile/mage/sh"
)

func Build() error {
	return sh.Run("go", "build", "-o", "build/artemis-relay", "main.go")
}

func Test() error {
	return sh.Run("go", "test", "./...")
}

func Lint() error {
	return sh.Run("revive", "-config", "revive.toml", "./...")
}

func Install() error {
	return sh.Run("go", "build", "-o", "$GOPATH/bin/artemis-relay", "main.go")
}
