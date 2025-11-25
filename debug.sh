#!/bin/bash
# debug_server.sh - Check server status and test endpoints

echo "🔍 Checking Hyrule server..."
echo ""

# Check if server is running
echo "1. Testing server health..."
HEALTH=$(curl -s http://localhost:3000/api/health 2>&1)
if [[ $? -eq 0 ]]; then
    echo "✓ Server is reachable"
    echo "Response: $HEALTH"
else
    echo "✗ Cannot connect to server"
    echo "Error: $HEALTH"
    echo ""
    echo "Is the server running? Start with:"
    echo "  cd ../Hyrule"
    echo "  cargo run"
    exit 1
fi

echo ""
echo "2. Testing signup endpoint..."
SIGNUP_RESPONSE=$(curl -v -X POST http://localhost:3000/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"username":"TestUser123","password":"password123"}' 2>&1)

echo "$SIGNUP_RESPONSE"

echo ""
echo "3. Checking server logs..."
echo "Look at the Hyrule server terminal for error messages"
echo ""
echo "Common issues:"
echo "  • Database not initialized: cargo run -- migrate"
echo "  • Wrong database schema: DROP DATABASE hyrule; CREATE DATABASE hyrule;"
echo "  • Server port conflict: Check if port 3000 is in use"
