//+build mage

package main

import (
	"github.com/magefile/mage/mg"
	"github.com/magefile/mage/sh"
)

func Build() {
	mg.Deps(BuildMain, BuildTools)
}

func BuildMain() error {
	return sh.Run("go", "build", "-o", "build/artemis-relay", "main.go")
}

func BuildTools() error {
	return sh.Run("go", "build", "-o", "build/list-events", "tools/list_events.go")
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
