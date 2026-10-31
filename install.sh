#!/usr/bin/env bash
# Nexus Panel - Universal Linux Installer
# Configures Nexus Panel as the default Windows 11-style clipboard manager & input hub

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$SCRIPT_DIR/nexus"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
APPLICATIONS_DIR="$PREFIX/share/applications"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
AUTOSTART_DIR="$HOME/.config/autostart"

# Color formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          NEXUS PANEL - UNIVERSAL LINUX INSTALLER            ║"
echo "║     Windows 11 Fluent Clipboard History & Emoji Picker       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Handle --uninstall flag
if [[ "$1" == "--uninstall" ]]; then
    echo -e "${YELLOW}🗑️ Uninstalling Nexus Panel...${NC}"
    systemctl --user stop nexus.service 2>/dev/null || true
    systemctl --user disable nexus.service 2>/dev/null || true
    rm -f "$BIN_DIR/nexus-ui" "$BIN_DIR/nexus"
    rm -f "$APPLICATIONS_DIR/nexus.desktop" "$APPLICATIONS_DIR/nexus-emoji.desktop"
    rm -f "$SYSTEMD_USER_DIR/nexus.service" "$AUTOSTART_DIR/nexus-daemon.desktop"
    echo -e "${GREEN}✓ Nexus Panel successfully uninstalled.${NC}"
    exit 0
fi

# 1. Detect Linux Distribution
echo -e "${BLUE}▶ 1. Detecting Linux distribution...${NC}"
OS_NAME="Linux"
PKG_MGR="unknown"

if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    OS_NAME="$NAME"
    echo -e "   Distro: ${BOLD}$OS_NAME ($ID)${NC}"
    
    if command -v pacman &>/dev/null; then
        PKG_MGR="pacman"
    elif command -v apt-get &>/dev/null; then
        PKG_MGR="apt"
    elif command -v dnf &>/dev/null; then
        PKG_MGR="dnf"
    elif command -v zypper &>/dev/null; then
        PKG_MGR="zypper"
    elif command -v apk &>/dev/null; then
        PKG_MGR="apk"
    elif command -v xbps-install &>/dev/null; then
        PKG_MGR="xbps"
    fi
fi
echo -e "   Package Manager: ${BOLD}$PKG_MGR${NC}"

# 2. Check and recommend dependencies
echo ""
echo -e "${BLUE}▶ 2. Checking required dependencies...${NC}"

check_tool() {
    local cmd="$1"
    local desc="$2"
    local pkg_arch="$3"
    local pkg_debian="$4"
    local pkg_fedora="$5"

    if command -v "$cmd" &>/dev/null; then
        echo -e "   ${GREEN}✓${NC} $desc ($cmd) found"
    else
        echo -e "   ${YELLOW}⚠️ Missing $desc ($cmd)${NC}"
        case "$PKG_MGR" in
            pacman) echo "      Install with: sudo pacman -S $pkg_arch" ;;
            apt)    echo "      Install with: sudo apt install $pkg_debian" ;;
            dnf)    echo "      Install with: sudo dnf install $pkg_fedora" ;;
            *)      echo "      Please install: $desc" ;;
        esac
    fi
}

check_tool "wl-copy" "Wayland clipboard manager" "wl-clipboard" "wl-clipboard" "wl-clipboard"
check_tool "sqlite3" "SQLite database engine" "sqlite" "sqlite3" "sqlite"

# Check emoji font for full color rendering
if fc-list : family | grep -iE "(noto color emoji|apple color emoji)" &>/dev/null; then
    echo -e "   ${GREEN}✓${NC} Color Emoji font detected"
else
    echo -e "   ${YELLOW}⚠️ Color Emoji font recommended for emoji rendering:${NC}"
    case "$PKG_MGR" in
        pacman) echo "      Install with: sudo pacman -S noto-fonts-emoji" ;;
        apt)    echo "      Install with: sudo apt install fonts-noto-color-emoji" ;;
        dnf)    echo "      Install with: sudo dnf install google-noto-color-emoji-fonts" ;;
        *)      echo "      Install Noto Color Emoji font" ;;
    esac
fi

# 3. Build & Install binary
echo ""
echo -e "${BLUE}▶ 3. Building and installing Nexus Panel...${NC}"
mkdir -p "$BIN_DIR" "$APPLICATIONS_DIR" "$SYSTEMD_USER_DIR" "$AUTOSTART_DIR"

cd "$WORKSPACE_DIR"
if [[ -f target/release/nexus-ui ]]; then
    echo "   Using existing release build..."
else
    echo "   Compiling optimized release binary (this may take a couple minutes)..."
    cargo build --release -p nexus-ui
fi

if command -v systemctl &>/dev/null; then
    systemctl --user stop nexus.service 2>/dev/null || true
fi

install -m 755 target/release/nexus-ui "$BIN_DIR/nexus-ui"
ln -sf "$BIN_DIR/nexus-ui" "$BIN_DIR/nexus"
echo -e "   ${GREEN}✓${NC} Binary installed to: ${BOLD}$BIN_DIR/nexus-ui${NC}"
echo -e "   ${GREEN}✓${NC} Symlink installed to: ${BOLD}$BIN_DIR/nexus${NC}"

# Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "   ${YELLOW}ℹ️ Adding $BIN_DIR to your shell PATH${NC}"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$rc" ]] && ! grep -q "$BIN_DIR" "$rc"; then
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$rc"
        fi
    done
fi

# 4. Install Desktop Files & Launchers
echo ""
echo -e "${BLUE}▶ 4. Registering desktop applications...${NC}"
cp "$WORKSPACE_DIR/packaging/nexus.desktop" "$APPLICATIONS_DIR/nexus.desktop"
cp "$WORKSPACE_DIR/packaging/nexus-emoji.desktop" "$APPLICATIONS_DIR/nexus-emoji.desktop"

# Fix binary path if installed in non-standard location
sed -i "s|Exec=nexus-ui|Exec=$BIN_DIR/nexus-ui|g" "$APPLICATIONS_DIR/nexus.desktop"
sed -i "s|Exec=nexus-ui --emoji|Exec=$BIN_DIR/nexus-ui --emoji|g" "$APPLICATIONS_DIR/nexus-emoji.desktop"

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APPLICATIONS_DIR" 2>/dev/null || true
fi
echo -e "   ${GREEN}✓${NC} Desktop entries registered in ${BOLD}$APPLICATIONS_DIR${NC}"

# 5. Background Daemon & Autostart Configuration
echo ""
echo -e "${BLUE}▶ 5. Setting up 24/7 background clipboard & screenshot watcher...${NC}"
cp "$WORKSPACE_DIR/packaging/nexus.service" "$SYSTEMD_USER_DIR/nexus.service"
sed -i "s|%h/.local/bin/nexus-ui|$BIN_DIR/nexus-ui|g" "$SYSTEMD_USER_DIR/nexus.service"

# Also setup XDG autostart for DEs without systemd user sessions
cp "$WORKSPACE_DIR/packaging/nexus-daemon.desktop" "$AUTOSTART_DIR/nexus-daemon.desktop"
sed -i "s|Exec=nexus-ui --daemon|Exec=$BIN_DIR/nexus-ui --daemon|g" "$AUTOSTART_DIR/nexus-daemon.desktop"

# Enable and start user service if systemd is available
if command -v systemctl &>/dev/null && systemctl --user is-active --quiet basic.target 2>/dev/null; then
    systemctl --user daemon-reload
    systemctl --user enable --now nexus.service 2>/dev/null || true
    echo -e "   ${GREEN}✓${NC} Systemd user service enabled and started (${BOLD}nexus.service${NC})"
else
    echo -e "   ${GREEN}✓${NC} XDG Autostart configured in ${BOLD}$AUTOSTART_DIR/nexus-daemon.desktop${NC}"
fi

# 6. Compositor and Window Manager Integration
echo ""
echo -e "${BLUE}▶ 6. Configuring compositor window rules & shortcuts...${NC}"

# A. Niri Compositor
if [[ -d "$HOME/.config/niri" ]]; then
    echo "   Detected Niri compositor config:"
    NIRI_RULES="$HOME/.config/niri/rules.kdl"
    NIRI_KEYBINDS="$HOME/.config/niri/keybinds.kdl"
    
    if [[ -f "$NIRI_RULES" ]]; then
        if ! grep -q "nexus" "$NIRI_RULES"; then
            cat << 'EOF' >> "$NIRI_RULES"

window-rule {
    match app-id=r#"(?i)nexus(-ui)?"#
    open-floating true
    default-column-width { fixed 420; }
    default-window-height { fixed 580; }
    geometry-corner-radius 12
    clip-to-geometry true
}
EOF
            echo -e "      ${GREEN}✓${NC} Floating & geometry rules added to rules.kdl"
        else
            echo -e "      ${GREEN}✓${NC} Window rules already present in rules.kdl"
        fi
    fi

    if [[ -f "$NIRI_KEYBINDS" ]]; then
        # Check if Mod+V is bound
        if grep -q "noctalia msg panel-toggle clipboard" "$NIRI_KEYBINDS"; then
            sed -i "s|noctalia msg panel-toggle clipboard|$BIN_DIR/nexus-ui|g" "$NIRI_KEYBINDS"
            echo -e "      ${GREEN}✓${NC} Updated Mod+V keybind to launch nexus-ui"
        elif ! grep -q "nexus-ui" "$NIRI_KEYBINDS"; then
            echo -e "      ${CYAN}ℹ️ To bind Mod+V to Nexus in ~/.config/niri/keybinds.kdl:${NC}"
            echo "         Mod+V { spawn \"$BIN_DIR/nexus-ui\"; }"
            echo "         Mod+Period { spawn \"$BIN_DIR/nexus-ui\" \"--emoji\"; }"
        fi
    fi
fi

# B. Hyprland
if [[ -f "$HOME/.config/hypr/hyprland.conf" ]]; then
    if ! grep -q "nexus-ui" "$HOME/.config/hypr/hyprland.conf"; then
        echo -e "   ${CYAN}ℹ️ Hyprland detected! Add to ~/.config/hypr/hyprland.conf:${NC}"
        echo "      windowrulev2 = float, class:^(nexus-ui)$"
        echo "      bind = \$mainMod, V, exec, $BIN_DIR/nexus-ui"
        echo "      bind = \$mainMod, PERIOD, exec, $BIN_DIR/nexus-ui --emoji"
    fi
fi

# C. Sway
if [[ -f "$HOME/.config/sway/config" ]]; then
    if ! grep -q "nexus-ui" "$HOME/.config/sway/config"; then
        echo -e "   ${CYAN}ℹ️ Sway detected! Add to ~/.config/sway/config:${NC}"
        echo "      for_window [app_id=\"nexus-ui\"] floating enable"
        echo "      bindsym \$mod+v exec $BIN_DIR/nexus-ui"
        echo "      bindsym \$mod+period exec $BIN_DIR/nexus-ui --emoji"
    fi
fi

echo ""
echo -e "${GREEN}${BOLD}══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}🎉 NEXUS PANEL INSTALLED SUCCESSFULLY!${NC}"
echo -e "${GREEN}${BOLD}══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "• ${BOLD}Clipboard Flyout${NC}:  Run ${CYAN}nexus-ui${NC} or press your configured ${BOLD}Super+V${NC}"
echo -e "• ${BOLD}Emoji/Kaomoji/GIF${NC}: Run ${CYAN}nexus-ui --emoji${NC} or press ${BOLD}Super+.${NC}"
echo -e "• ${BOLD}Daemon${NC}:            Active and recording all copied text & screenshots."
echo ""
