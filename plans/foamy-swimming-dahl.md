# Context

현재 `AstroResult`는 `AstroRequest`의 요청 shape를 거의 그대로 반영하는 raw result라서, future derivation layer가 항상 필요한 입력을 갖는다는 보장이 없다. 실제로 Dasha는 Moon을 요구하지만 현재는 `request.bodies`에 Moon이 빠지면 derive 단계에서 실패할 수 있다. 또한 `Reference-house`, `Bhava`, `Special Lagna` 같은 후속 차트는 body subset과 무관하게 항상 전체 핵심 천문 스냅샷을 전제로 하는 편이 구조적으로 안전하다.

이번 변경의 목표는 `AstroResult`를 “요청 결과”가 아니라 “모든 차트 파생을 위한 derivation-ready snapshot”으로 재정의하는 것이다. 사용자 요청에 따라 `AstroRequest.bodies`는 제거하고, `AstroResult`는 항상 Sun~Ketu 9개 body와 12개 house cusp를 보장하도록 바꾼다.

# Recommended approach

1. `AstroRequest`에서 `bodies`를 제거한다.
   - 대상: `src/kundli/astro/request.rs`
   - `AstroRequest::new` 시그니처를 `jd_ut, latitude, longitude`만 받도록 바꾼다.
   - validation에서 empty bodies 검사를 제거한다.
   - `request.bodies`를 전제하는 테스트/호출부를 전부 정리한다.

2. `AstroResult`를 guaranteed snapshot 계약으로 강화한다.
   - 대상: `src/kundli/astro/result.rs`
   - `bodies`는 항상 Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu를 모두 포함하도록 보장한다.
   - `house_cusps`는 `Vec<f64>` 대신 `[f64; 12]`로 고정해 shape 보장을 타입 수준으로 올린다.
   - 가능하면 `AstroResult::body(AstroBody) -> &AstroBodyPosition` 같은 accessor를 추가해 downstream이 body 존재 여부를 optional로 다루지 않게 만든다.
   - 테스트/수동 fixture에서도 malformed `AstroResult`를 만들기 어렵게 constructor 또는 invariant helper를 둔다.

3. 엔진 경계를 새 계약에 맞춘다.
   - 대상: `src/kundli/astro/engine.rs`
   - `SwissEphAstroEngine::calculate`는 요청 body 목록을 따르지 않고, canonical 9개 body를 항상 계산해 `AstroResult`에 채운다.
   - `raw.houses.cusps`를 `[f64; 12]`로 채우고, 길이/유한값 invariant를 여기서 확인한다.
   - `AstroMeta`는 지금처럼 실제 계산 설정(`jd_ut`, `zodiac`, `ayanamsha`, `ayanamsha_value`, `sidereal_time`)을 유지한다.

4. derive input을 guaranteed contract 위로 옮긴다.
   - 대상: `src/kundli/derive/input.rs`
   - `KundliDeriveInput::from_astro`가 arbitrary body vec를 snapshot으로 변환하는 구조에서, guaranteed body set을 그대로 normalize하는 구조로 바꾼다.
   - `house_cusps` 타입 변경(`[f64; 12]`)을 반영한다.
   - `body()` 조회는 total accessor를 사용할 수 있으면 그쪽으로 정리한다.

5. downstream derive의 optional 가정을 제거/축소한다.
   - 대상: `src/kundli/derive/d1.rs`, `src/kundli/derive/d9.rs`, `src/kundli/derive/dasha.rs`
   - D1/D9는 새 `house_cusps` 타입과 body accessor를 사용하도록 수정한다.
   - Dasha는 엔진-produced `AstroResult`에서 Moon 부재를 더 이상 정상 경로로 취급하지 않도록 단순화한다.
   - `DeriveError::MissingMoon`은 공개 생성 경로를 어떻게 둘지에 따라 제거하거나, malformed test fixture 방어용으로만 유지한다.

6. high-level 조립부와 문서를 새 계약에 맞춘다.
   - 대상: `src/kundli/calculate.rs`, `src/kundli/astro/mod.rs`, 관련 docs/tests
   - `body_count`는 항상 9가 되므로 이를 새 계약에 맞게 검증한다.
   - 모듈 문서에서 `AstroResult`가 raw mirror가 아니라 derivation-ready snapshot임을 명시한다.

# Critical files

- `src/kundli/astro/request.rs`
- `src/kundli/astro/result.rs`
- `src/kundli/astro/engine.rs`
- `src/kundli/derive/input.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`
- `src/kundli/derive/dasha.rs`
- `src/kundli/calculate.rs`
- `src/kundli/error.rs`
- `tests/astro_smoke.rs`
- `tests/astro_request_validation.rs`
- `tests/derive_d1.rs`
- `tests/derive_d9.rs`
- `tests/derive_dasha.rs`

# Existing code to reuse

- `SwissEphAstroEngine::calculate`의 raw -> public result 변환 경계 (`src/kundli/astro/engine.rs:47`)
- `KundliDeriveInput::from_astro` normalization path (`src/kundli/derive/input.rs:40`)
- D1 조립 패턴 (`src/kundli/derive/d1.rs:120`)
- D9의 transformed-input 재사용 구조 (`src/kundli/derive/d9.rs:21`)
- Dasha의 Moon/nakshatra 기반 계산 로직 (`src/kundli/derive/dasha.rs:10`)

# Verification

1. 단위 테스트 업데이트 및 추가
   - `AstroRequest` validation tests에서 bodies 관련 검증 제거
   - 엔진 테스트에서 요청과 무관하게 항상 9 bodies, 12 cusps가 나오는지 검증
   - derive tests fixture를 새 `AstroResult` shape로 갱신
   - subset request가 사라진 뒤에도 D1/D9/Dasha가 모두 정상 동작하는지 회귀 테스트 추가

2. 통합 테스트
   - `tests/astro_smoke.rs`에서 `result.bodies.len() == 9`, `result.house_cusps.len() == 12`를 확인
   - `calculate_kundli_with_engine` 경로에서 `body_count == 9`, optional outputs(D9/Dasha) 정상 생성 확인

3. 실행 확인
   - `cargo test`
   - 필요 시 관련 테스트만 개별 실행: `cargo test astro_smoke`, `cargo test derive_d1`, `cargo test derive_d9`, `cargo test derive_dasha`

# Notes

- 이 변경은 public API shape 변경(`AstroRequest::new`, `AstroRequest` fields, `AstroResult.house_cusps`)을 포함하므로, crate 내부 호출부와 테스트를 한 번에 맞춰야 한다.
- 설계 기준은 “요청 최소화”가 아니라 “파생 안정성 최대화”다. 즉, 이후 `Reference-house`, `Bhava`, `Special Lagna` 구현이 request shape와 무관하게 `AstroResult`만 믿고 진행될 수 있어야 한다.
