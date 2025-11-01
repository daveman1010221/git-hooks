# git-hooks

Tiny Rust hooks, built reproducibly with Nix.

Right now this repo ships **one** hook:

- `commit-msg` → validates / normalizes Conventional Commits headers, rewrites the file in-place

The crate is split into:

- `src/lib.rs` → shared logic (`normalize_commit_message(...)`)
- `src/main.rs` → the actual hook binary

Nix builds it and **runs the Rust tests** (including `proptest`) in the derivation.

---

## Layout

```text
.
├── Cargo.toml        # git_hooks lib + commit-msg-hook bin
├── src/lib.rs        # normalization + tests
├── src/main.rs       # CLI / hook entrypoint
├── flake.nix         # exposes package for x86_64-linux, aarch64-linux
├── nix/vendor.nix    # fixed-output derivation: `cargo vendor ...`
├── .cargo/config.toml# forces local vendored sources
└── .envrc            # `use flake`
```

The important bit: we do NOT commit vendor/. Nix generates it.

## Build it (Nix)

```bash
nix build .#commit-msg-hook
ls -l result/bin/commit-msg
```

That derivation:

1. runs a separate vendoring derivation,
2. writes a .cargo/config.toml pointing at the vendored tree,
3. builds with --frozen --offline,
4. runs cargo test.

So CI-style lockdown is already baked in.

## Use as a Git template

Drop the built hook into your global templates:

```text
environment.etc."git-templates".source = pkgs.runCommand "git-templates" {} ''
  mkdir -p $out/hooks
  install -m0755 ${pkgs.commit-msg-hook}/bin/commit-msg $out/hooks/commit-msg
'';
```

Then in Git:

```bash
git config --global init.templateDir /etc/git-templates
git init new-repo
```

Every new repo gets the hook.

## Adding another hook

You’ve got two options:

### 1. (Simplest): Another bin in this crate

Cargo.toml:

```text
[[bin]]
name = "pre-push-hook"
path = "src/pre_push.rs"
```

and in `flake.nix` add another package that copies it to `$out/bin/pre-push`.

### 2. Separate crate (more Nixy)

Create a sibling crate and package it the same way:

```bash
cargo new --bin pre-push-hook
```

Then:

```text
{
  pre-push-hook = pkgs.stdenv.mkDerivation {
    pname = "pre-push-hook";
    version = "0.1.0";
    src = ./pre-push-hook;
    # same pattern: vendor → build → install
  };
}
```

Drop it next to the existing one in the template:

```text
install -m0755 ${pkgs.pre-push-hook}/bin/pre-push-hook $out/hooks/pre-push
```

## Why the vendoring derivation?

Because Rust test deps (`proptest`, etc.) aren’t optional in Nix builds. If you want **repro + offline + tests**, you vendor in Nix, not in git. That’s what `nix/vendor.nix` is doing.

## Dev Shell

```bash
direnv allow .
# or
nix develop
```

You get cargo, rustc, matching toolchain, and the same hardening flags the derivation uses.

## Notes

- Commit message hook rewrites the commit message if it can clean it.
- If it can’t validate it → nonzero exit → Git blocks the commit.
- Conventional Commits only. No freestyle poetry.
