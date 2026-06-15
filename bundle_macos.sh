#!/usr/bin/env bash
set -e

APP_NAME="Browseraptor"
BUNDLE_ID="com.browseraptor.Browseraptor"
VERSION="0.2.0"
BINARY="target/release/browseraptor"
APP_DIR="dist/${APP_NAME}.app"

echo "==> Building release binary..."
cargo build --release

echo "==> Creating .app bundle structure..."
rm -rf "dist/${APP_NAME}.app"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

echo "==> Copying binary..."
cp "${BINARY}" "${APP_DIR}/Contents/MacOS/${APP_NAME}"

echo "==> Copying assets..."
mkdir -p "${APP_DIR}/Contents/Resources/assets/icons"
cp assets/icons/*.svg "${APP_DIR}/Contents/Resources/assets/icons/"

echo "==> Generating Info.plist..."
cat > "${APP_DIR}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>HTTP URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>http</string>
                <string>https</string>
            </array>
            <key>LSHandlerRank</key>
            <string>Default</string>
        </dict>
    </array>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>HTML Document</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Default</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.html</string>
                <string>public.xhtml</string>
            </array>
        </dict>
    </array>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
</dict>
</plist>
EOF

echo "==> Generating .icns icon..."
ICONSET="dist/AppIcon.iconset"
mkdir -p "${ICONSET}"

SRC="assets/browseraptor-logo.png"
sips -z 16 16     "${SRC}" --out "${ICONSET}/icon_16x16.png"    2>/dev/null
sips -z 32 32     "${SRC}" --out "${ICONSET}/icon_16x16@2x.png" 2>/dev/null
sips -z 32 32     "${SRC}" --out "${ICONSET}/icon_32x32.png"    2>/dev/null
sips -z 64 64     "${SRC}" --out "${ICONSET}/icon_32x32@2x.png" 2>/dev/null
sips -z 128 128   "${SRC}" --out "${ICONSET}/icon_128x128.png"  2>/dev/null
sips -z 256 256   "${SRC}" --out "${ICONSET}/icon_128x128@2x.png" 2>/dev/null
sips -z 256 256   "${SRC}" --out "${ICONSET}/icon_256x256.png"  2>/dev/null
sips -z 512 512   "${SRC}" --out "${ICONSET}/icon_256x256@2x.png" 2>/dev/null
sips -z 512 512   "${SRC}" --out "${ICONSET}/icon_512x512.png"  2>/dev/null
sips -z 1024 1024 "${SRC}" --out "${ICONSET}/icon_512x512@2x.png" 2>/dev/null

iconutil -c icns "${ICONSET}" -o "${APP_DIR}/Contents/Resources/AppIcon.icns"
rm -rf "${ICONSET}"

echo "==> Registering URL schemes with Launch Services..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
    -f "${PWD}/${APP_DIR}" 2>/dev/null || true

echo ""
echo "✓ Bundle created: ${APP_DIR}"
echo "  Bundle ID : ${BUNDLE_ID}"
echo "  Executable: ${APP_DIR}/Contents/MacOS/${APP_NAME}"
echo ""
echo "To install system-wide:"
echo "  cp -r ${APP_DIR} /Applications/"
echo ""
echo "To run directly:"
echo "  open ${APP_DIR}"
