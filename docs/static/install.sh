#!/bin/bash

# Default
REPO="cestef/rwalk"
SKIP_CHECK=false
VERSION=""
NO_COLOR=false
PRERELEASE=false

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -r, --repo REPO         GitHub repository (default: $REPO)
    -v, --version VERSION   Specific version to install
    -s, --skip-check        Skip version check
    -p, --prerelease        Include prereleases
    -n, --no-color          Disable colored output
    -h, --help              Show this help message

Environment variables (overridden by command line args):
    REPO, SKIP_CHECK, VERSION, NO_COLOR, PRERELEASE

Examples:
    $0                                     # Install latest version of default repo
    $0 -r owner/repo -v v1.2.3             # Install specific version
    $0 --prerelease --no-color             # Install latest prerelease without colors
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--repo)
            REPO="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -s|--skip-check)
            SKIP_CHECK=true
            shift
            ;;
        -p|--prerelease)
            PRERELEASE=true
            shift
            ;;
        -n|--no-color)
            NO_COLOR=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

REPO=${REPO:-"$REPO"}
SKIP_CHECK=${SKIP_CHECK:-false}
VERSION=${VERSION:-""}
NO_COLOR=${NO_COLOR:-false}
PRERELEASE=${PRERELEASE:-false}

# if debug, print all variables
if [ "$DEBUG" = true ]; then
    echo "DEBUG: REPO=$REPO"
    echo "DEBUG: SKIP_CHECK=$SKIP_CHECK"
    echo "DEBUG: VERSION=$VERSION"
    echo "DEBUG: NO_COLOR=$NO_COLOR"
    echo "DEBUG: PRERELEASE=$PRERELEASE"
fi

PROJECT_NAME=$(basename "$REPO")
INSTALLER_NAME="${PROJECT_NAME}-installer.sh"

if [ "$NO_COLOR" = true ]; then
    C_BOLD="" C_DIM="" C_GREEN="" C_RED="" C_YELLOW="" C_BLUE="" C_RESET=""
else
    C_BOLD="\033[1m"
    C_DIM="\033[2m"
    C_GREEN="\033[0;32m"
    C_RED="\033[0;31m"
    C_YELLOW="\033[0;33m"
    C_BLUE="\033[0;34m"
    C_RESET="\033[0m"
fi

msg() {
    local color="$1"
    local message="$2"
    local level="${3:-}"
    
    if [ -n "$level" ]; then
        echo -e "${color}[${level}] ${message}${C_RESET}"
    else
        echo -e "${color}${message}${C_RESET}"
    fi
}

echo -e "${C_BOLD}Installing $PROJECT_NAME...${C_RESET}"

if [ -z "$VERSION" ] && [ "$SKIP_CHECK" != "true" ]; then
    echo -e "${C_DIM}Fetching latest version information...${C_RESET}"
    if [ "$PRERELEASE" = true ]; then
        GITHUB_API_URL="https://api.github.com/repos/$REPO/releases?per_page=1"
    else
        GITHUB_API_URL="https://api.github.com/repos/$REPO/releases/latest"
    fi
    LATEST_VERSION=$(curl -s $GITHUB_API_URL | 
                    grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_VERSION" ]; then
        msg "$C_RED" "Failed to fetch latest version from GitHub API" "ERROR"
        exit 1
    fi
    
    msg "$C_GREEN" "Latest version: ${C_BOLD}$LATEST_VERSION${C_RESET}" "SUCCESS"
    VERSION_TO_INSTALL=$LATEST_VERSION
else
    VERSION_TO_INSTALL=$VERSION
    if [ -n "$VERSION" ]; then
        msg "$C_YELLOW" "Using specified version: ${C_BOLD}$VERSION_TO_INSTALL${C_RESET}" "VERSION"
    else
        echo -e "${C_DIM}Using latest available version${C_RESET}"
    fi
fi

echo -e "${C_BLUE}Downloading installer...${C_RESET}"
INSTALLER_URL="https://github.com/$REPO/releases/download/${VERSION_TO_INSTALL}/$INSTALLER_NAME"
echo -e "${C_DIM}From: $INSTALLER_URL${C_RESET}"

TEMP_INSTALLER=$(mktemp)
curl --proto '=https' --tlsv1.2 -LsSf "$INSTALLER_URL" -o "$TEMP_INSTALLER"
if [ $? -eq 0 ]; then
    chmod +x "$TEMP_INSTALLER"
    msg "$C_BLUE" "Running installer..."
    if command -v bash >/dev/null 2>&1; then
        bash "$TEMP_INSTALLER"
    else
        sh "$TEMP_INSTALLER"
    fi
    INSTALL_STATUS=$?
    rm "$TEMP_INSTALLER"
    exit $INSTALL_STATUS
else
    msg "$C_RED" "Failed to download installer" "ERROR"
    rm "$TEMP_INSTALLER"
    exit 1
fi