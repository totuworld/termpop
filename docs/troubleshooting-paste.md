# Troubleshooting: Cmd+Enter 후 텍스트 붙여넣기 안 되는 문제

## 증상

데몬 모드에서 Cmd+Shift+I → 텍스트 입력 → Cmd+Enter 시, 이전 앱에 텍스트가 붙여넣기되지 않음.

## 디버깅 과정

### 1차: clipboard.rs 타이밍 개선 시도

- `activate_app` 후 대기 시간 증가, 포커스 폴링 루프 추가, simulate_paste key-up/down 딜레이 추가
- 결과: 실패
- 디버그 로그 추가 후 확인 → 클립보드 설정 정상, 앱 전환 정상, simulate_paste 호출 정상

### 2차: simulate_paste CGEvent 방식 변경

- `HIDSystemState` → `CombinedSessionState`, `CGEventTapLocation::HID` → `AnnotatedSession`
- modifier 키 클리어 이벤트 추가
- 결과: 실패

### 3차: osascript 방식으로 변경

- CGEvent 대신 `osascript -e 'tell application "System Events" to keystroke "v" using command down'`
- 테스트 전 중단 (다른 접근으로 전환)

### 4차: 커밋 bisect

- `e2b7fa0` (원본) → 정상 동작
- `be51007` (WIP 커밋) → 실패
- 차이: `editor.rs` UI 변경 (힌트 라벨 attributed string, 텍스트 영문화), `Cargo.toml` 피처 추가

### 5차: editor.rs 변경 범위 좁히기

- `setAttributedStringValue` 제거, 원본 `setStringValue` 방식 복원 → 실패
- `Cargo.toml` 원본 복원 → 실패
- `build_hint_attributed_string` 함수 + `title` 필드 제거 → 실패
- 힌트 텍스트 한글→영문 변경만 적용 → 실패
- **원본 문자열 끝에 `!` 한 글자만 추가** → 실패

### 6차: 핵심 발견

원본 `e2b7fa0`을 아무 변경 없이 빌드하면 정상 동작하지만, **어떤 변경이든** 재컴파일하면 실패.

## 유력한 원인: macOS 접근성 권한 해시 검증

macOS는 접근성 권한(시스템 설정 > 개인정보 보호 및 보안 > 접근성)을 **바이너리의 코드 서명/해시** 기준으로 부여한다.

- 바이너리가 재컴파일되면 해시가 변경됨
- 접근성 권한이 자동으로 무효화됨
- `CGEvent::post()`는 접근성 권한 없이도 **에러 없이 조용히 실패**함
- 그래서 로그에는 모든 단계가 정상으로 보이지만 실제 키 이벤트는 전달되지 않음

### 왜 원본 빌드는 됐나?

처음 `e2b7fa0`을 빌드하고 `/usr/local/bin/termpop`에 복사한 시점에 접근성 권한을 부여했기 때문. 이후 재컴파일하면 바이너리 해시가 달라져서 권한이 풀림.

## 해결 방안 (검증 필요)

1. **재빌드 후 접근성 권한 재부여**: 매번 빌드 후 시스템 설정에서 termpop 토글 끄고 다시 켜기
2. **코드 서명**: `codesign`으로 바이너리에 서명하면 서명 기준으로 권한이 유지될 수 있음
3. **osascript 방식**: CGEvent 대신 AppleScript로 키 이벤트 전달 (접근성 권한 불필요할 수 있음)
4. **AXIsProcessTrusted() 체크 추가**: 런타임에 접근성 권한 여부를 확인하고 사용자에게 안내

## 현재 상태

**원인 확정**: macOS 접근성 권한이 바이너리 해시 기반으로 관리되어, 재컴파일 시 권한이 무효화됨.

**해결**: 재빌드 후 시스템 설정 > 접근성에서 termpop을 제거하고 다시 추가하면 정상 동작 확인됨.

`be51007` 커밋의 `editor.rs` 변경(attributed string, 영문화 등)은 문제 없음. 코드 변경이 아닌 바이너리 해시 변경이 원인이었음.
