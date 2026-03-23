# Context
- `plan/initial-plan.md` 기준으로 `astro` 모듈을 먼저 구현한다.
- 현재 저장소는 `src/lib.rs` 기본 템플릿만 있고, `astro`가 기대하는 상위 타입/모듈이 아직 없다.
- 이번 작업의 목표는 **raw astronomical values만 계산하는 compile-ready `astro` 모듈**을 만들고, 해석 로직(`derive`)과 orchestration(`calculate_kundli`)은 범위에서 제외하는 것이다.

# Recommended approach
1. **최소 상위 wiring 추가**
   - `src/lib.rs`를 `pub mod kundli;`로 교체한다.
   - `src/kundli/mod.rs`를 만들고 `pub mod astro;`만 노출한다.

2. **`astro`의 독립적인 공개 타입부터 확정**
   - `src/kundli/astro/request.rs`: `AstroRequest`, `AstroBody`, `ZodiacType`, `Ayanamsha`, `HouseSystem`, `NodeType` 정의
   - `src/kundli/astro/result.rs`: `AstroResult`, `AstroBodyPosition`, `AstroMeta` 정의
   - `src/kundli/astro/error.rs`: `AstroError` 정의
   - 이 단계에서는 `BirthInput`, `KundliConfig`, `derive` 모듈에 의존하지 않게 유지한다.

3. **엔진 추상화와 Swiss Ephemeris 연동 지점 분리**
   - `src/kundli/astro/engine.rs`: `AstroEngine` trait과 `SwissEphAstroEngine` 구현
   - `src/kundli/astro/ephemeris.rs`: Swiss Ephemeris 호출을 감싸는 얇은 wrapper
   - `engine.rs`는 `ephemeris.rs`만 호출하고, sign/house/nakshatra 해석은 절대 넣지 않는다.

4. **Swiss Ephemeris 의존성은 wrapper 뒤로 숨기기**
   - 구현 시작 시 실제 crate API를 먼저 확인한 뒤 `ephemeris.rs`에만 붙인다.
   - 외부 crate API 차이 또는 변경 가능성이 있어도 `AstroRequest`/`AstroResult`/`AstroEngine` public API는 흔들리지 않게 유지한다.

5. **테스트는 shape + validation 중심으로 시작**
   - 요청값 검증(위도/경도 범위, 빈 planet 목록 처리 등)
   - 엔진 결과 shape 검증(body 개수, ascendant/MC/house cusp 존재 여부)
   - 실제 Swiss Ephemeris가 붙으면 알려진 JD/좌표 1건으로 smoke test를 추가한다.

# Critical files to modify
- `Cargo.toml`
- `src/lib.rs`
- `src/kundli/mod.rs`
- `src/kundli/astro/mod.rs`
- `src/kundli/astro/request.rs`
- `src/kundli/astro/result.rs`
- `src/kundli/astro/error.rs`
- `src/kundli/astro/engine.rs`
- `src/kundli/astro/ephemeris.rs`

# Reuse / source of truth
- 재사용 가능한 Rust 구현은 현재 `src/` 아래에 없다.
- 아래 설계 문서를 구현 기준으로 재사용한다.
  - `plan/initial-plan.md:121`~`plan/initial-plan.md:166` — `astro` 인터페이스
  - `plan/initial-plan.md:248`~`plan/initial-plan.md:263` — 전체 orchestration에서 `astro`의 역할
  - `plan/initial-plan.md:288`~`plan/initial-plan.md:294` — `astro`는 raw values만 계산한다는 원칙

# Coupling constraints
- `AstroRequest`는 `jd_ut`, `latitude`, `longitude` 같은 primitive와 `astro` 내부 enum만 사용한다.
- `astro` 단계에서는 `BirthInput`, `KundliConfig`, `KundliResult`, `derive` 모듈을 참조하지 않는다.
- 상위 모듈이 아직 비어 있으므로, 공통 enum도 우선 `astro`에 두고 필요하면 후속 작업에서 상위로 승격하거나 re-export 한다.

# Verification
1. `cargo check`로 모듈 구조와 public API 컴파일 확인
2. `cargo test`로 request validation / engine smoke test 확인
3. `cargo fmt`로 포맷 정리
4. 실제 Swiss Ephemeris가 연결되면, 알려진 JD/위도/경도 입력 1건에 대해 아래를 검증
   - 요청한 body 수만큼 `AstroResult.bodies`가 반환되는지
   - `ascendant_longitude`, `mc_longitude`, `house_cusps`가 채워지는지
   - sidereal 설정 시 `AstroMeta`에 ayanamsha 관련 메타가 남는지
5. 구현 후 `derive` 관련 필드(sign/house/nakshatra/pada)가 `astro`에 새로 들어가지 않았는지 코드 리뷰로 확인

# Main risk to handle during implementation
- 가장 큰 불확실성은 Swiss Ephemeris Rust crate의 실제 API와 빌드 방식이다.
- 따라서 구현 순서는 **public type 고정 → engine trait 고정 → `ephemeris.rs`에 crate 연동** 순서로 간다.
- crate API가 계획과 다르더라도 수정 범위가 `ephemeris.rs`와 `engine.rs`에 국한되게 유지한다.
