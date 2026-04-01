## Context
이번 변경의 목적은 derive 계층에서 남아 있는 범용 전처리 컨테이너 `KundliDeriveInput`을 제거하고, 현재 chart와 dasha가 서로 다른 계산 성격을 갖는다는 점을 코드 구조에 반영하는 것이다. 지금은 D1/D9는 generic `Pipeline<P, R, ST, HT>`를 통해 chart derivation을 수행하지만, dasha는 별도로 `KundliDeriveInput::from_astro`를 거쳐 Moon의 nakshatra/progress를 계산한다. 이 구조는 chart용 단계형 pipeline과 dasha용 Moon 중심 계산을 한 추상화 아래에 느슨하게 묶고 있어 책임 경계가 흐린다. 목표는 chart는 명시적인 `ChartPipeline`으로, dasha는 별도의 dasha pipeline 객체로 분리하고, 범용 입력 객체 없이 각 도메인에 필요한 최소 snapshot/helper만 남기는 것이다.

## Recommended approach
1. chart pipeline의 이름과 역할을 고정한다.
   - `src/kundli/derive/pipeline/core.rs`의 `Pipeline<P, R, ST, HT>`를 `ChartPipeline<P, R, ST, HT>`로 rename한다.
   - `execute(&self, input: AstroResult) -> Result<ChartResult, DeriveError>` 시그니처는 유지하되, 문서와 테스트에서 chart derivation 전용 객체임을 명확히 한다.
   - `src/kundli/derive/pipeline/mod.rs`, `src/kundli/derive/d1.rs`, `src/kundli/derive/d9.rs`의 re-export/사용처를 함께 갱신한다.

2. `KundliDeriveInput` 제거 전에 dasha가 실제로 필요한 입력을 독립 타입으로 축소한다.
   - `src/kundli/derive/dasha.rs` 안에 dasha 전용 최소 snapshot을 둔다.
   - 권장 형태:
     - `MoonDashaSeed { jd_ut, zodiac, moon_nakshatra, moon_nakshatra_progress_ratio }`
     - 또는 `MoonDashaSnapshot { moon_longitude, jd_ut, zodiac }` + 내부 helper
   - 이 snapshot은 `AstroResult`에서 직접 생성한다. Moon lookup은 `AstroResult::body(AstroBody::Moon)`를 재사용한다.
   - nakshatra 관련 계산은 `src/kundli/derive/nakshatra.rs`의 `nakshatra_placement_from_longitude`, `nakshatra_progress_ratio`를 그대로 재사용한다.

3. dasha 전용 pipeline 객체를 도입한다.
   - `src/kundli/derive/dasha.rs` 또는 `src/kundli/derive/dasha/` 하위 모듈에 `DashaPipeline` 또는 `VimshottariDashaPipeline`을 추가한다.
   - 권장 인터페이스:
     - `pub(crate) struct DashaPipeline;`
     - `impl DashaPipeline { fn execute(&self, astro: &AstroResult) -> Result<VimshottariDasha, DeriveError> }`
   - 내부 단계 책임:
     - sidereal 여부 검증
     - Moon longitude 추출
     - Moon nakshatra / progress 계산
     - `dasha_lord_for_nakshatra`로 current lord 결정
     - `mahadasha_duration_days`를 사용해 current period와 full sequence materialize
   - 기존 `derive_vimshottari_dasha`는 public wrapper로 유지하되 내부에서 `DashaPipeline::execute`를 호출하게 바꾼다.

4. `KundliDeriveInput`와 관련 helper를 제거하고 필요한 전처리를 역할별로 재배치한다.
   - `src/kundli/derive/input.rs`의 `KundliDeriveInput`는 삭제한다.
   - 함께 삭제 대상:
     - `PreparedAngle`
     - `PreparedBody`
     - `KundliDeriveInput::from_astro`
     - `KundliDeriveInput::body`
     - `KundliDeriveInput::to_navamsa`
   - 단, 여기에 있던 계산 자체는 버리지 않고 역할별로 재배치한다.
     - chart에서 필요한 canonicalization은 이미 `pipeline/projection.rs`, `pipeline/sign.rs`가 대부분 담당하므로 중복 helper는 제거한다.
     - dasha에 필요한 nakshatra/progress 계산은 `dasha.rs` 내부 helper로 이동한다.
   - `src/kundli/derive/mod.rs`에서 `mod input;`를 제거한다.

5. D1/D9와 calculate 경로를 새 구조에 맞춰 정리한다.
   - `src/kundli/derive/d1.rs`, `src/kundli/derive/d9.rs`에서 `ChartPipeline::new(...)`를 사용하게 바꾼다.
   - `src/kundli/calculate.rs`는 공개 API를 유지하고, 내부적으로는 dasha 계산만 새 `DashaPipeline` wrapper 경로를 통하게 둔다.
   - `ChartResult`, `D1Chart`, `D9Chart`, `VimshottariDasha` 같은 공개 결과 타입은 유지한다.

## Critical files
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/core.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/pipeline/mod.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/d1.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/d9.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/dasha.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/input.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/derive/mod.rs`
- `/Users/koohyomin/Projects/kundli-rs/src/kundli/calculate.rs`
- 필요 시 `/Users/koohyomin/Projects/kundli-rs/src/kundli/error.rs`

## Reuse
- `src/kundli/astro/result.rs`
  - `AstroResult::body`
- `src/kundli/derive/nakshatra.rs`
  - `nakshatra_placement_from_longitude`
  - `nakshatra_progress_ratio`
  - `dasha_lord_for_nakshatra`
- `src/kundli/derive/sign.rs`
  - `normalize_longitude`
  - `sign_from_longitude`
  - `degrees_in_sign`
- `src/kundli/derive/dasha.rs`
  - `mahadasha_duration_days`
  - `mahadasha_years`
- `src/kundli/derive/pipeline/*`
  - projection/reference/sign/house/materialize 단계 타입과 trait는 chart 전용 구조로 유지

## Verification
- rename / 구조 회귀
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- chart pipeline 회귀
  - `cargo test pipeline`
  - `cargo test derive_d1`
  - `cargo test derive_d9`
- dasha 회귀
  - `cargo test derive_dasha`
  - 검증 포인트:
    - non-sidereal에서 `UnsupportedZodiac`
    - Moon lookup 실패 시 `MissingMoon`
    - Moon nakshatra 기반 current lord 유지
    - nakshatra progress 반영한 current period 유지
    - 9개 mahadasha sequence 유지
- 통합 회귀
  - `cargo test astro_smoke`
  - `cargo test calculate_with_engine`
  - `calculate_kundli_with_engine` 결과가 manual D1/D9/Dasha 조립과 동일한지 유지

## Notes
- `ChartPipeline`는 계산 과정, `ChartResult`는 산출물이라는 점이 이름상 헷갈리지 않도록 주석과 테스트 이름을 함께 정리한다.
- dasha는 chart pipeline trait 체계를 억지로 재사용하지 않는다. dasha 계산 단계는 Moon 중심이라 구조가 다르므로 별도 pipeline object로 두는 편이 더 명확하다.
- `input.rs` 제거 시 기존 테스트는 목적별로 재배치한다. navamsa 관련 검증은 D9/sign transform 쪽으로, Moon progress 관련 검증은 dasha 테스트로 옮긴다.
