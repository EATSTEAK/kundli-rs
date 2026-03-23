# Context

`plans/initial-plan.md`는 `BirthInput -> normalize -> astro -> derive -> KundliResult` 형태의 최종 API를 제안하지만, 현재 구현은 이미 `AstroRequest -> AstroResult -> derive::*` 구조로 대부분 완성되어 있다. 실제로 `src/kundli/astro/request.rs`, `src/kundli/astro/engine.rs`, `src/kundli/derive/d1.rs`, `src/kundli/derive/d9.rs`, `src/kundli/derive/dasha.rs`가 핵심 계산을 제공하고 있고, 비어 있는 부분은 최종 조립층과 결과 래퍼 모델이다.

사용자 결정에 따라 이번 단계의 최종 공개 입력은 `AstroRequest + KundliConfig`로 확정한다. 따라서 이번 변경의 목표는 기존 astro/derive 구현을 재사용해 한 번의 호출로 최종 쿤들리 결과를 반환하는 공개 API를 추가하는 것이다.

## Recommended approach

`AstroRequest + KundliConfig`를 입력으로 받는 최종 orchestration API를 구현한다.

예상 공개 API:

```rust
pub fn calculate_kundli(
    request: AstroRequest,
    config: KundliConfig,
) -> Result<KundliResult, KundliError>
```

내부 재사용과 테스트를 위해 engine 주입 버전도 함께 둔다.

```rust
pub fn calculate_kundli_with_engine<E: AstroEngine>(
    engine: &E,
    request: &AstroRequest,
    config: &KundliConfig,
) -> Result<KundliResult, KundliError>
```

## Critical files to modify

- `src/kundli/mod.rs`
  - `calculate` 모듈 공개
- `src/kundli/calculate.rs`
  - 최종 orchestration 구현
- `src/kundli/model.rs`
  - `KundliResult`, `CalculationMeta`, `CalculationWarning` 추가
- `src/lib.rs`
  - 필요 시 최상위 re-export 정리
- `tests/astro_smoke.rs`
  - 기존 manual pipeline 검증을 final API 기준으로 확장
- 필요 시 새 final API 테스트 파일 추가

## Existing functions and types to reuse

- `src/kundli/astro/request.rs`
  - `AstroRequest`, `validate()`
- `src/kundli/astro/engine.rs`
  - `AstroEngine`, `SwissEphAstroEngine`, `SwissEphConfig`
- `src/kundli/config.rs`
  - `KundliConfig` (`include_d9`, `include_dasha` 분기 사용)
- `src/kundli/derive/d1.rs`
  - `derive_d1_chart`
  - 필요 시 `derive_lagna`, `derive_planet_placements`, `derive_houses`
- `src/kundli/derive/d9.rs`
  - `derive_d9_chart`
- `src/kundli/derive/dasha.rs`
  - `derive_vimshottari_dasha`
- `src/kundli/derive/input.rs`
  - derive 공통 전처리 패턴 기준점 (`KundliDeriveInput::from_astro`, `to_navamsa`)
- `src/kundli/error.rs`
  - `KundliError`, `DeriveError`
- `src/kundli/model.rs`
  - 기존 `D1Chart`, `D9Chart`, `VimshottariDasha`, `LagnaResult`, `PlanetPlacement`, `HouseResult`

## Implementation outline

1. `src/kundli/model.rs` 확장
   - `KundliResult` 추가
   - `CalculationMeta` 추가
   - `CalculationWarning` 추가
   - 최종 결과는 아래 형태로 조립
     - `meta`
     - `lagna`
     - `planets`
     - `houses`
     - `d1`
     - `d9: Option<D9Chart>`
     - `dasha: Option<VimshottariDasha>`
     - `warnings`

2. `src/kundli/calculate.rs` 추가
   - `SwissEphAstroEngine::new(SwissEphConfig::new())`를 사용하는 기본 진입점 `calculate_kundli(request, config)` 구현
   - 테스트 친화적인 `calculate_kundli_with_engine(engine, request, config)` 구현
   - `engine.calculate(request)`로 `AstroResult` 생성
   - `derive_d1_chart`로 D1 조립
   - `lagna`, `planets`, `houses`는 D1에서 재사용해 중복 계산 제거
   - `config.include_d9`가 true면 `derive_d9_chart` 호출
   - `config.include_dasha`가 true면 `derive_vimshottari_dasha` 호출
   - astro/meta/config snapshot을 이용해 `KundliResult` 반환

3. 공개 API 연결
   - `src/kundli/mod.rs`에서 `pub mod calculate;` 추가
   - 필요 시 `src/lib.rs`에서 최종 API를 crate root로 re-export

4. 테스트 보강
   - final API smoke test 추가
   - `include_d9 = false`, `include_dasha = false` 분기 테스트 추가
   - invalid request/error propagation test 추가
   - 필요 시 manual pipeline 결과와 final API 결과 직접 비교 테스트 추가

## Verification

- `cargo test`
- 기존 `tests/astro_smoke.rs` fixture를 이용해 final API가 manual pipeline과 동일한 shape를 반환하는지 검증
- `include_d9 = true`, `include_dasha = true`에서 `d1` 필수 + `d9`/`dasha` optional 필드가 모두 채워지는지 검증
- `include_d9 = false`, `include_dasha = false`에서 `d9 == None`, `dasha == None`인지 검증
- invalid coordinates / empty bodies 요청에서 `KundliError::Astro`가 전파되는지 검증
- sidereal/house-system 제약에 맞지 않는 D9 요청에서 현재 derive 정책과 일관된 실패 동작을 유지하는지 검증
- 필요 시 final API test에서 manual pipeline (`engine.calculate` + `derive_*`) 결과와 `calculate_kundli` 결과를 직접 비교
