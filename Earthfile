# Set the Earthly version to 0.7
VERSION 0.7
FROM debian:stable-slim

rust-toolchain:
    FROM rust:1.71-slim-bullseye
    RUN rustup component add rustfmt

# Installs Cargo chef
install-chef:
    FROM +rust-toolchain
    RUN cargo install --debug cargo-chef

# Prepares the local cache
prepare-cache:
    FROM +install-chef
    COPY --dir vitup iapyx valgrind integration-tests registration-service registration-verify-service snapshot-trigger-service signals-handler .
    COPY Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json
    SAVE IMAGE --cache-hint

# Builds the local cache
build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./

    # Install build dependencies
    RUN apt-get update && \
        apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        libpq-dev \
        libsqlite3-dev \
        pkg-config \
        protobuf-compiler

    RUN cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

# This is the default builder that all other builders should inherit from
builder:
    FROM +rust-toolchain

    WORKDIR /src

    # Install build dependencies
    RUN apt-get update && \
        apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        libpq-dev \
        libsqlite3-dev \
        pkg-config \
        protobuf-compiler
    COPY --dir vitup iapyx valgrind integration-tests registration-service registration-verify-service snapshot-trigger-service signals-handler .
    COPY --dir Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    SAVE ARTIFACT /src

build:
    FROM +builder

    COPY --dir vitup iapyx valgrind integration-tests registration-service registration-verify-service snapshot-trigger-service signals-handler .
    COPY Cargo.toml Cargo.lock ./

    RUN cargo build --locked --release -p iapyx -p valgrind -p vitup

    SAVE ARTIFACT /src/target/release/iapyx iapyx
    SAVE ARTIFACT /src/target/release/iapyx-load iapyx-load
    SAVE ARTIFACT /src/target/release/valgrind valgrind
    SAVE ARTIFACT /src/target/release/vitup vitup