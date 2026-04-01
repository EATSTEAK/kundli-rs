## Context
이번 변경의 목적은 사용자가 제시한 2~6단계 chart pipeline 골격(`ProjectionOp -> ReferenceOp -> SignTransformOp -> HouseTransformOp -> Materialize`)을 현재 `kundli-rs`의 derive 계층에 실제로 연결하는 것이다. 핵심은 각 단계가 입력 타입과 출력 타입을 명시적으로 가지도록 만들고, `Pipeline<P, R, ST, HT>`가 그 연결을 타입으로 강제하게 하는 데 있다. 이때 pipeline의 2단계 입력은 별도 `AstroBase`를 도입하지 않고 현재 raw astro 계약인 `AstroResult`로 고정한다. 목표는 D1/D9/Moon/Bhava 같은 차트 정의가 operation 조합으로 드러나게 하면서도, 기존 derive 코드의 재사용성과 상위 API 호환성을 유지하는 것이다.

## Fixed pipeline outline to preserve
- 2단계 `ProjectionOp`: `AstroResult` -> `ProjectedBase`
- 3단계 `ReferenceOp`: `ProjectedBase` -> `ReferenceContext`
- 4단계 `SignTransformOp`: `ReferenceContext` -> `SignContext`
- 5단계 `HouseTransformOp`: `SignContext` -> `HouseContext`
- 6단계 `Materialize`: `HouseContext` -> `ChartResult`
- 7단계 `Pipeline<P, R, ST, HT>`: 위 단계를 순서대로 실행

이 아웃라인 자체는 유지하고, 현재 코드 적응은 이 단계 책임 안에서만 수행한다.

## Agreed design decisions
- pipeline의 직접 대상은 chart derivation이다. `VimshottariDasha`는 같은 pipeline에 억지로 넣지 않고 별도 derive 흐름으로 유지한다.
- `ProjectionOp` 출력 `ProjectedBase`는 longitude/cusp/reference용 raw 투영값 중심으로 유지한다. sign, degrees, nakshatra 같은 파생값은 여기서 만들지 않는다.
- `ReferenceOp`는 symbolic reference 선택만 하지 않고, 이후 단계가 바로 쓸 수 있는 resolved reference longitude/context까지 만든다.
- `SignTransformOp`는 transformed longitude, sign, degrees_in_sign까지 확정한 완전한 sign snapshot을 만든다.
- `HouseTransformOp`는 subject별 house 번호와 house 구조의 기준축을 확정한다. cusp/sign 직렬화 같은 최종 결과화는 `Materialize`가 맡는다.
- `Materialize`는 항상 새 공통 `ChartResult`를 만든다. 현재 `D1Chart`/`D9Chart`는 `ChartResult`에서 변환되는 API 결과 타입으로 정리한다.

## Recommended implementation approach
1. `src/kundli/astro/result.rs`의 현재 `AstroResult`를 pipeline의 raw 입력 계약으로 그대로 사용한다.
2. 신규 `src/kundli/derive/pipeline.rs`에 사용자가 제안한 형태의 핵심 trait와 `Pipeline<P, R, ST, HT>`를 도입한다.
   - `ProjectionOp { type Output; fn apply(&self, input: AstroResult) -> Self::Output; }`
   - `ReferenceOp<Input> { type Output; fn apply(&self, input: &Input) -> Self::Output; }`
   - `SignTransformOp<Input> { type Output; fn apply(&self, input: &Input) -> Self::Output; }`
   - `HouseTransformOp<Input> { type Output; fn apply(&self, input: &Input) -> Self::Output; }`
   - `Materialize { fn materialize(self) -> ChartResult; }`
   - `Pipeline<P, R, ST, HT>`는 `R: ReferenceOp<P::Output>`, `ST: SignTransformOp<R::Output>`, `HT: HouseTransformOp<ST::Output>`, `HT::Output: Materialize` 제약을 그대로 둔다.
3. 신규 또는 기존 derive 공통 타입 파일에 중간 컨텍스트를 추가한다.
   - `ProjectedBase`: 투영된 ascendant/body/cusp longitude 보유
   - `ReferenceContext`: projected data + resolved reference
   - `SignContext`: transformed longitude + sign + degrees snapshot
   - `HouseContext`: subject별 house assignment와 materialize 직전 상태
   - `ChartResult`: pipeline 공통 결과
4. `src/kundli/derive/d1.rs`와 `src/kundli/derive/d9.rs`는 기존 `derive_*_from_input` 구현을 즉시 버리지 말고, 새 pipeline 단계 구현체 내부에서 재사용 가능한 계산 로직만 추출하도록 정리한다.
   - D1: `IdentityProjection + LagnaReference + IdentitySignTransform + WholeSignHouseTransform/cusp-based house transform`
   - D9: `SiderealProjection + LagnaReference + VargaTransform<D9Rule> + WholeSignHouseTransform`
   - 이후 Moon/Bhava chart는 reference/house 단계 구현체 조합으로 확장 가능하게 둔다.
5. `src/kundli/model.rs`에 새 공통 `ChartResult`를 추가하고, `ChartResult -> D1Chart`, `ChartResult -> D9Chart` 변환 경로를 둔다. `KundliResult`와 top-level mirror 필드는 유지한다.
6. `src/kundli/calculate.rs`는 최종 orchestration만 담당한다. engine이 반환한 raw astro 데이터를 pipeline 입력으로 넣어 D1/D9를 만들고, `dasha`는 기존 별도 derive를 유지한다.

## Existing code to reuse
- `src/kundli/astro/result.rs`
  - `AstroResult`
  - `AstroResult::body`
- `src/kundli/derive/input.rs`
  - 현재 `prepare_angle`, `prepare_body`, `navamsa_longitude`, `transform_body_to_navamsa`에 들어 있는 계산은 새 단계 구현으로 이동하거나 재사용
  - `KundliDeriveInput::body` 접근 패턴은 reference/sign 단계 보조 로직으로 재사용 가능
- `src/kundli/derive/d1.rs`
  - `derive_lagna_from_input`
  - `derive_planet_placements_from_input`
  - `derive_houses_from_input`
  - `derive_d1_chart_from_input`
- `src/kundli/derive/d9.rs`
  - `derive_d9_chart_from_input`
  - `derive_d9_planet_placements_from_input`
- `src/kundli/derive/dasha.rs`
  - `derive_vimshottari_dasha_from_input`는 chart pipeline 밖 별도 derive로 유지
- `src/kundli/calculate.rs`
  - `build_calculation_meta`

## Critical files to modify
- `src/kundli/astro/result.rs`
- `src/kundli/derive/mod.rs`
- `src/kundli/derive/pipeline.rs` (new)
- `src/kundli/derive/input.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`
- 필요 시 `src/kundli/derive/house.rs`
- `src/kundli/calculate.rs`
- `src/kundli/model.rs`

## Implementation order
1. `derive/pipeline.rs`에 단계 trait, 중간 컨텍스트, `Pipeline<P, R, ST, HT>` 골격 추가
2. `astro/result.rs`와의 관계를 정리해 `AstroBase` alias/adapter 여부 확정
3. `ProjectionOp`, `ReferenceOp`, `SignTransformOp`, `HouseTransformOp`의 최소 구현체(`Identity/Sidereal`, `Lagna`, `Identity/Varga`, `WholeSign/CuspBased`) 추가
4. `model.rs`에 공통 `ChartResult`와 기존 `D1Chart`/`D9Chart` 매핑 정리
5. `d1.rs`, `d9.rs`를 pipeline 기반 chart assembly로 전환
6. `calculate.rs`를 새 D1/D9 pipeline 호출 방식으로 전환
7. `dasha.rs`는 분리 유지하되, 필요한 공통 계산 재사용만 최소 반영

## Key constraints
- 2~6단계 책임 분리는 유지한다. 단계 책임을 섞지 않는다.
- `ProjectionOp` 입력은 별도 `AstroBase`가 아니라 `AstroResult`다.
- `ProjectionOp`에서 sign/nakshatra를 미리 계산하지 않는다.
- `ReferenceOp`는 resolved reference context까지 만든다.
- `SignTransformOp`는 sign/degrees snapshot을 완성한다.
- `HouseTransformOp`는 house assignment를 책임지고 최종 직렬화는 `Materialize`가 맡는다.
- `Dasha`는 이번 chart pipeline 범위 밖이다.
- 현재 public calculate API와 `KundliResult` mirror 구조는 유지한다.

## Refactor follow-up: dead code cleanup and module split
- dead code의 직접 원인은 public 경로가 `derive_d1_chart_result` / `derive_d9_chart_result` -> `Pipeline::execute`로 바뀌면서, 기존 `derive_d1_chart_from_input`, `derive_d1_chart_result_from_input`, `derive_d9_chart_from_input`, `derive_d9_planet_placements_from_input`가 더 이상 호출되지 않게 된 것이다.
- 다음 리팩터링에서는 이 고립된 `from_input` 경로를 제거하거나 pipeline 내부 helper로 흡수해 경고를 없앤다.
- `src/kundli/derive/pipeline.rs`는 책임별로 분리한다.
  - `src/kundli/derive/pipeline/mod.rs`: `Pipeline<P,R,ST,HT>`와 공통 trait re-export
  - `src/kundli/derive/pipeline/projection.rs`: `ProjectedBase`, `ProjectedBody`, `ProjectionOp`, `IdentityProjection`, `SiderealProjection`
  - `src/kundli/derive/pipeline/reference.rs`: `ReferencePoint`, `ResolvedReference`, `ReferenceContext`, `LagnaReference`, `MoonReference`
  - `src/kundli/derive/pipeline/sign.rs`: `SignPlacement`, `SignContext`, `SignTransformOp`, `IdentitySignTransform`, `VargaRule`, `D9Rule`, `VargaTransform`
  - `src/kundli/derive/pipeline/house.rs`: `HouseContext`, `HousedPlacement`, `HouseSeed`, `HouseTransformOp`, `WholeSignHouseTransform`, `CuspBasedHouseTransform`
  - `src/kundli/derive/pipeline/materialize.rs`: `Materialize`와 `HouseContext -> ChartResult`
- `d1.rs`와 `d9.rs`는 orchestration 파일로 단순화한다. 구체 구현 디테일은 pipeline 하위 모듈로 이동하고, chart별 preset 조합만 남긴다.
- visibility는 기본적으로 `pub(crate)`를 유지하고, `derive::pipeline::mod.rs`에서 필요한 항목만 re-export한다. 외부 API로는 새 내부 타입이 새지 않게 한다.
- `input.rs`는 현재 `dasha.rs`와 테스트가 아직 사용 중이므로 즉시 제거하지 않는다. 다만 chart pipeline이 더 성숙하면 `prepared.rs`/`navamsa.rs`로 재배치할 수 있다.

## Verification
- 단위 테스트:
  - `ProjectionOp` 구현이 tropical 유지 / sidereal 변환을 올바르게 수행하는지 검증
  - `ReferenceOp` 구현이 Lagna/Moon 등 resolved reference를 정확히 산출하는지 검증
  - `SignTransformOp` 구현이 D1 identity / D9 varga longitude와 sign/degrees를 정확히 만드는지 검증
  - `HouseTransformOp` 구현이 whole-sign / reference renumbering / cusp-based house assignment를 정확히 수행하는지 검증
  - `Materialize`가 `HouseContext -> ChartResult`를 기대한 형태로 직렬화하는지 검증
- 회귀 테스트:
  - 기존 `derive_d1_chart`와 새 D1 pipeline 결과가 동일한지 비교
  - 기존 `derive_d9_chart`와 새 D9 pipeline 결과가 동일한지 비교
  - `calculate_kundli_with_engine`에서 `lagna == d1.lagna`, `planets == d1.planets`, `houses == d1.houses` mirror 유지 확인
  - optional `d9`, `dasha` on/off 동작 유지 확인
  - 리팩터링 후 `cargo test`에서 dead code warning이 제거되었는지 확인
- 실행 명령:
  - `cargo test`
  - 필요 시 `cargo test kundli::derive`
  - 필요 시 `cargo test calculate_with_engine`
