# Astro 레이어 변경 계획

## Context
- 이후 derive 레이어에서 실제 날짜 기반 dasha를 계산하려면 `AstroResult` 안에 출생 시점(`jd_ut`)이 남아 있어야 한다.
- 지금은 `AstroRequest.jd_ut`가 `AstroResult`로 전달되지 않아 astro 계산 시점을 결과에서 복원할 수 없다.

## 현재 작업할 부분
- `src/kundli/astro/result.rs`의 `AstroMeta`에 `jd_ut: f64` 추가
- `src/kundli/astro/engine.rs`에서 `request.jd_ut`를 `result.meta.jd_ut`로 전달
- `src/kundli/astro/engine.rs` 테스트와 `tests/astro_smoke.rs`에 해당 값 검증 추가

## 이 작업이 필요한 이유
- astro 레이어는 이미 행성 longitude, ascendant, house cusps 같은 **공간 정보**는 충분히 계산하고 있다.
- D1, D9 같은 차트는 이 공간 정보만으로도 derive 가능하다. 둘 다 기본적으로 특정 시점의 longitude를 sign/divisional sign으로 해석하는 정적 차트이기 때문이다.
- 반면 Vimshottari Dasha는 단순한 배치 해석이 아니라 **언제 시작되고 언제 끝나는지**를 계산해야 하는 시간축 정보다.
- Moon longitude로 dasha의 순서와 비율은 구할 수 있어도, 실제 `start/end` 날짜를 만들려면 그 비율을 어느 출생 시점에 적용할지 기준 시각이 꼭 필요하다.
- 지금 구조에서는 `AstroRequest.jd_ut`가 `AstroResult`로 전달되지 않아 derive 단계가 계산 기준 시각을 알 수 없다.
- 그래서 이번 astro 변경은 future derive 전체를 넓히기 위한 것이 아니라, 특히 **날짜가 있는 dasha 결과**를 가능하게 만드는 최소 선행 작업이다.
