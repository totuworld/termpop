# Undo/Redo 기능 계획

## 현재 상태 분석

### NSTextView 기본 Undo/Redo
`NSTextView`는 macOS AppKit이 제공하는 네이티브 텍스트 편집 컴포넌트로, **내장 UndoManager를 통해 Cmd+Z (undo)와 Cmd+Shift+Z (redo)를 이미 지원**한다.

현재 `editor.rs`의 이벤트 루프에서 `Cmd+Z`를 가로채지 않으므로, `app.sendEvent(event)`를 통해 NSTextView로 전달되어 기본 undo/redo가 동작하는 상태다.

### 확인이 필요한 사항
- `NSTextView`의 `allowsUndo` 속성이 기본값 `true`인지 확인 (문서상 NSTextView는 기본 true)
- 실제로 Cmd+Z / Cmd+Shift+Z가 정상 동작하는지 수동 테스트 필요

## 조사 결과 요약

### 접근 방식 비교

| 방식 | 설명 | 적합도 |
|------|------|--------|
| **NSTextView 내장 UndoManager 활용** | AppKit이 제공하는 기본 undo/redo 그대로 사용 | ✅ 최적 |
| Command 패턴 직접 구현 | 텍스트 변경을 커맨드 객체로 래핑 | ❌ 과잉 |
| Memento 패턴 (스냅샷) | 전체 텍스트 상태를 스택에 저장 | ❌ 과잉 |
| `undo` crate (evenorog) | Rust용 범용 undo 라이브러리 | ❌ NSTextView와 중복 |

### 결론
NSTextView가 이미 undo/redo를 내장하고 있으므로, **커스텀 구현 없이 네이티브 기능을 올바르게 활성화하고 UX를 보강하는 것이 최선**이다.

### GitHub/웹 조사 참고
- Helix, Xi, Zed 등 Rust 기반 에디터들은 자체 텍스트 버퍼를 사용하므로 Command 패턴이나 CRDT를 직접 구현
- TermPop은 NSTextView를 그대로 사용하므로 AppKit의 UndoManager에 위임하는 것이 올바른 접근
- `undo` crate (evenorog/undo)는 커스텀 데이터 구조에 undo를 붙일 때 유용하지만, NSTextView에는 불필요

## 구현 계획

### Task 1: NSTextView allowsUndo 명시적 활성화
`editor.rs`에서 NSTextView 설정 시 `allowsUndo`를 명시적으로 `true`로 설정한다.

```rust
text_view.setAllowsUndo(true);
```

NSTextView는 기본적으로 undo를 지원하지만, 명시적으로 설정하여 의도를 명확히 한다.

### Task 2: 힌트 바에 Undo/Redo 단축키 표시
현재 힌트 바:
```
TermPop  │  Enter: New line  │  ⌘+Enter: Submit  │  Esc: Cancel  │  ⌃+/⌃-: Font size  │  ⌃T: Theme
```

변경 후:
```
TermPop  │  Enter: New line  │  ⌘+Enter: Submit  │  Esc: Cancel  │  ⌘Z/⌘⇧Z: Undo/Redo  │  ⌃+/⌃-: Font  │  ⌃T: Theme
```

### Task 3: 수동 테스트 검증
- [ ] 텍스트 입력 후 Cmd+Z로 undo 동작 확인
- [ ] Cmd+Shift+Z로 redo 동작 확인
- [ ] 여러 단계 undo/redo 연속 동작 확인
- [ ] initial_text가 있는 경우 undo 시 initial_text까지 되돌아가는지 확인

## 변경 파일
- `src/editor.rs` — `setAllowsUndo(true)` 추가 + 힌트 텍스트 업데이트

## 예상 작업량
최소 변경. 코드 2~3줄 수정으로 완료 가능.
