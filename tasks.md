# TermPop 작업 체크리스트

## 작업 원칙
- TDD: 반드시 Red → Green → Refactor 순서
- 테스트 먼저 작성 → 실패 확인 → 통과하는 최소 코드 작성

## Phase 1: 코어 에디터 (MVP)
- [x] git init
- [x] Cargo 프로젝트 생성 + 의존성 (objc2, objc2-app-kit, clap)
- [x] editor.rs — NSWindow + NSTextView 팝업 생성
- [x] 키 바인딩 (Cmd+Enter 제출, Escape 취소)
- [x] main.rs — 팝업 열고 결과 stdout 출력 / 취소 시 exit 1
- [x] 검증: `result=$(cargo run) && echo "$result"`

## Phase 2: 데몬 + IPC
- [x] ipc.rs — Request/Response enum + 소켓 프로토콜 (4byte 길이 + JSON)
- [x] daemon.rs — tokio Unix socket 서버 + NSApp 이벤트 루프
- [x] CLI에서 소켓 존재 시 데몬 연결, 없으면 직접 팝업
- [x] `termpop daemon` / `termpop status` / `termpop stop` 명령어
- [x] 검증: 데몬 띄운 후 CLI로 소켓 통신 확인

## Phase 3: 글로벌 핫키 + 붙여넣기
- [x] global-hotkey 통합 (Cmd+Shift+E)
- [x] clipboard.rs — 클립보드 저장/복원
- [x] CGEvent로 Cmd+V 시뮬레이션
- [x] 이전 앱 포커스 복원
- [ ] 검증: 다른 앱에서 핫키 → 팝업 → 붙여넣기 확인

## Phase 4: 편의 기능
- [ ] --initial, --title CLI 옵션
- [ ] launchd plist 자동 생성 (daemon --install)
- [ ] 설정 파일 (~/.config/termpop/config.toml)
- [ ] 다크/라이트 모드 자동 대응

## Phase 5: UI/UX 폴리싱
- [ ] 말풍선 스타일 윈도우 (둥근 모서리, 꼬리 장식, 그림자)
- [ ] 커서 위치 기반 팝업 — 호출한 터미널의 커서 바로 위에 표시
- [ ] 글자 크기 조정 옵션 (--font-size, 설정 파일, 강의/프레젠테이션 용도)
