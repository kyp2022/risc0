#!/bin/bash

# Configuration
SERVER_IP="118.31.238.137"
USER="root"
PASSWORD="Aa123456"
REMOTE_DIR="/root/ghostlink-v2"
APP_NAME="ghostlink-service"
PORT=80 # Map host port 80 to container port 3000

echo "Deploying to Alibaba Cloud ($SERVER_IP)..."

# 1. Check for sshpass
if ! command -v sshpass &> /dev/null; then
    echo "Error: sshpass is not installed. Please install it first."
    exit 1
fi

# Function to execute remote command
remote_exec() {
    sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no $USER@$SERVER_IP "$1"
}

# 2. Check and Install Docker on Remote Server
echo "Checking Docker installation on server..."
if ! remote_exec "command -v docker" &> /dev/null; then
    echo "Docker not found. Installing Docker..."
    remote_exec "curl -fsSL https://get.docker.com | sh"
else
    echo "Docker is already installed."
fi

# 2.1 Check and Install rsync on Remote Server
echo "Checking rsync installation on server..."
if ! remote_exec "command -v rsync" &> /dev/null; then
    echo "rsync not found. Installing rsync..."
    # Try yum (CentOS/Alibaba Cloud Linux) then apt (Ubuntu/Debian)
    remote_exec "yum install -y rsync || apt-get update && apt-get install -y rsync"
fi

# 2.2 Configure Registry Mirror (for Podman/China)
echo "Configuring registry mirror (FORCE OVERWRITE)..."
remote_exec "
    # Backup existing config
    cp /etc/containers/registries.conf /etc/containers/registries.conf.bak_$(date +%s)
    
    # Write new config
    cat > /etc/containers/registries.conf <<EOF
unqualified-search-registries = [\"docker.io\"]

[[registry]]
prefix = \"docker.io\"
location = \"docker.io\"

[[registry.mirror]]
location = \"docker.m.daocloud.io\"

[[registry.mirror]]
location = \"mirror.baidubce.com\"
EOF
"

# Check connectivity
echo "Checking server connectivity..."
if ! remote_exec "ping -c 3 google.com || ping -c 3 baidu.com"; then
    echo "Warning: Server seems to have issues connecting to the internet."
fi

# 3. Create Remote Directory
echo "Creating remote directory..."
remote_exec "mkdir -p $REMOTE_DIR"

# 4. Sync Files
echo "Syncing files to server..."
# Using rsync with sshpass. Excluding files from .dockerignore logic manually or by using the file if rsync supports it
# Here we just list common exclusions
sshpass -p "$PASSWORD" rsync -avz -e "ssh -o StrictHostKeyChecking=no" \
    --exclude 'target' \
    --exclude '.git' \
    --exclude 'node_modules' \
    --exclude '.DS_Store' \
    --exclude 'tmp' \
    --exclude '.agent' \
    --exclude '.gemini' \
    . $USER@$SERVER_IP:$REMOTE_DIR/

# 5. Build and Run Docker Container
echo "Building and running Docker container on server used..."
remote_exec "cd $REMOTE_DIR && \
    docker build -t $APP_NAME . && \
    docker stop $APP_NAME || true && \
    docker rm $APP_NAME || true && \
    docker run -d -p $PORT:3000 --restart always --name $APP_NAME $APP_NAME"

echo "Deployment completed!"
echo "Service should be available at http://$SERVER_IP"
