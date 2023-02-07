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
                    libiconv

                    # cmake
                    # gcc
                    # libiconv
                    # openssl
                    # protobuf

                    cowsay
                ];

                shellHook = ''
                    eval "$(direnv hook bash)"

                    cowsay "Snowbridge Dev Environment"
                '';
            };
        }
    );

    nixConfig = {};
}
