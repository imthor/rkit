use std::io::{self, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::error::RkitResult;

pub fn list_repos(project_root: &PathBuf, full: bool) -> RkitResult<()> {
    log::debug!(
        "Listing repositories in workspace: {}",
        project_root.display()
    );
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == ".git" {
            let repo_path = entry.path().parent().unwrap();
            if full {
                if let Err(e) = writeln!(io::stdout(), "{}", repo_path.display()) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        // Exit gracefully on broken pipe
                        return Ok(());
                    }
                    return Err(e.into());
                }
            } else if let Ok(relative_path) = repo_path.strip_prefix(project_root) {
                if let Err(e) = writeln!(io::stdout(), "{}", relative_path.display()) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        // Exit gracefully on broken pipe
                        return Ok(());
                    }
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}
