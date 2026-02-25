use crate::{git, store};

pub fn run(args: &[String]) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    if args.is_empty() {
        return Err("No git arguments provided. Usage: mind git <args...>".to_string());
    }

    git::passthrough(args)
}
