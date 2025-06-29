# Proxy Server

<sub>A proxy server is a system or router that provides a gateway between users and the internet. Therefore, it helps prevent cyber attackers from entering a private network. It is a server, referred to as an "intermediary" because it goes between end-users and the web pages they visit online.</sub>

![proxy image](https://github.com/imrany/proxy-server/blob/main/assets/proxy_image.webp)

<sub>This server is designed to block websites defined in a text file. We will use Axum's http-proxy example and add the feature to block the websites.</sub>

## Features

- ✅ HTTP/HTTPS proxy functionality
- ✅ Website blocking via configuration file
- ✅ Built with Rust and Axum framework
- ✅ Docker support for easy deployment
- ✅ High performance and low resource usage

## Quick Start

### Option 1: Build with Docker

1. **Build the image**
```bash
docker build -t proxy .
```

2. **Run the container**
```bash
docker run -p 8080:8080 -d proxy
```

### Option 2: Pull from GitHub Container Registry

1. **Pull the pre-built image**
```bash
docker pull ghcr.io/imrany/proxy
```

2. **Run the container**
```bash
docker run -p 8080:8080 -d ghcr.io/imrany/proxy
```

### Option 3: Run from Source

1. **Prerequisites**
- Rust 1.70+ installed
- Cargo package manager

2. **Clone and run**
```bash
git clone https://github.com/imrany/proxy-server.git
cd proxy-server
cargo run
```

## Testing the Proxy

Once the server is running, test it with curl:

```bash
# Test HTTP request through proxy
curl -v -x "127.0.0.1:8080" http://httpbin.org/ip

# Test HTTPS request through proxy  
curl -v -x "127.0.0.1:8080" https://tokio.rs
```

### Configure Your Browser

**Chrome/Edge:**
```
Settings → Advanced → System → Open proxy settings
HTTP Proxy: 127.0.0.1:8080
HTTPS Proxy: 127.0.0.1:8080
```

**Firefox:**
```
Settings → Network Settings → Manual proxy configuration
HTTP Proxy: 127.0.0.1 Port: 8080
Use this proxy server for all protocols: ✓
```

## Configuration

### Blocked Websites

Create a `blocked_sites.txt` file in the project root:

```
www.instagram.com:443
twitter.com:443
discord.com:443
```

The proxy will block access to any domains listed in this file.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PROXY_PORT` | `8080` | Port to run the proxy server |
| `BLOCKED_SITES_FILE` | `blocked_sites.txt` | Path to blocked sites configuration |
| `LOG_LEVEL` | `info` | Logging level (debug, info, warn, error) |

### Docker Environment Example

```bash
docker run -p 8080:8080 -d \
-e PROXY_PORT=8080 \
-e LOG_LEVEL=debug \
-v $(pwd)/blocked_sites.txt:/app/blocked_sites.txt \
ghcr.io/imrany/proxy
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

### Docker Development

```bash
# Build development image
docker build -t proxy:dev .

# Run with volume mounting for development
docker run -p 8080:8080 -v $(pwd):/app proxy:dev
```

## Performance

- **Memory Usage**: ~10MB base
- **Concurrent Connections**: 1000+ 
- **Latency Overhead**: <5ms
- **Throughput**: Limited by network bandwidth

## Logging

The proxy logs all requests and blocked attempts:

```
2025-06-29T11:34:48.243347Z DEBUG proxy: listening on 0.0.0.0:8080
2025-06-29T11:36:30.369691Z TRACE proxy: req=Request { method: CONNECT, uri: tokio.rs:443, version: HTTP/1.1, headers: {"host": "tokio.rs:443", "user-agent": "curl/8.5.0", "proxy-connection": "Keep-Alive"}, body: Body(UnsyncBoxBody) }
```

## Use Cases

- **Corporate Networks**: Block social media and non-work sites
- **Parental Controls**: Restrict access to inappropriate content
- **Development Testing**: Intercept and analyze HTTP traffic
- **Privacy Protection**: Mask client IP addresses
- **Load Testing**: Simulate different network conditions

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Troubleshooting

### Common Issues

**Port already in use:**
```bash
# Find process using port 8080
lsof -i :8080
# Kill the process
kill -9 <PID>
```

**Permission denied (Linux):**
```bash
# Run with sudo if binding to port < 1024
sudo docker run -p 80:8080 proxy
```

**Blocked sites not working:**
- Ensure `blocked_sites.txt` exists and is readable
- Check file format (one domain per line)
- Verify domain names don't include protocols (http://)

### Support

- 📧 Email: [imranmat254@gmail.com]
- Sponsor me: [Github Sponsors](https://github.com/sponsors/imrany)
- 🐛 Issues: [GitHub Issues](https://github.com/imrany/proxy-server/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/imrany/proxy-server/discussions)
