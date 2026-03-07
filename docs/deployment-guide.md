# ProofZK Demo Deployment Guide
## DigitalOcean Droplet Setup for Airline Partnerships

### 🚀 Quick Deploy Script

```bash
#!/bin/bash
# deploy-proofzk-demo.sh

# 1. Set up domain
echo "Setting up demo.proofzk.com..."

# 2. Install dependencies
sudo apt update
sudo apt install -y nginx nodejs npm rust-all git certbot python3-certbot-nginx

# 3. Clone and build
git clone https://github.com/your-org/proof-zk.git
cd proof-zk

# Build Rust backend
cargo build --release

# Build PWA frontend
cd demos/customer-storage/web
npm install
npm run build

# 4. Nginx configuration
sudo tee /etc/nginx/sites-available/proofzk-demo << 'EOF'
server {
    listen 80;
    server_name demo.proofzk.com;
    
    # PWA frontend
    location / {
        root /var/www/proofzk/pwa;
        try_files $uri $uri/ /index.html;
        
        # PWA headers
        add_header Cache-Control "max-age=31536000";
        add_header Service-Worker-Allowed "/";
    }
    
    # API backend
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
    
    # Demo airline endpoints
    location /airline-demo/ {
        root /var/www/proofzk/airline;
        try_files $uri $uri/ /index.html;
    }
}
EOF

# 5. Enable site and SSL
sudo ln -s /etc/nginx/sites-available/proofzk-demo /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
sudo certbot --nginx -d demo.proofzk.com

# 6. Start services
cd /opt/proof-zk
sudo systemctl enable proofzk-api
sudo systemctl start proofzk-api

echo "✅ ProofZK demo deployed at https://demo.proofzk.com"
```

### 🎯 **Demo Scenarios for Airlines**

#### **Scenario 1: Age Verification**
```
URL: https://demo.proofzk.com/airline/age-check
Customer Story:
1. Customer visits "airline" website
2. Clicks "ProofZK Quick Check-in"
3. Browser prompts for Face ID/Touch ID
4. Instantly verified as 18+ (no other data revealed)
5. Check-in completed - no forms needed
```

#### **Scenario 2: Passport Verification**  
```
URL: https://demo.proofzk.com/airline/passport-check
Customer Story:
1. International flight check-in
2. ProofZK verifies passport validity
3. No passport data stored by airline
4. Instant verification vs 5-minute manual process
```

#### **Scenario 3: Cost Comparison**
```
URL: https://demo.proofzk.com/cost-calculator
Shows:
- Traditional KYC: $15 per verification
- ProofZK: $0.50 per verification  
- 97% cost reduction
- Zero breach liability
```

### 📊 **Analytics for Airline Meetings**

```javascript
// Track demo usage for partnership meetings
analytics.track('airline_demo_completed', {
  airline: 'qantas', // or 'westjet'
  demo_type: 'age_verification',
  time_saved: '4.2_minutes',
  cost_saved: '$14.50'
});
```

### 🎁 **Partnership Pitch Package**

Deploy these pages:
1. **Live Demo** - `demo.proofzk.com`
2. **ROI Calculator** - Shows exact savings for their passenger volume
3. **Integration Guide** - Copy-paste code for their developers
4. **Security Audit** - Technical specifications for their security teams
5. **Compliance Report** - GDPR, privacy law compliance documentation

### 💡 **Competitive Intelligence**

Monitor who's testing:
```bash
# Check access logs for airline IP ranges
grep -E "(qantas|westjet|air-canada)" /var/log/nginx/access.log
```

### 🔒 **Security for Demo**

```nginx
# Rate limiting to prevent abuse
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=demo:10m rate=30r/s;

# Geographic restrictions if needed
# Only allow from airline headquarters locations
```

## 📅 **Timeline**

- **Week 1:** Deploy basic PWA demo
- **Week 2:** Add airline integration scenarios  
- **Week 3:** Create ROI calculator and documentation
- **Week 4:** Schedule demos with Qantas/WestJet

**Cost:** ~$12/month vs potential **millions** in airline revenue

**ROI:** If you land even **one** mid-size airline, that's 100,000x return on this droplet investment! 🚀