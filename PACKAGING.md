Packaging
=========

This file contains quick reminders and notes on how to package Bloom.

We consider here the packaging flow of Bloom version `1.0.0` for Linux.

1. **How to bump Bloom version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`
    3. Bump Debian package version in `debian/rules` to `1.0`

2. **How to build Bloom, package it and release it on Crates, GitHub, Docker Hub and Packagecloud (multiple architectures):**
    1. Commit your changes locally, and push them (but do not tag them at this point)
    2. `git describe --always --long` eg. gives `8aca211` (copy this)
    3. `git tag -a 1.0` insert description eg. `1.0-0-8aca211` and save
    4. `git push origin 1.0:1.0`
    5. Wait for all release jobs to complete on the [actions](https://github.com/valeriansaliou/bloom/actions) page on GitHub
    6. Download all release archives, and sign them locally using: `./scripts/sign_binaries.sh --version=1.0`
    7. Publish a changelog and upload all the built archives, as well as their signatures on the [releases](https://github.com/valeriansaliou/bloom/releases) page on GitHub

Notice: upon packaging `x86_64` becomes `amd64`.
