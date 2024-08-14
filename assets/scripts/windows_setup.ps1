# Define the URL of the proxy script
$proxyScriptUrl = "https://raw.githubusercontent.com/imrany/proxy-server/main/assets/pac/proxy.pac"

# Set the proxy script for Windows Internet settings
$registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings"
Set-ItemProperty -Path $registryPath -Name AutoConfigURL -Value $proxyScriptUrl

# Enable the use of a proxy script
Set-ItemProperty -Path $registryPath -Name ProxyEnable -Value 0

# Notify the system of the proxy change
[System.Runtime.Interopservices.Marshal]::ReleaseComObject([System.Runtime.Interopservices.Marshal]::GetActiveObject("Shell.Application")).Windows() | foreach { $_.Refresh() }

Write-Output "Proxy script address configured in Windows settings."
