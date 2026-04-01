## Context
현재 kundli 라이브러리는 상위 공개 API(`calculate_kundli`, `calculate_kundli_with_engine`) 기준으로는 `Result`를 반환하지만, 내부 derive 계층에 `expect`/`unwrap`이 남아 있어 비정상 입력이나 내부 불변식 위반 시 panic 경로가 존재합니다. 이번 변경의 목적은 파이프라인뿐 아니라 `src/kundli`의 프로덕션 코드 전반에서 panic 가능 지점을 `DeriveError`로 승격해, 라이브러리 사용자가 panic 대신 명시적인 에러를 받도록 만드는 것입니다. 의도한 결과는 derive/dasha/pipeline 흐름 전체가 `Result`로 일관되게 종료되고, 테스트 코드에만 `unwrap`이 남는 상태입니다.

## Recommended approach
1. `src/kundli/derive/pipeline/materialize.rs`
   - `Materialize::materialize(self) -> ChartResult`를 `-> Result<ChartResult, DeriveError>`로 변경합니다.
   - `body.placement.body.expect(...)`를 신규 `DeriveError` variant로 바꿉니다.
   - `sign_from_longitude(...).expect(...)`는 기존 `DeriveError::InvalidLongitude(f64)`로 매핑합니다.

2. `src/kundli/derive/pipeline/core.rs`
   - `ChartPipeline::execute(...) -> Result<ChartResult, DeriveError>` 시그니처는 유지합니다.
   - 마지막 `housed.materialize()`를 그대로 `Result`로 전파하도록 연결합니다.

3. `src/kundli/derive/pipeline/house.rs`
   - `HouseNumber::new(...).expect(...)` 호출들을 모두 `Result` 기반으로 바꿉니다.
   - `renumber_house`를 `Result<HouseNumber, DeriveError>`로 바꿔 panic을 제거합니다.
   - `WholeSignHouseTransform::apply`, `CuspBasedHouseTransform::apply`, `derive_house_from_cusps`에서 새 반환형을 `?`로 전파합니다.

4. `src/kundli/derive/house.rs`
   - `derive_house_whole_sign`, `derive_house_from_cusps` 안의 `expect`를 `ok_or(...)`로 교체합니다.
   - `longitude_to_sign_index(longitude: f64) -> usize`는 내부 `expect`를 없애기 위해 `-> Result<usize, DeriveError>`로 변경하고, 호출부에서 `?`로 전파합니다.
   - 이 파일은 이미 `Result<HouseNumber, DeriveError>`를 반환하므로 시그니처 파급은 제한적입니다.

5. `src/kundli/derive/dasha.rs`
   - `DashaLord::SEQUENCE.iter().position(...).expect(...)`를 panic 대신 명시적 에러로 바꿉니다.
   - `current_lord`가 시퀀스에 없다는 것은 도메인 불변식 위반이므로, 신규 `DeriveError` variant로 드러내는 것이 적절합니다.

6. `src/kundli/error.rs`
   - `DeriveError`에 최소 범위의 신규 variant를 추가합니다.
   - 권장: `MissingPlacementBody`, `InvalidHouseNumber(u8)`, `InvalidDashaSequenceLord(DashaLord)`.
   - 기존 수동 `Display`/`Error` 구현 패턴과 상위 `KundliError` 수렴 구조는 유지합니다.

## Scope notes
- `src/kundli` 기준 프로덕션 코드의 panic 경로는 현재 다음 파일들에 집중되어 있습니다:
  - `src/kundli/derive/pipeline/materialize.rs`
  - `src/kundli/derive/pipeline/house.rs`
  - `src/kundli/derive/house.rs`
  - `src/kundli/derive/dasha.rs`
- `astro/engine.rs`, `calculate.rs`, `config.rs`, `derive/sign.rs`, `derive/nakshatra.rs`, `derive/pipeline/mod.rs`의 `unwrap`은 조사 시점 기준 테스트 코드에만 있습니다.
- `src/kundli/astro/result.rs`의 `AstroResult::body(...)`는 `debug_assert_eq!`만 사용하며 프로덕션 panic 경로가 아니므로 이번 범위에 포함하지 않습니다.

## Reuse existing patterns
- `src/kundli/derive/pipeline/core.rs`의 단계별 `?` 전파 패턴을 그대로 재사용합니다.
- `src/kundli/error.rs`의 수동 `Display` 및 상위 에러 수렴 패턴을 그대로 따릅니다.
- `calculate_kundli_with_engine(...) -> Result<_, KundliError>`까지 이미 `From<DeriveError>` 전파가 준비되어 있으므로, derive 계층 panic 제거만으로 상위 공개 API는 자연스럽게 안전해집니다.

## Critical files
- `src/kundli/derive/pipeline/materialize.rs`
- `src/kundli/derive/pipeline/core.rs`
- `src/kundli/derive/pipeline/house.rs`
- `src/kundli/derive/house.rs`
- `src/kundli/derive/dasha.rs`
- `src/kundli/error.rs`

## Verification
- `cargo test` 실행
- `src/kundli/derive/pipeline/*`, `src/kundli/derive/house.rs`, `src/kundli/derive/dasha.rs` 관련 테스트가 기존 정상 케이스를 유지하는지 확인합니다.
- 신규/수정 테스트로 다음 panic 제거를 검증합니다:
  - body 누락 materialize 경로가 `DeriveError`를 반환하는지
  - invalid cusp/house number 경로가 panic 대신 `Err`로 끝나는지
  - dasha sequence lookup 실패가 panic 대신 `Err`로 끝나는지
- 상위 진입점 `calculate_kundli_with_engine`까지 확인해 derive 실패가 `KundliError::Derive`로 수렴하는지 검증합니다.
- 최종적으로 `src/kundli` 프로덕션 코드에 남은 `expect`/`unwrap`이 없는지 재검색으로 확인합니다.
