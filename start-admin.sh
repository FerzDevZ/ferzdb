#!/bin/bash

# Start the Bridge server in the background
echo "Starting FerzAdmin Bridge..."
cd admin/bridge
node index.js &
BRIDGE_PID=$!

# Start the Frontend dev server
echo "Starting FerzAdmin Frontend..."
cd ../frontend
npm run dev &
FRONTEND_PID=$!

# Function to cleanup on exit
trap "kill $BRIDGE_PID $FRONTEND_PID" EXIT

wait
