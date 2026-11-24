# 정답 파일

Kotlin 원본(FGA-Modified)이 내보낸 값입니다. Rust 쪽 테스트가 이 값과 같은지 비교합니다. 손으로 고치지 않습니다.

- `locations/<서버>_<가로>x<세로>.json`: 서버와 화면 크기마다 `Locations`의 모든 `Location`/`Region`(스크립트 좌표, 화면 좌표, 이미지 좌표). 속성은 리플렉션으로 찾으므로 FGA-Modified에 속성이 늘면 코드 수정 없이 파일에 나타납니다. 계산에 실패한 속성은 `{"error": "예외 클래스"}`로 기록하고 파일의 `errors`에 나열합니다.

  `battle.master`의 다섯 값(`locate(Skill.Master.A/B/C)`, `masterSkillOpenClick`, `stageCountRegion`)은 화면에서 전투 메뉴 버튼을 찾아 정해집니다. 내보내기는 가짜 화면(`golden/GoldenComponent.kt`)으로 실행하며, `values`의 이 다섯 값은 아무것도 없는 검은 화면에서 나온 값입니다. 파일의 `masterOutcomes`는 같은 다섯 값을 두 경우로 따로 적습니다.
  - `buttonNotFound`: 검은 화면. 버튼을 못 찾아 기본 위치(`Location(-298, 0).xFromRight()`)를 씁니다.
  - `buttonFound`: 그 서버의 `battle_menu.png`를 코드가 기본 위치로 삼는 곳에 붙인 화면. 찾은 버튼의 중심을 씁니다.
  - `defaultOffsetLogged`: 코드가 기본 위치를 썼다고 기록했는지(`ScriptLog.DefaultMasterOffset`). 두 경우의 좌표가 1픽셀 안쪽으로 같거나 달라도 어느 경우인지 구별됩니다.
  화면 캡처는 쓰지 않고, 템플릿 이미지는 서브모듈의 것을 그대로 씁니다.
- `prefs/<이름>.json`: `core/testdata/fga/`의 입력 파일마다 전투 설정을 읽은 결과. 이 값은 Android의 `prefs` 모듈이 아니라 `host/.../prefs`의 손으로 옮긴 읽기 코드가 낸 것입니다.

다시 만들기: `git submodule update --init` 후 `./gradlew :host:golden`. 두 번 실행해도 파일은 바뀌지 않아야 합니다.
