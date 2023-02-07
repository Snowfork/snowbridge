{
    description = "A flake for Snowbridge";

    inputs = {
        nixpkgs.url = "nixpkgs/nixos-22.11";

        /* rust-overlay = { */
        /* url = "github:oxalica/rust-overlay"; */
        /* inputs.nixpkgs.follows = "nixpkgs"; */
        /* }; */
    };

    outputs = { self, nixpkgs }: # , rust-overlay
        let
            pkgs = import nixpkgs { system = "aarch64-darwin"; };
            dependencies = (with pkgs; [
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
                openssl
                libiconv
                cmake
                protobuf
                clang

                cowsay
            ]);
        in {
            devShell.aarch64-darwin = pkgs.mkShell {
                buildInputs = dependencies;
            };
        };

    nixConfig = {};
}
