Packaging
=========

This file contains quick reminders and notes on how to package Bloom.

We consider here the packaging flow of Bloom version `1.0.0` for Linux.

1. **How to bump Bloom version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`
    3. Bump Debian package version in `debian/rules` to `1.0`

2. **How to build Bloom, package it and release it on GitHub (multiple architectures):**
    1. Install the cross-compilation utility: `cargo install cross`
    2. Release all binaries: `./scripts/release_binaries.sh --version=1.0`
    3. Publish all the built archives on the [releases](https://github.com/valeriansaliou/bloom/releases) page on GitHub

3. **How to build install packages from Bloom version:**
    1. Ensure Docker is running, and that the target build archive is published on GitHub Releases
    2. Build all packages: `./scripts/build_packages.sh --version=1.0`

4. **How to update Bloom on Crates:**
    1. Publish package on Crates: `cargo publish --no-verify`

5. **How to update Docker:**
    1. `docker build .`
    2. `docker tag [DOCKER_IMAGE_ID] valeriansaliou/bloom:v1.0.0` (insert the built image identifier)
    3. `docker push valeriansaliou/bloom:v1.0.0`

Notice: upon packaging `x86_64` becomes `amd64`.
