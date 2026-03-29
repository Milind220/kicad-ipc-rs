# Introduction

`kicad-ipc-rs` is a production-ready Rust client for KiCad's IPC API.

## Why this crate?

`kicad-ipc-rs` gives you programmatic control over KiCad with an ergonomic, type-safe Rust API. Whether you're building automation tools, integrating KiCad into CI/CD pipelines, or creating custom workflows, this crate provides the most complete and well-documented interface to KiCad's API.

### Key Features

- **100% API Coverage**: All 57 KiCad v10.0.0 API commands implemented
- **Type-Safe Models**: Native Rust structs for tracks, vias, footprints, nets, and more
- **Dual API**: Async-first design with full synchronous support via `blocking` feature
- **Zero Protobuf Hassle**: Pre-generated types — no KiCad source checkout needed
- **Battle-Tested**: Used in real automation and integration workflows

### API Comparison

| Capability | `kicad-ipc-rs` | Python bindings | Official Rust |
|------------|---------------|-----------------|---------------|
| Rust-native API | ✅ Production-ready | ❌ Python only | ⚠️ Preview |
| Async + Sync | ✅ Both supported | ⚠️ Event-loop | ⚠️ Preview |
| Complete coverage | ✅ 57/57 commands | Unknown | Unknown |
| Active maintenance | ✅ Yes | ✅ Official | ⚠️ Preview |

## Project Goals

- Rust-native API for all KiCad IPC commands
- Typed, ergonomic models for board and editor operations
- Full parity between async and blocking APIs
- Clear documentation and real-world examples
- Stable, maintainable release workflow

## Current Scope

- KiCad API proto snapshot pinned in repo (`src/proto/generated/`)
- 57/57 wrapped command families from KiCad v10.0.0
- Runtime compatibility verified against KiCad 10.0.0

## Core Entrypoints

- **Async**: `kicad_ipc_rs::KiCadClient`
- **Blocking**: `kicad_ipc_rs::KiCadClientBlocking` (enable `blocking` feature)
- **Errors**: `kicad_ipc_rs::KiCadError`

## Getting Started

Jump to [Quickstart](quickstart.md) to connect to KiCad and run your first commands.

## Related Docs

- [Crate README](https://github.com/Milind220/kicad-ipc-rs/blob/main/README.md)
- [API Reference on docs.rs](https://docs.rs/kicad-ipc-rs)
- [Examples](examples.md) for real-world patterns
