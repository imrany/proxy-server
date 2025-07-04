function FindProxyForURL(url, host) {
    // Bypass proxy for local addresses
    if (isPlainHostName(host) || dnsDomainIs(host, ".localdomain.com")) {
        return "DIRECT";
    }

    // Directly access private networks
    if (isInNet(host, "192.168.0.0", "255.255.0.0") || 
        isInNet(host, "10.0.0.0", "255.0.0.0") ||
        isInNet(host, "172.16.0.0", "255.240.0.0")) {
        return "DIRECT";
    }

    // Your main proxy with fallback
    return "PROXY prxy.villebiz.com; DIRECT";
}