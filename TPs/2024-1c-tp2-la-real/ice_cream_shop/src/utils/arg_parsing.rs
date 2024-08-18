use std::env;
use tracing::Level;

#[derive(Debug)]
pub struct Args {
    pub port: String,
    pub tracing_level: Level,
    pub input_file: String,
    pub backup_file_prefix: String,
    pub max_orders_before_backup: usize,
}

impl Args {
    /// Parse command line args. Return parsed arguments or defaults if some or all were not given.
    pub fn parse_args() -> Option<Args> {
        let port = "9999".to_string();
        let mut tracing_level = Level::INFO;
        let mut input_file = "data/stock_files/test_stock.csv".to_string();
        let mut backup_file_prefix = "backup".to_string();
        let mut max_orders_before_backup = 25;

        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-d" | "--debug-level" => {
                    if i + 1 < args.len() {
                        match args[i + 1].clone().parse() {
                            Ok(level) => {
                                tracing_level = level;
                            }
                            Err(_) => {
                                println!("Invalid tracing level: {}. Use one of [TRACE, DEBUG, INFO, WARN, ERROR, FATAL]", args[i + 1]);
                                return None;
                            }
                        }
                        i += 1;
                    }
                }
                "-i" | "--inventory" => {
                    if i + 1 < args.len() {
                        input_file = args[i + 1].clone();
                        i += 1;
                    }
                }
                "-b" | "--backup-file-prefix" => {
                    if i + 1 < args.len() {
                        backup_file_prefix = args[i + 1].clone();
                        i += 1;
                    }
                }
                "-f" | "--backup-frequency" => {
                    if i + 1 < args.len() {
                        match args[i + 1].clone().parse() {
                            Ok(freq) => {
                                max_orders_before_backup = freq;
                            }
                            Err(_) => {
                                println!(
                                    "Invalid backup frequency: {}. Use a positive integer",
                                    args[i + 1]
                                );
                                return None;
                            }
                        }
                        i += 1;
                    }
                }
                "-h" | "--help" => {
                    print_help();
                    return None;
                }
                _ => {
                    println!(
                        "Unknown option: {}. Use -h | --help for possible arguments",
                        args[i]
                    );
                    return None;
                }
            }
            i += 1;
        }

        Some(Args {
            port,
            tracing_level,
            input_file,
            backup_file_prefix,
            max_orders_before_backup,
        })
    }
}
fn print_help() {
    println!("Usage: cargo run -- [options]");
    println!("Options:");
    println!("  -d, --debug-level LEVEL  Tracing level for output, can be one of [TRACE, DEBUG, INFO, WARN, ERROR, FATAL] (default: info)");
    println!("  -i, --inventory FILE     File with initial stock (default: ice_cream_shop/stock_files/test_stock.csv)");
    println!("  -b, --backup-file-prefix PREFIX  Prefix for backup files (default: backup)");
    println!(
        "  -f, --backup-frequency   FREQ  Number of orders before creating a backup (default: 10)"
    );
    println!("  -h, --help               Print this help message");
}
