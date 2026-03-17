# CLAUDE.md

## 프로젝트 개요
TermPop — 터미널에서 호출하는 네이티브 macOS 팝업 텍스트 에디터 (Rust + AppKit)

## 빌드 & 적용

```bash
# 릴리스 빌드
cargo build --release

# 바이너리 설치
sudo cp target/release/termpop /usr/local/bin/termpop
```

### 재빌드 후 데몬 적용 절차
macOS는 접근성 권한을 바이너리 해시 기준으로 관리한다. 재빌드하면 권한이 무효화되어 CGEvent(Cmd+V 붙여넣기)가 조용히 실패한다.

```bash
# 1. 데몬 중지
termpop daemon --uninstall
pkill -9 -f "termpop"

# 2. 새 바이너리 복사
sudo cp target/release/termpop /usr/local/bin/termpop

# 3. 접근성 권한 재등록
#    시스템 설정 → 개인정보 보호 및 보안 → 접근성 → termpop을 `-`로 제거 후 `+`로 다시 추가

# 4. 데몬 재시작
termpop daemon --install
```

## 테스트

```bash
cargo test
```

## 프로젝트 구조

```
src/
├── main.rs        # 엔트리포인트, CLI 라우팅
├── cli.rs         # clap 기반 CLI 파싱
├── editor.rs      # NSWindow + NSTextView 네이티브 UI
├── daemon.rs      # tokio Unix socket 서버 + 이벤트 루프
├── ipc.rs         # 4byte 길이 + JSON 프로토콜
├── clipboard.rs   # 클립보드 저장/복원 + CGEvent 붙여넣기
├── config.rs      # TOML 설정 파일 (~/.config/termpop/config.toml)
└── launchd.rs     # launchd plist 생성/제거
```

## 주요 의존성
- `objc2` + `objc2-app-kit` — 네이티브 macOS UI (NSTextView/NSWindow)
- `tokio` — 비동기 Unix socket IPC
- `global-hotkey` — 글로벌 단축키
- `core-graphics` — CGEvent (Cmd+V 시뮬레이션)
- `clap` — CLI 파싱
- `serde` + `serde_json` + `toml` — 직렬화/설정

## 편집기 동작 참고
- NSTextView가 텍스트 편집 전체를 담당 (undo/redo, 복사/붙여넣기 등 내장)
- 이벤트 루프에서 특정 키만 가로챔 (Cmd+Enter, Esc, Ctrl+/-, Ctrl+0, Ctrl+T)
- 나머지 키 이벤트는 `app.sendEvent()`로 NSTextView에 그대로 전달
