#!/bin/bash
# Installation script for auto.ui module in AutoLang stdlib
#
# This script copies or symlinks the ui.at file to the AutoLang stdlib path.
#
# Usage:
#   ./install-ui-module.sh [path/to/auto-lang]
#
# Example:
#   ./install-ui-module.sh ../../../auto-lang

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default AutoLang path (relative to auto-ui)
AUTOLANG_PATH="${1:-../../../auto-lang}"

# Resolve absolute paths
AUTOUI_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AUTOLANG_ROOT="$(cd "${AUTOUI_ROOT}/${AUTOLANG_PATH}" && pwd 2>/dev/null || echo "")"

UI_SOURCE="${AUTOUI_ROOT}/stdlib/auto/ui.at"
UI_DEST="${AUTOLANG_ROOT}/stdlib/auto/ui.at"

echo "======================================================================"
echo "AutoUI Module Installation Script"
echo "======================================================================"
echo ""
echo "AutoUI Root:  ${AUTOUI_ROOT}"
echo "AutoLang Root: ${AUTOLANG_ROOT}"
echo ""

# Check if AutoLang path exists
if [ -z "${AUTOLANG_ROOT}" ] || [ ! -d "${AUTOLANG_ROOT}" ]; then
    echo -e "${RED}Error: AutoLang path not found: ${AUTOLANG_ROOT}${NC}"
    echo ""
    echo "Please provide the correct path to AutoLang:"
    echo "  $0 [path/to/auto-lang]"
    echo ""
    echo "Example:"
    echo "  $0 ../../../auto-lang"
    echo "  $0 /path/to/auto-lang"
    exit 1
fi

# Check if ui.at source exists
if [ ! -f "${UI_SOURCE}" ]; then
    echo -e "${RED}Error: ui.at not found at: ${UI_SOURCE}${NC}"
    exit 1
fi

# Create destination directory if it doesn't exist
echo -e "${YELLOW}Creating destination directory...${NC}"
mkdir -p "$(dirname "${UI_DEST}")"

# Check if ui.at already exists in AutoLang
if [ -f "${UI_DEST}" ]; then
    echo -e "${YELLOW}Warning: ui.at already exists at: ${UI_DEST}${NC}"
    echo ""
    read -p "Do you want to overwrite it? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi

    # Backup existing file
    BACKUP="${UI_DEST}.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}Backing up existing file to: ${BACKUP}${NC}"
    cp "${UI_DEST}" "${BACKUP}"
fi

# Ask for installation method
echo ""
echo "Choose installation method:"
echo "  1) Symlink (recommended for development)"
echo "  2) Copy (for distribution)"
echo ""
read -p "Enter choice (1 or 2): " -n 1 -r
echo ""

if [[ $REPLY == "1" ]]; then
    # Create symlink
    echo -e "${YELLOW}Creating symlink...${NC}"
    ln -sf "${UI_SOURCE}" "${UI_DEST}"
    echo -e "${GREEN}✓ Symlink created: ${UI_DEST} -> ${UI_SOURCE}${NC}"
else
    # Copy file
    echo -e "${YELLOW}Copying file...${NC}"
    cp "${UI_SOURCE}" "${UI_DEST}"
    echo -e "${GREEN}✓ File copied to: ${UI_DEST}${NC}"
fi

# Verify installation
echo ""
echo -e "${YELLOW}Verifying installation...${NC}"
if [ -e "${UI_DEST}" ]; then
    echo -e "${GREEN}✓ Installation successful!${NC}"
    echo ""
    echo "Module location: ${UI_DEST}"
    echo ""
    echo "You can now use the auto.ui module in your .at files:"
    echo '  use auto.ui: View, Widget, App, Center, Text'
    echo ""
    echo "Test it with:"
    echo "  cd ${AUTOLANG_ROOT}"
    echo "  cargo run --bin auto-lang -- parse path/to/your/file.at"
else
    echo -e "${RED}✗ Installation failed: ${UI_DEST} not found${NC}"
    exit 1
fi
