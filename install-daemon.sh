#!/bin/bash

# Snakepit Daemon Installation Script
# This script installs and configures the snakepit daemon as a systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="snakepit-daemon"
SERVICE_FILE="snakepit-daemon.service"
INSTALL_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/snakepit"

echo -e "${BLUE}🐍 Snakepit Daemon Installation${NC}"
echo "=================================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Check if snakepit binary exists
if [ ! -f "./target/release/snakepit" ]; then
    echo -e "${YELLOW}Building snakepit binary...${NC}"
    cargo build --release
fi

# Install snakepit binary
echo -e "${BLUE}Installing snakepit binary...${NC}"
cp target/release/snakepit $INSTALL_DIR/
chmod +x $INSTALL_DIR/snakepit

# Create configuration directory
echo -e "${BLUE}Creating configuration directory...${NC}"
mkdir -p $CONFIG_DIR
mkdir -p /home/$SUDO_USER/.config/snakepit

# Copy service file
echo -e "${BLUE}Installing systemd service...${NC}"
cp $SERVICE_FILE $SERVICE_DIR/

# Reload systemd
echo -e "${BLUE}Reloading systemd daemon...${NC}"
systemctl daemon-reload

# Enable service (but don't start yet)
echo -e "${BLUE}Enabling snakepit daemon service...${NC}"
systemctl enable $SERVICE_NAME

# Create default daemon configuration
echo -e "${BLUE}Creating default daemon configuration...${NC}"
cat > /home/$SUDO_USER/.config/snakepit/daemon.toml << EOF
enabled = true
auto_install = true
check_interval = 5
max_install_attempts = 3
whitelist_modules = []
blacklist_modules = ["sys", "os", "builtins"]
log_file = "/home/$SUDO_USER/.config/snakepit/daemon.log"
pid_file = "/home/$SUDO_USER/.config/snakepit/snakepit.pid"
EOF

# Set proper permissions
chown -R $SUDO_USER:$SUDO_USER /home/$SUDO_USER/.config/snakepit
chmod 755 /home/$SUDO_USER/.config/snakepit
chmod 644 /home/$SUDO_USER/.config/snakepit/daemon.toml

echo -e "${GREEN}✅ Snakepit daemon installed successfully!${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  Start daemon:    sudo systemctl start $SERVICE_NAME"
echo "  Stop daemon:     sudo systemctl stop $SERVICE_NAME"
echo "  Status daemon:   sudo systemctl status $SERVICE_NAME"
echo "  Enable auto-start: sudo systemctl enable $SERVICE_NAME"
echo "  Disable auto-start: sudo systemctl disable $SERVICE_NAME"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Config file: /home/$SUDO_USER/.config/snakepit/daemon.toml"
echo "  Log file:    /home/$SUDO_USER/.config/snakepit/daemon.log"
echo "  PID file:    /home/$SUDO_USER/.config/snakepit/snakepit.pid"
echo ""
echo -e "${YELLOW}Note: The daemon will auto-start on boot if enabled.${NC}"
echo -e "${YELLOW}To start the daemon now, run: sudo systemctl start $SERVICE_NAME${NC}"
