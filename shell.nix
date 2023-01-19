{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/2f9fd351ec37f5d479556cd48be4ca340da59b8f.tar.gz") {} }:

pkgs.mkShell {
    buildInputs = with pkgs; [
        jq
        moreutils
        direnv
        typos
        go-ethereum

        # typescript packages
        nodejs-18_x
        nodePackages.pnpm

        # relayer
        go
        mage
        revive

        # parachain
        rustup
        gcc
        openssl
        libiconv
        cmake
        protobuf

        pkgs.cowsay
    ];


    shellHook = ''
        export CARGO_HOME=$PWD/.cargo
        export RUSTUP_HOME=$PWD/.rustup
        export PATH=$CARGO_HOME/bin:$PATH

        rustup install 1.66.1
        rustup install nightly-2022-11-15
        rustup target add wasm32-unknown-unknown --toolchain nightly-2022-11-15
        rustup default 1.66.1

        cowsay "Snowbridge Dev Environment"
    '';
}
