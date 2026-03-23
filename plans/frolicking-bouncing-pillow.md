# Context

- 현재 파이프라인은 `src/kundli/astro/engine.rs:40`의 `SwissEphAstroEngine::calculate()`가 `src/kundli/astro/result.rs:21`의 `AstroResult`를 반환하고, 각 차트/도메인 derive가 이 `AstroResult`를 직접 다시 해석하는 구조다.
- 그 결과 동일한 raw longitude에 대해 아래 계산이 여러 번 반복된다.
  - longitude normalization
  - sign / degrees-in-sign
  - nakshatra / pada
  - Moon progress ratio
  - D1/D9별 planet 순회
- 중복 해석 지점은 주로 다음에 있다.
  - `src/kundli/derive/d1.rs:12`
  - `src/kundli/derive/d1.rs:22`
  - `src/kundli/derive/d1.rs:49`
  - `src/kundli/derive/d9.rs:11`
  - `src/kundli/derive/d9.rs:49`
  - `src/kundli/derive/dasha.rs:11`
- 목표는 `AstroResult`를 한 번만 Kundli 해석용 중간 표현으로 변환한 뒤, D1 / D9 / Dasha가 그 결과를 재사용하도록 만들어 중복 해석을 줄이는 것이다.
- astro 레이어는 그대로 유지하고, 현재 public derive API의 동작도 가능한 한 깨지지 않게 유지하는 것이 좋다.

# Recommended approach

## 1. derive 내부에 AstroResult -> Kundli intermediate 변환 레이어 추가

수정/추가 대상:
- `src/kundli/derive/mod.rs`
- `src/kundli/derive/input.rs` (new)

구현 방향:
- 새 모듈에 `AstroResult`를 해석해 재사용 가능한 중간 타입을 둔다.
- 이 타입은 chart 결과가 아니라, chart 조립 전에 공통으로 필요한 해석 결과만 담는다.
- intermediate 타입 이름은 `KundliDeriveInput`으로 한다.
- 최소 포함 정보:
  - meta snapshot (`jd_ut`, `zodiac`, `ayanamsha`, `ayanamsha_value`)
  - ascendant snapshot (normalized longitude, sign, degrees-in-sign)
  - body snapshots in original order
    - `body`
    - normalized longitude
    - sign
    - degrees-in-sign
    - `NakshatraPlacement`
    - Moon progress 재사용용 ratio
    - retrograde flag
  - `house_cusps` raw pass-through
- 중요한 제약:
  - 여기서 house cusp count를 전역 검증하지 않는다.
  - D1 non-WholeSign에서만 기존처럼 cusp 검증을 유지한다.
  - astro layer 타입은 변경하지 않는다.

## 2. D1은 새 intermediate를 소비하도록 내부 구현만 교체

수정 대상:
- `src/kundli/derive/d1.rs`
- `tests/derive_d1.rs`

구현 방향:
- 아래 prepared-input 기반 내부 함수를 추가한다.
  - `derive_lagna_from_input(...)`
  - `derive_planet_placements_from_input(...)`
  - `derive_houses_from_input(...)`
  - `derive_d1_chart_from_input(...)`
- 기존 public 함수 시그니처는 유지하고, 내부에서 intermediate 변환 후 위 함수를 호출하는 wrapper로 바꾼다.
- D1이 계속 담당할 일:
  - `config.house_system`에 따른 house derivation
  - D1 house result assembly
- D1이 intermediate에서 바로 재사용할 값:
  - ascendant sign / degrees
  - body sign / degrees / nakshatra / retrograde

## 3. D9는 intermediate의 navamsa 변환본을 소비하도록 재구성

수정 대상:
- `src/kundli/derive/d9.rs`
- `tests/derive_d9.rs`

구현 방향:
- base intermediate에 대해 `to_navamsa()` 같은 변환을 추가하거나, 동등한 internal helper를 `input.rs`에 둔다.
- D9는 transformed intermediate를 받아 아래만 조립한다.
  - D9 lagna
  - D9 planet placements
  - D9 WholeSign houses
- 유지할 계약:
  - sidereal-only guard 유지
  - `HouseSystem::WholeSign`만 허용하는 contract 유지
- 제거할 중복:
  - D9 내부에서 raw `AstroResult` body를 다시 normalize/sign/nakshatra로 해석하던 흐름

## 4. Dasha는 Moon을 intermediate에서 바로 사용하도록 변경

수정 대상:
- `src/kundli/derive/dasha.rs`
- `tests/derive_dasha.rs`

구현 방향:
- intermediate에 body lookup helper를 둔다. 예: `body(AstroBody) -> Option<&PreparedBody>`
- dasha는 더 이상 raw Moon longitude를 다시 nakshatra/progress로 해석하지 않고, intermediate의 Moon snapshot을 재사용한다.
- 유지할 계약:
  - sidereal-only guard
  - MissingMoon error
  - existing mahadasha sequence logic
- 제거할 중복:
  - Moon lookup + nakshatra placement + progress ratio 재계산

## 5. 공개 API는 일단 유지하고, 새 intermediate는 내부 레이어로 먼저 도입

수정 대상:
- `src/kundli/derive/mod.rs`
- 필요 시 `docs/derive-implementation-overview.md`
- `tests/astro_smoke.rs`

구현 방향:
- 현재 public surface는 그대로 유지한다.
  - `derive_d1_chart(&AstroResult, &KundliConfig)`
  - `derive_d9_chart(&AstroResult, &KundliConfig)`
  - `derive_vimshottari_dasha(&AstroResult)`
- wrapper 내부에서만 intermediate를 생성한다.
- smoke test는 public API 그대로 호출하되, 내부 구현이 intermediate로 바뀌어도 end-to-end가 유지되는지 확인한다.
- 문서는 파이프라인을 아래처럼 갱신한다.
  - `AstroResult -> Prepared Kundli input -> D1 / D9 / Dasha`

## 6. 권장 구현 순서

1. `derive/input.rs` 추가
2. intermediate constructor + body lookup + navamsa transform 구현
3. D1 internal refactor
4. Dasha internal refactor
5. D9 internal refactor
6. 기존 integration/smoke tests 갱신
7. 문서 갱신
8. 전체 `cargo test` / `cargo clippy` 검증

# Critical files to modify

- `src/kundli/derive/mod.rs`
- `src/kundli/derive/input.rs` (new)
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`
- `src/kundli/derive/dasha.rs`
- `tests/derive_d1.rs`
- `tests/derive_d9.rs`
- `tests/derive_dasha.rs`
- `tests/astro_smoke.rs`
- `docs/derive-implementation-overview.md`

# Existing functions and utilities to reuse

- `src/kundli/derive/sign.rs:18`의 `normalize_longitude`
- `src/kundli/derive/sign.rs:35`의 `sign_from_longitude`
- `src/kundli/derive/sign.rs:46`의 `degrees_in_sign`
- `src/kundli/derive/nakshatra.rs:60`의 `nakshatra_placement_from_longitude`
- `src/kundli/derive/nakshatra.rs:82`의 `moon_progress_ratio`
- `src/kundli/derive/nakshatra.rs:93`의 `dasha_lord_for_nakshatra`
- `src/kundli/derive/house.rs:23`의 `derive_house`
- `src/kundli/astro/result.rs:21`의 `AstroResult`
- `src/kundli/astro/result.rs:13`의 `AstroMeta`

# Edge cases to preserve

- `AstroResult` body order는 유지한다.
- longitude validation/normalization semantics는 기존 helper와 동일해야 한다.
- D1만 `config.house_system`에 따라 cusp validation을 수행한다.
- D9는 계속 sidereal + WholeSign만 허용한다.
- Dasha는 계속 sidereal만 허용하고 Moon이 없으면 실패한다.
- astro layer 타입/계산 로직은 변경하지 않는다.
- public derive wrapper의 외부 동작은 가능한 한 그대로 유지한다.

# Verification

## Unit tests
- new intermediate layer
  - ascendant/body normalization
  - sign/degrees/nakshatra/progress precompute
  - body order preservation
  - Moon lookup helper
  - navamsa transform correctness

## Integration tests
- `tests/derive_d1.rs`
  - wrapper path 결과가 기존 의미론과 동일한지 검증
- `tests/derive_d9.rs`
  - sidereal / WholeSign guard 유지
  - transformed intermediate 기반 결과 동일성 검증
- `tests/derive_dasha.rs`
  - MissingMoon / sidereal guard / dasha sequence 유지
- `tests/astro_smoke.rs`
  - public API end-to-end path 유지

## Commands
- `cargo test --lib`
- `cargo test --test astro_smoke --test derive_d1 --test derive_d9 --test derive_dasha`
- `cargo clippy --all-targets --all-features -- -D warnings`
