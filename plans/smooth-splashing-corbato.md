# Context

- 직전 리뷰에서 `src/kundli/derive/d1.rs`와 `src/kundli/derive/d9.rs`가 이미 존재하는 house helper를 쓰지 않고, WholeSign/cusp house 판정과 sign-index 계산을 각각 다시 구현하고 있다는 점이 확인됐다.
- 이 중복은 `src/kundli/derive/house.rs`의 규칙이 바뀔 때 D1/D9가 서로 다른 semantics로 drift할 위험을 만든다.
- 이번 변경의 목표는 `KundliDeriveInput` 기반 구조와 public API는 유지한 채, D1/D9의 house 계산을 `src/kundli/derive/house.rs`의 단일 구현으로 수렴시키는 것이다.
- 범위는 리뷰의 **개선 제안 두 건 반영**으로 제한한다. `calculate_kundli` 같은 상위 orchestration 추가, wrapper 간 shared intermediate 도입, 문서 정리는 이번 작업에서 제외한다.

# Recommended approach

## 1. D1의 중복 house 판정 로직 제거

수정 대상:
- `src/kundli/derive/d1.rs`

구현 방향:
- `src/kundli/derive/house.rs:23`의 `derive_house(...)`를 import해서 `derive_planet_placements_from_input(...)`의 house 계산에 직접 사용한다.
- 호출 형태는 `derive_house(body.longitude, input.ascendant.longitude, &input.house_cusps, config.house_system)`로 통일한다.
- 더 이상 필요 없는 아래 private helper는 삭제한다.
  - `derive_house_from_input(...)`
  - `derive_house_whole_sign(...)`
  - `derive_house_from_cusps(...)`
  - `longitude_to_sign_index(...)`
  - `is_in_house(...)`
- `derive_houses_from_input(...)`와 `derive_d1_chart_from_input(...)`는 유지한다. 이 함수들은 house **판정**이 아니라 `HouseResult` / `D1Chart` 조립 책임을 가지므로 그대로 두는 편이 맞다.

## 2. D9의 local WholeSign 계산을 공통 helper로 교체

수정 대상:
- `src/kundli/derive/d9.rs`

구현 방향:
- `derive_d9_planet_placements_from_input(...)`에서 house 계산을 `src/kundli/derive/house.rs:23`의 `derive_house(...)`로 위임한다.
- D9는 이미 `src/kundli/derive/d9.rs:16`에서 `HouseSystem::WholeSign` guard를 통과한 뒤에만 조립되므로, 실제 호출은 `derive_house(body.longitude, input.ascendant.longitude, &[], HouseSystem::WholeSign)`로 단순화한다.
- 더 이상 필요 없는 아래 local helper는 삭제한다.
  - `derive_d9_house(...)`
  - `longitude_to_sign_index(...)`
- `src/kundli/derive/input.rs:53`의 `to_navamsa()` 경로와 sidereal/WholeSign guard는 그대로 유지한다.

## 3. 동작 보존 중심으로 회귀 검증

관련 검증 파일:
- `tests/derive_d1.rs`
- `tests/derive_d9.rs`
- `tests/derive_dasha.rs`
- `tests/astro_smoke.rs`

검증 방향:
- D1 WholeSign/cusp 기반 house 결과가 기존과 동일한지 확인한다.
- D9 Navamsa 변환 후 WholeSign house 결과가 기존과 동일한지 확인한다.
- sidereal/WholeSign guard 및 invalid longitude/cusp count 에러 계약이 유지되는지 확인한다.
- public derive API를 타는 smoke test가 계속 통과하는지 확인한다.
- 의미론 변경이 목적이 아니므로, 테스트는 가능하면 기존 계약을 그대로 재사용하고 필요한 경우에만 회귀 assertion을 추가한다.

# Critical files to modify

- `src/kundli/derive/d1.rs`
- `src/kundli/derive/d9.rs`

# Existing functions and utilities to reuse

- `src/kundli/derive/house.rs:23` — `derive_house(...)`
- `src/kundli/derive/input.rs:36` — `KundliDeriveInput::from_astro(...)`
- `src/kundli/derive/input.rs:53` — `KundliDeriveInput::to_navamsa()`
- `src/kundli/derive/d1.rs:11` — `derive_lagna_from_input(...)`
- `src/kundli/derive/d1.rs:53` — `derive_houses_from_input(...)`
- `src/kundli/derive/d1.rs:107` — `derive_d1_chart_from_input(...)`

# Verification

1. `cargo test --test derive_d1`
2. `cargo test --test derive_d9`
3. `cargo test --test astro_smoke --test derive_dasha`
4. `cargo test`
5. `cargo clippy --all-targets --all-features -- -D warnings`

검증 시 특히 확인할 점:
- D1/D9 planet house 번호가 기존 테스트 기대값과 동일한지
- non-WholeSign D1의 cusp validation contract가 그대로인지
- D9의 `UnsupportedD9HouseSystem` / `UnsupportedZodiac` contract가 그대로인지
- smoke test에서 body order와 finite longitude 보장이 깨지지 않는지
