@echo off

REM Define the proxy server and port
REM monorail.proxy.rlwy.net:31125
set PROXY_SERVER=http://monorail.proxy.rlwy.net:31125

REM Set proxy for HTTP and HTTPS
setx HTTP_PROXY %PROXY_SERVER%
setx HTTPS_PROXY %PROXY_SERVER%

REM Optionally, set no proxy for specific addresses
set NO_PROXY=localhost;127.0.0.1;.example.com
setx NO_PROXY %NO_PROXY%

echo Proxy configuration is set.
pause
