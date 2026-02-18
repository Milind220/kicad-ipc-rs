# KiCad IPC Protocol Buffer Definitions

These `.proto` files are copied from the official KiCad source tree:

- https://gitlab.com/kicad/code/kicad/-/tree/master/api/proto

Rules for this repository:

1. Keep copied files verbatim when possible.
2. Preserve original upstream copyright/license headers.
3. Use these files for build-time code generation.
4. Do not commit generated Rust protobuf output.

Crate licensing:

- Hand-written Rust code in this repository is MIT licensed.
- Upstream `.proto` files retain their original upstream licensing/headers.
