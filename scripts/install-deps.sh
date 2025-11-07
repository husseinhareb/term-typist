#!/usr/bin/env bash
set -euo pipefail

# Script to install system dependencies required to build and run term-typist
# Run with: sudo ./scripts/install-deps.sh

# Function to detect Linux distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo $ID
    elif type lsb_release >/dev/null 2>&1; then
        lsb_release -si | tr '[:upper:]' '[:lower:]'
    elif [ -f /etc/redhat-release ]; then
        echo "rhel"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    else
        echo "unknown"
    fi
}

# Function to install dependencies based on distribution
install_dependencies() {
    local distro=$1
    
    echo "Detected Linux distribution: $distro"
    
    case "$distro" in
        ubuntu|debian|pop|mint|elementary)
            echo "Installing dependencies for Debian/Ubuntu-based distribution..."
            apt-get update
            apt-get install -y pkg-config libasound2-dev
            echo "Installed: pkg-config, libasound2-dev"
            ;;
        fedora|centos|rhel|rocky|almalinux)
            echo "Installing dependencies for Red Hat-based distribution..."
            if command -v dnf >/dev/null 2>&1; then
                dnf install -y pkgconfig alsa-lib-devel
            else
                yum install -y pkgconfig alsa-lib-devel
            fi
            echo "Installed: pkgconfig, alsa-lib-devel"
            ;;
        opensuse*|suse)
            echo "Installing dependencies for openSUSE..."
            zypper install -y pkg-config alsa-devel
            echo "Installed: pkg-config, alsa-devel"
            ;;
        arch|manjaro|endeavouros|garuda)
            echo "Installing dependencies for Arch-based distribution..."
            pacman -S --noconfirm pkg-config alsa-lib
            echo "Installed: pkg-config, alsa-lib"
            ;;
        alpine)
            echo "Installing dependencies for Alpine Linux..."
            apk add --no-cache pkgconfig alsa-lib-dev
            echo "Installed: pkgconfig, alsa-lib-dev"
            ;;
        gentoo)
            echo "Installing dependencies for Gentoo..."
            emerge --ask=n dev-util/pkgconfig media-libs/alsa-lib
            echo "Installed: dev-util/pkgconfig, media-libs/alsa-lib"
            ;;
        void)
            echo "Installing dependencies for Void Linux..."
            xbps-install -y pkg-config alsa-lib-devel
            echo "Installed: pkg-config, alsa-lib-devel"
            ;;
        *)
            echo "Unknown or unsupported distribution: $distro"
            echo ""
            echo "Please install the following dependencies manually:"
            echo "• pkg-config (or pkgconfig)"
            echo "• ALSA development libraries"
            echo ""
            echo "For your distribution, this might be:"
            echo "• Debian/Ubuntu: pkg-config libasound2-dev"
            echo "• Fedora/RHEL: pkgconfig alsa-lib-devel"
            echo "• Arch: pkg-config alsa-lib"
            echo "• openSUSE: pkg-config alsa-devel"
            echo "• Alpine: pkgconfig alsa-lib-dev"
            exit 1
            ;;
    esac
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script needs to be run with sudo privileges"
    echo "Usage: sudo ./scripts/install-deps.sh"
    exit 1
fi

# Main execution
distro=$(detect_distro)
install_dependencies "$distro"

echo ""
echo "System dependencies successfully installed!"
echo "You can now build term-typist with: make build" 
