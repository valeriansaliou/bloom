Packaging
=========

This file contains quick reminders and notes on how to package Bloom.

We consider here the packaging flow of Bloom version `1.0` for Debian, for target architecture `x86_64` (the steps are alike for `i686`):

1. **How to setup `rust-musl-builder` on MacOS:**
    1. Follow setup instructions from: [rust-musl-builder](https://github.com/emk/rust-musl-builder)
    2. Pull the stable Docker image: `docker pull ekidd/rust-musl-builder:stable`

2. **How to bump Bloom version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`
    3. Bump Debian package version in `debian/rules` to `1.0`

3. **How to build Bloom for Linux on MacOS:**
    1. `rust-musl-builder-stable cargo build --target=x86_64-unknown-linux-musl --release`
    2. `rust-musl-builder-stable strip ./target/x86_64-unknown-linux-musl/release/bloom`

4. **How to package built binary and release it on GitHub:**
    1. `mkdir bloom`
    2. `mv target/x86_64-unknown-linux-musl/release/bloom bloom/`
    3. `cp config.cfg bloom/`
    4. `tar -czvf v1.0-amd64.tar.gz bloom`
    5. `rm -r bloom/`

5. **How to trigger a Debian build from Travis CI:**
    1. Commit your changes locally
    2. `git describe --always --long` eg. gives `8aca211` (copy this)
    3. `git tag -a 1.0` insert description eg. `1.0-0-8aca211` and save
    4. `git push origin 1.0:1.0`
    5. Quickly upload the archive files as GitHub releases before the build triggers, named as eg. `v1.0-amd64.tar.gz`

6. **How to update Crates:**
    1. Publish package on Crates: `cargo publish`

7. **How to update Docker:**
    1. `docker build .`
    2. `docker tag [DOCKER_IMAGE_ID] valeriansaliou/bloom:v1.0.0` (insert the built image identifier)
    3. `docker push valeriansaliou/bloom:v1.0.0`

Notice: upon packaging `x86_64` becomes `amd64` and `i686` becomes `i386`.
