mod shop_connection;
mod shop_stock;
mod utils;

fn main() -> Result<(), String> {
    match utils::arg_parsing::Args::parse_args() {
        Some(args) => utils::setup_shop::setup_shop(args),
        None => Ok(()),
    }
}
