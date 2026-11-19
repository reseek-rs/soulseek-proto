default:
    @just --list

# cargo nextest without and with the features, then the doc examples nextest does not run
test:
    cargo nextest run
    cargo nextest run --all-features
    cargo test --all-features --doc

# formatting of the crate and the fuzz crate, and clippy without and with the features
lint:
    cargo fmt --all --check
    cargo fmt --manifest-path fuzz/Cargo.toml --all --check
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-features --all-targets -- -D warnings

# the library alone on the manifest's rust-version, without and with the features; its tests
# need a newer Rust
msrv:
    CARGO_TARGET_DIR=target/msrv nix develop .#msrv -c cargo build --locked
    CARGO_TARGET_DIR=target/msrv nix develop .#msrv -c cargo build --locked --all-features

# fails when a committed Cargo.lock does not match its manifest; the other recipes would
# rewrite it and pass
locked:
    cargo metadata --locked --format-version 1 > /dev/null

# the gate for every push; ci.yml runs it
ci: locked lint test msrv

# package and build the crate as crates.io would, without uploading; release.yml runs it
publish-check:
    cargo publish --dry-run

# the fuzz recipes need nightly cargo and cargo-fuzz: run them inside `nix develop .#fuzz`

[doc("write the round-trip vectors into fuzz/corpus/<target> as seeds")]
fuzz-seed:
    cargo test --lib -- --ignored --exact testing::write_fuzz_corpus

# the fuzz crate is a workspace of its own, so its lockfile starts as a copy of the crate's: the
# targets build against the versions the crate is tested with, and cargo adds only the fuzz crates
[private]
fuzz-lock:
    cp Cargo.lock fuzz/Cargo.lock

# run one fuzz target, indefinitely unless SECONDS is given
fuzz TARGET SECONDS="": fuzz-seed fuzz-lock
    cargo fuzz run {{ TARGET }} {{ if SECONDS == "" { "" } else { "-- -max_total_time=" + SECONDS } }}

# split HOURS evenly across every fuzz target
fuzz-all HOURS: fuzz-seed fuzz-lock
    #!/usr/bin/env bash
    set -euo pipefail
    targets=$(cargo fuzz list)
    count=$(echo "$targets" | wc -l)
    per_target=$(( {{ HOURS }} * 3600 / count ))
    for target in $targets; do
      cargo fuzz run "$target" -- -max_total_time="$per_target"
    done
