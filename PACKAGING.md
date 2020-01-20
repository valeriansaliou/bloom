Packaging
=========

This file contains quick reminders and notes on how to package Bloom.

We consider here the packaging flow of Bloom version `1.0.0` for Linux.

1. **How to setup `rust-musl-builder` on MacOS (required to build binaries):**
    1. Follow setup instructions from: [rust-musl-builder](https://github.com/emk/rust-musl-builder)
    2. Pull the stable Docker image: `docker pull ekidd/rust-musl-builder:stable`

2. **How to bump Bloom version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`
    3. Bump Debian package version in `debian/rules` to `1.0`

3. **How to build Bloom, package it and release it on GitHub (multiple architectures):**
    1. `./scripts/release_binaries.sh --version=1.0.0`
    2. Publish all the built archives on the [releases](https://github.com/valeriansaliou/bloom/releases) page on GitHub

4. **How to trigger a Debian build from Travis CI:**
    1. Commit your changes locally
    2. `git describe --always --long` eg. gives `8aca211` (copy this)
    3. `git tag -a 1.0` insert description eg. `1.0-0-8aca211` and save
    4. `git push origin 1.0:1.0`
    5. Quickly upload the archive files as GitHub releases before the build triggers, named as eg. `v1.0-amd64.tar.gz`

5. **How to update Bloom on Crates:**
    1. Publish package on Crates: `cargo publish --no-verify`

6. **How to update Docker:**
    1. `docker build .`
    2. `docker tag [DOCKER_IMAGE_ID] valeriansaliou/bloom:v1.0.0` (insert the built image identifier)
    3. `docker push valeriansaliou/bloom:v1.0.0`

Notice: upon packaging `x86_64` becomes `amd64`.
