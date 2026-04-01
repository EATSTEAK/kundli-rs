## Context
사용자는 브랜치 리뷰에서 나온 세 가지 개선 제안을 모두 실제 코드에 반영하길 원한다. 대상은 chart kind 의미를 더 명확히 드러내는 것, house helper의 에러 의미를 정확히 맞추는 것, 그리고 chart/house mode 조합 오류가 misleading한 derive 에러로 흘러가지 않게 validation 단계에서 더 구체적으로 막는 것이다. 의도한 결과는 다음과 같다: (1) `Bhava`/`Chalit`의 현재 구현 상태가 테스트와 코드에서 숨지지 않고, (2) `whole_sign_house`의 실패 경로가 정확한 에러를 반환하며, (3) 잘못된 chart selection은 `config.validate()`에서 명확한 selection error로 조기에 거부된다.

## Recommended approach
1. `src/kundli/error.rs`
   - `ChartSelectionError`에 bhava/chalit 계열이 cusp-based house mode를 요구한다는 의미의 variant를 추가한다.
   - `Display`와 `From<ChartSelectionError> for KundliError` 흐름은 기존 패턴을 그대로 유지한다.

2. `src/kundli/config.rs`
   - `KundliConfig::validate()`에서 `ChartKind::Bhava`, `ChartKind::Chalit`, `ChartKind::DivisionalBhava { .. }`가 `HouseMode::CuspBased(_)`가 아닌 경우를 새 `ChartSelectionError`로 조기 거부한다.
   - 기존 `division == 0`, empty selection, sort/dedup 로직은 그대로 재사용한다.
   - 현재 `ChartSpec::bhava()`, `ChartSpec::chalit()`, `ChartSpec::divisional_bhava()`가 기본적으로 `HouseMode::Configured`를 사용하므로, `Configured + non-cusp config`가 validate에서 명확히 실패하도록 만든다.

3. `src/kundli/calculate.rs`
   - `resolve_house_mode()`에서 `Bhava`/`Chalit`/`DivisionalBhava`에 대해 misleading한 `DeriveError::UnsupportedHouseSystem(config.house_system)`를 만들던 분기를 제거하거나, validation 이후 내부 불변식 기반의 `debug_assert!`/`unreachable!` 수준으로 축소한다.
   - `derive_chart_result()` 인근에 `Bhava`/`Chalit`가 현재는 `Rasi`와 동일한 pipeline assembly를 사용한다는 사실을 짧게 명시한다. 핵심은 기능 추가가 아니라 현재 의미를 숨기지 않는 것이다.

4. `src/kundli/derive/pipeline/house.rs`
   - `whole_sign_house()`에서 `HouseNumber::new(...)` 실패 시 `DeriveError::InvalidHouseCusps(NUM_HOUSES)`가 아니라 `DeriveError::InvalidHouseNumber(value)`를 반환하도록 수정한다.
   - `renumber_house()`와 동일한 에러 의미를 유지해 helper 간 일관성을 맞춘다.

5. 테스트 보강
   - `src/kundli/config.rs` tests:
     - `Bhava`가 whole-sign/configured-non-cusp에서 validation 실패하는지
     - `Chalit`가 whole-sign/configured-non-cusp에서 validation 실패하는지
     - `DivisionalBhava`가 `HouseMode::None`뿐 아니라 non-cusp mode에서도 validation 실패하는지
   - `src/kundli/calculate.rs` tests 또는 `tests/astro_smoke.rs`:
     - cusp-based 설정에서 `ChartSpec::rasi()`, `ChartSpec::bhava()`, `ChartSpec::chalit()`가 현재 동일 chart result를 내는 회귀 테스트를 추가한다. 이 테스트는 “현재는 semantic placeholder” 상태를 고정하는 목적이다.
   - 필요 시 `src/kundli/derive/pipeline/house.rs` unit test:
     - whole-sign helper가 산출하는 house number가 1..=12 범위를 유지함을 검증한다.

## Critical files
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/error.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/config.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/calculate.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/house.rs`
- `/Users/koohyomin/Projects/kundli-rs/tests/astro_smoke.rs`

## Reuse
- `src/kundli/config.rs:193` `KundliConfig::validate()`의 기존 selection 검증 패턴
- `src/kundli/calculate.rs:68` `derive_chart_result()`의 chart kind별 pipeline assembly
- `src/kundli/calculate.rs:144` `resolve_house_mode()`의 resolved house mode 계산 뼈대
- `src/kundli/derive/pipeline/house.rs:153` `renumber_house()`의 `InvalidHouseNumber` 매핑 패턴
- `src/kundli/error.rs:18` 이하 수동 `Display`/`Error` 구현 패턴

## Verification
- `cargo test`
- 집중 확인:
  - `cargo test config`
  - `cargo test calculate_with_engine`
  - `cargo test astro_smoke`
  - `cargo test pipeline`
- 확인 포인트:
  - 잘못된 `Bhava`/`Chalit`/`DivisionalBhava` house mode 조합이 `KundliError::ChartSelection(...)`로 조기 실패하는지
  - `Bhava`/`Chalit`가 현재 구현에서 `Rasi`와 동일 결과라는 사실이 테스트로 고정되는지
  - whole-sign house helper 수정 후 전체 회귀 테스트가 그대로 통과하는지
