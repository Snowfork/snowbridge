//go:build mage
// +build mage

package main

import (
	"github.com/magefile/mage/mg"
	"github.com/magefile/mage/sh"
)

func Build() {
	mg.Deps(BuildMain)
}

func BuildMain() error {
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
