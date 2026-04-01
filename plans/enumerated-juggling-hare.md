## Context
사용자는 두 가지를 원한다. 첫째, derive pipeline 관련 공개 문서와 주석을 현재 구현 계약에 맞게 정리하는 것. 둘째, 리뷰에서 지적된 핵심 불일치인 "reference 단계가 파이프라인에 존재하지만 실제 house 계산에는 쓰이지 않는 문제"를 실제 구현으로 바로잡는 것이다. 목표는 `core -> reference -> sign -> house -> materialize` 흐름을 유지하면서, D1/D9의 현재 결과는 유지하고 향후 Moon/Bhava/reference-house 확장이 가능한 형태로 house 단계가 `reference`를 실사용하도록 만드는 것이다.

## Recommended approach
1. 문서/주석을 현재 계약에 맞게 정리한다.
   - `src/kundli/astro/result.rs`
     - `AstroBodyPosition.body` 주석의 "request order" 표현을 제거하고 canonical `AstroBody::ALL` snapshot 계약으로 수정한다.
     - `AstroResult` 주석을 "derivation-ready full snapshot" 중심으로 정리한다.
   - `src/kundli/model.rs`
     - `ChartResult`/`D1Chart`의 planets 주석에서 "requested bodies" 표현을 제거한다.
     - `HouseResult` 주석은 house 번호가 reference-relative assignment 결과이며 cusp/sign은 materialized house seed를 나타낸다는 현재 구현 기준으로 보강한다.
   - `docs/derive-implementation-overview.md`
     - requested bodies/Vec 중심 설명을 canonical full snapshot과 reference-aware house 단계 기준으로 갱신한다.

2. `reference`가 실제로 house 계산에 반영되도록 `src/kundli/derive/pipeline/house.rs`를 수정한다.
   - `SignContext.reference`를 house 단계에서 실제로 사용한다.
   - Whole-sign house:
     - 현재 `input.ascendant.longitude`를 1H 기준으로 쓰는 대신 `input.reference.longitude`가 속한 sign boundary를 1H 시작점으로 사용한다.
     - 기존 `whole_sign_house`와 `sign_start_longitude` 로직은 재사용한다.
     - D1/D9는 계속 `LagnaReference`를 사용하므로 결과는 기존과 동일해야 한다.
   - Cusp-based house:
     - 절대 구간 판정 자체는 기존 `derive_house_from_cusps`를 그대로 사용한다.
     - 다만 `reference.longitude`가 속한 cusp index를 "1번 house"로 재번호화하도록 helper를 추가한다.
     - 각 행성의 판정 결과와 `houses` seed의 번호를 같은 방식으로 renumber한다.
   - 필요 helper 예시:
     - `reference_house_index_from_cusps(reference_longitude, cusps)`
     - `renumber_house(absolute_house, first_house)`

3. `materialize`와 결과 타입 정합성을 점검한다.
   - `src/kundli/derive/pipeline/materialize.rs`
     - renumber된 `HouseSeed.house`와 `HousedPlacement.house`를 그대로 직렬화하면 되는지 확인하고, 필요하면 주석만 정리한다.
   - `src/kundli/model.rs`
     - `HouseResult` 문서가 renumbered house assignment를 설명하도록 맞춘다.

4. 테스트를 회귀 + 신규 케이스로 보강한다.
   - 기존 회귀:
     - `tests/derive_d1.rs`, `tests/derive_d9.rs`, `tests/astro_smoke.rs`가 계속 통과해야 한다.
   - 신규 unit test:
     - `src/kundli/derive/pipeline/mod.rs` 또는 `house.rs` 테스트에 `MoonReference + WholeSignHouseTransform`를 추가해 moon sign이 1H가 되는지 검증한다.
     - cusp-based 기준에서도 같은 cusp 배열에 대해 Lagna reference와 Moon reference가 서로 다른 house 번호 체계를 만들 수 있음을 검증한다.
   - `cargo test`로 전체 회귀를 확인한다.

## Critical files
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/house.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/materialize.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/astro/result.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/model.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/mod.rs`
- `/Users/koohyomin/Projects/kundli-rs/docs/derive-implementation-overview.md`

## Reuse
- `src/kundli/derive/pipeline/house.rs`
  - `sign_start_longitude`
  - `whole_sign_house`
  - `derive_house_from_cusps`
- `src/kundli/derive/pipeline/core.rs`
  - pipeline execution order는 유지
- `src/kundli/derive/pipeline/reference.rs`
  - `ResolvedReference`와 `MoonReference`를 그대로 활용

## Verification
- `cargo test`
- 필요 시 집중 확인:
  - `cargo test derive_d1`
  - `cargo test derive_d9`
  - `cargo test astro_smoke`
  - `cargo test pipeline`
- 확인 포인트:
  - D1/D9 결과가 기존 Lagna 기준에서 유지되는지
  - Moon reference를 사용할 때 whole-sign/cusp-based house numbering이 reference-relative로 바뀌는지
  - 주석/문서가 canonical full snapshot 계약과 reference-aware house 계약을 반영하는지
