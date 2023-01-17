{pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
    buildInputs = [
        pkgs.nodejs-18_x
        pkgs.nodePackages.pnpm
        pkgs.go
        pkgs.rustup

        pkgs.go-ethereum
        pkgs.gcc
        pkgs.jq
        pkgs.moreutils

        pkgs.cowsay
    ];

    shellHook = ''
        cowsay "Hello $FOO"
    '';

    FOO = "Hello";
}