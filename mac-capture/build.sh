#!/bin/sh
set -e
cd "$(dirname "$0")"
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
APP=FGACapture.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp Info.plist "$APP/Contents/Info.plist"
xcrun swiftc -O main.swift -o "$APP/Contents/MacOS/fga-capture"
codesign --force --sign - "$APP"
echo BUILD_OK
