# TermPop 배포 가이드

TermPop을 일반 사용자에게 전달하기 위한 패키징 및 배포 방법을 정리합니다.

## 배포 옵션 비교

| 방법 | 난이도 | 대상 사용자 | 장점 | 단점 |
|------|--------|------------|------|------|
| GitHub Release + 바이너리 | ★☆☆ | 개발자 | 가장 간단 | 수동 다운로드 필요 |
| Homebrew Tap | ★★☆ | macOS 개발자 | `brew install` 한 줄 | Tap 저장소 관리 필요 |
| cargo install | ★☆☆ | Rust 개발자 | crates.io 등록 시 자동 | Rust 툴체인 필요 |
| cargo-dist (자동화) | ★★☆ | 모든 사용자 | CI/CD 자동화, 멀티 아키텍처 | 초기 설정 필요 |

TermPop은 CLI 도구이므로 `.app` 번들이나 DMG는 불필요합니다.
추천 조합: **GitHub Release + Homebrew Tap** (+ cargo-dist로 자동화)

---

## 1. GitHub Release + 바이너리 배포

가장 기본적인 방법. 빌드한 바이너리를 GitHub Release에 첨부합니다.

### 수동 빌드 및 릴리스

```bash
# Apple Silicon (M1/M2/M3/M4)
cargo build --release --target aarch64-apple-darwin

# Intel Mac
cargo build --release --target x86_64-apple-darwin

# Universal Binary (두 아키텍처 합치기)
lipo -create \
  target/aarch64-apple-darwin/release/termpop \
  target/x86_64-apple-darwin/release/termpop \
  -output termpop-universal

# 압축
tar -czf termpop-v0.1.0-macos-arm64.tar.gz -C target/aarch64-apple-darwin/release termpop
tar -czf termpop-v0.1.0-macos-x86_64.tar.gz -C target/x86_64-apple-darwin/release termpop
tar -czf termpop-v0.1.0-macos-universal.tar.gz termpop-universal

# SHA256 체크섬 생성
shasum -a 256 termpop-v0.1.0-*.tar.gz > checksums.txt
```

GitHub에서 Release 생성 후 `.tar.gz` 파일들을 첨부합니다.

### 사용자 설치 방법

```bash
# 다운로드 후
tar -xzf termpop-v0.1.0-macos-arm64.tar.gz
chmod +x termpop
sudo mv termpop /usr/local/bin/
```

---

## 2. Homebrew Tap

macOS 사용자에게 가장 친숙한 설치 경험을 제공합니다.

### Tap 저장소 생성

GitHub에 `homebrew-tap` 저장소를 만들고 `Formula/termpop.rb`를 작성합니다.

```ruby
class Termpop < Formula
  desc "터미널에서 호출하는 네이티브 macOS 팝업 텍스트 에디터"
  homepage "https://github.com/totuworld/text-bubble"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/totuworld/text-bubble/releases/download/v0.1.0/termpop-v0.1.0-macos-arm64.tar.gz"
      sha256 "ARM64_SHA256_HERE"
    end
    on_intel do
      url "https://github.com/totuworld/text-bubble/releases/download/v0.1.0/termpop-v0.1.0-macos-x86_64.tar.gz"
      sha256 "X86_64_SHA256_HERE"
    end
  end

  def install
    bin.install "termpop"
  end

  test do
    assert_match "termpop", shell_output("#{bin}/termpop --help")
  end
end
```

### 사용자 설치 방법

```bash
brew tap totuworld/tap
brew install termpop
```

---

## 3. cargo-dist로 릴리스 자동화 (추천)

[cargo-dist](https://github.com/axodotdev/cargo-dist)는 GitHub Actions로 빌드, 릴리스, Homebrew 포뮬러 업데이트까지 자동화합니다.

### 초기 설정

```bash
# cargo-dist 설치
cargo install cargo-dist

# 프로젝트에 cargo-dist 초기화
cargo dist init
```

`Cargo.toml`에 다음이 추가됩니다:

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.27.0"
ci = "github"
installers = ["shell", "homebrew"]
tap = "totuworld/homebrew-tap"
publish-jobs = ["homebrew"]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
]
```

### 릴리스 프로세스

```bash
# 버전 태그 생성 → GitHub Actions가 자동으로 빌드 + 릴리스
git tag v0.1.0
git push origin v0.1.0
```

자동으로 수행되는 작업:
- aarch64, x86_64 바이너리 빌드
- GitHub Release 생성 및 바이너리 첨부
- Homebrew 포뮬러 자동 업데이트
- 설치 스크립트 생성

### 사용자 설치 방법 (자동 생성됨)

```bash
# 원라인 설치 스크립트
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/totuworld/text-bubble/releases/latest/download/termpop-installer.sh | sh

# 또는 Homebrew
brew tap totuworld/tap
brew install termpop
```

---

## 4. crates.io 등록

Rust 개발자를 위한 배포 채널입니다.

### 준비

`Cargo.toml`에 메타데이터를 추가합니다:

```toml
[package]
name = "termpop"
version = "0.1.0"
edition = "2021"
description = "터미널에서 호출하는 네이티브 macOS 팝업 텍스트 에디터"
license = "MIT"
repository = "https://github.com/totuworld/text-bubble"
keywords = ["terminal", "editor", "macos", "popup", "clipboard"]
categories = ["command-line-utilities"]
```

### 배포

```bash
cargo login
cargo publish
```

### 사용자 설치 방법

```bash
cargo install termpop
```

---

## 추천 배포 전략

### Phase 1 (지금 바로)
1. GitHub Release에 수동 빌드 바이너리 첨부
2. README에 설치 방법 안내

### Phase 2 (사용자 늘어나면)
1. cargo-dist 설정으로 릴리스 자동화
2. Homebrew Tap 생성
3. crates.io 등록

### Phase 3 (선택)
1. 코드 서명 + 공증 (Gatekeeper 경고 제거)
   - Apple Developer 계정 필요 ($99/년)
   - `codesign --sign "Developer ID" termpop`
   - `xcrun notarytool submit termpop.zip`

---

## 주의사항

### macOS 접근성 권한
데몬 모드의 글로벌 핫키와 CGEvent 붙여넣기는 접근성 권한이 필요합니다. 사용자에게 안내가 필요합니다:

> 시스템 설정 → 개인정보 보호 및 보안 → 접근성 → termpop 허용

### Gatekeeper
코드 서명 없이 배포하면 "확인되지 않은 개발자" 경고가 표시됩니다. 사용자에게 우회 방법을 안내합니다:

```bash
# Gatekeeper 우회 (다운로드한 바이너리)
xattr -d com.apple.quarantine /usr/local/bin/termpop
```

### macOS 전용
TermPop은 AppKit, CGEvent 등 macOS 네이티브 API에 의존하므로 macOS에서만 동작합니다.
