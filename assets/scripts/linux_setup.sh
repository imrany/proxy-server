#!/bin/bash

# Define the proxy server and port
#monorail.proxy.rlwy.net:31125
PROXY_SERVER="http:/monorail.proxy.rlwy.net:31125"

# Configure environment variables for HTTP, HTTPS, and FTP proxies
export http_proxy=$PROXY_SERVER
export https_proxy=$PROXY_SERVER
export ftp_proxy=$PROXY_SERVER

# Optional: No proxy for specific addresses (e.g., localhost)
export no_proxy="localhost,127.0.0.1,.example.com"

# Persist the configuration across sessions
echo "export http_proxy=$PROXY_SERVER" >> ~/.bashrc
echo "export https_proxy=$PROXY_SERVER" >> ~/.bashrc
echo "export ftp_proxy=$PROXY_SERVER" >> ~/.bashrc
echo "export no_proxy=\"localhost,127.0.0.1,.example.com\"" >> ~/.bashrc

# Reload the shell configuration
source ~/.bashrc

echo "Proxy configuration is set."
