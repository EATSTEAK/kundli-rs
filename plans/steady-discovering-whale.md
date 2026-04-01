## Context

이번 변경은 multi-chart API를 더 안전한 public contract로 정리하기 위한 것이다. 현재 `KundliConfig::charts`가 비어 있어도 `calculate_kundli_with_engine()`가 정상처럼 빈 `KundliResult`를 반환하고, 중복 chart 요청도 `BTreeMap` insert 과정에서 암묵적으로 dedupe된다. 또한 `AstroResult::body()`는 이미 canonical order가 보장된 배열을 선형 탐색하고 있다. 목표는 빈 chart 요청을 명시적으로 거부하고, chart dedupe를 config validation 단계에서 처리하며, body 접근을 canonical order 기반 O(1) lookup으로 맞추는 것이다.

## Recommended approach

1. `src/kundli/error.rs`
   - `KundliError`에 chart selection validation용 새 에러 variant를 추가한다.
   - 필요하면 세부 사유를 나타내는 enum(예: empty selection, duplicate chart)을 함께 도입한다.
   - `Display`와 `source()`를 업데이트해 top-level API 에러로 노출한다.

2. `src/kundli/config.rs`
   - `KundliConfig`에 명시적 validation entrypoint를 추가한다.
   - validation에서 다음 정책을 적용한다.
     - `charts.is_empty()`면 에러 반환
     - duplicate `KnownChart`는 config 단계에서 dedupe
   - dedupe는 validation 안에서 수행되도록 하고, `with_charts()`는 단순 setter로 유지한다.
   - 필요하면 `validate`를 `&mut self` 또는 정규화 결과를 반환하는 형태로 구성해 dedupe된 `charts`를 확정한다.

3. `src/kundli/calculate.rs`
   - `calculate_kundli_with_engine()`에서 astro engine 호출 전에 config validation을 수행한다.
   - `validate_request_matches_config()`는 현재처럼 request/config duplicated setting 비교만 담당하게 유지한다.
   - 기존의 암묵적 dedupe(`BTreeMap` overwrite)에 기대지 않도록 흐름을 정리한다.

4. `src/kundli/astro/request.rs`
   - `AstroBody`에 canonical slot helper(예: `index() -> usize`)를 추가한다.
   - `AstroBody::ALL` 순서와 body 배열 인덱스의 대응 관계를 여기서 단일 source of truth로 만든다.

5. `src/kundli/astro/result.rs`
   - `AstroResult::body()`를 선형 탐색 대신 `AstroBody::index()` 기반 직접 인덱싱으로 변경한다.
   - canonical order invariant를 문서/구현 모두에서 더 명확히 드러낸다.

6. `src/kundli/derive/pipeline/reference.rs`
   - `MoonReference::apply()`의 `iter().find(...)` Moon lookup을 canonical index 기반 접근으로 바꾼다.
   - dasha 파이프라인의 `astro.body(AstroBody::Moon)` 사용 방식과 일관되게 맞춘다.

## Critical existing code to reuse

- `src/kundli/calculate.rs`
  - `validate_request_matches_config()` — request/config duplicated setting 검증 패턴 재사용
- `src/kundli/astro/request.rs`
  - `AstroBody::ALL` — canonical body ordering source
- `src/kundli/derive/dasha.rs`
  - `astro.body(AstroBody::Moon)` — 개선된 accessor의 기존 사용처

## Tests to update/add

- `src/kundli/calculate.rs`
  - empty charts가 더 이상 성공하지 않고 validation error를 반환하는 테스트로 교체
  - duplicate chart input이 config validation 이후 dedupe된 결과로 계산되는지 테스트 추가/수정
  - request/config mismatch 테스트는 유지
- `src/kundli/astro/result.rs` 또는 인접 테스트
  - `AstroResult::body(AstroBody::Moon)`가 canonical slot을 직접 반환하는지 검증
- `src/kundli/derive/pipeline/reference.rs` 관련 테스트
  - Moon reference 경로가 기존과 동일하게 동작하는지 회귀 확인

## Verification

1. `cargo test`
2. 필요 시 touched module 중심의 테스트 재확인
3. `cargo fmt`
4. `cargo fmt --all -- --check`로 포맷 검증

## Files expected to change

- `src/kundli/error.rs`
- `src/kundli/config.rs`
- `src/kundli/calculate.rs`
- `src/kundli/astro/request.rs`
- `src/kundli/astro/result.rs`
- `src/kundli/derive/pipeline/reference.rs`
- 관련 테스트 파일