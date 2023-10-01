# lol

**This project in under complete reworking. Please wait for a while.**

![CI](https://github.com/akiradeveloper/lol/workflows/CI/badge.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/akiradeveloper/lol/blob/master/LICENSE)
[![Tokei](https://tokei.rs/b1/github/akiradeveloper/lol)](https://github.com/akiradeveloper/lol)

A Raft implementation in Rust language. To support this project please give it a ⭐

![](https://user-images.githubusercontent.com/785824/146726060-63b12378-ecb7-49f9-8025-a65dbd37e9b2.jpeg)

## Features

- Implements all basic [Raft](https://raft.github.io/) features: Replication, Leader Election, Log Compaction, Persistency, Dynamic Membership Change, Streaming Snapshot, etc.
- Based on [Tonic](https://github.com/hyperium/tonic) and efficient gRPC streaming is exploited in log replication and snapshot.
- [Phi Accrual Failure Detector](https://github.com/akiradeveloper/phi-detector) is used for leader failure detection. The adaptive algorithm allows you to not choose a fixed timeout number in prior to deployment and makes it possible to deploy Raft node in even Geo-distributed environment.

## Development

- `docker compose build` to build test servers.
- TERM1: `./log` to start log watcher.
- TERM2: `./dev` to start the dev container.
- TERM2: run `cargo test`.
