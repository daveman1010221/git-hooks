{ pkgs, src }:

pkgs.stdenv.mkDerivation {
  name = "git-hooks-cargo-vendor";
  inherit src;

  nativeBuildInputs = [ pkgs.cargo pkgs.cacert ];

  # this is a fixed-output derivation, so NO patching
  dontFixup = true;
  dontPatchShebangs = true;

  # we only need Cargo.toml + Cargo.lock, but src is fine for now
  buildPhase = ''

    # give cargo a writable home
    export HOME=$TMPDIR
    export CARGO_HOME=$TMPDIR/.cargo

    # Help cargo find certificates
    export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
    export NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt

    mkdir -p "$CARGO_HOME"
    cargo vendor --versioned-dirs --locked $out
  '';

  installPhase = "true";

  outputHashMode = "recursive";
  outputHashAlgo = "sha256";
  outputHash = "sha256-+tpRgPhkC2dg6vO4o6FQzCnfNm5GsnwakNH655+IPn8=";
}
