#!/bin/bash

# GOBLIN Quick Start Script
# This script helps you get GOBLIN running quickly

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════╗"
echo "║   GOBLIN DNS Redirector Quick Start   ║"
echo "╔═══════════════════════════════════════╝"
echo -e "${NC}"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v podman &>/dev/null; then
  echo -e "${RED}Error: Podman is not installed${NC}"
  exit 1
fi

if ! command -v podman-compose &>/dev/null; then
  echo -e "${RED}Error: Podman Compose is not installed${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Podman is installed${NC}"
echo -e "${GREEN}✓ Podman Compose is installed${NC}"

# Ask user what they want to do
echo ""
echo "What would you like to do?"
echo "1) Start GOBLIN with Podman Compose (recommended)"
echo "2) Build and run backend only"
echo "3) Build and run frontend only"
echo "4) Stop all GOBLIN services"
echo "5) View logs"
echo "6) Clean up (remove containers and volumes)"

read -p "Enter your choice (1-6): " choice

case $choice in
1)
  echo -e "${BLUE}Starting GOBLIN with Podman Compose...${NC}"
  podman compose up -d
  echo ""
  echo -e "${GREEN}✓ GOBLIN is starting!${NC}"
  echo ""
  echo "Services:"
  echo "  DNS Server:  localhost:53 (requires root/sudo)"
  echo "  REST API:    http://localhost:8080/api/v1"
  echo "  Web UI:      http://localhost:3000"
  echo ""
  echo "View logs with: podman compose logs -f"
  echo "Stop with: podman compose down"
  ;;
2)
  echo -e "${BLUE}Building and running backend...${NC}"
  cd backend
  if [ ! -f "goblin.db" ]; then
    echo "Initializing database..."
    sqlite3 goblin.db <migrations/001_initial.sql
  fi
  cargo run -- --dns-bind-address 0.0.0.0:15353 --api-bind-address 0.0.0.0:8080
  ;;
3)
  echo -e "${BLUE}Building and running frontend...${NC}"
  cd frontend
  if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
  fi
  npm run dev
  ;;
4)
  echo -e "${BLUE}Stopping GOBLIN services...${NC}"
  podman compose down
  echo -e "${GREEN}✓ Services stopped${NC}"
  ;;
5)
  echo -e "${BLUE}Viewing logs (Ctrl+C to exit)...${NC}"
  podman compose logs -f
  ;;
6)
  read -p "This will DELETE all data. Are you sure? (yes/no): " confirm
  if [ "$confirm" = "yes" ]; then
    echo -e "${BLUE}Cleaning up...${NC}"
    podman compose down -v
    echo -e "${GREEN}✓ Cleanup complete${NC}"
  else
    echo "Cleanup cancelled"
  fi
  ;;
*)
  echo -e "${RED}Invalid choice${NC}"
  exit 1
  ;;
esac

echo ""
echo -e "${GREEN}Done!${NC}"
