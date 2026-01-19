use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::BTreeMap, process::Command};

use chrono::{Local, NaiveDateTime};

pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub commit_hash: String,
    pub username: String,
    pub email: String,
    pub datetime: NaiveDateTime, // will parse later (focus on one thing at a time)
    pub message: String,
}

impl Commit {
    pub fn new() -> Self {
        Commit {
            tree: String::from(""),
            parent: Some(String::from("")),
            commit_hash: String::from(""),
            username: String::from(""),
            email: String::from(""),
            datetime: NaiveDateTime::parse_from_str("2002-10-05 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(), // my bday! &
            // simple
            // unwrap cause
            // this works,
            // 4 sho
            message: String::from(""),
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nCommit {}\nTree: {}\nParent: {}\nBy: {} <{}>\nMessage: {}\n",
            self.commit_hash,
            self.tree,
            self.parent.clone().unwrap_or(String::from("None")),
            self.username,
            self.email,
            self.message
        )
    }
}

pub struct Log {
    // really want to make this a BTreeMap with time as the key and
    // associated commits as a Queue
    pub commits: BTreeMap<NaiveDateTime, Vec<Commit>>,
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

pub fn parse_commits() {
    let mut ancestor = 0;
    loop {
        let mut commit = Commit::new();

        let git_commit_cmd = Command::new("git")
            .arg("cat-file")
            .arg("commit")
            .arg(format!("HEAD~{ancestor}"))
            .output();

        let git_head_cmd = Command::new("git")
            .arg("rev-parse")
            .arg(format!("HEAD~{ancestor}"))
            .output();

        if let Ok(op) = git_commit_cmd {
            if String::from_utf8_lossy(&op.stderr).contains("fatal: Not a valid object name") {
                break;
            } else {
                let commit_str_blob = String::from_utf8_lossy(&op.stdout);
                let commit_str_blob_iter = commit_str_blob.split("\n");

                for (i, c) in commit_str_blob_iter.enumerate() {
                    /* note the format:
                     *      - tree
                     *      - parent (opt.)
                     *      - author
                     *      - committer
                     *      - \n
                     *      - message
                     */
                    if c.starts_with("tree") {
                        let tree_hash = c.strip_prefix("tree ").unwrap();
                        commit.tree = tree_hash.to_string();
                    }

                    if c.starts_with("parent") {
                        let parent_hash = c.strip_prefix("parent ").unwrap();
                        commit.parent = Some(parent_hash.to_string());
                    } else {
                        commit.parent = None;
                    }

                    if c.starts_with("author") {
                        let author_info = c.strip_prefix("author ").unwrap();
                        let split_author_info: Vec<&str> = author_info.split(" ").collect();
                        // note the format: username <email> timestamp tz
                        let username = split_author_info[0];
                        let email = split_author_info[1]
                            .strip_prefix("<")
                            .unwrap()
                            .strip_suffix(">")
                            .unwrap();
                        let timestamp = split_author_info[2];

                        commit.username = username.to_string();
                        commit.email = email.to_string();

                        commit.datetime = NaiveDateTime::from_timestamp_millis(
                            timestamp.trim().parse::<i64>().unwrap(),
                        )
                        .unwrap();
                    }

                    if i > 2 && !c.starts_with("committer") {
                        commit.message.push_str(c.trim());
                    }
                }

                if let Ok(o) = git_head_cmd {
                    commit.commit_hash = String::from_utf8_lossy(&o.stdout).trim().to_string();
                }
            }

            println!("{commit}");
        } else {
            println!("No output");
        }

        ancestor += 1;
    }
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
    #[test]
    fn test_parse_commits() {
        parse_commits();
    }
}
