// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::process::Command;

use camino::Utf8Path;
use clap::Args;

use crate::paths::absolute_msys_path;
use crate::run::run_command;

#[derive(Args)]
pub struct RsyncArgs {
    #[arg(long, value_delimiter(','), allow_hyphen_values(true))]
    extra_args: Vec<String>,
    #[arg(long)]
    prefix: String,
    #[arg(long, required(true), num_args(..))]
    inputs: Vec<String>,
    #[arg(long)]
    output_dir: String,
}

fn copy_recursive(src: &std::path::Path, dest: &std::path::Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            copy_recursive(&entry_path, &dest_path)?;
        }
    } else {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

pub fn rsync_files(args: RsyncArgs) {
    if cfg!(windows) {
        let prefix = std::path::Path::new(&args.prefix);
        let output_dir = std::path::Path::new(&args.output_dir);
        for input in &args.inputs {
            let src = prefix.join(input);
            let dest = output_dir.join(input);
            copy_recursive(&src, &dest).unwrap_or_else(|e| {
                panic!("failed to copy {} to {}: {}", src.display(), dest.display(), e)
            });
        }
    } else {
        let output_dir = absolute_msys_path(Utf8Path::new(&args.output_dir));
        run_command(
            Command::new("rsync")
                .current_dir(&args.prefix)
                .arg("--relative")
                .args(args.extra_args)
                .args(args.inputs.iter())
                .arg(output_dir),
        );
    }
}
