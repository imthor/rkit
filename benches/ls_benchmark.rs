use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::env;
use std::path::PathBuf;

fn bench_ls_command(c: &mut Criterion) {
    // Check if the platform is Unix-like
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )))]
    {
        eprintln!("Warning: This benchmark is designed for Unix-like systems. Behavior may be unexpected on this platform.");
    }

    // Allow the project root to be overridden by an environment variable
    let project_root = match env::var("RKIT_BENCH_ROOT") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            // Fallback to HOME, or use a default if HOME is not set
            match env::var("HOME") {
                Ok(home) => PathBuf::from(home).join("projects"),
                Err(_) => {
                    eprintln!("Warning: HOME environment variable not set. Using default path '/tmp/rkit_bench'.");
                    PathBuf::from("/tmp/rkit_bench")
                }
            }
        }
    };

    c.bench_function("ls_command", |b| {
        b.iter(|| rkit::commands::ls::list_repos(black_box(&project_root), black_box(false), None))
    });
}

criterion_group!(benches, bench_ls_command);
criterion_main!(benches);
