# API hardening plan

## Context

The task has changed from documentation work to API hardening. The goal now is to implement the previously identified weak points in the public API so the crate behaves more like a production-quality Rust library, while keeping the blast radius proportionate.

From exploration of the current codebase:

- `KundliError::InputConfigMismatch(&'static str)` is centralized in `src/kundli/calculate.rs`, so it is a good target for an immediate typed redesign.
- `Pada::new` and `HouseNumber::new` already exist in `src/kundli/model.rs`, but `HouseNumber(...)` is still constructed directly in `src/kundli/derive/d1.rs`, `src/kundli/derive/house.rs`, and several tests.
- `Pada(...)` direct construction is already minimal.
- `AstroRequest` and `KundliConfig` are used widely as public-field struct literals across `src/`, tests, and crate docs, so making all fields private immediately would be a much broader breaking change.

The intended outcome is to harden invariants and error typing now, while keeping `AstroRequest` and `KundliConfig` on the narrower compatibility-preserving path for this pass.

## Recommended approach

### 1. Replace stringly request/config mismatch errors with a typed design

Introduce a dedicated typed mismatch descriptor in `src/kundli/error.rs` and change `KundliError::InputConfigMismatch(&'static str)` into a typed variant that identifies which duplicated field mismatched.

Mismatch kinds:

- zodiac
- ayanamsha
- house_system
- node_type

Then update `validate_request_matches_config` in `src/kundli/calculate.rs` to return the typed variant while preserving clear `Display` output.

Why first:

- all production construction is centralized in one function
- only one exact test currently depends on the string payload
- docs mention the variant name but not a strict payload contract

### 2. Harden invariant-bearing domain types in `src/kundli/model.rs`

For `Pada` and `HouseNumber`:

- keep checked construction through `new`
- add a stable accessor such as `get()` / `as_u8()` for raw extraction
- migrate all first-party code and tests away from `.0` reads and direct tuple construction
- then make the inner fields private for real invariant enforcement

Priority:

- `Pada` is easy because direct tuple construction is already rare
- `HouseNumber` needs a broader migration because direct `HouseNumber(...)` construction exists in derive code and tests

Files that will need migration for this step:

- `src/kundli/model.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/house.rs`
- `tests/astro_smoke.rs`
- `tests/derive_d1.rs`
- `tests/derive_d9.rs`
- any internal tests still asserting `HouseNumber(...)` or reading `.0`

### 3. Improve the public construction path for `AstroRequest`

Add an additive constructor and/or fluent setters/builder-style helpers in `src/kundli/astro/request.rs` so users no longer need to rely on raw struct literals as the only ergonomic construction path.

Minimum useful improvement:

- a constructor covering the required core input
- fluent methods for zodiac/ayanamsha/house system/node type if needed
- accessors if we later decide to reduce direct field access

Existing logic to reuse:

- `AstroRequest::validate` in `src/kundli/astro/request.rs`

### 4. Improve the public construction path for `KundliConfig`

Add a more guided configuration API in `src/kundli/config.rs`, while preserving the existing defaults.

Implementation direction:

- retain `Default`
- add a constructor and/or builder-style methods
- make enabling D9 and dasha easy without requiring raw field mutation

Existing logic to reuse:

- `Default for KundliConfig` in `src/kundli/config.rs`

### 5. Migrate first-party code, tests, and docs to the hardened APIs

After introducing the new error and construction/accessor APIs:

- update internal derive code to stop constructing `HouseNumber(...)` directly
- update tests to stop reading `planet.house.0`
- update crate docs/examples away from raw struct literals where new constructors/builders are added
- update the mismatch test in `src/kundli/calculate.rs` to assert on typed mismatch semantics instead of a string literal payload

### 6. Scope decision for `AstroRequest` and `KundliConfig`

Use the narrower hardening scope for this pass.

Implementation decision:

- fully harden `Pada` / `HouseNumber`
- replace stringly mismatch errors with a typed design
- add constructor/builder-style APIs for `AstroRequest` / `KundliConfig`
- keep `AstroRequest` and `KundliConfig` public fields for compatibility in this pass

This preserves most of the API-safety benefit with a much smaller blast radius, while still establishing a better public construction path for future tightening.

## Critical files to modify

- `src/kundli/error.rs`
- `src/kundli/calculate.rs`
- `src/kundli/model.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/house.rs`
- `src/kundli/astro/request.rs`
- `src/kundli/config.rs`
- `src/lib.rs`
- `tests/astro_smoke.rs`
- `tests/derive_d1.rs`
- `tests/derive_d9.rs`

## Existing functions and utilities to reuse

- `validate_request_matches_config` in `src/kundli/calculate.rs`
- `AstroRequest::validate` in `src/kundli/astro/request.rs`
- `Pada::new` in `src/kundli/model.rs`
- `HouseNumber::new` in `src/kundli/model.rs`
- `Default for KundliConfig` in `src/kundli/config.rs`

## Verification

1. Run `cargo test`
   - verify all unit and integration tests still pass after the error-type and invariant changes.

2. Run `cargo doc --no-deps`
   - verify docs still build after public API updates.

3. Run `cargo test --doc`
   - verify crate-level and item-level examples compile after constructor/builder updates.

4. Add focused checks for the hardening changes
   - mismatch validation returns the correct typed error
   - `Pada` and `HouseNumber` reject invalid values through checked constructors
   - first-party code no longer depends on tuple-field `.0` access
