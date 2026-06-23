# Visor Rust SDK Build Workflow

This file is a compact workflow guide for standing up the Rust SDK port. It is
not the full implementation spec. Use `docs/design/visor-rust-impl.md` as the
canonical technical handoff, and use this file to keep the build process small,
phased, and test-driven.

## Goal

Build a Rust SDK for the Visor Public API that preserves the behavior of the
existing Python SDK while presenting an idiomatic Rust API.

Primary reference files:

- `docs/design/visor-rust-impl.md` - Rust implementation handoff and API shape
- `C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\` - source of truth for current Python SDK behavior
- `C:\Users\mille\PycharmProjects\visor-api-sdk\tests\` - Python SDK behavioral expectations and integration patterns
- `docs/decisions/backlog.md` - known follow-up work and edge-case notes


The Rust SDK is a behavioral port of the Python SDK.

Reference implementation:

- Local path: `C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\`
- Key files:
  - `C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\_client.py`
  - `C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\_transport.py`
  - `C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\models\`

Use this code to confirm endpoint behavior, response envelope handling, filter serialization, validation rules, and error mapping. Do not import from or depend on the Python SDK at runtime.

External Rust references:

- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html
- Cargo SemVer compatibility: https://doc.rust-lang.org/cargo/reference/semver.html
- rustdoc documentation tests: https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html

## Pre-Flight Cleanup

Before writing crate code, make sure the implementation handoff is internally
consistent:

- Filters use explicit composition only: `ListingsFilter.base`,
  `FacetsFilter.base`, and `DealerFilter` where applicable.
- `FacetsFilter::new(facets)` is infallible; validation happens in
  `validate()`, and client methods call validation before sending requests.
- `get_usage` returns the full usage envelope:
  `{ data: [...], totals: {...}, meta: {...} }`.
- Any `PageMeta` reference is either defined explicitly or replaced with
  `HashMap<String, serde_json::Value>`.
- Non-2xx malformed error responses fall back to `unknown_error` plus raw body
  text rather than failing while parsing the error.
- Constructor behavior is decided before implementation. Prefer `Result` for
  runtime configuration failures unless the design intentionally documents a
  panic as a programmer error.

## Build Contract

Do not implement broad areas all at once. Work in small slices, and run the
quality gates after each slice.

Required gates:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo doc --no-deps
```

Optional gates before release:

```bash
cargo test --doc
cargo test --all-features
cargo tree -e features
```

To run phase-contract tests (written ahead of their implementation phase):

```bash
# Linux/macOS
RUSTFLAGS='--cfg phase_contracts' cargo test

# Windows PowerShell
$env:RUSTFLAGS='--cfg phase_contracts'; cargo test
```

These tests intentionally fail until the relevant phase lands. They use a
custom `cfg` flag rather than a Cargo feature so that `cargo test --all-features`
remains a healthy check at all times.

Do not run live API tests unless `VISOR_API_KEY` is present. Mocked tests should
be the default.

## Phase 1: Crate Skeleton

Create the Rust crate and module tree first, without endpoint logic.

Expected structure:

```text
src/
  lib.rs
  error.rs
  transport/
  client/
  pagination/
  models/
tests/
  common/
```

Initial tasks:

- Define crate metadata in `Cargo.toml`.
- Choose dependency versions at crate creation time rather than copying stale
  versions from a design document.
- Decide whether the sync client ships unconditionally or behind a Cargo
  feature such as `blocking`.
- Re-export the intended public API from `lib.rs`.
- Keep private implementation modules private unless callers need them.

## Phase 2: Contract Tests First

Write tests that lock behavior from the Python SDK before filling in all
implementation details.

Start with:

- filter serialization golden tests
- validation tests for invalid filters
- serde/defaulting tests for sparse API responses
- error dispatch tests for 400, 401, 403, 404, 429, and 5xx
- malformed success-body and malformed error-body tests
- base URL composition tests
- usage response envelope tests
- public re-export compile tests

These tests are the main protection against building an attractive Rust API
that accidentally changes SDK behavior.

## Phase 3: Transport

Implement transport before client convenience methods.

Requirements:

- Add bearer auth on every request.
- Use the configured base URL without hard-coding the production URL in tests.
- Parse successful responses from bytes so `serde_json::Error` can map to
  `VisorError::InvalidResponse`.
- Parse non-2xx error bodies according to the Python transport behavior.
- Preserve `Retry-After` handling for rate limits.
- Keep async and sync transports behaviorally identical.

## Phase 4: Models And Filters

Port response models and filters in a focused pass.

Requirements:

- Use `#[serde(default)]` for response fields that the API may omit.
- Rename reserved JSON fields, such as dealer `type`, with serde attributes.
- Use typed enums for constrained API values.
- Manually serialize query params with `Vec<(String, String)>`.
- Preserve separator quirks from the Python SDK.
- Do not use `HashMap` for query params; test ordering should remain stable.

## Phase 5: Clients

Implement public client methods after transport and models are stable.

Required methods:

- `filter_listings`
- `get_listing`
- `lookup_vin`
- `filter_facets`
- `search_dealers`
- `get_dealer`
- `dealer_inventory`
- `get_usage`

Rules:

- Validate filters before sending requests.
- Return full page/envelope structs where the Python SDK does.
- Unwrap top-level `data` only for single-resource methods where the Python SDK
  does.
- Keep async and sync method behavior aligned.

## Phase 6: Pagination

Add pagination helpers after the underlying list methods are tested.

Expected API:

- Async helpers return streams.
- Sync helpers return iterators.
- Helpers own or clone their working filter copy so callers are not mutated.
- Stop when the API returns an empty page or pagination metadata indicates there
  are no more results.

## Phase 7: Documentation And Examples

Add examples only after the API has settled.

Include:

- crate-level docs with a minimal quickstart
- docs for auth/configuration
- docs for async and sync clients
- docs for filters and pagination
- examples that compile without a live API key where possible
- ignored live examples where an API key is required

Prefer rustdoc examples that compile as doctests when possible.

## API Review Checklist

Before calling the first Rust version complete, review:

- Public names follow Rust naming conventions.
- Public constructors use `Result` for recoverable runtime errors.
- `VisorError` is suitable for downstream matching; consider
  `#[non_exhaustive]`.
- Public enums that may grow are protected against future API expansion.
- Public structs expose fields intentionally.
- Cargo features are additive and documented.
- Examples show realistic SDK usage without requiring users to understand
  internals.
- All tests pass under the default feature set and all intended feature sets.

## Done Criteria

The first implementation is ready for deeper review when:

- The crate builds from a clean checkout.
- All mocked tests pass.
- `cargo fmt --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo doc --no-deps` passes.
- The public API matches `docs/design/visor-rust-impl.md` or any differences are
  intentionally documented.
- Live tests are either passing with `VISOR_API_KEY` or clearly marked ignored.
