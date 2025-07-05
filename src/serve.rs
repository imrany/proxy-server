use axum::response::Html;

pub async fn index_page() -> Html<String>{
    Html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Proxy Server</title>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                html, body {
                    height: 100%;
                    margin: 0;
                    padding: 0;
                }
                body {
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background: var(--bg, #f5f5f5);
                    color: var(--fg, #222);
                    font-family: system-ui, sans-serif;
                    transition: background 0.3s, color 0.3s;
                }
                .card {
                    max-width: 400px;
                    width: 100%;
                    /* text-align: center; */
                }
                h1 {
                    margin-top: 0;
                    font-size: 2.2rem;
                    letter-spacing: -1px;
                }
                .links {
                    margin: 2rem 0 0 0;
                    display: flex;
                    flex-direction: column;
                    gap: 1rem;
                }
                .links a {
                    display: inline-block;
                    padding: 0.7em 1.2em;
                    border-radius: 6px;
                    text-decoration: none;
                    font-weight: 500;
                    background: var(--btn-bg, #24292f);
                    color: var(--btn-fg, #fff);
                    transition: background 0.2s;
                }
                .links a.sponsor {
                    background: #db61a2;
                }
                .links a.email {
                    background: #0078d4;
                }
                .links a:hover {
                    filter: brightness(1.1);
                }
                @media (prefers-color-scheme: dark) {
                    :root {
                        --bg: #181a1b;
                        --fg: #f3f3f3;
                        --card-bg: #23272e;
                        --btn-bg: #24292f;
                        --btn-fg: #fff;
                    }
                    .links a.email { background: #2899f5; }
                }
                @media screen and (max-width: 600px) {
                    body {
                        padding: 1rem;
                        font-size: 0.9rem;
                    }
                    .card {
                        width: 100%;
                        max-width: 90%;
                    }
                    h1 {
                        font-size: 1.8rem;
                    }

                    
                }
            </style>
        </head>
        <body>
            <div class="card">
                <h1>👋 Welcome!</h1>
                <p>
                    This is a simple, fast, and modern <b>HTTP proxy server</b> written in Rust.<br>
                    It allows you to forward requests, inspect traffic, and easily integrate with your own applications.<br>
                    <span style="font-size:1.5em;">🚀</span>
                    <br>
                    
                </p>
                <div>
                    <p><b>Features:</b></p>
                    <ul style="text-align:left; margin: 0 auto; display: inline-block;">
                    <li>Lightweight and efficient</li>
                    <li>Easy to configure and extend</li>
                    <li>Modern async Rust stack</li>
                    <li>Open source and community-driven</li>
                    </ul>
                </div>
                <div class="links">
                    <a href="https://github.com/imrany/proxy-server" target="_blank" rel="noopener">🌐 GitHub Repo</a>
                    <a class="sponsor" href="https://github.com/sponsors/imrany" target="_blank" rel="noopener">💖 Sponsor on GitHub</a>
                    <a class="email" href="mailto:imranmat254@gmail.com">📧 Contact via Email</a>
                </div>
            </div>
        </body>
        </html>
    "#.to_string())
}

pub async fn notfound_page() -> Html<String>{
    Html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>NotFound Page</title>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                html, body {
                    height: 100%;
                    margin: 0;
                    padding: 0;
                }
                body {
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background: var(--bg, #f5f5f5);
                    color: var(--fg, #222);
                    font-family: system-ui, sans-serif;
                    transition: background 0.3s, color 0.3s;
                }
                .card {
                    max-width: 400px;
                    width: 100%;
                    /* text-align: center; */
                }
                h1 {
                    margin-top: 0;
                    font-size: 2.2rem;
                    letter-spacing: -1px;
                }
                .icon {
                    font-size: 64px;
                    color: #ffa000;
                    margin-bottom: 16px;
                }
                .links {
                    margin: 2rem 0 0 0;
                    display: flex;
                    flex-direction: column;
                    gap: 1rem;
                }
                .links a {
                    display: inline-block;
                    padding: 0.7em 1.2em;
                    border-radius: 6px;
                    text-decoration: none;
                    font-weight: 500;
                    background: var(--btn-bg, #24292f);
                    color: var(--btn-fg, #fff);
                    transition: background 0.2s;
                }
                .links a.sponsor {
                    background: #db61a2;
                }
                .links a.email {
                    background: #0078d4;
                }
                .links a:hover {
                    filter: brightness(1.1);
                }
                @media (prefers-color-scheme: dark) {
                    :root {
                        --bg: #181a1b;
                        --fg: #f3f3f3;
                        --card-bg: #23272e;
                        --btn-bg: #24292f;
                        --btn-fg: #fff;
                    }
                    .links a.email { background: #2899f5; }
                }
                @media screen and (max-width: 600px) {
                    body {
                        padding: 1rem;
                        font-size: 0.9rem;
                    }
                    .card {
                        width: 100%;
                        max-width: 90%;
                    }
                    h1 {
                        font-size: 1.8rem;
                    }

                    
                }
            </style>
        
        </head>
        <body>
            <div class="card">
                <div class="icon">🔍</div>
                <h1>Not Found</h1>
                <p>The requested URL was not found on this server.</p>
                <div class="links">
                    <a href="https://github.com/imrany/proxy-server/issues" target="_blank" rel="noopener">🐞 Report Issue</a>
                    <a class="sponsor" href="https://github.com/sponsors/imrany" target="_blank" rel="noopener">💖 Sponsor on GitHub</a>
                </div>
            </div>
        </body>
        </html>
    "#.to_string())
}