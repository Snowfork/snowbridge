{
    description = "Snowbridge flake";

    inputs = {
        nixpkgs.url = "nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
        flake-utils.url  = "github:numtide/flake-utils";
        foundry.url = "github:shazow/foundry.nix/monthly";
    };

    outputs = { self, nixpkgs, rust-overlay, flake-utils, foundry }:
    let
        supportedSystems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" ];
        overlays = [ (import rust-overlay) foundry.overlay ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
        let
            pkgs = import nixpkgs { inherit system overlays; };
            cwd = builtins.toString ./.;
            rust =
              pkgs.rust-bin.fromRustupToolchainFile "${cwd}/parachain/rust-toolchain.toml";
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
                    # ps for zombienet, required in pure shells on Linux
                    ps

                    # typescript
                    nodePackages.pnpm
                    nodejs-18_x

                    # ethereum
                    foundry-bin
                    # go-ethereum
                    # gnupg for forge install
                    gnupg

                    # relayer
                    go
                    mage
                    revive

                    # parachain
                    clang
                    gcc
                    libiconv
                    protobuf
                    rust

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

                    # LIBCLANG_PATH points rocksdb to a clang.so on Linux
                    export LIBCLANG_PATH="$(readlink -f ${pkgs.clang}/resource-root/include | xargs dirname | xargs dirname | xargs dirname)"

                    echo "Initializing Snowbridge Dev Environment..."
                    (cd core && pnpm install)

                    cowsay "Snowbridge Dev Environment Ready"
                '';
            };
        }
    );

    nixConfig = {};
}
