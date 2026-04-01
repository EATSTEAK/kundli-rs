# Context
초기 spec-driven 구조 변경은 이미 반영되었고, 이제 남은 범위는 3가지다: (1) `Jupiter Bhava`와 `GL Bhava` 같은 reference 확장, (2) D2~D60을 단순 `longitude * division`이 아니라 각 분할 차트의 전통 규칙으로 개별화, (3) 현재 동일하게 materialize되는 `Bhava`와 `Chalit`를 의미적으로 분리하는 것. 현재 코드상 `Jupiter` 기준점은 `ReferenceKey::Planet(AstroBody)`로 바로 수용 가능하지만, `GL`은 별도 special point가 없고, `Bhava/Chalit`는 `src/kundli/calculate.rs`에서 사실상 같은 파이프라인을 공유하며, divisional charts도 `src/kundli/derive/pipeline/sign.rs::DivisionalSignTransform`의 단순 remap에 머물러 있다.

# Recommended approach
1. **Reference 확장을 두 층으로 나눈다.**
   - `src/kundli/config.rs`
   - `src/kundli/derive/pipeline/reference.rs`
   - `src/kundli/model.rs`
   - `Jupiter Bhava`는 새 public alias/helper만 추가해 바로 지원한다.
   - `GL`은 `ReferenceKey`에 `Special(...)` 축을 추가하고, derive 계층에서 longitude를 계산해 `ResolvedReference`로 주입한다.
   - special point가 외부 응답에도 보여야 한다면 `ChartResult`/metadata에 별도 필드를 추가하고, 기준점으로만 쓰면 내부 derive 값으로만 유지한다.

2. **GL 계산은 astro가 아니라 derive helper에 둔다.**
   - `src/kundli/derive/reference_points.rs` 같은 전용 모듈을 추가하는 방향
   - 현재 `AstroResult`는 canonical body + ascendant/mc/house cusps만 가지므로, GL을 raw astro snapshot에 넣기보다 reference resolution 시 계산하는 편이 blast radius가 작다.
   - 이 helper는 `AstroResult`, `HouseSystem`, 필요 시 weekday/day-night 판정에 필요한 값만 받아 longitude를 계산한다.

3. **Varga rule을 data-driven table/strategy로 치환한다.**
   - `src/kundli/derive/pipeline/sign.rs`
   - 현재 `map_divisional_longitude(longitude, division)`는 모든 Dn에 동일 규칙을 적용한다.
   - 이를 `VargaScheme` 또는 `DivisionRule` 테이블로 바꿔 D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60 각각의 sign mapping 규칙을 분리한다.
   - `DivisionalSignTransform`는 `division`만 받지 말고 “resolved rule”을 받아 동작하게 바꾼다.
   - D9 전용 `D9Rule`는 새 rule table의 한 케이스로 흡수한다.

4. **Bhava와 Chalit를 결과 shape에서 분리한다.**
   - `src/kundli/model.rs`
   - `src/kundli/derive/pipeline/materialize.rs`
   - 현재 둘 다 `ChartLayer::Chart(ChartResult)`로 동일하게 materialize되어 `tests/astro_smoke.rs`에서도 같은 결과로 검증된다.
   - 권장 방향은 `ChartResult` 공통 기반은 유지하되, `Chalit`에만 “sign placement는 유지, house occupancy는 cusp-based로 재해석했다”는 의미가 드러나도록 별도 payload 또는 명시 필드를 둔다.
   - `Bhava`는 cusp-based houses 자체가 중심인 차트, `Chalit`는 sign anchoring과 house occupancy를 분리해 설명 가능한 차트로 정의한다.

5. **구현 순서를 고정한다.**
   - 1단계: `Jupiter Bhava` alias/helper 추가
   - 2단계: `ReferenceKey::Special(...)` 및 GL 계산 helper 도입
   - 3단계: divisional rule table로 D2~D60 개별화
   - 4단계: `Bhava`/`Chalit` payload 분리 및 materialize 업데이트
   - 5단계: smoke/integration fixtures 확장

# Critical files to modify
- `src/kundli/config.rs`
- `src/kundli/error.rs`
- `src/kundli/calculate.rs`
- `src/kundli/model.rs`
- `src/kundli/derive/pipeline/reference.rs`
- `src/kundli/derive/pipeline/sign.rs`
- `src/kundli/derive/pipeline/materialize.rs`
- `src/kundli/derive/pipeline/house.rs`
- 새 special-point helper 모듈 (`src/kundli/derive/*`)
- 테스트:
  - `tests/astro_smoke.rs`
  - `tests/derive_d9.rs`
  - 새 varga rule table tests
  - 새 reference/special-point tests

# Existing code to reuse
- `src/kundli/calculate.rs::derive_chart_result`
- `src/kundli/derive/pipeline/core.rs::ChartPipeline`
- `src/kundli/derive/pipeline/reference.rs::ReferenceTransform`
- `src/kundli/derive/pipeline/sign.rs::build_sign_placement`
- `src/kundli/derive/pipeline/house.rs::{WholeSignHouseTransform, CuspBasedHouseTransform}`
- `src/kundli/derive/pipeline/materialize.rs::Materialize`

# Verification
1. 단위 테스트
   - `Jupiter`/`Moon`/`Sun`/`GL` 기준점이 기대 longitude로 해석되는지 검증
   - D2~D60 rule table에서 대표 longitude boundary 케이스 검증
   - `Bhava`와 `Chalit`가 더 이상 동일 payload semantics로 수렴하지 않는지 검증

2. 통합 테스트
   - `calculate_kundli_with_engine` 경유로 `Jupiter Bhava`, `GL Bhava`, `D10`, `D24`, `D60`, `Divisional Bhava` 요청 검증
   - config validation에서 `Bhava/Chalit/DivisionalBhava`는 명시적 `CuspBased` mode를 요구하는지 검증

3. 회귀 테스트
   - 기존 `D1`, `D9`, `VimshottariDasha` 및 현재 reference/bhava 관련 테스트 유지

# GL decision
`GL Bhava`의 `GL`은 `Gulika`로 구현한다. 따라서 special reference 축은 최소 `Gulika`를 포함해야 하며, `ReferenceTransform` 또는 별도 special-point helper에서 Gulika longitude를 계산해 `ResolvedReference`로 변환한다.
