{
  description = "Zelos development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        fenixPkgs = fenix.packages.${system};
        rustToolchain = fenixPkgs.latest.toolchain; # nightly toolchain incl. rustc/cargo/rustfmt/clippy
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Core tooling
            git
            just
            treefmt

            # Rust toolchain (nightly via fenix)
            rustToolchain

            # Go toolchain + protobuf compiler and plugins
            go
            protobuf
            protoc-gen-go
            protoc-gen-go-grpc

            # Python tooling
            ruff
            uv

            # Linters/formatters
            shfmt
            shellcheck
            taplo

            # Utilities
            jq
            unzip
            typos
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ]
            ++ [ pkgs.pkg-config ];

          shellHook = ''
            # Ensure Go-installed binaries are accessible
            if command -v go >/dev/null 2>&1; then
              export PATH="$PATH:$(go env GOPATH)/bin"
            fi
            echo "Loaded Zelos dev shell."
          '';
        };
      });
}


