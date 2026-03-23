# Context
- 현재 `astro` 구현은 기본 골격과 raw result 경계는 맞지만, 테스트와 public API 정리가 아직 덜 끝난 상태다.
- 이번 변경의 목표는 다음 4가지를 한 번에 마무리하는 것이다.
  1. 좌표 검증을 fixture 기반 테스트로 확장
  2. `AstroRequest.planets`를 `bodies` 계열 용어로 통일
  3. Swiss Ephemeris wrapper의 두 가지 핵심 리스크 수정
     - sidereal house 계산 실패 시 원본 에러 메시지 보존
     - 전역 상태 변경을 caller에게 노출하지 않고 선언적 API 뒤로 캡슐화
  4. `cargo clippy` 경고까지 정리
- 최종 상태에서도 `astro`는 raw astronomical values만 반환하고, `derive` 의미 해석 로직은 추가하지 않는다.

# Recommended approach
1. **public API 용어를 먼저 고정한다**
   - `src/kundli/astro/request.rs`의 `AstroRequest.planets`를 `bodies`로 rename 한다.
   - `src/kundli/astro/engine.rs`, `src/kundli/astro/ephemeris.rs`, 기존 테스트 코드를 모두 같은 용어로 맞춘다.
   - `AstroResult.bodies`, `AstroBodyPosition.body`는 그대로 유지해 API 전체가 `AstroBody` / `bodies` / `body` 축으로 일관되게 보이게 한다.

2. **선언적 engine config를 도입하고 Swiss 전역 상태를 내부에 가둔다**
   - `src/kundli/astro/engine.rs`에 `SwissEphConfig` 같은 설정 타입을 추가하고 `SwissEphAstroEngine`는 이 config로만 생성되게 바꾼다.
   - `AstroRequest`에는 계산 의미(`zodiac`, `ayanamsha`, `house_system`, `node_type`, `bodies`)만 남기고, `ephemeris_path` 같은 실행 환경 설정은 engine config로만 이동/유지한다.
   - `src/kundli/astro/ephemeris.rs` 안에서만 `safe::set_ephe_path`, `safe::set_sidereal_mode`를 호출하게 하고, 한 번의 calculation 동안 Swiss Ephemeris 호출 전체를 process-wide lock으로 감싸 전역 상태가 요청 간에 interleave 되지 않게 한다.
   - 이렇게 해서 public API는 계속 `engine.calculate(&request)` 형태의 선언적 인터페이스만 노출하고, imperative global mutation은 외부에 드러나지 않게 한다.

3. **sidereal house 에러를 generic string 대신 실제 Swiss 에러로 전달한다**
   - `src/kundli/astro/ephemeris.rs`의 `houses_ex` wrapper를 수정해 `swe_houses_ex` 실패 시 Swiss Ephemeris의 에러 버퍼를 읽어 `AstroError`로 올린다.
   - `src/kundli/astro/error.rs`는 필요하면 operation context를 함께 담을 수 있게 조금 확장하되, 에러를 다시 뭉개지 않는다.
   - `engine.rs`는 low-level detail을 재가공하지 말고 wrapper가 만든 상세 에러를 그대로 전파한다.

4. **fixture 기반 테스트를 integration test로 추가한다**
   - fixture는 `tests/fixtures/astro/` 아래에 두고, 테스트 엔트리는 `tests/` 아래 integration test로 만든다.
   - `tests/fixtures/astro/coordinate_validation.json`
     - 유효 좌표, 경계값(`±90`, `±180`), 범위 밖 좌표, empty bodies, non-finite 좌표를 케이스별로 정의한다.
   - `tests/fixtures/astro/smoke_case.json`
     - 알려진 `jd_ut` / `latitude` / `longitude` 입력 1건과 요청 `bodies`를 정의한다.
   - `tests/astro_request_validation.rs`
     - fixture를 읽어 `AstroRequest::validate` 결과와 `AstroError` variant를 검증한다.
   - `tests/astro_smoke.rs`
     - `SwissEphAstroEngine::calculate`를 호출해 body 개수, `house_cusps.len() == 12`, finite `ascendant_longitude`/`mc_longitude`, sidereal일 때 `meta.ayanamsha_value.is_some()`를 검증한다.
   - 현재 `request.rs` / `engine.rs`의 inline unit test는 완전히 중복되는 것은 줄이고, 구현 가까이에 둘 가치가 있는 최소 테스트만 남긴다.

5. **clippy와 작은 구현 정리를 마지막에 마무리한다**
   - `src/kundli/astro/ephemeris.rs`의 house cusp copy loop는 slice copy로 바꿔 `manual_memcpy`를 제거한다.
   - rename / config 도입 과정에서 생기는 추가 clippy 경고를 함께 정리한다.
   - scope는 `astro` 내부 정리로 제한하고, `derive` 관련 필드나 helper는 추가하지 않는다.

# Critical files to modify
- `Cargo.toml` — fixture JSON 파싱에 필요한 최소 `dev-dependencies` 추가가 필요하면 수정
- `src/kundli/astro/request.rs`
- `src/kundli/astro/engine.rs`
- `src/kundli/astro/ephemeris.rs`
- `src/kundli/astro/error.rs`
- `src/kundli/astro/mod.rs`
- `tests/astro_request_validation.rs`
- `tests/astro_smoke.rs`
- `tests/fixtures/astro/coordinate_validation.json`
- `tests/fixtures/astro/smoke_case.json`

# Reuse / source of truth
- 설계 기준
  - `plans/abstract-watching-crystal.md:26`~`29` — validation + known JD/좌표 smoke test 요구사항
  - `plans/initial-plan.md:121`~`166` — `AstroRequest` / `AstroResult` / `AstroEngine` 인터페이스
  - `plans/initial-plan.md:288`~`294` — `astro`는 raw values만 계산한다는 최종 원칙
- 재사용할 기존 구현
  - `src/kundli/astro/request.rs:55` `AstroRequest::validate` — fixture validation의 기준 함수로 재사용
  - `src/kundli/astro/engine.rs:26` `AstroEngine::calculate` — public execution boundary 유지
  - `src/kundli/astro/ephemeris.rs:16` `Ephemeris::calculate` — Swiss 호출 orchestration의 중심으로 재사용
  - `src/kundli/astro/ephemeris.rs:87` `calc_flags` — zodiac별 Swiss flag 조합 재사용
  - `src/kundli/astro/ephemeris.rs:97` `to_sidereal_mode` / `src/kundli/astro/ephemeris.rs:105` `to_house_system` / `src/kundli/astro/ephemeris.rs:114` `node_body` — wrapper 내부 mapping 로직 재사용
  - `src/kundli/astro/ephemeris.rs:135` `houses_ex` — 상세 에러 보존과 clippy 수정이 필요한 핵심 wrapper로 재사용

# Verification
1. `cargo fmt --check`
2. `cargo check`
3. `cargo test --lib`
4. `cargo test --test astro_request_validation --test astro_smoke`
5. `cargo clippy --all-targets --all-features -- -D warnings`
6. 가능하면 마지막에 `cargo test` 전체를 다시 돌려 doctest까지 확인한다. 현재 harness에서는 `/tmp/claude/...` 임시 디렉터리 문제로 doctest가 실패할 수 있으므로, 코드 검증의 1차 기준은 3~5번으로 둔다.
7. 코드 리뷰로 아래를 다시 확인한다.
   - public API에 imperative setter가 새로 생기지 않았는지
   - `AstroRequest`가 여전히 primitive + astro enum만 사용하는지
   - `sign` / `house placement` / `nakshatra` / `pada` 같은 derive 필드가 `astro`에 추가되지 않았는지

# Main risk to handle during implementation
- Swiss Ephemeris는 backend 자체가 전역 상태를 사용하므로, “전역 상태를 없애는 것”이 아니라 “전역 상태를 caller에게 노출하지 않고, 요청 간 경쟁 없이 내부에서 직렬화하는 것”이 이번 수정의 핵심이다.
- 따라서 구현 순서는 **request rename → declarative engine config → synchronized ephemeris wrapper → fixture tests → clippy 정리** 순으로 가져간다.