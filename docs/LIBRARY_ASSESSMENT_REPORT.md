# Library Assessment Report

Date: 2026-03-29  
Repository: `kicad-ipc-rs`  
Branch assessed: `chore/kicad-v10-stable-protos`  
Related PR: #23 (`feat: bump vendored KiCad protos to v10.0.0` + follow-up tests)

## Purpose

This report evaluates the library's structure, correctness, and practical usefulness using a layered review sequence inspired by:

1. `m15-anti-pattern` (idiomatic/code-smell review)
2. `plan-eng-review` (architecture/maintainability review)
3. `review` (change-level correctness review)
4. `document-release` (docs + release/readiness review)

No fixes were applied while producing this report.

## Scope and Method

### In-scope

- Core crate code under `src/`
- Public docs and usage docs:
  - `README.md`
  - `docs/book/src/*.md`
- Current PR diff vs `origin/main`
- Test/lint/validation signals from local command runs

### Out-of-scope

- KiCad upstream implementation internals beyond proto surface validation
- Runtime behavior on all KiCad OS/channel combinations
- Performance benchmarking

### Evidence-gathering commands run

- Test inventory and counts (`rg` on `#[test]` and `#[tokio::test]`)
- LOC and public API counts (`wc -l`, `rg -n "pub async fn"`, `rg -n "pub "`)
- Diff/PR scope checks (`git diff origin/main...HEAD`)
- Validation checks:
  - `cargo test -q`
  - `cargo test --features blocking -q`
  - `cargo clippy --all-targets --all-features -- -D warnings`

## Executive Summary

Overall status: **Good functional correctness with strong protocol-awareness, moderate maintainability risk, and moderate documentation/discoverability risk.**

- Correctness: **Strong** for current scope and PR #23 changes.
- Architecture: **Solid layering** but **high centralization** in a large `client.rs` module.
- API usefulness: Good typed surface, with intentional raw escape hatches.
- Release/docs hygiene: Version-snippet drift is now resolved; remaining weaknesses are cross-link integrity, rustdoc coverage depth, and onboarding examples/prerequisites.
- Test posture: Healthy and improving; protocol-contract tests were a high-value addition.

## Baseline Metrics

### Code footprint

#### Full non-generated source tree (LOC)

- `src/client.rs`: 5448
- `src/model/board.rs`: 746
- `src/blocking.rs`: 667
- `src/model/common.rs`: 550
- `src/transport.rs`: 126
- `src/lib.rs`: 112
- `src/envelope.rs`: 110
- `src/error.rs`: 90
- `src/proto/mod.rs`: 38
- `src/commands/mod.rs`: 4
- `src/model/mod.rs`: 2
- `src/kicad_api_version.rs`: 2
- `src/commands/project.rs`: 2
- `src/commands/editor.rs`: 2
- `src/commands/board.rs`: 2
- `src/commands/base.rs`: 2

**Non-generated total:** 7903 LOC  
**Generated proto total:** 4863 LOC  
**Overall total:** 12766 LOC

Interpretation: The crate remains heavily concentrated in `src/client.rs`, which acts as both API facade and a significant portion of mapping/decoding logic.

### Dependency and module surface snapshot

- Direct dependencies: 6 (`nng`, `prost`, `prost-types`, `thiserror`, `tokio`, `tracing`)
- Public modules exported from `lib.rs`: 7
  - `client`
  - `commands`
  - `envelope`
  - `error`
  - `model`
  - `transport`
  - `blocking` (feature-gated)
- Total public API items (excluding generated code): 139

Interpretation: API breadth is substantial relative to crate size; discoverability and consistency controls are important.

### API surface size

- Public async methods in `KiCadClient`: 107
- Public blocking methods in `KiCadClientBlocking`: 26 explicitly counted wrappers (plus macro-generated parity set)

Interpretation: This is a broad API surface for a single crate module; drift control and discoverability are key concerns.

### Test inventory (unit tests in `src/`)

- `src/client.rs`: 50
- `src/blocking.rs`: 7
- `src/model/common.rs`: 6
- `src/model/board.rs`: 4
- `src/envelope.rs`: 2

Total identified unit tests: 69

Interpretation: Coverage appears substantial for a library of this size, with strongest emphasis on client behavior and mapping.

## Pass 1: `m15-anti-pattern` Findings

### What looks good

- No `unsafe` usage in `src/`, reducing a major class of memory-safety risk.
- Error mapping is explicit and specific in `src/error.rs`.

### Verified clean signals

- **Zero `.unwrap()` calls in production code** (confirmed via exhaustive grep).
- **All 56 `.expect()` calls are in test code only** (zero in production paths).
- **Zero production `panic!` usage** (all 7 instances in `src/client.rs` are in `#[cfg(test)]` modules).
- **No TODO/FIXME/HACK/XXX markers** in non-generated source.

### Notable anti-pattern/style findings

#### AP-1: `clone_on_copy` in production mapping path

- Evidence: `src/client.rs:2329` (`Option<StrokeAttributes>` clone called where type is `Copy`)
- Impact: Low runtime impact, but signals unnecessary ownership noise and can obscure intent.
- Severity: Low

#### AP-2: Strict clippy fails on generated protobuf enums

- Evidence:
  - `src/proto/generated/kiapi.common.rs:56`
  - `src/proto/generated/kiapi.common.commands.rs:424`, `:459`, `:490` (and similar)
- Lint class: `clippy::enum_variant_names` under `-D warnings`
- Impact: High CI/tooling friction if strict clippy is expected to pass globally.
- Severity: Medium (process/tooling risk)

#### AP-3: Test-style bool asserts flagged

- Evidence: test assertions around `src/client.rs:4648`, `:4657`, `:4663`, `:4692`, `:4698`
- Impact: Low; style-level issue only.
- Severity: Low

#### AP-4: Heavy repeated RPC boilerplate in `client.rs`

- Repeated pattern appears across many methods:
  1. Build command
  2. `send_command(envelope::pack_any(&command, CMD_*))`
  3. `response_payload_as_any(response, RES_*)`
  4. `decode_any(...)`
- Evidence:
  - `send_command + pack_any`: `src/client.rs:367`, `:411`, `:434`, `:459`, `:475`, `:508`, `:1665`
  - `response_payload_as_any`: `src/client.rs:369`, `:413`, `:436`, `:477`, `:510`, `:530`, `:1842`
  - `decode_any`: `src/client.rs:379`, `:422`, `:445`, `:484`, `:536`, `:589`, `:1698`
- Impact: Boilerplate proliferation increases maintenance drag and inconsistency risk.
- Severity: Medium

#### AP-5: Silenced results via `let _ =` in production code

- Evidence:
  - `src/client.rs`: `:519`, `:564`, `:1422`, `:1716`, `:1745`, `:1797`, `:1828`, `:1846`
  - `src/transport.rs:36` (channel send during shutdown)
  - `src/blocking.rs`: `:41`, `:46`, `:77`, `:103`
- Notes: Some cases are intentional (e.g., benign send failure during shutdown), but several cases could hide useful operational failures.
- Impact: Potential silent failure paths and debugging opacity.
- Severity: Low-Medium

#### AP-6: Pervasive unchecked `as i32` casts for protobuf enum discriminants

- Evidence: 77 production instances across `src/client.rs` and `src/model/common.rs`.
  - Examples: `src/model/common.rs:42-48`, `:107-112`; `src/client.rs:127-195`, `:1213`, `:1219`, `:2145-2162`, `:2691-2705`
- Notes: This is common in prost-backed code, but it is still unchecked narrowing.
- Impact: Low in current protocol-constrained context; type-safety remains weaker than explicit conversion helpers.
- Severity: Low

### Anti-pattern conclusion

No major architectural anti-patterns like pervasive `unwrap` in production paths, unsafe shortcuts, or panic-driven control flow were found. Primary concern is maintainability/process friction: boilerplate repetition, strict clippy policy mismatch with generated code, and a handful of silent-result patterns.

## Pass 2: `plan-eng-review` Findings (Structure and Maintainability)

### Structural strengths

- Clean conceptual layering reflected in module organization:
  - `transport` (IPC boundary)
  - `envelope` (protobuf Any/type URL handling)
  - client-level typed wrappers and conversions
- Blocking facade includes a strong parity guard:
  - `src/blocking.rs:586` (`sync_wrapper_covers_async_method_names`)
  - This is an excellent protection against async/blocking drift.
- Rich typed models provide ergonomic APIs over raw protobuf payloads:
  - `src/model/board.rs`
  - `src/model/common.rs`

### Transport architecture

- Implemented in a single file: `src/transport.rs` (126 LOC), not a transport directory.
- Uses an `nng` `Req0` socket with an async-to-blocking bridge.
- Tokio MPSC queue (capacity = 64) feeds a dedicated OS worker thread.
- Worker performs blocking `socket_roundtrip` (send then recv).
- Async side awaits completion via oneshot response channels.
- Timeout and transport failures are clearly mapped to `KiCadError::{Timeout, TransportSend, TransportReceive, Connection}`.

### Feature flag architecture

- `default = ["async"]`
- `async = ["dep:nng", "dep:prost", "dep:prost-types", "dep:tokio"]`
- `blocking = ["async"]` (additive, not replacement)
- `tracing = ["dep:tracing"]`
- Blocking facade runs a dedicated single-thread Tokio runtime worker and dispatches sync calls through a bounded channel.

### Model layer cross-dependencies

- `model::common` depends on board-layer types (`PcbItem`, `Vector2Nm`, etc.).
- Effective layering is common → board-aware, not fully independent.
- This is pragmatically acceptable today, but worth noting if future domain split/modularization is pursued.

### Structural risks

#### ST-1: Monolithic client module

- Evidence: `src/client.rs` at 5448 LOC with 107 public async methods.
- Why it matters:
  - Larger review blast radius for changes.
  - Mixed responsibilities (command dispatch + mapping + helpers + tests) in one file.
  - Harder onboarding and higher chance of incidental coupling.
- Severity: Medium

##### Natural splitting points for `client.rs`

- `client/core.rs`: connect/send_command/context resolution (~lines 284-348)
- `client/common.rs`: ping/version/paths/open docs/run_action/text/netclass (~lines 359-666)
- `client/items.rs`: item CRUD, item decoding, by-id/by-type queries (~lines 671-861, 1211-1409)
- `client/board.rs`: board layers/origin/stackup/appearance/nets (~lines 885-1040, 1572-1744)
- `client/selection.rs`: selection mutation + summaries/details (~lines 1047-1201)
- `client/geometry.rs`: text extents/shapes, bounding box, hit test, pad polygon (~lines 1426-1572, 1879-1954)
- `client/document.rs`: save/revert/string serialization/title block (~lines 1750-1865)
- `client/mappers.rs`: pure proto↔model mapping helpers (post-1954)

#### ST-2: Public `commands::*` modules appear as placeholders

- Evidence:
  - `src/commands/base.rs`
  - `src/commands/board.rs`
  - `src/commands/editor.rs`
  - `src/commands/project.rs`
- Each contains only a trivial empty struct.
- Why it matters: Public API surface may imply supported low-level builder functionality that is not yet substantive.
- Severity: Low to Medium (usability/signaling risk)

#### ST-3: Very broad public API requires stronger discoverability discipline

- With 100+ async methods, docs quality and examples become critical for practical use.
- Missing docs warnings and uneven method-level docs indicate discoverability debt.
- Severity: Medium

### Structure conclusion

Architecture is fundamentally sound, but maintainability risk is rising due to centralization and API breadth. Modularizing client domains and reducing repeated RPC boilerplate are the highest-value structural improvements.

## Pass 3: `review` Findings (PR #23 Correctness)

### What changed in PR scope

Diff vs `origin/main` (10 files changed):

- `kicad` submodule pin update to KiCad `10.0.0`
- Generated proto refresh:
  - `src/proto/generated/kiapi.board.commands.rs`
- New method wiring:
  - `src/client.rs` (`GetBoardLayerName` request/response constants + method)
  - `src/blocking.rs` parity method
- Coverage/document updates:
  - `README.md`
  - `docs/book/src/intro.md`
  - `docs/book/src/validation.md`
  - `src/lib.rs`
  - `src/kicad_api_version.rs`
  - `test-scripts/kicad-ipc-cli.rs`

### Correctness assessment

#### CR-1: KiCad v10 stable proto delta appears correctly applied

- The newly-added command (`GetBoardLayerName`) is reflected in generated types and wrapped in high-level API.

#### CR-2: Protocol-contract tests are meaningful and non-trivial

Added tests verify runtime `type_url` contracts, which the Rust type system cannot enforce by itself:

- decode succeeds on expected type URL
- decode fails on mismatched type URL
- command Any packing uses expected proto command name

This is a strong testing choice for IPC/protobuf Any boundaries.

#### CR-3: Blocking parity maintained

- New async method has matching blocking exposure and remains protected by parity test strategy.

### Correctness conclusion

PR #23 quality is good. Changes are focused, functionally coherent, and backed by targeted tests that protect real protocol failure modes.

## Pass 4: `document-release` Findings (Usefulness and Release Readiness)

### Positive documentation signals

- README contains detailed compatibility matrix and runtime notes.
- Book includes API reference links and practical usage patterns.
- Validation commands are clearly documented.
- Version snippet drift previously identified in DR-1 has been corrected (see **Resolved Issues**).

### Active documentation and release-readiness findings

#### DR-2: Broken anchor in `validation.md`

- `docs/book/src/validation.md:21` links to README anchor `#kicad-v1000-api-completion-matrix`
- README heading is `## KiCad v10.0.0 API Reference` (`README.md:118`)
- Anchor slug does not match current heading
- Severity: Low (broken cross-reference)

#### DR-3: Filesystem artifact in docs tree

- `docs/book/src/https:/docs.rs/` exists as a literal directory
- This appears to be an accidental malformed path/link artifact
- Severity: Low (docs hygiene)

#### DR-4: Rustdoc coverage gap in `client.rs`

- 120 public items in `src/client.rs`
- 47 documented (39% coverage)
- 73 public items undocumented
- Gaps notably include several `*_raw` variants and some typed wrappers
- Evidence examples of undocumented public methods: `src/client.rs:359`, `:403`, `:426`, `:472`, `:861`, `:908`
- Severity: Medium (API discoverability)

#### DR-5: Narrow examples set

- Only one example: `examples/selection_deep_dump.rs` (632 LOC)
- Current example is advanced and blocking-feature-gated
- No beginner-oriented examples (e.g., connect+ping, simple list/query)
- Severity: Medium (onboarding friction)

#### DR-6: README missing explicit prerequisites section

- No dedicated prerequisites section explicitly stating KiCad runtime requirements (running KiCad with IPC available/enabled)
- Runtime preconditions are discoverable via book quickstart, but not surfaced early in README
- Severity: Low-Medium (onboarding clarity)

### Documentation conclusion

Documentation quality is generally strong, and version alignment has materially improved. Remaining gaps are low-to-medium severity but user-visible: cross-link correctness, explicit prerequisites, broader examples, and improved rustdoc coverage for the Tier 1 API surface.

## Quality Gates and Validation Status

### Tests

- `cargo test -q`: pass
- `cargo test --features blocking -q`: pass

### Linting

- `cargo clippy --all-targets --all-features -- -D warnings`: fails

Observed root causes:

1. Generated proto enum naming lints under strict clippy
2. A few fixable local style issues (clone-on-copy, bool assert style in tests)

Implication: Either clippy policy needs explicit generated-code handling, or strict global clippy will remain noisy/fragile.

## Resolved Issues

### DR-1: Stale version snippets in docs (resolved)

Previously reported version drift has been fixed and is no longer an active risk.

Resolved evidence:

- `README.md:27` → `0.4.1` ✓
- `README.md:67` → `0.4.1` ✓
- `docs/book/src/quickstart.md:15` → `0.4.1` ✓
- `docs/book/src/quickstart.md:42` → `0.4.1` ✓
- `Cargo.toml` crate version → `0.4.1` ✓
- `CHANGELOG.md` includes `[0.4.1]` entry ✓

Impact: The highest-friction onboarding mismatch from the initial report has been addressed.

## Risk Register

| ID | Risk | Area | Severity | Likelihood | Notes |
| --- | --- | --- | --- | --- | --- |
| R2 | Monolithic `client.rs` slows safe evolution | Structure | Medium | High | Confirmed at 5448 LOC + broad method count |
| R3 | Strict clippy friction due to generated code | Process | Medium | High | Reproducible with current strict command |
| R4 | Public placeholder command modules confuse users | API clarity | Low/Med | Medium | Can be fixed with docs or visibility adjustment |
| R5 | Missing docs on public items in `client.rs` | DX/discoverability | Medium | Medium | 73/120 public items undocumented (39% documented) |
| R6 | Broken doc links/anchors | Docs | Low | Medium | `validation.md` → README anchor mismatch |
| R7 | Filesystem artifact in docs tree | Hygiene | Low | Low | `docs/book/src/https:/docs.rs/` directory |
| R8 | Narrow examples coverage | Onboarding | Medium | High | Single advanced example only |
| R9 | Repeated RPC boilerplate | Maintainability | Medium | High | Repetition compounds monolith risk (R2) |

## Prioritized Action Plan (Report-only)

### P0: Immediate (highest ROI)

1. Fix broken `validation.md` anchor to correctly reference current README heading.
2. Document newly added and existing Tier 1 public methods; target **80%+ rustdoc coverage** for Tier 1 public methods.
3. Define clippy policy for generated files (allowlist/scope strategy) and document it.
4. Clean obvious non-generated clippy findings (`clone_on_copy`, bool assert style in tests).
5. Remove `docs/book/src/https:` filesystem artifact from docs tree.

Expected outcome: Cleaner user navigation, better first-run success, improved CI signal quality, reduced contributor confusion.

### P1: Maintainability upgrades

1. Split `src/client.rs` by functional domains while preserving public API signatures, using the proposed module seams:
   - `client/core.rs`
   - `client/common.rs`
   - `client/items.rs`
   - `client/board.rs`
   - `client/selection.rs`
   - `client/geometry.rs`
   - `client/document.rs`
   - `client/mappers.rs`
2. Group command/response type URL constants near their domain methods.
3. Keep and extend protocol-contract test helpers to reduce repeated literal contract strings.
4. Extract a generic typed RPC dispatch helper to reduce repeated `send_command`/`pack_any`/`response_payload_as_any`/`decode_any` boilerplate.
5. Add 2–3 beginner-friendly examples (e.g., connect+ping, list-nets, simple query).

Expected outcome: Smaller review units, lower regression risk, reduced boilerplate drift, faster onboarding.

### P2: API clarity and polish

1. Clarify intent of public `commands::*` modules (document as placeholders or reduce visibility until substantive).
2. Add a concise versioning model section (crate version vs proto pin vs tested KiCad runtime).
3. Add one focused quickstart snippet showing `get_board_layer_name` usage.
4. Add an explicit README **Prerequisites** section describing KiCad runtime requirements.
5. Improve rustdoc coverage to 80%+ for Tier 1 API surface.
6. Audit and fix all cross-document links between mdBook and README.

Expected outcome: Reduced user misinterpretation and smoother docs-driven adoption.

## Recommended Documentation Policy (Three Tiers)

Given the current API breadth, a tiered documentation policy is the best balance between usability and maintenance.

### Tier definitions

1. Tier 1 (primary client-facing API)
   - Examples: `KiCadClient`, `KiCadClientBlocking`, core typed models used by normal consumers.
   - Policy: Fully documented (method docs + parameter behavior + return semantics + error notes where relevant).
   - Goal: New users should succeed from rustdoc + README/book without reading internals.

2. Tier 2 (advanced public API)
   - Examples: advanced helper modules/surfaces intended for power users.
   - Policy: Public but intentionally light docs.
   - Minimum docs: one module-level explanation describing intended audience and "prefer Tier 1 first" guidance.

3. Tier 3 (low-level/raw plumbing)
   - Examples: transport/protobuf-oriented internals exposed for specialized integration.
   - Policy: Public for escape hatches, minimal docs, and clearly labeled as advanced.
   - Optional: hide from rustdoc navigation via `#[doc(hidden)]` for especially noisy surfaces while keeping symbols public.

### Lint and docs strategy

1. Keep strict documentation quality for Tier 1.
   - Continue `#![warn(missing_docs)]` and drive Tier 1 toward zero missing-doc warnings.

2. Scope missing-doc relaxations only to Tier 2/3 module boundaries.
   - Prefer local `#[allow(missing_docs)]` on specific modules over crate-wide suppression.
   - This preserves signal for user-facing APIs while reducing low-value warning noise.

3. Add clear module-level labels for advanced surfaces.
   - Recommended wording pattern:
     - "Advanced API surface"
     - "May change more frequently than Tier 1"
     - "Prefer Tier 1 APIs unless you need lower-level control"

### Why this approach is recommended

- It aligns with user needs: newcomers get high-quality guidance where it matters.
- It avoids documentation bloat in internal/advanced layers.
- It keeps compiler/rustdoc warnings actionable instead of overwhelming.
- It preserves public extensibility without forcing exhaustive docs for every low-level symbol.

## Suggested Acceptance Criteria for Follow-up Work

### For documentation consistency and integrity

- README/book cross-links resolve correctly.
- No malformed artifact directories remain under docs source.
- README includes an explicit prerequisites section for KiCad IPC runtime conditions.
- Tier 1 public API rustdoc coverage reaches 80%+.

### For lint/process hygiene

- `cargo clippy --all-targets --all-features -- -D warnings` is either:
  - fully green, or
  - intentionally scoped with documented generated-code exemptions.

### For architecture improvements

- `client.rs` split into coherent domain modules.
- Repeated RPC boilerplate consolidated into typed helper(s) where practical.
- No public API breakage in function names/signatures.
- Existing parity and protocol tests remain green.

## Detailed Notes and Evidence Pointers

- Core API and layering: `src/lib.rs`
- Main API implementation and constants: `src/client.rs`
- Blocking parity guard test: `src/blocking.rs:586`
- Transport bridge implementation: `src/transport.rs`
- Error boundary taxonomy: `src/error.rs`
- Generated proto lint hotspot examples:
  - `src/proto/generated/kiapi.common.rs:56`
  - `src/proto/generated/kiapi.common.commands.rs:424`
- Documentation issues:
  - `docs/book/src/validation.md:21`
  - `README.md:118`
  - `docs/book/src/https:/docs.rs/`
- Rustdoc coverage evidence samples (undocumented public methods):
  - `src/client.rs:359`
  - `src/client.rs:403`
  - `src/client.rs:426`
  - `src/client.rs:472`
  - `src/client.rs:861`
  - `src/client.rs:908`

## Final Assessment

The library is in a strong position functionally and continues to show disciplined protocol-aware testing. The most impactful near-term improvements are documentation integrity (anchors/prerequisites/examples), rustdoc depth for Tier 1 APIs, and lint policy clarity around generated code. Medium-term effort should focus on modularizing `src/client.rs` and reducing repeated RPC boilerplate to sustain quality as API breadth grows.

## Report Revision History

- 2026-03-29: Initial report generated (partial)
- 2026-03-29: Comprehensive completion pass — verified all metrics against codebase, expanded anti-pattern scan (3→6 findings + verified clean signals), added transport/feature-flag/model architecture details, identified 5 new documentation issues, corrected resolved DR-1, expanded risk register (5→9 entries), and updated action plan
