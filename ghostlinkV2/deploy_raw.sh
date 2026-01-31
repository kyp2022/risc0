#!/bin/bash

# Configuration
SERVER_IP="118.31.238.137"
USER="root"
PASSWORD="Aa123456"
REMOTE_DIR="/root/ghostlink-v2"

echo "Deploying GhostLink V2 (Raw Mode + Static ELF) to $SERVER_IP..."

# Function to execute remote command
remote_exec() {
    sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no $USER@$SERVER_IP "$1"
}

# 1. Cleanup Old Processes
echo "Cleaning up old processes..."
remote_exec "
    pkill -f ghostlink || true
    pkill -f 'cargo run' || true
    pkill -f 'docker build' || true
    pkill -f 'podman build' || true
    # Free up port 80 if occupied
    fuser -k 80/tcp || true
"

# 2. Sync Files
echo "Syncing code and static ELF to server..."
sshpass -p "$PASSWORD" rsync -avz -e "ssh -o StrictHostKeyChecking=no" \
    --exclude 'target' \
    --exclude '.git' \
    --exclude 'node_modules' \
    --exclude '.DS_Store' \
    --exclude 'tmp' \
    --exclude '.agent' \
    --exclude '.gemini' \
    --delete \
    . $USER@$SERVER_IP:$REMOTE_DIR/

# 2.1 Final Cleanup on server
remote_exec "rm -f $REMOTE_DIR/methods/build.rs"

# 3. Remote Setup & Run
echo "Starting remote build and execution..."
remote_exec "
    cd $REMOTE_DIR
    
    # 3.1 Install System Dependencies
    if command -v apt-get &> /dev/null; then
        apt-get update && apt-get install -y build-essential clang curl libssl-dev pkg-config
    else
        yum groupinstall -y 'Development Tools' || true
        yum install -y clang openssl-devel || true
    fi

    # 3.2 Ensure Cargo config uses SJTU mirror
    mkdir -p \$HOME/.cargo
    cat > \$HOME/.cargo/config.toml <<EOF
[source.crates-io]
replace-with = 'sjtu'

[source.sjtu]
registry = \"sparse+https://mirrors.sjtug.sjtu.edu.cn/crates.io-index/\"

[net]
git-fetch-with-cli = true
EOF

    # 3.3 Ensure Rust is in path
    [ -f \$HOME/.cargo/env ] && source \$HOME/.cargo/env

    # 3.4 Build & Run
    echo 'Building and running release version (Static ELF Mode)...'
    # RISC0_SKIP_BUILD=1 avoids searching for toolchain
    export RISC0_SKIP_BUILD=1
    nohup cargo run --release --bin ghostlink-v2-host --features prove > server.log 2>&1 &
    
    echo 'Server started in background. Logs:'
    sleep 5
    tail -n 20 server.log
"

echo "Deployment completed!"
