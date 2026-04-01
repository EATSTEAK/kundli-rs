## Context
사용자가 생성할 차트를 `&[KnownChart]`처럼 명시적으로 선택할 수 있게 하고, 기존 D1/D9 중심 구조를 Moon chart 같은 reference-variant chart까지 확장할 수 있도록 derive orchestration을 일반화한다. 목표는 기존 `ChartPipeline` 조합과 개별 derive 함수는 유지하면서, 상위 API가 불리언 플래그(`include_d9`, `include_dasha`) 대신 선택된 chart enum 집합을 기준으로 결과를 조립하게 만드는 것이다. 사용자 선택에 따라 **명시한 차트만 생성하는 완전 선택형**으로 동작한다.

## Recommended approach
1. `KnownChart` 공개 enum 추가
   - 파일: `src/kundli/config.rs` 또는 공개 re-export가 자연스러운 위치
   - 항목: 최소 `D1`, `D9`, `Moon`, `VimshottariDasha`
   - `Copy + Eq + Hash` 등 기존 공개 enum 패턴에 맞춘다.
   - 필요하면 `KnownChart::ALL` 상수도 함께 둔다.

2. `KundliConfig`를 chart selection 기반으로 전환
   - 파일: `src/kundli/config.rs`
   - `include_d9`, `include_dasha` 제거 후 `charts: Vec<KnownChart>` 추가
   - `new`/`from_request` 기본값은 빈 목록으로 둔다.
   - `with_charts(&[KnownChart]) -> Self` 추가
   - 기존 `with_include_d9`, `with_include_dasha` 호출부는 새 API로 치환한다.

3. pipeline alias와 Moon chart derive 추가
   - 파일: `src/kundli/derive/pipeline/core.rs`, `src/kundli/derive/pipeline/mod.rs`, `src/kundli/derive/d1.rs`, `src/kundli/derive/d9.rs`, 신규 `src/kundli/derive/moon.rs`
   - 재사용할 기존 구성요소:
     - `ChartPipeline` (`src/kundli/derive/pipeline/core.rs`)
     - `LagnaReference`, `MoonReference` (`src/kundli/derive/pipeline/reference.rs`)
     - `WholeSignHouseTransform`, `CuspBasedHouseTransform`
   - `D1`/`D9`/`Moon`에 대해 읽기 좋은 alias 또는 전용 생성 함수를 추가해 pipeline intent를 드러낸다.
   - `derive_moon_chart_result`는 `MoonReference` 기반으로 D1과 같은 materialization shape를 반환하게 한다.

4. 결과 타입 확장
   - 파일: `src/kundli/model.rs`
   - `MoonChart` 타입 alias 수준의 얇은 래퍼 추가(패턴은 `D1Chart`, `D9Chart`와 동일)
   - `KundliResult`는 완전 선택형에 맞춰 다음처럼 option 필드 기반으로 조정:
     - `d1: Option<D1Chart>`
     - `d9: Option<D9Chart>`
     - `moon: Option<MoonChart>`
     - `dasha: Option<VimshottariDasha>`
   - top-level convenience mirror(`lagna`, `planets`, `houses`)는 더 이상 항상 보장되지 않으므로 같이 optional로 바꾸거나 제거해야 한다. 추천은 `Option`으로 전환해 API churn을 최소화하는 것이다.

5. orchestration을 enum dispatch 구조로 변경
   - 파일: `src/kundli/calculate.rs`
   - `config.charts`를 순회하거나 `contains`로 검사해 필요한 derive 함수만 실행
   - `KnownChart::D1` → `derive_d1_chart_result`
   - `KnownChart::D9` → `derive_d9_chart_result`
   - `KnownChart::Moon` → `derive_moon_chart_result`
   - `KnownChart::VimshottariDasha` → `derive_vimshottari_dasha`
   - convenience mirror는 D1이 있을 때만 채운다.
   - request/config 일치 검증 로직은 그대로 유지한다.

## Critical files
- `src/kundli/config.rs`
- `src/kundli/calculate.rs`
- `src/kundli/model.rs`
- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`
- `src/kundli/derive/moon.rs`
- `src/kundli/derive/pipeline/core.rs`
- `src/kundli/derive/pipeline/mod.rs`
- `src/kundli/derive/pipeline/reference.rs`

## Verification
1. 단위 테스트 갱신/추가
   - `config.rs`: `with_charts(&[...])`가 선택 목록을 정확히 저장하는지
   - `calculate.rs`: 선택한 chart만 결과에 채워지는지, D1 미선택 시 mirror가 비어 있는지
   - `derive/moon.rs`: Moon reference 기준으로 house/planet anchoring이 기대대로 바뀌는지
2. 기존 derive 테스트 보정
   - `tests/derive_d1.rs`, `tests/derive_d9.rs`, `tests/derive_dasha.rs`, `tests/astro_smoke.rs`
   - 새 `tests/derive_moon.rs` 또는 `calculate.rs` 테스트로 Moon chart 경로 검증
3. 실행
   - 관련 Rust 테스트 실행으로 end-to-end 확인 (`cargo test` 중심)
   - 필요 시 `calculate_kundli_with_engine`의 stub engine 테스트로 chart selection 조합별 결과를 검증