#!/bin/bash

# Test script for peer discovery with two processes
# Usage: ./test_discovery.sh

echo "🚀 Starting mDNS peer discovery test..."
echo "📝 This will start two processes that should discover each other"
echo ""

# Function to run the example with a nickname
run_instance() {
    local nickname=$1
    echo "🏃 Starting instance with nickname: $nickname"
    cd "$(dirname "$0")"
    timeout 35s cargo run --example basic_usage -- --nickname "$nickname" 2>&1 | \
    sed "s/^/[$nickname] /" &
    local pid=$!
    echo "📍 Started instance $nickname with PID: $pid"
    echo $pid
}

# Start first instance
echo "🎯 Starting first instance..."
pid1=$(run_instance "device-1")
sleep 2

# Start second instance
echo "🎯 Starting second instance..."
pid2=$(run_instance "device-2")
sleep 2

echo ""
echo "⏳ Both instances are running. They should discover each other within 30 seconds."
echo "📊 Watch for 'Peer discovered' messages in the output below."
echo ""

# Wait for both processes to complete
wait $pid1
exit_code1=$?
wait $pid2
exit_code2=$?

echo ""
echo "✅ Test completed!"
echo "📋 Exit codes: device-1=$exit_code1, device-2=$exit_code2"

if [ $exit_code1 -eq 124 ] || [ $exit_code2 -eq 124 ]; then
    echo "⏰ Test timed out (expected - instances run for 35 seconds)"
fi

echo ""
echo "💡 Tips:"
echo "   - Look for '🎉 Peer discovered' messages"
echo "   - Check for '🔄 Nickname updated' messages"
echo "   - Each instance should show the other's nickname"