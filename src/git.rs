use std::fmt;
use std::path::PathBuf;
use std::{collections::BTreeMap, process::Command};

use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<Vec<String>>,
    pub commit_hash: String,
    pub author_username: String,
    pub author_email: String,
    pub committer_username: String,
    pub committer_email: String,
    pub author_datetime: NaiveDateTime,
    pub committer_datetime: NaiveDateTime,
    pub message: String,
}

impl Commit {
    pub fn new() -> Self {
        Commit {
            tree: String::from(""),
            parent: None,
            commit_hash: String::from(""),
            author_username: String::from(""),
            author_email: String::from(""),
            committer_username: String::from(""),
            committer_email: String::from(""),
            author_datetime: NaiveDateTime::parse_from_str(
                "2002-10-05 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(), // my bday! &
            committer_datetime: NaiveDateTime::parse_from_str(
                "2002-10-05 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
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
            "\nCommit {}\nTree: {}\nParent: {:?}\nAuthor: {} <{}>\nCommitter: {} <{}>\nMessage: {}\nAuthorAt: {}\nCommitterAt: {}\n",
            self.commit_hash,
            self.tree,
            self.parent.clone().unwrap_or(vec![]),
            self.author_username,
            self.author_email,
            self.committer_username,
            self.committer_email,
            self.message,
            self.author_datetime,
            self.committer_datetime,
        )
    }
}

#[derive(Debug)]
pub struct Log {
    // really want to make this a BTreeMap with time as the key and
    // associated commits as a Queue (DONE!!!! yay)
    pub commits: BTreeMap<NaiveDateTime, Vec<Commit>>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            commits: BTreeMap::<NaiveDateTime, Vec<Commit>>::new(),
        }
    }
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

pub fn parse_commits(dir: PathBuf, start_index: Option<usize>) -> Log {
    const _BATCH_SIZE: usize = 100; // show only hundred commits on one go
    let mut ancestor = 0;
    let mut log = Log::new();

    loop {

        if ancestor > _BATCH_SIZE {
            break;
        }

        let mut commit = Commit::new();

        let mut handling_gpgsig = false;

        let git_commit_cmd = Command::new("git")
            .arg("cat-file")
            .arg("commit")
            .arg(format!("HEAD~{ancestor}"))
            .current_dir(dir.canonicalize().unwrap())
            .output();

        let git_head_cmd = Command::new("git")
            .arg("rev-parse")
            .arg(format!("HEAD~{ancestor}"))
            .current_dir(dir.canonicalize().unwrap())
            .output();

        if let Ok(op) = git_commit_cmd {
            if String::from_utf8_lossy(&op.stderr).contains("fatal: Not a valid object name") {
                break;
            } else {
                let commit_str_blob = String::from_utf8_lossy(&op.stdout);
                let commit_str_blob_iter = commit_str_blob.split("\n");

                for (i, c) in commit_str_blob_iter.enumerate() {
                    let mut line_p = false; // is the line parsed?
                    /* note the format:
                     *      - tree
                     *      - parent (opt.)
                     *      - author
                     *      - committer
                     *      - \n
                     *      - message
                     */
                    // println!("{i}: {c}");
                    if c.trim().starts_with("tree ") {
                        let tree_hash = c.strip_prefix("tree ").unwrap();
                        commit.tree = tree_hash.to_string();
                        line_p = true;
                    }

                    if c.trim().starts_with("parent ") {
                        let parent_hash = c.strip_prefix("parent ").unwrap();
                        //commit.parent = Some(parent_hash.to_string());
                        let mut cp = commit.parent.clone().unwrap_or(vec![]);
                        cp.push(parent_hash.to_string());
                        commit.parent = Some(cp);
                        line_p = true;
                    }

                    if c.trim().starts_with("author ") {
                        let author_info = c.strip_prefix("author ").unwrap();
                        let mut split_author_info: Vec<&str> = author_info.split(" ").collect();
                        // note the format: username <email> timestamp tz
                        let mut username = String::from("");
                        for s in split_author_info.clone() {
                            if !s.starts_with("<") {
                                username.push_str(s);
                                username.push_str(" ");
                                split_author_info.remove(0);
                            } else {
                                break;
                            }
                        }
                        let email = split_author_info[0]
                            .strip_prefix("<")
                            .unwrap()
                            .strip_suffix(">")
                            .unwrap();
                        let timestamp = split_author_info[1];

                        commit.author_username = username.to_string();
                        commit.author_email = email.to_string();

                        commit.author_datetime = NaiveDateTime::from_timestamp(
                            timestamp.trim().parse::<i64>().unwrap(),
                            0,
                        );
                        line_p = true;
                    }

                    if c.trim().starts_with("committer ") {
                        let author_info = c.strip_prefix("committer ").unwrap();
                        let mut split_author_info: Vec<&str> = author_info.split(" ").collect();
                        // note the format: username <email> timestamp tz
                        let mut username = String::from("");
                        for s in split_author_info.clone() {
                            if !s.starts_with("<") {
                                username.push_str(s);
                                username.push_str(" ");
                                split_author_info.remove(0);
                            } else {
                                break;
                            }
                        }
                        let email = split_author_info[0]
                            .strip_prefix("<")
                            .unwrap()
                            .strip_suffix(">")
                            .unwrap();
                        let timestamp = split_author_info[1];

                        commit.committer_username = username.to_string();
                        commit.committer_email = email.to_string();

                        commit.committer_datetime = NaiveDateTime::from_timestamp(
                            timestamp.trim().parse::<i64>().unwrap(),
                            0,
                        );
                        line_p = true;
                    }

                    // ignore the gpg signatures, and the change-id

                    // handle gpgsig fields
                    if c.trim().starts_with("gpgsig ") {
                        handling_gpgsig = true;
                    }

                    if c.trim().starts_with("-----") {
                        handling_gpgsig = false;
                        continue;
                    }

                    if handling_gpgsig {
                        continue;
                    } 

                    if !c.trim().starts_with("change-id ") && !line_p {
                        if c != "\n" {
                            commit.message.push_str(c);
                        }
                    }

                }

                if let Ok(o) = git_head_cmd {
                    commit.commit_hash = String::from_utf8_lossy(&o.stdout).trim().to_string();
                }
            }

            println!("{commit}");
            log.commits
                .entry(commit.committer_datetime)
                .and_modify(|v| v.push(commit.clone()))
                .or_insert(vec![commit]);
        } else {
            println!("No output");
        }

        ancestor += 1;
    }

    log
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
        // println!(
        //     "{:?}",
        //     parse_commits(PathBuf::new().join("test").join("gitlogue"))
        // );
        parse_commits(PathBuf::new().join("test").join("gitlogue"), None);
    }
}
