## 설치 방법

### 1. DMG 다운로드 (권장)

| 칩 | 파일 |
|---|---|
| Apple Silicon (M1/M2/M3/M4) | `TermPop-v__VERSION__-macos-arm64.dmg` |
| Intel | `TermPop-v__VERSION__-macos-x86_64.dmg` |
| Universal (둘 다 지원) | `TermPop-v__VERSION__-macos-universal.dmg` |

### 2. 설치

1. DMG 열기 → `TermPop.app`을 `/Applications`로 드래그
2. `Install.command` 더블클릭 → 데몬 자동 설치
3. 시스템 설정 → 개인정보 보호 및 보안 → 접근성 → `TermPop.app` 추가 및 토글 활성화
4. 끝! `Cmd+Shift+I`로 사용하세요

> ⚠️ macOS Tahoe(26)부터 CLI 바이너리는 접근성 목록에 직접 등록이 불가합니다.
> DMG(.app 번들)로 설치해주세요.

### 3. Gatekeeper 경고가 뜨는 경우

코드 서명이 없는 빌드를 사용할 경우 다음 명령으로 우회할 수 있습니다:

```bash
xattr -d com.apple.quarantine /Applications/TermPop.app
```

---

### CLI 바이너리 (개발자용)

```bash
tar -xzf termpop-v__VERSION__-macos-arm64.tar.gz
chmod +x termpop
sudo mv termpop /usr/local/bin/
```
