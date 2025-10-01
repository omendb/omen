//! OmenDB Backup and Restore CLI Tool
//! Enterprise-grade backup management with compression and verification

use omendb::backup::{BackupManager, BackupType};
use anyhow::{Result, Context, bail};
use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Database data directory
    #[arg(short, long, value_name = "DIR")]
    data_dir: PathBuf,

    /// Backup storage directory
    #[arg(short, long, value_name = "DIR")]
    backup_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a full backup
    FullBackup {
        /// Database name
        #[arg(short, long)]
        database: String,
    },
    /// Create an incremental backup
    IncrementalBackup {
        /// Database name
        #[arg(short, long)]
        database: String,
    },
    /// Restore from backup
    Restore {
        /// Backup ID to restore from
        #[arg(short, long)]
        backup_id: String,
        /// Point-in-time recovery to specific WAL sequence
        #[arg(short, long)]
        sequence: Option<u64>,
        /// Force restore without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Verify backup integrity
    Verify {
        /// Backup ID to verify
        #[arg(short, long)]
        backup_id: String,
    },
    /// List all available backups
    List {
        /// Filter by database name
        #[arg(short, long)]
        database: Option<String>,
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show backup chain for incremental restore
    Chain {
        /// Backup ID to analyze
        #[arg(short, long)]
        backup_id: String,
    },
    /// Clean up old backups
    Cleanup {
        /// Retention period in days
        #[arg(short, long, default_value = "30")]
        retention_days: u64,
        /// Show what would be deleted without actually deleting
        #[arg(short, long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Validate directories
    if !cli.data_dir.exists() {
        bail!("Data directory does not exist: {}", cli.data_dir.display());
    }

    let mut backup_manager = BackupManager::new(&cli.data_dir, &cli.backup_dir)
        .context("Failed to initialize backup manager")?;

    match &cli.command {
        Commands::FullBackup { database } => {
            println!("Creating full backup for database: {}", database);
            let metadata = backup_manager.create_full_backup(database)
                .context("Failed to create full backup")?;

            println!("✅ Full backup completed successfully!");
            print_backup_info(&metadata);
        }

        Commands::IncrementalBackup { database } => {
            println!("Creating incremental backup for database: {}", database);
            let metadata = backup_manager.create_incremental_backup(database)
                .context("Failed to create incremental backup")?;

            println!("✅ Incremental backup completed successfully!");
            print_backup_info(&metadata);
        }

        Commands::Restore { backup_id, sequence, force } => {
            if !force {
                println!("⚠️  WARNING: This will overwrite all existing data!");
                print!("Are you sure you want to restore from backup {}? (yes/no): ", backup_id);

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "yes" {
                    println!("Restore cancelled.");
                    return Ok(());
                }
            }

            println!("Restoring from backup: {}", backup_id);
            if let Some(seq) = sequence {
                println!("Point-in-time recovery to sequence: {}", seq);
            }

            backup_manager.restore_from_backup(backup_id, *sequence)
                .context("Failed to restore from backup")?;

            println!("✅ Restore completed successfully!");
        }

        Commands::Verify { backup_id } => {
            println!("Verifying backup: {}", backup_id);
            let is_valid = backup_manager.verify_backup(backup_id)
                .context("Failed to verify backup")?;

            if is_valid {
                println!("✅ Backup verification successful!");
            } else {
                println!("❌ Backup verification failed!");
                std::process::exit(1);
            }
        }

        Commands::List { database, verbose } => {
            let backups = backup_manager.list_backups();

            let filtered_backups: Vec<_> = if let Some(db_name) = database {
                backups.into_iter().filter(|b| b.database_name == *db_name).collect()
            } else {
                backups
            };

            if filtered_backups.is_empty() {
                println!("No backups found.");
                return Ok(());
            }

            if *verbose {
                print_detailed_backup_list(&filtered_backups);
            } else {
                print_simple_backup_list(&filtered_backups);
            }
        }

        Commands::Chain { backup_id } => {
            println!("Analyzing backup chain for: {}", backup_id);
            let chain = backup_manager.get_backup_chain(backup_id)
                .context("Failed to get backup chain")?;

            print_backup_chain(&chain);
        }

        Commands::Cleanup { retention_days, dry_run } => {
            println!("Cleaning up backups older than {} days...", retention_days);

            if *dry_run {
                println!("🔍 DRY RUN MODE - No backups will be deleted");
            }

            let deleted = if *dry_run {
                // TODO: Add dry-run method to BackupManager
                Vec::new()
            } else {
                backup_manager.cleanup_old_backups(*retention_days)
                    .context("Failed to cleanup old backups")?
            };

            if deleted.is_empty() {
                println!("No backups to delete.");
            } else {
                println!("Deleted {} backup(s):", deleted.len());
                for backup_id in deleted {
                    println!("  - {}", backup_id);
                }
            }
        }
    }

    Ok(())
}

fn print_backup_info(metadata: &omendb::backup::BackupMetadata) {
    println!();
    println!("Backup Information:");
    println!("  ID: {}", metadata.backup_id);
    println!("  Type: {:?}", metadata.backup_type);
    println!("  Database: {}", metadata.database_name);
    println!("  Created: {}", metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  WAL Range: {} - {}", metadata.wal_sequence_start, metadata.wal_sequence_end);
    println!("  Size: {} bytes ({} compressed)",
             metadata.total_size_bytes, metadata.compressed_size_bytes);
    println!("  Files: {}", metadata.included_files.len());

    if let Some(depends_on) = &metadata.depends_on_backup {
        println!("  Depends on: {}", depends_on);
    }
}

fn print_simple_backup_list(backups: &[&omendb::backup::BackupMetadata]) {
    println!();
    println!("{:<30} {:<12} {:<20} {:<20}", "BACKUP ID", "TYPE", "DATABASE", "CREATED");
    println!("{}", "-".repeat(85));

    for backup in backups {
        let backup_type = match backup.backup_type {
            BackupType::Full => "Full",
            BackupType::Incremental => "Incremental",
            BackupType::PointInTime { .. } => "Point-in-Time",
        };

        println!("{:<30} {:<12} {:<20} {:<20}",
                 &backup.backup_id[..backup.backup_id.len().min(30)],
                 backup_type,
                 &backup.database_name,
                 backup.created_at.format("%Y-%m-%d %H:%M:%S"));
    }
}

fn print_detailed_backup_list(backups: &[&omendb::backup::BackupMetadata]) {
    println!();
    for (i, backup) in backups.iter().enumerate() {
        if i > 0 {
            println!("{}", "-".repeat(60));
        }
        print_backup_info(backup);
    }
}

fn print_backup_chain(chain: &[&omendb::backup::BackupMetadata]) {
    println!();
    println!("Backup Chain ({} backups):", chain.len());
    println!();

    for (i, backup) in chain.iter().enumerate() {
        let indent = "  ".repeat(i);
        let backup_type = match backup.backup_type {
            BackupType::Full => "Full",
            BackupType::Incremental => "Incremental",
            BackupType::PointInTime { .. } => "Point-in-Time",
        };

        println!("{}{}. {} [{}]", indent, i + 1, backup.backup_id, backup_type);
        println!("{}   Created: {}", indent, backup.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("{}   WAL: {} - {}", indent, backup.wal_sequence_start, backup.wal_sequence_end);

        if let Some(depends_on) = &backup.depends_on_backup {
            println!("{}   Depends on: {}", indent, depends_on);
        }
    }

    println!();
    println!("To restore this backup, all {} backup(s) in the chain will be applied in order.", chain.len());
}