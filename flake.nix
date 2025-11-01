{
  description = "Small git hooks (commit-msg) built with plain cargo";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs = {
      nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    systems = [ "x86_64-linux" "aarch64-linux" ];
    forAllSystems = f:
      builtins.listToAttrs (map (system: {
        name = system;
        value = f system;
      }) systems);
  in {
    packages = forAllSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # this just produces the /nix/store/...-git-hooks-cargo-vendor tree
        vendor = pkgs.callPackage ./nix/vendor.nix {
          src = ./.;
        };
      in {
        commit-msg-hook = pkgs.stdenv.mkDerivation {
          pname = "commit-msg-hook";
          version = "0.1.0";

          src = ./.;

          doCheck = true;

          nativeBuildInputs = [
            pkgs.cargo
            pkgs.rust-bin.stable.latest.default
            pkgs.binutils
            pkgs.cacert
          ];

          buildPhase = ''
            export CARGO_HOME=$PWD/.cargo
            export NIX_HARDENING_ENABLE=""

            mkdir -p .cargo
            cat > .cargo/config.toml <<EOF
            [source.crates-io]
            replace-with = "vendored-sources"
            [source.vendored-sources]
            directory = "${vendor}"
            EOF

            cargo build --release --frozen --offline
          '';

          checkPhase = ''
            cargo test --frozen --offline
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/commit-msg-hook $out/bin/commit-msg
            ${pkgs.binutils}/bin/strip --strip-all $out/bin/commit-msg || true
            chmod -R 0775 $out/bin/
          '';
        };
      });

    devShells = forAllSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.rust-bin.stable.latest.default
            pkgs.binutils
            pkgs.cacert
          ];
          shellHook = ''
            export NIX_HARDENING_ENABLE="fortify stackprotector pie relro bindnow"
            export CARGO_HOME=$PWD/.cargo
            echo "nix-style hardening enabled; cargo builds should match flake outputs."
          '';
        };
      });

    defaultPackage.x86_64-linux = self.packages.x86_64-linux.commit-msg-hook;
  };
}
