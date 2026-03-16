# TermPop — 터미널 멀티라인 입력 팝업 도구

## Context

터미널에서 긴 텍스트나 여러 줄 입력이 필요할 때 (AI 에이전트 프롬프트, 커밋 메시지, 복잡한 명령어 등) 줄바꿈, 선택, 복사/붙여넣기가 불편하다. 이 문제는 어떤 터미널을 쓰든 동일하게 존재한다.

cmux-tb 프로젝트의 TextBox 기능에서 착안하여, **터미널에 독립적인 범용 네이티브 팝업 텍스트 에디터**를 만든다. 단축키 하나로 팝업을 띄우고, 편하게 입력한 뒤, 결과를 호출한 터미널로 돌려보낸다.

## 기술 스택

- **언어**: Rust
- **네이티브 UI**: `objc2` + `objc2-app-kit` (NSTextView/NSWindow)
- **글로벌 핫키**: `global-hotkey` (0.7)
- **IPC**: Unix socket (tokio)
- **직렬화**: serde_json
- **타겟**: macOS 우선 (아키텍처는 크로스플랫폼 대비)

## 아키텍처

```
┌─────────────────────────────────────────────┐
│                  termpop                     │
│                                              │
│  ┌──────────┐    Unix Socket    ┌─────────┐ │
│  │ CLI 모드  │ ◄──────────────► │ 데몬 모드 │ │
│  │ (termpop) │                  │(termpopd)│ │
│  └──────────┘                   └────┬────┘ │
│       │                              │      │
│       │ stdout                  글로벌 핫키  │
│       ▼                              │      │
│  호출한 셸로                    ┌────▼────┐ │
│  텍스트 반환                    │ NSWindow │ │
│                                 │+NSTextView│
│                                 └─────────┘ │
└─────────────────────────────────────────────┘
```

## 두 가지 사용 모드

### 모드 1: CLI 직접 호출
```bash
# 셸에서 직접 호출 — 팝업 열리고, 입력 후 stdout으로 반환
result=$(termpop)
echo "$result"

# 초기 텍스트 전달
termpop --initial "기존 텍스트"

# 파이프와 조합
termpop | pbcopy
git commit -m "$(termpop --title 'Commit Message')"
```

동작 흐름:
1. `termpop` CLI 실행
2. 데몬이 떠있으면 소켓으로 요청, 없으면 직접 NSWindow 생성
3. 네이티브 팝업 표시 (NSTextView)
4. 사용자가 Cmd+Enter로 확인 또는 Escape로 취소
5. 확인 시 텍스트를 stdout으로 출력, 취소 시 exit code 1

### 모드 2: 글로벌 핫키 (데몬)
```bash
# 데몬 시작 (로그인 시 자동 시작 가능)
termpop daemon

# 또는 launchd plist로 등록
termpop daemon --install
```

동작 흐름:
1. `termpopd` 데몬이 글로벌 핫키 등록 (기본: Cmd+Shift+E)
2. 어떤 앱에서든 핫키 누르면 팝업 표시
3. 입력 완료 시 클립보드에 복사 + 이전 앱으로 Cmd+V 시뮬레이션
4. (선택) bracket paste로 직접 터미널에 전송 가능

## 프로젝트 구조

```
termpop/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 엔트리포인트 (clap)
│   ├── daemon.rs            # 데몬 모드 (소켓 서버 + 핫키 + 이벤트 루프)
│   ├── editor.rs            # 네이티브 팝업 에디터 (NSWindow + NSTextView)
│   ├── ipc.rs               # Unix 소켓 프로토콜 (Request/Response)
│   ├── clipboard.rs         # 클립보드 + 붙여넣기 시뮬레이션
│   └── platform/
│       ├── mod.rs            # 플랫폼 trait 정의
│       └── macos.rs          # macOS 구현 (AppKit, CGEvent)
└── resources/
    └── Info.plist            # Accessibility 권한 설명
```

## 핵심 구현 상세

### 1. 네이티브 에디터 윈도우 (`editor.rs`)

`objc2-app-kit`으로 구현:
- `NSWindow` (floating panel, level = `.floating`)
- `NSScrollView` > `NSTextView` (plain text, 줄바꿈 허용)
- 하단 버튼 바: "Cancel (Esc)" / "Submit (⌘↵)"
- 윈도우 크기: 600x300, 리사이즈 가능
- 터미널 폰트 사용 (SF Mono 또는 시스템 모노스페이스)
- 다크/라이트 모드 자동 대응

키 바인딩:
- `Cmd+Enter` → 제출
- `Escape` → 취소
- `Cmd+A/C/V/X/Z` → 표준 텍스트 편집 (NSTextView 기본 제공)
- `Enter` → 줄바꿈 (일반 텍스트 편집)

### 2. IPC 프로토콜 (`ipc.rs`)

소켓 경로: `~/Library/Application Support/termpop/daemon.sock`

```rust
// 4바이트 길이 프리픽스 + JSON
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Request {
    Open { initial_text: Option<String>, title: Option<String> },
    Status,
    Shutdown,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    Result { text: String, cancelled: bool },
    Status { running: bool, hotkey: String },
    Ok,
}
```

### 3. 글로벌 핫키 + 붙여넣기 시뮬레이션 (`daemon.rs`, `clipboard.rs`)

```
핫키 감지 → 현재 포커스 앱 기억 → 팝업 표시 → 입력 완료
→ 클립보드에 텍스트 복사 → 이전 앱 활성화 → Cmd+V 시뮬레이션
→ 클립보드 원래 내용 복원
```

Cmd+V 시뮬레이션은 `CGEvent`로 구현:
```rust
// core-graphics crate
let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
let cmd_down = CGEvent::new_keyboard_event(source, 0x37, true);  // Cmd
let v_down = CGEvent::new_keyboard_event(source, 0x09, true);    // V
v_down.set_flags(CGEventFlags::CGEventFlagMaskCommand);
v_down.post(CGEventTapLocation::HID);
```

### 4. CLI 모드 (`main.rs`)

```rust
#[derive(Parser)]
#[command(name = "termpop")]
enum Cli {
    /// 팝업 에디터 열기 (기본 동작)
    #[command(default)]
    Open {
        #[arg(long)]
        initial: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// 데몬 모드 시작
    Daemon {
        #[arg(long)]
        install: bool,  // launchd 등록
    },
    /// 데몬 상태 확인
    Status,
    /// 데몬 종료
    Stop,
}
```

CLI 실행 시:
1. 데몬 소켓 존재 확인
2. 있으면 → 소켓으로 `Open` 요청 전송, 응답 대기
3. 없으면 → 직접 NSApplication 루프 시작, 팝업 표시, 결과 stdout 출력

## 구현 순서

### Phase 1: 코어 에디터 (MVP)
1. Cargo 프로젝트 생성, 의존성 설정
2. `editor.rs` — NSWindow + NSTextView 팝업 구현
3. `main.rs` — `termpop` 실행 시 팝업 열고 stdout 반환
4. 키 바인딩 (Cmd+Enter 제출, Escape 취소)

### Phase 2: 데몬 + IPC
5. `ipc.rs` — 소켓 프로토콜 정의
6. `daemon.rs` — tokio 기반 소켓 서버
7. CLI에서 데몬 연결 로직 추가
8. `termpop daemon` 명령어

### Phase 3: 글로벌 핫키 + 붙여넣기
9. `global-hotkey` 통합
10. `clipboard.rs` — 클립보드 저장/복원 + CGEvent Cmd+V 시뮬레이션
11. 이전 앱 포커스 복원

### Phase 4: 편의 기능
12. `--initial`, `--title` 옵션
13. launchd plist 자동 생성 (`daemon --install`)
14. 설정 파일 (~/.config/termpop/config.toml) — 핫키, 폰트, 윈도우 크기
15. 다크/라이트 모드 자동 대응

## 검증 방법

- Phase 1 완료 후: `result=$(cargo run) && echo "$result"` 로 팝업 → 입력 → stdout 확인
- Phase 2 완료 후: `cargo run -- daemon &` 후 `cargo run` 으로 소켓 통신 확인
- Phase 3 완료 후: 다른 앱(터미널)에서 글로벌 핫키 → 팝업 → 텍스트가 터미널에 붙여넣기 되는지 확인
- Accessibility 권한 요청 다이얼로그가 정상 표시되는지 확인 (CGEvent 사용 시 필요)
