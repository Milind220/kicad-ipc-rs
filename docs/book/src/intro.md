# Introduction

`kicad-ipc-rs` is a Rust client for KiCad's IPC API.

## Why this crate?

`kicad-ipc-rs` gives you programmatic control over KiCad with an ergonomic, type-safe Rust API. Whether you're building automation tools, integrating KiCad into CI/CD pipelines, or creating custom workflows, this crate provides the most complete and well-documented interface to KiCad's API.

### Key Features

- **KiCad 11 Groundwork**: protos regenerated from KiCad `10.99.0-991-gd3bb3db575`
- **Raw KiCad 11 Access**: generated protobuf modules plus command helpers for new IPC families
- **KiCad 10 Wrapper Coverage**: existing typed wrappers remain available while KiCad 11 wrappers are tracked
- **Type-Safe Models**: Native Rust structs for tracks, vias, footprints, nets, and more
- **Dual API**: Async-first design with full synchronous support via `blocking` feature
- **Checked-In Protos**: no KiCad source checkout needed for consumers
- **Field-Used**: Applied in automation and integration workflows

### API Comparison
| Capability | `kicad-ipc-rs` | Python bindings | Official Rust |
|------------|---------------|-----------------|---------------|
| Rust-native API | KiCad 10 typed, KiCad 11 raw-first | Python only | Preview |
| Async + Sync | Both supported | Event-loop | Preview |
| KiCad 11 coverage | Raw proto/helper groundwork | Unknown | Unknown |
| Active maintenance | Yes | Official | Preview |

## Project Goals
- Rust-native API for all KiCad IPC commands
- Typed, ergonomic models for board and editor operations
- Full parity between async and blocking APIs
- Clear documentation and real-world examples
- Stable, maintainable release workflow

## Current Scope

- KiCad API proto snapshot pinned in repo (`src/proto/generated/`)
- KiCad 11 / 10.99 generated modules exposed under `kicad_ipc_rs::proto`
- Raw command helpers for new project, editor, board, schematic, and job command families
- Existing KiCad 10 typed wrappers retained pending KiCad 11 wrapper design

## Core Entrypoints
- **Async**: `kicad_ipc_rs::KiCadClient`
- **Blocking**: `kicad_ipc_rs::KiCadClientBlocking` (enable `blocking` feature)
- **Errors**: `kicad_ipc_rs::KiCadError`
- **Raw KiCad 11**: `kicad_ipc_rs::commands::*`, `kicad_ipc_rs::proto`, `KiCadClient::send_raw_command`

## Getting Started

Jump to [Quickstart](quickstart.md) to connect to KiCad and run your first commands.

## Related Docs

- [Crate README](https://github.com/Milind220/kicad-ipc-rs/blob/main/README.md)
- [API Reference on docs.rs](https://docs.rs/kicad-ipc-rs)
- [Examples](examples.md) for real-world patterns
