# GOBLIN Deployment Guide

This guide covers deploying GOBLIN in various environments.

## Table of Contents

- [Quick Start with Docker](#quick-start-with-docker)
- [Production Deployment](#production-deployment)
- [Manual Deployment](#manual-deployment)
- [Security Considerations](#security-considerations)
- [Monitoring and Maintenance](#monitoring-and-maintenance)

## Quick Start with Docker

The fastest way to get GOBLIN running is with Podman Compose.

### Prerequisites

- Podman 5.8.2
- Podman Compose 1.6.0

### Step 1: Clone and Build

```bash
# Build and start all services
podman compose up -d
```

### Step 2: Verify Services

```bash
# Check service status
podman compose ps

# View logs
podman compose logs -f

# Test DNS server
dig @localhost evil.com

# Test API
curl http://localhost:8080/api/v1/health

# Access Web UI
open http://localhost:3000
```

### Step 3: Stop Services

```bash
podman compose down

# Stop and remove volumes (WARNING: deletes database)
podman compose down -v
```

## Production Deployment

### Using Docker Compose (Recommended)

1. **Configure Environment**

Create a `.env` file:

```bash
# Backend
DNS_BIND_ADDRESS=0.0.0.0:53
API_BIND_ADDRESS=0.0.0.0:8080
DATABASE_PATH=/data/goblin.db
LOG_LEVEL=info
DEFAULT_UPSTREAM=8.8.8.8:53
RULES_CACHE_TTL=60

# Frontend
VITE_API_URL=http://your-domain.com:8080/api/v1
```

1. **Update podman compose.yml**

For production, consider:

- Using specific version tags instead of `latest`
- Adding resource limits
- Configuring logging
- Setting up backup volumes

1. **Deploy**

```bash
# Build and start in detached mode
podman compose up -d --build

# View logs
podman compose logs -f backend
podman compose logs -f frontend
```

### Reverse Proxy Setup (nginx)

For production, put GOBLIN behind a reverse proxy:

```nginx
# /etc/nginx/sites-available/goblin
server {
    listen 80;
    server_name goblin.yourdomain.com;

    # Frontend
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # API
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # DNS over HTTPS (optional)
    location /dns-query {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
    }
}
```

Enable and restart nginx:

```bash
sudo ln -s /etc/nginx/sites-available/goblin /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### SSL/TLS with Let's Encrypt

```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d goblin.yourdomain.com

# Auto-renewal is configured automatically
```

## Manual Deployment

### Backend Deployment

1. **Build the binary**

```bash
cd backend
cargo build --release
```

1. **Create systemd service**

```ini
# /etc/systemd/system/goblin.service
[Unit]
Description=GOBLIN DNS Redirector
After=network.target

[Service]
Type=simple
User=goblin
Group=goblin
WorkingDirectory=/opt/goblin
Environment="DATABASE_PATH=/var/lib/goblin/goblin.db"
Environment="LOG_LEVEL=info"
ExecStart=/opt/goblin/goblin
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/goblin

[Install]
WantedBy=multi-user.target
```

1. **Deploy**

```bash
# Create user
sudo useradd -r -s /bin/false goblin

# Create directories
sudo mkdir -p /opt/goblin /var/lib/goblin
sudo chown goblin:goblin /var/lib/goblin

# Copy binary
sudo cp target/release/goblin /opt/goblin/
sudo cp -r migrations /opt/goblin/

# Start service
sudo systemctl daemon-reload
sudo systemctl enable goblin
sudo systemctl start goblin
sudo systemctl status goblin
```

### Frontend Deployment

1. **Build the frontend**

```bash
cd frontend
npm install
npm run build
```

1. **Deploy to nginx**

```bash
# Copy built files
sudo cp -r dist/* /var/www/goblin/

# Configure nginx
sudo nano /etc/nginx/sites-available/goblin-frontend
```

```nginx
server {
    listen 80;
    server_name goblin-ui.yourdomain.com;
    root /var/www/goblin;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

```bash
# Enable and restart
sudo ln -s /etc/nginx/sites-available/goblin-frontend /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

## Security Considerations

### DNS Server Security

1. **Firewall Rules**

```bash
# Allow DNS queries only from specific networks
sudo ufw allow from 192.168.1.0/24 to any port 53
sudo ufw allow from 10.0.0.0/8 to any port 53

# Allow API from localhost only (if using reverse proxy)
sudo ufw allow from 127.0.0.1 to any port 8080
```

1. **Rate Limiting**

Consider implementing rate limiting at the firewall level:

```bash
# iptables example
sudo iptables -A INPUT -p udp --dport 53 -m state --state NEW -m recent --set
sudo iptables -A INPUT -p udp --dport 53 -m state --state NEW -m recent --update --seconds 1 --hitcount 20 -j DROP
```

### API Security

1. **Add Authentication** (Future Enhancement)

Consider adding JWT-based authentication for the API in production.

1. **CORS Configuration**

Limit CORS to specific origins in production:

```bash
# Set environment variable
API_CORS_ENABLED=false  # Disable if using reverse proxy
```

### Database Security

1. **Backup Strategy**

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/var/backups/goblin"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR
sqlite3 /var/lib/goblin/goblin.db ".backup $BACKUP_DIR/goblin_$DATE.db"

# Keep only last 30 days
find $BACKUP_DIR -name "goblin_*.db" -mtime +30 -delete
```

Add to cron:

```bash
# /etc/cron.daily/goblin-backup
0 2 * * * /usr/local/bin/goblin-backup.sh
```

1. **Permissions**

```bash
# Restrict database file permissions
chmod 600 /var/lib/goblin/goblin.db
chown goblin:goblin /var/lib/goblin/goblin.db
```

## Monitoring and Maintenance

### Health Checks

```bash
# Backend health
curl http://localhost:8080/api/v1/health

# Frontend health
curl -I http://localhost:3000

# DNS resolution test
dig @localhost test.com
```

### Logging

1. **View logs (Docker)**

```bash
podman compose logs -f backend
podman compose logs -f frontend
```

1. **View logs (systemd)**

```bash
journalctl -u goblin -f
```

1. **Log Rotation**

```bash
# /etc/logrotate.d/goblin
/var/log/goblin/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 goblin goblin
    sharedscripts
    postrotate
        systemctl reload goblin
    endscript
}
```

### Database Maintenance

```bash
# Vacuum database (reclaim space)
sqlite3 /var/lib/goblin/goblin.db "VACUUM;"

# Check integrity
sqlite3 /var/lib/goblin/goblin.db "PRAGMA integrity_check;"

# View statistics
sqlite3 /var/lib/goblin/goblin.db "SELECT COUNT(*) FROM statistics;"
```

### Performance Monitoring

1. **Monitor DNS queries**

```bash
# Watch DNS logs
tail -f /var/log/goblin/dns.log | grep "Resolved"
```

1. **Monitor API requests**

```bash
# API request statistics
curl http://localhost:8080/api/v1/stats/summary | jq
```

1. **Resource usage**

```bash
# Docker stats
docker stats goblin-backend goblin-frontend

# System resources
htop
iotop
```

## Troubleshooting

### DNS Server Won't Start

**Error:** "Address already in use"

```bash
# Check what's using port 53
sudo lsof -i :53

# Stop systemd-resolved if conflicting
sudo systemctl stop systemd-resolved
sudo systemctl disable systemd-resolved
```

### Database Migration Errors

```bash
# Reset database (WARNING: deletes all data)
rm /var/lib/goblin/goblin.db
# Restart service to recreate
sudo systemctl restart goblin
```

### Permission Denied on Port 53

Port 53 requires root privileges. Options:

1. Run as root (not recommended)
2. Use capabilities:

   ```bash
   sudo setcap 'cap_net_bind_service=+ep' /opt/goblin/goblin
   ```

3. Use a different port and redirect with iptables:

   ```bash
   # Run on port 5353, redirect 53 to 5353
   sudo iptables -t nat -A PREROUTING -p udp --dport 53 -j REDIRECT --to-port 5353
   ```

## Scaling

For high-traffic environments:

1. **Multiple DNS servers** behind a load balancer
2. **PostgreSQL** instead of SQLite for better concurrency
3. **Redis** cache for rule lookups
4. **Separate API and DNS** into different containers
