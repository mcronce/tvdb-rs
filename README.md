# `tvdb-rs`

[![Build Status](https://travis-ci.org/dbr/tvdb-rs.png?branch=master)](https://travis-ci.org/dbr/tvdb-rs)

[TheTVDB.com][tvdb] interface for the Rust programming langauge

## Status

Work in progress

[tvdb]: http://thetvdb.com/


## Release procedure
1. Make changes
2. Ensure CHANGELOG.md is updated
3. `cargo test` etc
4. Bump version in Cargo.toml
5. `cargo publish` pushes new version to cargo
6. Commit version bump
7. Tag release `git tag -a v0.1.0`
8. `git push --tags`
