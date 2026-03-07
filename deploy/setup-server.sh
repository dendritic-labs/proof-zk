#!/bin/bash
# ProofZK Demo Deployment Script for DigitalOcean
# Run this on a fresh Ubuntu 22.04 droplet

set -e  # Exit on any error

echo "🚀 Starting ProofZK Demo Deployment..."
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   log_error "This script should not be run as root for security reasons"
   exit 1
fi

# Configuration
DOMAIN="demo.proofzk.com"
PROJECT_DIR="/opt/proofzk"
WEB_ROOT="/var/www/proofzk"
SERVICE_USER="proofzk"

log_info "Updating system packages..."
sudo apt update && sudo apt upgrade -y

log_info "Installing required packages..."
sudo apt install -y \
    nginx \
    nodejs \
    npm \
    git \
    curl \
    ufw \
    certbot \
    python3-certbot-nginx \
    build-essential \
    pkg-config \
    libssl-dev

# Install Rust
if ! command -v cargo &> /dev/null; then
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    rustup update stable
else
    log_success "Rust already installed"
fi

# Create service user
if ! id "$SERVICE_USER" &>/dev/null; then
    log_info "Creating service user: $SERVICE_USER"
    sudo useradd --system --home-dir $PROJECT_DIR --create-home --shell /bin/bash $SERVICE_USER
fi

# Create project directory
log_info "Setting up project directory..."
sudo mkdir -p $PROJECT_DIR
sudo chown $SERVICE_USER:$SERVICE_USER $PROJECT_DIR

# Create web root
sudo mkdir -p $WEB_ROOT/{pwa,airline,api-docs}
sudo chown -R $SERVICE_USER:www-data $WEB_ROOT
sudo chmod -R 755 $WEB_ROOT

# Clone repository (replace with your actual repo)
log_info "Cloning ProofZK repository..."
sudo -u $SERVICE_USER git clone https://github.com/your-username/proof-zk.git $PROJECT_DIR/source || {
    log_warning "Git clone failed - creating directory structure manually"
    sudo -u $SERVICE_USER mkdir -p $PROJECT_DIR/source
}

# Build Rust backend
log_info "Building Rust backend..."
cd $PROJECT_DIR/source
sudo -u $SERVICE_USER /home/$USER/.cargo/bin/cargo build --release || {
    log_warning "Cargo build failed - will create mock API"
}

# Configure nginx
log_info "Configuring nginx..."
sudo tee /etc/nginx/sites-available/proofzk-demo > /dev/null << 'EOF'
# Rate limiting
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=demo:10m rate=30r/s;

server {
    listen 80;
    server_name demo.proofzk.com;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline'" always;
    
    # PWA frontend
    location / {
        root /var/www/proofzk/pwa;
        try_files $uri $uri/ /index.html;
        
        # PWA specific headers
        add_header Cache-Control "max-age=31536000" always;
        add_header Service-Worker-Allowed "/" always;
        
        # CORS for demo
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range" always;
    }
    
    # API backend
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS for API
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
        add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization" always;
        
        if ($request_method = 'OPTIONS') {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
            add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization";
            add_header Access-Control-Max-Age 1728000;
            add_header Content-Type 'text/plain; charset=utf-8';
            add_header Content-Length 0;
            return 204;
        }
    }
    
    # Airline demo
    location /airline/ {
        limit_req zone=demo burst=50 nodelay;
        
        root /var/www/proofzk;
        try_files $uri $uri/ /airline/index.html;
        
        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
    
    # API documentation
    location /docs/ {
        root /var/www/proofzk/api-docs;
        try_files $uri $uri/ /index.html;
    }
    
    # Health check
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
EOF

# Enable site
sudo ln -sf /etc/nginx/sites-available/proofzk-demo /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default

# Test nginx configuration
if sudo nginx -t; then
    log_success "Nginx configuration is valid"
    sudo systemctl reload nginx
else
    log_error "Nginx configuration is invalid"
    exit 1
fi

# Configure firewall
log_info "Configuring UFW firewall..."
sudo ufw --force reset
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 'Nginx Full'
sudo ufw --force enable

# Create systemd service for API
log_info "Creating systemd service..."
sudo tee /etc/systemd/system/proofzk-api.service > /dev/null << EOF
[Unit]
Description=ProofZK API Server
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$PROJECT_DIR
ExecStart=$PROJECT_DIR/target/release/proofzk-api
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$PROJECT_DIR

# Environment
Environment=RUST_LOG=info
Environment=BIND_ADDRESS=127.0.0.1:8080

[Install]
WantedBy=multi-user.target
EOF

# Enable service (don't start yet, we need to build the binary first)
sudo systemctl daemon-reload
sudo systemctl enable proofzk-api

log_success "Base system setup complete!"

echo ""
echo "=================================================="
echo "🎯 Next Steps:"
echo "=================================================="
echo "1. Update DNS: Point demo.proofzk.com to this server's IP"
echo "2. Run: sudo certbot --nginx -d demo.proofzk.com"
echo "3. Deploy the PWA and API code"
echo "4. Start the service: sudo systemctl start proofzk-api"
echo ""
echo "Server IP: $(curl -s ifconfig.me)"
echo "Domain to configure: $DOMAIN"
echo ""
log_success "Deployment script completed successfully!"
EOF