Packaging
=========

This file contains quick reminders and notes on how to package Bloom.

We consider here the packaging flow of Bloom version `0.1` for Debian, for target architecture `x86_64` (the steps are alike for `i686`):

1. **How to setup Rustup Linux toolchain on MacOS:**
    1. `brew install filosottile/musl-cross/musl-cross` (see: [FiloSottile/homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross))
    2. `rustup target add x86_64-unknown-linux-musl`

2. **How to bump Bloom version before a release:**
    1. Bump version in `Cargo.toml` to `0.1.0`
    2. Execute `cargo update` to bump `Cargo.lock`
    3. Bump Debian package version in `debian/rules` to `0.1`

3. **How to build Bloom for Linux on macOS:**
    1. `cargo build --target=x86_64-unknown-linux-musl --release`

4. **How to package built binary and release it on GitHub:**
    1. `mkdir bloom`
    2. `mv target/x86_64-unknown-linux-musl/release/bloom bloom/`
    3. `cp config.cfg bloom/`
    4. `tar -czvf v0.1-x86_64.tar.gz bloom`
    5. `rm -r bloom/`

5. **How to trigger a Debian build from Travis CI:**
    1. `git describe --always --long` eg. gives `8aca211` (copy this)
    2. `git tag -a 0.1` insert description eg. `0.1-0-8aca211` and save
    3. `git push origin 0.1:0.1`
    4. Quickly upload the archive files as GitHub releases before the build triggers, named as eg. `v0.1-x86_64.tar.gz`

Cargo configuration for custom Linux linkers (`~/.cargo/config`):

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.i686-unknown-linux-musl]
linker = "i486-linux-musl-gcc"
```
