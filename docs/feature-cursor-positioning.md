# Feature: 터미널 윈도우 기반 팝업 포지셔닝

## 배경

기존에는 `NSEvent::mouseLocation()`으로 마우스 커서 위치에 팝업을 띄웠다. 이제 **포커스된 터미널 윈도우의 하단 중앙**에 팝업을 배치한다.

## 구현

### 접근 방식

`CGWindowListCopyWindowInfo`로 frontmost 앱의 윈도우 bounds를 CG 좌표로 직접 가져온다. AX API의 텍스트 커서 위치(`AXBoundsForRange`)는 터미널별 지원이 불안정하고 멀티 모니터 좌표 변환 문제가 있어 채택하지 않았다.

### 포지셔닝 로직

```
┌─── 터미널 윈도우 ───────────────┐
│                                  │
│  $ ls -la                        │
│  $ git status                    │
│  $ termpop█                      │
│                                  │
│  ┌── TermPop 팝업 ──────────┐   │  ← 윈도우 하단에서 80px 위
│  │                           │   │     수평: 윈도우 중앙 정렬
│  │                           │   │
│  └───────────────────────────┘   │
└──────────────────────────────────┘
```

- Y: 윈도우 하단에서 `WINDOW_BOTTOM_MARGIN`(80px) 위
- X: 윈도우 수평 중앙 - 팝업 너비/2
- 실패 시 마우스 커서 위치로 폴백

### 변경 파일

- `src/ax_position.rs` — `CGWindowListCopyWindowInfo` 기반 `get_frontmost_window_bounds()` 구현
- `src/editor.rs` — 윈도우 하단 중앙 포지셔닝 로직
- `src/main.rs` — `mod ax_position` 추가
- `Cargo.toml` — `core-foundation = "0.10"` 의존성 추가

### 터미널 호환성

모든 터미널에서 동작 확인:
- Terminal.app ✅
- Ghostty ✅
- WezTerm ✅
- tmux ✅
