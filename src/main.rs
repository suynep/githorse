use std::path::PathBuf;

mod git;

fn main() {
    git::walk_dir(PathBuf::new().join(".git"));
}
