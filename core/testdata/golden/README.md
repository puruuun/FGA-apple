# 정답 파일

Kotlin 원본(FGA-Modified)이 내보낸 값입니다. Rust 쪽 테스트가 이 값과 같은지 비교합니다. 손으로 고치지 않습니다.

- `locations/<서버>_<가로>x<세로>.json`: 서버와 화면 크기마다 `Locations`의 모든 `Location`/`Region`(스크립트 좌표, 화면 좌표, 이미지 좌표). 속성은 리플렉션으로 찾으므로 FGA-Modified에 속성이 늘면 코드 수정 없이 파일에 나타납니다. 계산에 실패한 속성은 `{"error": "예외 클래스"}`로 기록하고 파일의 `errors`에 나열합니다.
- `prefs/<이름>.json`: `core/testdata/fga/`의 입력 파일마다 전투 설정을 읽은 결과. 이 값은 Android의 `prefs` 모듈이 아니라 `host/.../prefs`의 손으로 옮긴 읽기 코드가 낸 것입니다.

다시 만들기: `git submodule update --init` 후 `./gradlew :host:golden`. 두 번 실행해도 파일은 바뀌지 않아야 합니다.
