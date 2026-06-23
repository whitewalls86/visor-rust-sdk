# Cleanup Notes

Small follow-up notes that are worth revisiting but do not need to block the
current phase.

## DealerFilter query field

- `DealerFilter` currently exposes `q: Option<String>`.
- `q` is a common API/query-string convention for free-text search, but it is
  terse for a public Rust SDK field.
- Consider renaming the Rust field to `query: Option<String>` while still
  serializing it as the wire parameter `q` in `DealerFilter::to_params()`.

## Completed filter-design work

The domain types, closed-vocabulary enums, `GeoFilter`, `InventoryModeFilter`,
and unsigned filter bounds discussed in earlier versions of this file were
implemented in Phases 3.5 and 3.6. Their current contracts live in:

- `docs/design/phase-3-5-listings-filter-contract.md`
- `docs/design/phase-3-6-facets-contract.md`

Keep this cleanup file for work that remains deferred rather than duplicating
active implementation requirements.

## Dynamic facet catalog and code generation

Explore an optional utility that discovers the API's current categorical values
by walking down the facet hierarchy:

1. Fetch all observed makes with `facets=make`.
2. For each make, fetch observed models with `facets=model&make=...`.
3. For each make/model pair, fetch observed years with
   `facets=year&make=...&model=...`.
4. Optionally continue into trim, version, body type, powertrain, or other
   categorical facets.

The durable output should be a timestamped catalog snapshot, probably JSON,
rather than generated Rust source as the only artifact. The snapshot should
record the filter context used to build it, such as inventory mode, geography,
and snapshot date.

Possible consumers:

- Runtime discovery helpers for search forms and autocomplete.
- Advisory checks and spelling suggestions for caller-provided values.
- A separate `visor-codegen` CLI or companion crate that turns a pinned snapshot
  into Rust constants or enums for compiler autocomplete.
- Other generators, such as TypeScript values, UI dropdown data, or database
  seed files.

Generated Rust source should be explicit and reproducible:

```text
Visor facets API
      -> visor-codegen catalog
      -> visor-catalog.json
      -> visor-codegen rust
      -> generated Rust source committed by the consuming application
```

Do not fetch live API data from `build.rs`. Network-dependent builds would be
slow, fragile, and non-reproducible. Prefer an explicitly refreshed, committed
snapshot.

Treat discovered values as observed data, not an authoritative allowlist:

- Facet values change with inventory.
- Results depend on the filters used during discovery.
- `facet_value_limit` is capped at 100, so a response may be truncated.
- Rare, historical, aliased, or newly added values may be absent.
- Catalog validation should therefore be opt-in and advisory, not part of core
  `ListingsFilter::validate()`.

Questions to resolve later:

- Generate constants, enums, or both. Constants are less breaking when values
  change; enums provide stronger typing and richer compiler autocomplete.
- How to sanitize values such as `F-150` into stable identifiers and handle
  collisions after normalization.
- How to detect incomplete 100-value facet responses.
- Rate-limit handling, bounded concurrency, progress, retries, and resumable
  catalog generation.
- Whether years should remain explicit sets or be summarized as ranges.
- Whether generated types should model make -> model relationships directly or
  expose independent lookup modules.

### Python SDK follow-up

Think through the equivalent feature for the Python SDK rather than assuming
the Rust code-generation design maps directly.

Potential Python shapes include:

- Runtime facet discovery and cached catalog objects.
- Optional advisory validation and spelling suggestions.
- Generated `Enum` or `Literal` definitions for static type checkers and IDE
  autocomplete.
- Generated `.pyi` stubs or modules from the same language-neutral JSON
  snapshot used by Rust.

The shared JSON catalog should remain language-neutral so Rust and Python can
offer idiomatic interfaces without duplicating the expensive discovery crawl.

## Fallible constructors and panic boundaries

- Current constructors use panics for some setup-time failures:
  `with_config()` asserts that `api_key` is non-empty, and transport
  construction calls `expect()` if `reqwest` client construction fails.
- This is acceptable for early scaffolding, but production-grade SDKs should be
  careful about panics because an SDK panic can crash the caller's application.
- Consider adding fallible constructors such as:

```rust
pub fn try_new(api_key: String) -> Result<Self, VisorError>
pub fn try_with_config(config: ClientConfig) -> Result<Self, VisorError>
```

- These should return a typed error for invalid configuration or client
  construction failures instead of panicking.
- Existing panicking constructors could remain as convenience wrappers, or the
  public API could move fully toward fallible construction.
- Keep the boundary clear: request-time failures already return
  `Result<_, VisorError>` and should continue to do so. Panics should be
  reserved for internal invariant violations or explicitly documented
  convenience methods.
