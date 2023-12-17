use crate::Ctx;

use cli_parser::CheckPathArgs;
use db::pkg::find_path_owners;
use ehandle::{lpm::LpmError, MainError};

pub fn check_path(ctx: Ctx, args: &CheckPathArgs) -> Result<(), LpmError<MainError>> {
    let path = std::path::PathBuf::from(args.path).canonicalize()?;
    let owners = find_path_owners(&ctx.core_db, &path)?;

    if owners.is_empty() {
        println!("Is not owned by any package.");
    } else {
        println!(
            "'{}' is currently owned by the following packages:",
            path.display()
        );
        for p in owners {
            println!("  - {}", p);
        }
    }

    Ok(())
}
