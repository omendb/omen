//! ZenDB main entry point
//!
//! Command-line interface for ZenDB database server.

use anyhow::Result;
use clap::{Arg, Command};
use zendb::{Config, ZenDB};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("zendb")
        .version("0.1.0")
        .about("The database that grows with you: zen balance from embedded to distributed")
        .arg(Arg::new("embedded")
            .long("embedded")
            .help("Run in embedded mode")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("server")
            .long("server")
            .help("Run in server mode")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("data-path")
            .long("data-path")
            .value_name("PATH")
            .help("Database file path")
            .default_value("zendb.db"))
        .arg(Arg::new("bind")
            .long("bind")
            .value_name("ADDRESS")
            .help("Server bind address")
            .default_value("127.0.0.1:5432"))
        .arg(Arg::new("cluster")
            .long("cluster")
            .help("Enable clustering mode")
            .action(clap::ArgAction::SetTrue))
        .get_matches();
    
    let mut config = Config::default();
    
    // Parse command line arguments
    if let Some(data_path) = matches.get_one::<String>("data-path") {
        config.data_path = Some(data_path.clone());
    }
    
    if matches.get_flag("server") || matches.get_flag("cluster") {
        config.distributed = true;
        if let Some(bind_addr) = matches.get_one::<String>("bind") {
            config.bind_address = Some(bind_addr.clone());
        }
    }
    
    // Print startup banner
    println!("🚀 ZenDB v0.1.0");
    println!("   Find zen in your data's natural flow");
    println!();
    
    if config.distributed {
        println!("🌐 Starting in distributed mode");
        println!("   Bind address: {}", config.bind_address.as_ref().unwrap());
        
        // TODO: Start distributed server
        println!("❌ Distributed mode not yet implemented");
        return Ok(());
    } else {
        println!("💾 Starting in embedded mode");
        println!("   Data path: {}", config.data_path.as_ref().unwrap());
        
        // Start embedded database
        let db = ZenDB::with_config(config)?;
        
        // TODO: Add interactive REPL or keep-alive mechanism
        println!("✅ ZenDB ready! (embedded mode)");
        println!("   Press Ctrl+C to exit");
        
        // Wait for interrupt
        tokio::signal::ctrl_c().await?;
        println!("\n👋 Shutting down ZenDB...");
    }
    
    Ok(())
}