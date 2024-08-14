# Define the proxy server and port
# monorail.proxy.rlwy.net:31125
$proxyServer = "http:/ monorail.proxy.rlwy.net:31125"

# Set environment variables for the current session
$env:HTTP_PROXY = $proxyServer
$env:HTTPS_PROXY = $proxyServer

# Optionally, set no proxy for specific addresses
$env:NO_PROXY = "localhost,127.0.0.1,.example.com"

# Persist the settings across sessions by adding them to the user's environment variables
[Environment]::SetEnvironmentVariable("HTTP_PROXY", $proxyServer, "User")
[Environment]::SetEnvironmentVariable("HTTPS_PROXY", $proxyServer, "User")
[Environment]::SetEnvironmentVariable("NO_PROXY", "localhost,127.0.0.1,.example.com", "User")

Write-Output "Proxy configuration is set."
