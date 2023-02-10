{
    description = "Snowbridge flake";

    inputs = {
        nixpkgs.url = "nixpkgs/nixos-22.11";
        rust-overlay.url = "github:oxalica/rust-overlay";
        flake-utils.url  = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    let
        supportedSystems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" ];
        overlays = [ (import rust-overlay) ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
        let
            pkgs = import nixpkgs { inherit system overlays; };
        in
        with pkgs;
        {
            devShells.default = pkgs.mkShell {
                buildInputs = [
                    cacert
                    curl
                    direnv
                    git
                    jq
                    moreutils
                    typos
                    # required by zombienet & not available in pure shells on Linux
                    ps

                    # typescript
                    nodejs-18_x
                    nodePackages.pnpm

                    # ethereum
                    go-ethereum

                    # relayer
                    go
                    mage
                    revive

                    # parachain
                    gcc
                    libiconv
                    protobuf
                    rustup

                    clang
                    cmake
                    openssl

                    cowsay
                ];

                shellHook = ''
                    # set HOME for direnv and go
                    #
                    # direnv needs config, cache & data dirs (DIRENV_CONFIG, XDG_CACHE_HOME & XDG_DATA_HOME
                    # respectively) that can be automatically set when HOME is available
                    #
                    # relayer builds fail without GOPATH & GOCACHE set
                    # explicitly setting HOME allows go to infer these vars
                    #
                    export HOME=~

                    eval "$(direnv hook bash)"

                    # rocksdb requires a clang.so available in LIBCLANG_PATH on Linux
                    export LIBCLANG_PATH="$(readlink -f ${pkgs.clang}/resource-root/include | xargs dirname | xargs dirname | xargs dirname)"

                    cowsay "Snowbridge Dev Environment"
                '';
            };
        }
    );

    nixConfig = {};
}
