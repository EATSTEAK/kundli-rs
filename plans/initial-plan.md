# Rust Kundli 라이브러리 최종 구현 계획서

## 1. 목표

출생 정보와 계산 설정을 입력받아 아래 결과를 반환하는 Rust 라이브러리를 구현한다.

- D1(Rashi) 차트
- D9(Navamsa) 차트
- Vimshottari Dasha
- Lagna
- 행성별 sign / house / nakshatra / pada
- 계산 메타데이터

## 2. 상위 구조

```text
src/
├─ lib.rs
├─ kundli/
│  ├─ mod.rs
│  ├─ calculate.rs
│  ├─ config.rs
│  ├─ input.rs
│  ├─ model.rs
│  ├─ error.rs
│  ├─ normalize.rs
│  ├─ astro/
│  │  ├─ mod.rs
│  │  ├─ engine.rs
│  │  ├─ request.rs
│  │  ├─ result.rs
│  │  ├─ ephemeris.rs
│  │  └─ error.rs
│  └─ derive/
│     ├─ mod.rs
│     ├─ sign.rs
│     ├─ house.rs
│     ├─ nakshatra.rs
│     ├─ d1.rs
│     ├─ d9.rs
│     └─ dasha.rs
```

## 3. 핵심 흐름

전체 계산은 4단계로 고정한다.

1. **입력 정규화**
   - 출생 시각, 위치, 설정값 정리
   - Julian day, 위도/경도, precision 확정

2. **천문 계산**
   - `kundli::astro`에서 `swiss-eph` 사용
   - graha 위치, ascendant, house 정보 계산

3. **Kundli 해석**
   - longitude를 sign / house / nakshatra / pada로 변환
   - D1 생성
   - D9 생성
   - Dasha 생성

4. **최종 결과 조립**
   - meta + D1 + D9 + Dasha + warnings 반환

## 4. 공개 인터페이스

### 최상위 함수

```rust
pub fn calculate_kundli(
    input: BirthInput,
    config: KundliConfig,
) -> Result<KundliResult, KundliError>;
```

## 5. 입력 인터페이스

```rust
pub struct BirthInput {
    pub birth_datetime: BirthDateTimeInput,
    pub location: LocationInput,
    pub subject_name: Option<String>,
}
```

```rust
pub enum BirthDateTimeInput {
    OffsetDateTime(time::OffsetDateTime),
    ApproximateDate {
        date: time::Date,
    },
}
```

```rust
pub enum LocationInput {
    Coordinates {
        lat: f64,
        lon: f64,
    },
    PlaceName {
        name: String,
        country_code: Option<String>,
    },
}
```

## 6. 설정 인터페이스

```rust
pub struct KundliConfig {
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub include_d9: bool,
    pub include_dasha: bool,
}
```

## 7. astro 모듈 인터페이스

### 요청

```rust
pub struct AstroRequest {
    pub jd_ut: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub planets: Vec<AstroBody>,
}
```

### 결과

```rust
pub struct AstroResult {
    pub bodies: Vec<AstroBodyPosition>,
    pub ascendant_longitude: f64,
    pub mc_longitude: f64,
    pub house_cusps: Vec<f64>,
    pub meta: AstroMeta,
}
```

### 엔진

```rust
pub trait AstroEngine {
    fn calculate(&self, request: &AstroRequest) -> Result<AstroResult, AstroError>;
}
```

```rust
pub struct SwissEphAstroEngine;
```

역할:

- Swiss 기반 행성 위치 계산
- ascendant / house cusps 계산
- raw astronomy result 반환

## 8. derive 모듈 인터페이스

### 공통 해석 함수

```rust
pub fn derive_lagna(astro: &AstroResult) -> Result<LagnaResult, DomainError>;

pub fn derive_planet_placements(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<PlanetPlacement>, DomainError>;

pub fn derive_houses(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<HouseResult>, DomainError>;
```

### D1

```rust
pub fn derive_d1_chart(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<D1Chart, DomainError>;
```

### D9

```rust
pub fn derive_d9_chart(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<D9Chart, DomainError>;
```

### Dasha

```rust
pub fn derive_vimshottari_dasha(
    astro: &AstroResult,
) -> Result<VimshottariDasha, DomainError>;
```

## 9. 최종 결과 인터페이스

```rust
pub struct KundliResult {
    pub meta: CalculationMeta,
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
    pub houses: Vec<HouseResult>,
    pub d1: D1Chart,
    pub d9: Option<D9Chart>,
    pub dasha: Option<VimshottariDasha>,
    pub warnings: Vec<CalculationWarning>,
}
```

## 10. 주요 로직

### D1

- astro 결과의 graha longitude 사용
- sign / house / nakshatra / pada 계산
- Lagna와 12 house 구조 생성

### D9

- 각 graha longitude를 Navamsa 규칙으로 변환
- D9 sign 배치 생성
- 별도 D9 chart 모델로 반환

### Dasha

- Moon longitude 기반
- Moon nakshatra 및 잔여 구간 계산
- Vimshottari sequence 생성
- 현재 / 다음 dasha timeline 반환

## 11. 내부 orchestration

`calculate_kundli()`는 아래 순서만 책임진다.

```rust
normalize_birth_input(...)
build_astro_request(...)
engine.calculate(...)
derive_lagna(...)
derive_planet_placements(...)
derive_houses(...)
derive_d1_chart(...)
derive_d9_chart(...)          // include_d9 == true
derive_vimshottari_dasha(...) // include_dasha == true
assemble_result(...)
```

## 12. 구현 우선순위

### Phase 1

- 입력/설정/결과 모델
- normalize
- astro 계산
- D1

### Phase 2

- D9

### Phase 3

- Dasha

### Phase 4

- 에러 정리
- precision/warnings
- 테스트 고정

## 13. 최종 원칙

- `astro`는 raw astronomical values만 계산한다.
- `derive`는 Kundli 의미 해석만 담당한다.
- 공개 인터페이스는 `calculate_kundli()` 하나로 단순화한다.
- D9와 Dasha는 옵션이지만 결과 모델에는 정식 포함한다.
- D1은 항상 필수다.
