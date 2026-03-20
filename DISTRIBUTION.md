# TermPop 배포 가이드

## 배포 형태

| 형태 | 대상 | 접근성 권한 |
|------|------|------------|
| DMG (.app 번들) | 일반 사용자 | 시스템 설정에서 바로 등록 가능 |
| tar.gz (CLI 바이너리) | 개발자 | macOS 버전에 따라 등록 불가할 수 있음 |
| Homebrew Tap | macOS 개발자 | 위와 동일 |

macOS Tahoe(26)부터 CLI 바이너리를 접근성 목록에 직접 등록하는 것이 불가능해졌습니다.
`.app` 번들로 감싸야 macOS가 접근성 권한 대상으로 인식합니다.
따라서 DMG 배포가 기본 권장 방식입니다.

---

## DMG 배포 (권장)

### 구조

```
TermPop.app/
├── Contents/
│   ├── Info.plist          ← 번들 메타데이터 + 접근성 설명
│   ├── MacOS/
│   │   └── termpop         ← 바이너리
│   └── Resources/
│       └── termpop.icns    ← 앱 아이콘
```

### 로컬 빌드

```bash
cargo build --release
./scripts/package-dmg.sh
```

### 코드 서명 + 공증 포함 빌드

```bash
./scripts/package-dmg.sh \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --notarize
```

공증에는 사전에 `notarytool` 자격 증명 저장이 필요합니다:

```bash
xcrun notarytool store-credentials "termpop-notary" \
  --apple-id "your@email.com" \
  --team-id "TEAMID" \
  --password "app-specific-password"
```

### 사용자 설치 방법

1. DMG 다운로드 후 열기
2. `TermPop.app`을 `/Applications`로 드래그
3. `Install.command` 더블클릭 → 데몬 자동 설치 + 접근성 설정 안내
4. 시스템 설정 → 개인정보 보호 및 보안 → 접근성 → `+` → `TermPop.app` 추가

### "Apple could not verify" 경고 대응

공증 없이 배포하면 macOS Gatekeeper가 앱과 `Install.command` 실행을 차단합니다.
사용자에게 아래 방법을 안내하세요.

**방법 1: 우클릭으로 열기 (가장 간단)**

1. Finder → 응용 프로그램 → `TermPop` 찾기
2. Control + 클릭 → **열기** 선택
3. "확인되지 않은 개발자" 경고에서 **열기** 클릭
4. 이후 `Install.command`도 같은 방식으로 실행

> 처음 한 번만 이렇게 열면 이후부터는 더블클릭으로 정상 실행됩니다.

**방법 2: 터미널 명령어 (quarantine 속성 제거)**

```bash
xattr -cr /Applications/TermPop.app
```

`Install.command`도 차단되는 경우 DMG 마운트 경로에서:

```bash
xattr -cr /Volumes/TermPop/Install.command
```

**방법 3: 시스템 설정에서 허용**

1. `TermPop.app`을 더블클릭 (차단됨)
2. 시스템 설정 → 개인정보 보호 및 보안
3. 하단에 "TermPop이(가) 차단되었습니다" 옆 **확인 없이 열기** 클릭

---

## GitHub Actions 자동 릴리스

태그 푸시 시 자동으로 빌드 → 서명 → 공증 → 릴리스가 진행됩니다.

```bash
git tag v0.1.0
git push origin v0.1.0
```

### 필요한 GitHub Secrets

코드 서명 + 공증을 사용하려면 다음 시크릿을 설정하세요:

| Secret | 설명 |
|--------|------|
| `MACOS_CERTIFICATE` | Developer ID 인증서 (.p12) base64 인코딩 |
| `MACOS_CERTIFICATE_PWD` | .p12 파일 비밀번호 |
| `KEYCHAIN_PWD` | CI 키체인 비밀번호 (임의 값) |
| `APPLE_ID` | Apple ID 이메일 |
| `APPLE_TEAM_ID` | Apple Developer Team ID |
| `APPLE_APP_SPECIFIC_PASSWORD` | 앱 전용 비밀번호 (appleid.apple.com에서 생성) |

시크릿이 없으면 ad-hoc 서명으로 빌드됩니다. 이 경우 사용자에게 Gatekeeper 우회 안내가 필요합니다:

```bash
xattr -d com.apple.quarantine /Applications/TermPop.app
```

### 인증서 base64 인코딩 방법

```bash
base64 -i certificate.p12 | pbcopy
```

클립보드에 복사된 값을 `MACOS_CERTIFICATE` 시크릿에 붙여넣기.

---

## 릴리스 산출물

| 파일 | 설명 |
|------|------|
| `TermPop-v{VERSION}-macos-arm64.dmg` | Apple Silicon DMG |
| `TermPop-v{VERSION}-macos-x86_64.dmg` | Intel DMG |
| `TermPop-v{VERSION}-macos-universal.dmg` | Universal DMG |
| `termpop-v{VERSION}-macos-arm64.tar.gz` | Apple Silicon 바이너리 |
| `termpop-v{VERSION}-macos-x86_64.tar.gz` | Intel 바이너리 |
| `termpop-v{VERSION}-macos-universal.tar.gz` | Universal 바이너리 |
| `checksums.txt` | SHA256 체크섬 |

---

## 접근성 권한이 필요한 이유

TermPop 데몬은 다음 기능에 macOS 접근성 API를 사용합니다:

- 글로벌 핫키 등록 (`global-hotkey`)
- Cmd+V 키 이벤트 시뮬레이션 (`CGEvent`)

`.app` 번들로 배포하면 `Info.plist`의 `NSAccessibilityUsageDescription`을 통해
사용자에게 권한 요청 사유를 명확히 전달할 수 있습니다.
