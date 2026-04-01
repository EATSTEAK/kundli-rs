# Context
사용자는 현재 `kundli-rs`가 지원하는 제한된 차트 집합(D1, D9, VimshottariDasha)에서 확장해, 표에 있는 기본 차트 / divisional charts / reference-based charts / bhava·chalit 계열을 모두 구현할 수 있는 계획을 원한다. 핵심 목표는 단순히 enum variant를 늘리는 것이 아니라, D2~D60, Moon/Sun/Jupiter/GL 기준 전환, divisional bhava/chalit까지 수용할 수 있는 구조로 API와 derive pipeline을 일반화하는 것이다. 사용자 확인 결과, 외부 API 모델은 개별 차트 enum 나열보다 `ChartSpec { kind, reference, house_mode }` 같은 spec 조합형이 우선이다.

# Recommended approach
1. **차트 taxonomy를 spec 중심으로 재설계한다.**
   - `src/kundli/config.rs`
   - 현재 `KnownChart`는 `D1`, `D9`, `VimshottariDasha`만 표현 가능하므로 확장성이 부족하다.
   - `ChartSpec` 계층으로 전환한다.
     - 예시 축:
       - `ChartKind`: `Rasi`, `Varga { division }`, `Bhava`, `Chalit`, `DivisionalBhava { division }`, `Dasha`
       - `ReferenceKey`: `Lagna`, `Moon`, `Sun`, `Planet(AstroBody)`, `Special(...)`
       - `HouseMode`: `WholeSign`, `CuspBased(HouseSystem)`, `None`
   - `Moon Chart`, `Sun Chart`, `Moon Bhava`, `D9 Bhava` 같은 차트는 새 top-level enum을 계속 추가하지 않고 spec 조합으로 표현한다.

2. **결과 타입을 공통 chart payload 중심으로 정리한다.**
   - `src/kundli/model.rs`
   - 현재 `ChartLayer::D1(D1Chart)`, `ChartLayer::D9(D9Chart)` 구조는 chart 종류가 늘수록 폭증한다.
   - 공통 `ChartResult`를 중심 payload로 승격하고, 다샤만 별도 payload로 유지하는 방향으로 정리한다.
   - `KundliResult.charts`의 key도 `KnownChart`에서 새 spec/identifier 타입으로 교체한다.

3. **calculate entrypoint를 spec-dispatch 구조로 바꾼다.**
   - `src/kundli/calculate.rs`
   - 현재 `calculate_kundli_with_engine`는 `match KnownChart`로 고정 분기한다.
   - 이를 `ChartSpec -> pipeline builder -> materialize` 흐름으로 바꾸고, 다샤만 별도 분기로 남긴다.
   - 이 단계가 전체 구조 변경의 핵심이며, 현재 구조는 요청된 전체 차트 구현에 충분하지 않다.

4. **기존 derive pipeline을 재사용 가능한 팩토리 형태로 일반화한다.**
   - `src/kundli/derive/pipeline/core.rs`
   - `src/kundli/derive/pipeline/reference.rs`
   - `src/kundli/derive/pipeline/sign.rs`
   - 이미 있는 `ChartPipeline`, `IdentitySignTransform`, `VargaTransform<R>`, `WholeSignHouseTransform`, `CuspBasedHouseTransform`는 그대로 재사용한다.
   - 새 작업은 “차트별 전용 derive 함수 추가”보다 “spec에 따라 reference/sign/house 연산을 조합하는 builder”를 만드는 쪽으로 잡는다.

5. **reference layer를 일반화한다.**
   - `src/kundli/derive/pipeline/reference.rs`
   - 현재 `LagnaReference`, `MoonReference`, `ReferencePoint::Planet(AstroBody)`가 이미 존재하므로 이를 기반으로 `Sun`, `Jupiter`, `GL` 등으로 확장한다.
   - 우선순위:
     - `Moon Chart`, `Sun Chart`
     - `Moon Bhava`, `Jupiter Bhava`, `GL Bhava` 같은 reference-based bhava/chalit
   - `GL` 같은 special point가 아직 astro/model에 없으면 별도 계산 경로가 필요한지 먼저 확인해야 한다.

6. **varga support를 D9 전용에서 범용 division rule 집합으로 확장한다.**
   - `src/kundli/derive/pipeline/sign.rs`
   - 현재 `D9Rule`만 있으므로 `VargaRule` 구현을 D2, D3, D4, D7, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60까지 확장한다.
   - 구현 순서는 사용자가 표에 제시한 순서를 따르되, 내부적으로는 rule table/strategy를 먼저 정리한 뒤 차트를 추가하는 방식이 낫다.
   - `D1`은 `Rasi + Lagna + IdentityTransform`, `D9`는 `Varga(9)`의 특수 케이스로 흡수한다.

7. **bhava/chalit 계열을 chart kind로 분리하되, house transform 조합으로 구현한다.**
   - `src/kundli/derive/d1.rs`
   - `src/kundli/derive/pipeline/house.rs`
   - 현재 D1에서 이미 `WholeSignHouseTransform | CuspBasedHouseTransform` 분기가 있으므로 이를 재사용한다.
   - 계획상 의미 구분:
     - `Bhava Chart`: 하우스 재산정 결과를 독립 chart로 노출
     - `Bhava Chalit`: sign 배치와 house 점유 표현 방식을 분리해 결과 shape를 명확히 정의
     - `Divisional Bhava`: varga transform 이후 house transform을 다시 적용

8. **구현 순서를 고정한다.**
   - 1단계: config/model/calculate의 spec 기반 구조 변경
   - 2단계: reference 일반화(Moon/Sun/Planet/Special)
   - 3단계: 범용 varga rule 도입(D2~D60)
   - 4단계: bhava/chalit/divisional bhava 조합 지원
   - 5단계: alias/프리셋 정리(예: Moon Chart = `Rasi + Moon ref`)

# Critical files to modify
- `src/kundli/config.rs`
- `src/kundli/model.rs`
- `src/kundli/calculate.rs`
- `src/kundli/derive/pipeline/reference.rs`
- `src/kundli/derive/pipeline/sign.rs`
- `src/kundli/derive/pipeline/house.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`
- 필요 시 `src/kundli/astro/*` 또는 special-point 계산 위치
- 테스트 파일:
  - `tests/derive_d1.rs`
  - `tests/derive_d9.rs`
  - `tests/astro_smoke.rs`
  - 새 fixture/golden test 파일들

# Existing code to reuse
- `src/kundli/derive/pipeline/core.rs::ChartPipeline`
- `src/kundli/derive/pipeline/reference.rs::{ReferencePoint, LagnaReference, MoonReference}`
- `src/kundli/derive/pipeline/sign.rs::{IdentitySignTransform, VargaTransform, VargaRule, D9Rule}`
- `src/kundli/derive/pipeline/house.rs::{WholeSignHouseTransform, CuspBasedHouseTransform}`
- `src/kundli/model.rs::ChartResult`

# Verification
1. 단위 테스트
   - 각 `VargaRule`의 longitude mapping 검증
   - reference 변경 시 lagna/house renumbering 검증
   - bhava/chalit/divisional bhava 조합별 invariant 검증

2. 통합 테스트
   - `src/kundli/calculate.rs` 경유로 `ChartSpec` 조합 요청이 올바른 payload를 반환하는지 확인
   - 중복 spec 정규화/정렬/validation 확인

3. fixture 기반 검증
   - 현재 `tests/fixtures/astro/smoke_case.json` 패턴을 확장해 D10, D24, D60, Moon Chart, Sun Chart, Bhava, Chalit, Moon Bhava 같은 대표 케이스 추가

4. 회귀 테스트
   - 기존 D1/D9/VimshottariDasha 테스트가 그대로 통과해야 함
   - D9의 기존 제약(예: whole-sign only)을 유지할지, 범용 spec 구조에서 재정의할지 명시적으로 검증

# Conclusion
현재 구조는 요청된 전체 차트 집합을 무리 없이 수용하기에 충분하지 않다. 특히 `KnownChart`/`ChartLayer`/`calculate_kundli_with_engine`의 고정 분기 구조는 반드시 바뀌어야 한다. 다만 derive pipeline의 핵심 부품(`ChartPipeline`, reference/sign/house transform)은 재사용 가치가 높아서, 전면 폐기보다 spec-driven 조합 구조로 감싸는 방향이 가장 적합하다.
