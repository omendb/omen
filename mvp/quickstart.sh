#!/bin/bash

echo "🚀 pgAnalytics - PostgreSQL Real-Time Analytics MVP"
echo "Building for YC S26 Application"
echo ""

# Setup Python environment
echo "1. Setting up environment..."
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Start the server
echo "2. Starting analytics server..."
echo "   Access at: http://localhost:8000"
echo "   API docs: http://localhost:8000/docs"
echo ""

python app.py