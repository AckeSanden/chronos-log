#!/bin/bash
# Build script for creating Windows executable from Linux

set -e

echo "=========================================="
echo "Chronos Log - Windows Build Script"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Windows target is installed
echo "Checking for Windows target..."
if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    echo -e "${YELLOW}Windows target not found. Installing...${NC}"
    rustup target add x86_64-pc-windows-gnu
else
    echo -e "${GREEN}✓ Windows target already installed${NC}"
fi

# Check if MinGW is installed
echo "Checking for MinGW-w64..."
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${RED}MinGW-w64 not found!${NC}"
    echo "Please install it with: sudo pacman -S mingw-w64-gcc"
    exit 1
else
    echo -e "${GREEN}✓ MinGW-w64 found${NC}"
fi

echo ""
echo "Building for Windows (x86_64)..."
echo ""

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}=========================================="
    echo "✓ Build successful!"
    echo "==========================================${NC}"
    echo ""
    echo "Windows executable location:"
    echo "  $(pwd)/target/x86_64-pc-windows-gnu/release/chronos-log.exe"
    echo ""

    # Show file size
    SIZE=$(du -h target/x86_64-pc-windows-gnu/release/chronos-log.exe | cut -f1)
    echo "Executable size: $SIZE"
    echo ""

    echo "To deploy to Windows 11:"
    echo "  1. Copy chronos-log.exe to your Windows machine"
    echo "  2. Double-click to run (no installation needed)"
    echo "  3. Data will be stored in: %LOCALAPPDATA%\\chronos-log\\"
    echo ""

    # Optional: Create a deployment zip
    echo -e "${YELLOW}Create deployment package? (y/n)${NC}"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        DEPLOY_DIR="chronos-log-windows"
        ZIP_NAME="chronos-log-windows-$(date +%Y%m%d).zip"

        echo "Creating deployment package..."
        rm -rf "$DEPLOY_DIR"
        mkdir -p "$DEPLOY_DIR"

        # Copy executable
        cp target/x86_64-pc-windows-gnu/release/chronos-log.exe "$DEPLOY_DIR/"

        # Create README for Windows users
        cat > "$DEPLOY_DIR/README.txt" << 'EOF'
Chronos Log - Work Activity Time Tracker
=========================================

INSTALLATION:
1. Extract all files to a folder of your choice
2. Double-click chronos-log.exe to run
3. No installation or admin rights required

DATA STORAGE:
Your data is stored in:
%LOCALAPPDATA%\chronos-log\chronos_log.db

FIRST USE:
- The application will create example projects on first run
- You can delete these and create your own

OPTIONAL - Add to Startup:
1. Press Win+R, type: shell:startup
2. Create a shortcut to chronos-log.exe in that folder

SUPPORT:
For issues or questions, refer to the main README.md

License: MIT
EOF

        # Create zip
        zip -r "$ZIP_NAME" "$DEPLOY_DIR"
        rm -rf "$DEPLOY_DIR"

        echo -e "${GREEN}✓ Deployment package created: $ZIP_NAME${NC}"
        echo ""
    fi
else
    echo -e "${RED}=========================================="
    echo "✗ Build failed!"
    echo "==========================================${NC}"
    exit 1
fi
