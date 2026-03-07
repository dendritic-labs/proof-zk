#!/bin/bash

# ProofZK Deployment Script
# This script uploads all deployment files to your DigitalOcean droplet

set -e

# Configuration
DROPLET_IP="${1:-your-droplet-ip}"
DOMAIN="${2:-demo.proofzk.com}"
SSH_KEY="${3:-~/.ssh/id_rsa}"

if [ "$DROPLET_IP" = "your-droplet-ip" ]; then
    echo "❌ Usage: $0 <droplet-ip> [domain] [ssh-key-path]"
    echo "Example: $0 104.248.1.123 demo.proofzk.com ~/.ssh/id_rsa"
    exit 1
fi

echo "🚀 Deploying ProofZK to $DROPLET_IP..."
echo "🌐 Domain: $DOMAIN"

# Create deployment directory
DEPLOY_DIR="./deploy-package"
mkdir -p $DEPLOY_DIR

# Copy all deployment files
echo "📦 Preparing deployment package..."
cp -r web/ $DEPLOY_DIR/
cp -r api/ $DEPLOY_DIR/
cp setup-server.sh $DEPLOY_DIR/
cp proofzk-api.service $DEPLOY_DIR/

# Update domain in setup script
sed -i.bak "s/demo\.proofzk\.com/$DOMAIN/g" $DEPLOY_DIR/setup-server.sh

# Upload to droplet
echo "⬆️ Uploading files to droplet..."
scp -i $SSH_KEY -r $DEPLOY_DIR root@$DROPLET_IP:/tmp/proofzk-deploy

# Run setup script on droplet
echo "🔧 Running setup script on droplet..."
ssh -i $SSH_KEY root@$DROPLET_IP << 'EOF'
cd /tmp/proofzk-deploy
chmod +x setup-server.sh
./setup-server.sh
EOF

echo "✅ Deployment complete!"
echo ""
echo "🌐 Your ProofZK demo should be available at:"
echo "   https://$DOMAIN"
echo "   https://$DOMAIN/airline-demo.html"
echo ""
echo "📊 API endpoints:"
echo "   POST https://$DOMAIN/api/verify"
echo "   GET  https://$DOMAIN/api/roi/<passengers>"
echo "   GET  https://$DOMAIN/api/stats"
echo ""
echo "🔍 To check status:"
echo "   ssh -i $SSH_KEY root@$DROPLET_IP 'systemctl status proofzk-api'"
echo "   ssh -i $SSH_KEY root@$DROPLET_IP 'journalctl -u proofzk-api -f'"

# Clean up
rm -rf $DEPLOY_DIR