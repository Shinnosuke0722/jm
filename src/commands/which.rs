use anyhow::Result;
use jm_core::storage::StorageDirs;
use jm_install::link;

use crate::output;

pub fn run(binary: &str) -> Result<()> {
    let dirs = StorageDirs::resolve()?;
    let current = link::read_current_link(&dirs.current_link())?;

    match current {
        Some(target) => {
            let bin_path = target.join("bin").join(binary);
            if bin_path.exists() {
                println!("{}", bin_path.display());
            } else {
                output::print_error(&format!(
                    "'{}' not found in current JDK ({})",
                    binary,
                    target.display()
                ));
            }
        }
        None => {
            output::print_error("No JDK version active. Use 'jm install <version>' first.");
        }
    }

    Ok(())
}
