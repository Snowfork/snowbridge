{ system ? builtins.currentSystem }:

let
    nixpkgs = fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/2f9fd351ec37f5d479556cd48be4ca340da59b8f.tar.gz";
        sha256 = "0w3ysrhbqhgr1qnh0r9miyqd7yf7vsd4wcd21dffwjlb99lynla8";
    };

    pkgs = (import nixpkgs {
        config = {};
        overlays = [];
        inherit system;
    });
in

pkgs.mkShell {
    buildInputs = with pkgs; [
        git
        curl
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
        libiconv
        protobuf

        /* openssl */
        /* cmake */
        /* clang */

        cowsay
    ];


    shellHook = ''
        export CARGO_HOME=$PWD/.cargo
        export RUSTUP_HOME=$PWD/.rustup
        export PATH=$CARGO_HOME/bin:$PATH

        rustup install 1.66.1
        rustup install nightly-2022-11-15
        rustup target add wasm32-unknown-unknown --toolchain nightly-2022-11-15
        rustup default 1.66.1

        # rocksdb requires a clang.so
        export LIBCLANG_PATH="$(readlink -f ${pkgs.clang}/resource-root/include | xargs dirname | xargs dirname | xargs dirname)"

        eval "$(direnv hook bash)"

        cowsay "Snowbridge Dev Environment"
    '';
}
