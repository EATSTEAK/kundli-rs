## Context
사용자는 현재 `KundliResult`의 고정 필드(`d1`, `d9`, `dasha`) 중심 응답도 더 일반화해서, 요청한 chart 조합을 일관된 multi-chart 구조로 받고 싶어 한다. 지금 구조는 selection은 `KnownChart` enum 기반으로 일반화됐지만, 응답은 여전히 chart 종류별 고정 필드에 묶여 있어 새 chart 타입이 늘어날 때마다 `KundliResult`를 계속 수정해야 한다. 이번 변경의 목표는 설정과 orchestration뿐 아니라 **응답 모델까지 chart-keyed collection**으로 정리해, 미래 chart 타입 추가 시 고정 필드 확장 없이 같은 패턴으로 수용하게 만드는 것이다.

## Recommended approach
1. `KundliResult`를 chart collection 기반으로 재구성한다.
   - 수정 파일: `src/kundli/model.rs`
   - `d1`, `d9`, `dasha` 고정 optional 필드를 제거하고, 예를 들어 `charts: BTreeMap<KnownChart, ChartLayer>` 같은 일반화된 컬렉션 필드를 도입한다.
   - `ChartLayer`는 서로 다른 payload를 담는 공개 enum으로 정의한다. 권장 shape:
     - `ChartLayer::D1(D1Chart)`
     - `ChartLayer::D9(D9Chart)`
     - `ChartLayer::VimshottariDasha(VimshottariDasha)`
   - `KnownChart`는 map key로 안정적으로 쓰기 위해 `Ord + PartialOrd`까지 derive한다.
   - `CalculationMeta`, `warnings`는 top-level에 그대로 둔다.

2. 소비자 ergonomics를 enum-accessor 메서드로 보완한다.
   - 수정 파일: `src/kundli/model.rs`
   - `KundliResult::chart(kind) -> Option<&ChartLayer>` 같은 generic accessor와 함께,
     `ChartLayer::as_d1()`, `as_d9()`, `as_vimshottari_dasha()` 같은 typed accessor를 추가한다.
   - 이렇게 하면 응답 shape는 일반화하면서도 호출부는 `result.chart(KnownChart::D1).and_then(ChartLayer::as_d1)`처럼 안전하게 읽을 수 있다.

3. `calculate_kundli_with_engine`를 map assembly 방식으로 바꾼다.
   - 수정 파일: `src/kundli/calculate.rs`
   - astro 계산 후 `config.charts`를 순회하면서 해당 `KnownChart`에 맞는 derive 함수를 호출하고, 결과를 `ChartLayer`로 감싸 `charts` 컬렉션에 넣는다.
   - 현재처럼 `contains()`로 개별 필드를 채우지 말고, chart enum dispatch를 한 군데로 모은다.
   - 동일 chart가 중복 요청되어도 최종 응답에는 한 번만 담기도록 `BTreeMap` 삽입 semantics를 사용한다.
   - 기존 request/config 일치 검증과 D9 제약 검증은 그대로 재사용한다.

4. 문서와 테스트를 새 응답 계약에 맞게 갱신한다.
   - 수정 파일: `src/lib.rs`, `docs/derive-implementation-overview.md`, `tests/astro_smoke.rs`, 필요 시 `src/kundli/calculate.rs` 내부 테스트
   - quick start는 고정 필드 접근 대신 typed accessor 예시로 바꾼다.
   - smoke test는 manual derive 결과를 `ChartLayer`로 감싼 map entry와 비교하도록 바꾼다.
   - "D1만 선택", "D1+D9+dasha 선택", "중복 chart 요청" 케이스를 추가해 collection contract를 고정한다.

## Existing code to reuse
- `src/kundli/config.rs`
  - `KnownChart`
  - `KundliConfig::with_charts`
- `src/kundli/derive/d1.rs`
  - `derive_d1_chart_result`
- `src/kundli/derive/d9.rs`
  - `derive_d9_chart_result`
- `src/kundli/derive/dasha.rs`
  - `derive_vimshottari_dasha`
- `src/kundli/calculate.rs`
  - `validate_request_matches_config`
  - `build_calculation_meta`
- `tests/astro_smoke.rs`
  - manual pipeline vs final API 비교 패턴

## Critical files
- `src/kundli/model.rs`
- `src/kundli/calculate.rs`
- `src/kundli/config.rs`
- `src/lib.rs`
- `docs/derive-implementation-overview.md`
- `tests/astro_smoke.rs`
- 필요 시 `src/kundli/calculate.rs` 내부 테스트

## Verification
1. 모델 테스트
   - `ChartLayer` accessor가 올바른 variant에서만 값을 반환하는지 검증
   - `KundliResult::chart(KnownChart::...)`가 기대한 entry를 찾는지 검증
2. calculate 테스트
   - 요청한 chart만 map에 들어가는지 검증
   - 동일 chart를 중복 요청해도 결과 entry가 하나만 생기는지 검증
   - D9 unsupported 조합이 기존과 동일하게 에러를 내는지 검증
3. smoke/integration 테스트
   - final API의 `charts` 컬렉션이 manual derive 결과와 일치하는지 검증
   - D1-only / full-selection 케이스 검증
4. 실행 검증
   - `cargo test`