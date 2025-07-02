
pub async fn serve_pac() -> String {
   r#"
    function FindProxyForURL(url, host) {
        // Set the start and end dates (YYYY, MM-1, DD)
        var startDate = new Date(2025, 6, 1); // July 1, 2025 (months are 0-based)
        var endDate = new Date(2025, 6, 30);  // July 30, 2025

        var now = new Date();

        // If outside allowed date range, bypass proxy
        if (now < startDate || now > endDate) {
            return "DIRECT";
        }

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
        return "PROXY triple-ts-mediclinic.com:8080; DIRECT";
    }
    "#.to_string()
}
