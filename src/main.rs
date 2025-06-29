use axum::{
    body::Body,
    extract::Request,
    http::{
        Method, StatusCode
    },
    response::{
        IntoResponse, Response
    },
    routing::{get, post},
    Router,
};

mod handlers;
use handlers::{
    users::{
        UserStatsState,
        get_user_stats,
        update_user_stats,
    },
    connections::{
        ConnectionInfo,
        MonitoringState,
        get_connections,
        get_active_connections,
        log_connection,
        update_connection_status,
    },   
    payments::{
        PaymentState,
        record_payment,
        check_payment_status,
        get_all_payments,
        has_valid_payment,
    }
};

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::upgrade::Upgraded;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tower::Service;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer}
};
use hyper_util::rt::TokioIo;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing::Level;
use chrono::Utc;

mod read_txt;
use read_txt::check_address_block;


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true)
        )
        .init();

    // Initialize monitoring state
    let monitoring_state: MonitoringState = Arc::new(RwLock::new(HashMap::new()));
    let user_stats_state: UserStatsState = Arc::new(RwLock::new(HashMap::new()));
    let payment_state: PaymentState = Arc::new(RwLock::new(HashMap::new()));

    let monitoring_api = Router::new()
        .route("/connections", get(get_connections))
        .route("/active", get(get_active_connections))
        .with_state(monitoring_state.clone());

    let stats_api = Router::new()
        .route("/stats", get(get_user_stats))
        .with_state(user_stats_state.clone());

    let payment_api = Router::new()
        .route("/payment/record", post(record_payment))
        .route("/payment/status", get(check_payment_status))
        .route("/payment/all", get(get_all_payments))
        .with_state(payment_state.clone());

    let api_routes = Router::new()
        .merge(monitoring_api)
        .merge(stats_api)
        .merge(payment_api);
        
    let router_svc = Router::new()
        .nest_service(
            "/", ServeDir::new("assets/static")
            .not_found_service(ServeFile::new("assets/static/not_found.html")),
        )
        .nest("/api", api_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new()
                        .level(Level::INFO)
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::INFO)
                ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("🚀 Proxy server listening on {}", addr);
    tracing::info!("📊 Monitor endpoints:");
    tracing::info!("  - GET /api/connections - All connections");
    tracing::info!("  - GET /api/stats - User statistics");
    tracing::info!("  - GET /api/active - Active connections");
    tracing::info!("💰 Payment endpoints:");
    tracing::info!("  - POST /api/payment/record - Record payment");
    tracing::info!("  - GET /api/payment/status?ip=<client_ip> - Check payment status");
    tracing::info!("  - GET /api/payment/all - All payment records");

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (stream, client_addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("Failed to accept connection: {:?}", e);
                continue;
            }
        };
        tracing::info!("🔗 New connection from: {}", client_addr.ip());

        let router_svc = router_svc.clone();
        let monitoring_state = monitoring_state.clone();
        let user_stats_state = user_stats_state.clone();
        let payment_state = payment_state.clone();

        tokio::spawn(async move {
            let tower_service = tower::service_fn(move |req: Request<_>| {
                let router_svc = router_svc.clone();
                let monitoring_state = monitoring_state.clone();
                let user_stats_state = user_stats_state.clone();
                let payment_state = payment_state.clone();
                let req = req.map(Body::new);

                async move {
                    if req.method() == Method::CONNECT {
                        proxy(req, monitoring_state, user_stats_state, payment_state, client_addr.ip().to_string()).await
                    } else {
                        router_svc.oneshot(req)
                            .await
                            .map_err(|err| {
                                tracing::error!("Router service error: {:?}", err);
                                match err {}
                            })
                    }
                }
            });

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let io = TokioIo::new(stream);
            let hyper_service = hyper_service.clone();

            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, hyper_service)
                .with_upgrades()
                .await
            {
                tracing::warn!("❌ Failed to serve connection from {}: {:?}", client_addr.ip(), err);
            }
        });
    }
}

async fn proxy(
    req: Request, 
    monitoring_state: MonitoringState,
    user_stats_state: UserStatsState,
    payment_state: PaymentState,
    client_ip: String
) -> Result<Response, hyper::Error> {
    let headers = req.headers().clone();
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    tracing::info!("🔍 CONNECT request from: {} | User-Agent: {:?}", 
        client_ip, user_agent);

    // Check payment status first
    if !has_valid_payment(&payment_state, &client_ip).await {
        tracing::warn!("💳 PAYMENT REQUIRED: {} has no valid payment", client_ip);
        
        let timestamp = Utc::now();
        let conn_info = ConnectionInfo {
            client_ip: client_ip.clone(),
            target_host: req.uri().authority().map(|auth| auth.to_string()).unwrap_or_default(),
            timestamp,
            user_agent: user_agent.clone(),
            bytes_sent: 0,
            bytes_received: 0,
            status: "payment_required".to_string(),
            duration_ms: Some(0),
        };
        
        log_connection(monitoring_state.clone(), conn_info).await;
        
        return Ok(Response::builder()
            .status(StatusCode::PAYMENT_REQUIRED)
            .header("Content-Type", "text/html")
            .body(Body::from(format!(
                r#"<!DOCTYPE html>
                <html><head><title>Payment Required</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
                    .container {{ background: white; padding: 20px; border-radius: 8px; max-width: 600px; margin: 0 auto; }}
                    .error {{ color: #d63384; font-size: 24px; margin-bottom: 20px; }}
                    .info {{ background: #e7f3ff; padding: 15px; border-radius: 5px; margin: 20px 0; }}
                    .payment-methods {{ background: #f8f9fa; padding: 15px; border-radius: 5px; margin: 20px 0; }}
                </style>
                </head>
                <body>
                <div class="container">
                    <h1 class="error">💳 Payment Required</h1>
                    <p>Access to the proxy service requires a monthly payment of <strong>KSH 150</strong>.</p>
                    
                    <div class="info">
                        <strong>Your IP:</strong> {}<br>
                        <strong>Timestamp:</strong> {}<br>
                        <strong>Monthly Fee:</strong> KSH 150
                    </div>
                    
                    <div class="payment-methods">
                        <h3>Payment Methods:</h3>
                        <ul>
                            <li><strong>M-Pesa:</strong> Send KSH 150 to [Your M-Pesa Number]</li>
                            <li><strong>Bank Transfer:</strong> [Your Bank Details]</li>
                            <li><strong>Cash:</strong> Visit our office</li>
                        </ul>
                        <p><em>After payment, contact admin to activate your access.</em></p>
                    </div>
                    
                    <p>Once payment is confirmed, you'll have 30 days of unlimited proxy access.</p>
                </div>
                </body></html>"#,
                client_ip, timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )))
            .unwrap());
    }

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        let timestamp = Utc::now();
        
        // Check if address should be blocked (existing block list check)
        if check_address_block(&host_addr) {
            tracing::warn!("🚫 BLOCKED: {} attempting to connect to {}", client_ip, host_addr);
            
            let conn_info = ConnectionInfo {
                client_ip: client_ip.clone(),
                target_host: host_addr.clone(),
                timestamp,
                user_agent: user_agent.clone(),
                bytes_sent: 0,
                bytes_received: 0,
                status: "blocked".to_string(),
                duration_ms: Some(0),
            };
            
            log_connection(monitoring_state.clone(), conn_info).await;
            update_user_stats(user_stats_state.clone(), &client_ip, &host_addr, 0, true).await;
            
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "text/html")
                .body(Body::from(format!(
                    r#"<!DOCTYPE html>
                    <html><head><title>Access Denied</title></head>
                    <body><h1>🚫 Access Denied</h1>
                    <p>Connection to <strong>{}</strong> blocked by policy.</p>
                    <p>Your IP: {}</p><p>Timestamp: {}</p></body></html>"#,
                    host_addr, client_ip, timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                )))
                .unwrap());
        }

        tracing::info!("✅ ALLOWED: {} → {} (Payment verified)", client_ip, host_addr);
        
        // Log connection attempt
        let conn_info = ConnectionInfo {
            client_ip: client_ip.clone(),
            target_host: host_addr.clone(),
            timestamp,
            user_agent,
            bytes_sent: 0,
            bytes_received: 0,
            status: "active".to_string(),
            duration_ms: None,
        };
        
        log_connection(monitoring_state.clone(), conn_info).await;
        update_user_stats(user_stats_state.clone(), &client_ip, &host_addr, 0, false).await;

        // Spawn tunnel task
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    let start_time = Utc::now();
                    match tunnel(upgraded, host_addr.clone()).await {
                        Ok((bytes_sent, bytes_received)) => {
                            let duration = Utc::now().signed_duration_since(start_time);
                            let duration_ms = duration.num_milliseconds() as u64;
                            
                            tracing::info!("✅ Tunnel completed: {} → {} | ⬆️ {} bytes ⬇️ {} bytes | ⏱️ {}ms", 
                                client_ip, host_addr, bytes_sent, bytes_received, duration_ms);
                            
                            update_connection_status(
                                monitoring_state, 
                                &client_ip, 
                                &host_addr, 
                                bytes_sent, 
                                bytes_received, 
                                duration_ms,
                                "completed"
                            ).await;
                            
                            update_user_stats(user_stats_state, &client_ip, &host_addr, bytes_sent + bytes_received, false).await;
                        }
                        Err(e) => {
                            tracing::error!("❌ Tunnel error: {} → {} | Error: {}", client_ip, host_addr, e);
                            update_connection_status(
                                monitoring_state, 
                                &client_ip, 
                                &host_addr, 
                                0, 
                                0, 
                                0,
                                "failed"
                            ).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("❌ Upgrade error: {} → {} | Error: {}", client_ip, host_addr, e);
                    update_connection_status(
                        monitoring_state, 
                        &client_ip, 
                        &host_addr, 
                        0, 
                        0, 
                        0,
                        "failed"
                    ).await;
                }
            }
        });

        Ok(Response::new(Body::empty()))
    } else {
        tracing::warn!("⚠️ Invalid CONNECT request from {}: {:?}", client_ip, req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        ).into_response())
    }
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<(u64, u64)> {
    let mut server = TcpStream::connect(&addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    Ok((from_client, from_server))
}