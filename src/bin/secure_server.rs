//! Secure OmenDB monitoring server with authentication
//! Run with: cargo run --release --bin secure_server

use omendb::security::SecurityContext;
use omendb::server::start_secure_monitoring_server;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔒 OmenDB Secure Monitoring Server");
    println!("===================================");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse().unwrap_or(3000)
    } else {
        3000
    };

    // Load security configuration from environment
    let security_ctx = match SecurityContext::from_env() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to load security configuration: {}", e);
            eprintln!("Using default configuration");
            SecurityContext::default()
        }
    };

    println!("🚀 Starting secure monitoring server...");
    println!();

    // Print security configuration
    if security_ctx.auth.enabled {
        println!("🔐 Security Status: ENABLED");
        println!("   Users configured: {}", security_ctx.auth.users.len());
        println!("   Session timeout: {}s", security_ctx.auth.session_timeout);

        if security_ctx.auth.users.contains_key("admin") {
            println!("   ⚠️  Default admin user detected - CHANGE PASSWORD IN PRODUCTION!");
        }
    } else {
        println!("⚠️  Security Status: DISABLED");
        println!("   Set OMENDB_AUTH_DISABLED=false to enable authentication");
    }

    if security_ctx.tls.enabled {
        println!("🔒 TLS Status: ENABLED");
        println!("   Certificate: {}", security_ctx.tls.cert_file);
        println!("   Private Key: {}", security_ctx.tls.key_file);
    } else {
        println!("🔓 TLS Status: DISABLED");
        println!("   Set OMENDB_TLS_ENABLED=true to enable TLS");
    }

    println!();
    println!("🌐 Environment Configuration:");
    println!(
        "   OMENDB_AUTH_DISABLED={}",
        env::var("OMENDB_AUTH_DISABLED").unwrap_or_default()
    );
    println!(
        "   OMENDB_ADMIN_USER={}",
        env::var("OMENDB_ADMIN_USER").unwrap_or("admin".to_string())
    );
    println!(
        "   OMENDB_TLS_ENABLED={}",
        env::var("OMENDB_TLS_ENABLED").unwrap_or_default()
    );

    println!();
    println!("📖 Usage Examples:");
    println!("   # Access metrics (requires auth):");
    println!(
        "   curl -u admin:admin123 http://localhost:{}/metrics",
        port
    );
    println!();
    println!("   # Check health (requires auth):");
    println!("   curl -u admin:admin123 http://localhost:{}/health", port);
    println!();
    println!("   # Ready check (no auth):");
    println!("   curl http://localhost:{}/ready", port);
    println!();
    println!("   # Basic status (no auth):");
    println!("   curl http://localhost:{}/status", port);

    println!();
    println!("🔧 Configuration via Environment Variables:");
    println!("   OMENDB_AUTH_DISABLED=true|false");
    println!("   OMENDB_ADMIN_USER=username");
    println!("   OMENDB_ADMIN_PASSWORD=password");
    println!("   OMENDB_JWT_SECRET=secret");
    println!("   OMENDB_SESSION_TIMEOUT=3600");
    println!("   OMENDB_TLS_ENABLED=true|false");
    println!("   OMENDB_TLS_CERT=path/to/cert.pem");
    println!("   OMENDB_TLS_KEY=path/to/key.pem");

    println!();

    // Start the server
    start_secure_monitoring_server(port, security_ctx).await
}
