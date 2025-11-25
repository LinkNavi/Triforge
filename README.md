# TriForge Production Deployment Guide

**Version:** 0.1.0  
**Onion Service:** `hyrule4e3tu7pfdkvvca43senvgvgisi6einpe3d3kpidlk3uyjf7lqd.onion`

## Overview

TriForge is production-ready with full Tor integration for anonymous, secure version control over the Hyrule network.

## Features

✅ **Privacy-First Design**
- All traffic routed through Tor by default
- Onion service for maximum anonymity
- No IP address exposure
- No centralized tracking

✅ **Production Ready**
- Robust error handling
- Connection retries
- Timeout management
- Clear status indicators

✅ **Full Git Compatibility**
- Native Git repository support
- Complete version control workflow
- Seamless push/pull operations

## Prerequisites

### Required Software

1. **Tor**
   ```bash
   # Ubuntu/Debian
   sudo apt install tor
   
   # macOS
   brew install tor
   
   # Arch Linux
   sudo pacman -S tor
   ```

2. **Rust & Cargo** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### System Requirements

- **OS:** Linux, macOS, Windows (with WSL)
- **RAM:** 512MB minimum
- **Disk:** 100MB for binary + repository storage
- **Network:** Tor network access (port 9050)

## Installation

### 1. Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/TriForge
cd TriForge

# Build production binary
cargo build --release

# Install globally (optional)
sudo cp target/release/triforge /usr/local/bin/
```

### 2. Start Tor

```bash
# Enable and start Tor service
sudo systemctl enable tor
sudo systemctl start tor

# Verify Tor is running
systemctl status tor
```

### 3. Run Startup Check

```bash
# Make executable
chmod +x startup-check.sh

# Run comprehensive checks
./startup-check.sh
```

Expected output:
```
✓ Tor is installed
✓ Tor service is running
✓ Tor SOCKS proxy is listening on port 9050
✓ Tor circuits are working
✓ Hyrule onion service is reachable
```

## Configuration

### Default Configuration

TriForge ships with production-ready defaults:

```toml
# ~/.config/triforge/config.toml
hyrule_server = "http://hyrule4e3tu7pfdkvvca43senvgvgisi6einpe3d3kpidlk3uyjf7lqd.onion"
use_tor = true
tor_proxy = "socks5://127.0.0.1:9050"
verify_ssl = false
default_private = false
```

### View Configuration

```bash
triforge config show
```

### Modify Configuration

```bash
# Change Tor proxy port
triforge config set proxy socks5://127.0.0.1:9150

# Disable Tor (not recommended for production)
triforge config set tor false

# Use clearnet endpoint (not recommended)
triforge config set server http://example.com:3000
```

## Usage

### 1. Create Account

```bash
triforge signup
```

- Creates account on Hyrule network
- All registration traffic via Tor
- No personal information required
- Anonymous by default

### 2. Initialize Repository

```bash
# Create new repository
mkdir my-project
cd my-project
triforge init

# Add files
echo "# My Project" > README.md
triforge add README.md

# Commit
triforge commit -m "Initial commit"
```

### 3. Push to Hyrule Network

```bash
triforge push
```

- Uploads repository via Tor
- Content-addressed storage
- Distributed across anchor nodes
- Returns repository hash for cloning

### 4. Clone Repository

```bash
triforge clone <repo-hash> my-clone
```

### 5. Discovery Features

```bash
# Search repositories
triforge search rust

# View trending repos
triforge trending

# Network statistics
triforge stats

# List storage nodes
triforge nodes
```

## Security Best Practices

### 1. Tor Configuration

**Verify Tor is working:**
```bash
# Check your Tor exit IP
torsocks curl https://ifconfig.me

# Should show Tor exit node, not your real IP
```

**Configure Tor for better performance:**
```bash
# Edit /etc/tor/torrc
sudo nano /etc/tor/torrc

# Add these lines:
SocksPort 9050
ControlPort 9051
CookieAuthentication 1
```

### 2. Operational Security

- **Never** disable Tor in production
- **Always** verify onion service certificates
- **Use** VPN + Tor for additional layers (optional)
- **Keep** Tor updated: `sudo apt update && sudo apt upgrade tor`
- **Monitor** Tor logs: `journalctl -u tor -f`

### 3. Repository Security

```bash
# Make repositories private by default
triforge config set private true

# Use strong authentication
# Passwords should be 12+ characters
```

### 4. Data Privacy

- TriForge does **not** collect telemetry
- No analytics or tracking
- All metadata stays local
- Server only knows: username, repository hashes

## Troubleshooting

### Tor Connection Issues

**Problem:** "Tor is enabled but not reachable"

```bash
# Check Tor status
systemctl status tor

# Restart Tor
sudo systemctl restart tor

# Check SOCKS proxy
nc -z 127.0.0.1 9050

# View Tor logs
journalctl -u tor -n 50
```

**Problem:** "Cannot reach Hyrule onion service"

```bash
# Test Tor connectivity
torsocks curl https://check.torproject.org/

# Wait for circuit establishment (can take 30-60 seconds)
# Run startup check
./startup-check.sh
```

### Network Timeouts

```bash
# Onion services can be slow, especially on first connect
# Default timeouts:
# - Connect: 30 seconds
# - Request: 60 seconds

# If timeouts persist:
# 1. Check Tor logs for circuit issues
# 2. Try different Tor circuit: sudo systemctl restart tor
# 3. Check onion service status
```

### Authentication Errors

```bash
# Clear authentication
triforge logout

# Re-login
triforge login

# Verify config
triforge config show
```

## Performance

### Expected Latencies

- **Login/Signup:** 5-15 seconds
- **Push (100 objects):** 20-60 seconds
- **Clone (1000 objects):** 1-5 minutes
- **Search/Browse:** 3-10 seconds

### Optimization Tips

1. **Batch Operations**
   - Commit multiple files at once
   - Reduces network round-trips

2. **Tor Circuit Warmup**
   - First connection is slowest
   - Subsequent requests use cached circuit

3. **Compression**
   - Enabled by default
   - Reduces bandwidth by 60-80%

## Monitoring

### Health Checks

```bash
# Network status
triforge stats

# Node availability
triforge nodes

# Your repositories
triforge list
```

### Logs

```bash
# Tor logs
journalctl -u tor -f

# TriForge verbose mode
RUST_LOG=debug triforge push
```

## Production Checklist

- [ ] Tor installed and running
- [ ] Startup check passes all tests
- [ ] Configuration verified (`triforge config show`)
- [ ] Account created (`triforge signup`)
- [ ] Test repository pushed successfully
- [ ] Clone test successful
- [ ] Tor circuits stable (check logs)

## Support

### Documentation
- Full CLI reference: `triforge --help`
- Command help: `triforge <command> --help`

### Common Commands

```bash
# Quick reference
triforge init              # Initialize repository
triforge add <files>       # Stage files
triforge commit -m "msg"   # Commit changes
triforge push              # Upload to network
triforge clone <hash>      # Download repository
triforge config show       # View configuration
triforge stats             # Network statistics
triforge list              # Your repositories
```

## Architecture

### Network Flow

```
User -> TriForge -> Tor SOCKS Proxy (9050) -> Tor Network -> Onion Service
```

### Data Path

1. **Local:** `.git` directory with native Git objects
2. **Staging:** In-memory before upload
3. **Transport:** Compressed, encrypted over Tor
4. **Storage:** Content-addressed, replicated storage
5. **Retrieval:** Direct from anchor nodes

### Security Layers

1. **Application:** TriForge client
2. **Transport:** Tor (3 hop onion routing)
3. **Service:** Hidden service (no IP exposure)
4. **Storage:** Encrypted at rest
5. **Access:** Authenticated API

## License

See LICENSE file for details.

## Version History

- **0.1.0** (Current)
  - Initial production release
  - Full Tor integration
  - Onion service support
  - Complete Git workflow

---

**Ready for Production:** Yes  
**Tor Required:** Yes  
**Anonymous by Default:** Yes  
**Clearnet Support:** Available but not recommended
