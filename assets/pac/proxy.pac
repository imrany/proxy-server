// monorail.proxy.rlwy.net:31125
function FindProxyForURL(url, host) {
    // Bypass proxy for local addresses
    if (isPlainHostName(host) || dnsDomainIs(host, ".localdomain.com")) {
        return "DIRECT";
    }

    // Use a specific proxy for certain domains
    if (dnsDomainIs(host, ".example.com") || shExpMatch(host, "*.example.org")) {
        return "PROXY proxy1.example.com:8080";
    }

    // Use a different proxy for other domains
    if (dnsDomainIs(host, ".anotherdomain.com") || shExpMatch(host, "*.another.org")) {
        return "PROXY proxy2.example.com:8080";
    }

    // Directly access the specified IP range without a proxy
    if (isInNet(host, "192.168.0.0", "255.255.0.0")) {
        return "DIRECT";
    }

    // Default to a fallback proxy for all other requests
    return "PROXY monorail.proxy.rlwy.net:31125; DIRECT";
}
