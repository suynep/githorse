use std::path::PathBuf;

pub struct Commit {
    commit_hash: String,
    author: String,
    date: String, // will parse later (focus on one thing at a time)
    message: String,
}

pub struct Log {
    commits: Vec<Commit>, // really want to make this a BTreeMap with time as the key and
                          // associated commits as a Queue
}

// recursively navigate the parents until a .git folder is encountered
pub fn check_git(dir: PathBuf) -> bool {
    let mut current = dir;
    let mut depth: u8 = 5;

    loop {
        if depth == 0 {
            break false;
        }
        if let Ok(c) = current.read_dir() {
            for fd in c {
                if let Ok(c) = fd {
                    if c.file_name() == ".git" {
                        println!("Found .git folder: {:?}", current);
                        return true;
                    } else {
                        current = PathBuf::new().join(current).join("..");
                    }
                }
                depth -= 1;
            }
        }
    }
}

pub fn parse_commits(c_file: PathBuf) -> Log {
    todo!()
}

pub fn walk_dir(dir: PathBuf) {
    let content_iter = dir.read_dir().unwrap();
    for c in content_iter {
        println!("{:?}", c);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check_git() {
        check_git(PathBuf::new().join("."));
    }

    #[test]
    fn test_walk_dir() {
        walk_dir(PathBuf::new().join(".git"));
    }
}
