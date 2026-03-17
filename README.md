# TermPop

터미널에서 호출하는 네이티브 macOS 팝업 텍스트 에디터.

AI 프롬프트, 커밋 메시지, 복잡한 명령어 등 여러 줄 텍스트를 터미널에서 편하게 작성하고 stdout으로 받거나, 글로벌 핫키로 어디서든 호출해서 바로 붙여넣기까지.

```
┌─────────────────────────────────────────┐
│                                         │
│  여기에 텍스트를 입력하세요...            │
│                                         │
│  Enter: 줄바꿈 │ ⌘+Enter: 제출 │ Esc: 취소 │
└─────────────────────────────────────────┘
```

## 데모

<video src="assets/termpop_introduce.mp4" controls width="100%"></video>

## 특징

- Atom One Dark/Light 테마 (⌃T로 즉시 전환)
- 모노스페이스 폰트, 둥근 모서리, 테두리
- 커서 위치에 팝업 표시
- 데몬 모드 — 글로벌 핫키로 어디서든 호출
- 결과를 클립보드에 복사 후 자동 붙여넣기 (Cmd+V 시뮬레이션)
- 이전 앱 포커스 자동 복원
- 설정 파일로 핫키, 폰트 크기, 테마 등 커스터마이징
- 변경한 폰트 크기와 테마는 자동 저장

## 설치

### 소스에서 빌드

```bash
# 필수: macOS 10.13+, Rust 1.70+, Xcode Command Line Tools
git clone https://github.com/totuworld/termpop.git
cd termpop
cargo build --release
cp target/release/termpop /usr/local/bin/
```

### cargo install

```bash
cargo install --path .
```

## 사용법

### 기본 — 팝업 열고 결과 받기

```bash
# 팝업 열기, 제출하면 stdout으로 출력
result=$(termpop)
echo "$result"

# 초기 텍스트와 제목 지정
termpop --initial "기존 텍스트" --title "커밋 메시지"

# 폰트 크기 지정
termpop --font-size 20
```

### 데몬 모드 — 글로벌 핫키

```bash
# 데몬 시작 (기본 핫키: Cmd+Shift+I)
termpop daemon

# 로그인 시 자동 시작 등록
termpop daemon --install

# 자동 시작 해제
termpop daemon --uninstall

# 상태 확인
termpop status

# 데몬 종료
termpop stop
```

데몬 모드에서 핫키를 누르면:
1. 팝업이 열림
2. 텍스트 작성 후 ⌘+Enter로 제출
3. 결과가 클립보드에 복사되고 이전 앱에 자동 붙여넣기

### 데몬 자동 시작

글로벌 핫키를 사용하려면 데몬이 실행 중이어야 합니다. 매번 수동으로 띄우기 번거로우니 자동 시작을 설정하세요.

#### 방법 1: launchd (시스템 레벨)

```bash
termpop daemon --install
```

macOS 로그인 시 자동으로 데몬이 시작됩니다. 해제하려면 `termpop daemon --uninstall`.

#### 방법 2: .zshrc (셸 레벨)

`~/.zshrc` (또는 `~/.bashrc`)에 다음을 추가합니다:

```bash
# TermPop 데몬 자동 시작
if ! termpop status &>/dev/null; then
  termpop daemon &>/dev/null &
  disown
fi
```

터미널을 열 때 데몬이 꺼져 있으면 자동으로 백그라운드에서 시작합니다.

## 단축키

| 키 | 동작 |
|---|---|
| `Enter` | 줄바꿈 |
| `⌘+Enter` | 제출 |
| `Esc` | 취소 |
| `⌃+` | 폰트 크기 증가 |
| `⌃-` | 폰트 크기 감소 |
| `⌃0` | 폰트 크기 기본값 복원 |
| `⌃T` | 다크/라이트 테마 전환 |

## 설정

`~/.config/termpop/config.toml`

```toml
hotkey = "Cmd+Shift+I"
font_size = 14.0
window_width = 600.0
window_height = 300.0
theme = "dark"
```

폰트 크기와 테마는 팝업 안에서 변경하면 자동으로 저장됩니다.

## 아키텍처

```
src/
├── main.rs        # 엔트리포인트, CLI 라우팅
├── cli.rs         # clap 기반 CLI 파싱
├── editor.rs      # NSWindow + NSTextView 네이티브 UI
├── daemon.rs      # tokio Unix socket 서버 + 이벤트 루프
├── ipc.rs         # 4byte 길이 + JSON 프로토콜
├── clipboard.rs   # 클립보드 저장/복원 + CGEvent 붙여넣기
├── config.rs      # TOML 설정 파일 관리
└── launchd.rs     # launchd plist 생성/제거
```

## 권한

데몬 모드에서 글로벌 핫키와 자동 붙여넣기를 사용하려면 macOS 접근성 권한이 필요합니다.

시스템 설정 → 개인정보 보호 및 보안 → 접근성 → termpop 허용

### ⚠️ 재빌드 후 접근성 권한 재등록 필요

macOS는 접근성 권한을 **바이너리 해시** 기준으로 관리합니다. `cargo build`로 재컴파일하면 바이너리가 변경되어 기존 접근성 권한이 무효화됩니다. 이때 CGEvent(Cmd+V 시뮬레이션)가 **에러 없이 조용히 실패**하므로, 붙여넣기가 안 되는데 로그에는 정상으로 보입니다.

재빌드 후 붙여넣기가 안 되면:

```bash
# 1. 데몬 중지
termpop daemon --uninstall
pkill -9 -f "termpop"

# 2. 새 바이너리 복사
sudo cp target/release/termpop /usr/local/bin/termpop

# 3. 접근성 권한 재등록
#    시스템 설정 → 개인정보 보호 및 보안 → 접근성(손쉬운사용)에서
#    termpop을 `-` 버튼으로 제거 후 `+`로 다시 추가

# 4. 데몬 재시작
termpop daemon --install
```


## 라이선스

MIT
