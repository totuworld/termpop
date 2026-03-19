#!/bin/bash
clear
echo "================================"
echo "  TermPop 설치"
echo "================================"
echo ""

APP_PATH="/Applications/TermPop.app"
BINARY="$APP_PATH/Contents/MacOS/termpop"
OLD_BINARY="/usr/local/bin/termpop"
OLD_PLIST="$HOME/Library/LaunchAgents/com.termpop.daemon.plist"

if [ ! -d "$APP_PATH" ]; then
  echo "❌ TermPop.app이 /Applications에 없습니다."
  echo "   먼저 TermPop.app을 Applications 폴더로 드래그해주세요."
  echo ""
  read -n 1 -s -r -p "아무 키나 누르면 종료합니다..."
  exit 1
fi

echo "✅ TermPop.app 확인됨"

if [ -f "$OLD_BINARY" ] || [ -f "$OLD_PLIST" ]; then
  echo ""
  echo "→ 이전 버전(CLI) 설치가 감지되었습니다. 정리합니다..."

  pkill -f "termpop" 2>/dev/null || true
  sleep 1

  if [ -f "$OLD_PLIST" ]; then
    launchctl bootout "gui/$(id -u)" "$OLD_PLIST" 2>/dev/null || true
    launchctl unload "$OLD_PLIST" 2>/dev/null || true
    rm -f "$OLD_PLIST"
    echo "  ✓ 이전 데몬 plist 제거됨"
  fi

  if [ -f "$OLD_BINARY" ]; then
    sudo rm -f "$OLD_BINARY"
    echo "  ✓ 이전 바이너리 제거됨 ($OLD_BINARY)"
  fi

  echo ""
  echo "⚠️  접근성 목록에 이전 termpop이 남아있다면 수동으로 제거해주세요."
  echo "   (시스템 설정 → 접근성 → termpop 선택 → - 버튼)"
  echo ""
fi

echo ""
echo "→ 데몬을 설치합니다..."
echo ""

"$BINARY" daemon --install

echo ""
echo "================================"
echo "  거의 다 됐습니다!"
echo "================================"
echo ""
echo "마지막으로 접근성 권한을 설정해주세요:"
echo ""
echo "  1. 시스템 설정 → 개인정보 보호 및 보안 → 접근성"
echo "  2. + 버튼 클릭 → TermPop.app 선택"
echo "  3. 토글 켜기"
echo ""
echo "지금 설정 화면을 열까요? (y/n) "
read -n 1 answer
echo ""

if [[ "$answer" == "y" || "$answer" == "Y" ]]; then
  open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
  echo ""
  echo "설정 화면이 열렸습니다. TermPop.app을 추가해주세요."
fi

echo ""
echo "✅ 설치 완료! Cmd+Shift+I 로 사용하세요."
echo ""
read -n 1 -s -r -p "아무 키나 누르면 종료합니다..."
