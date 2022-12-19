use std::fs::DirEntry;
use std::process;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::vec::Vec;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "rdgrep")]
// Command line arguments.
struct RdgrepArgs {
    #[structopt(name = "PATH")]
    path: String,
}

const RUBY_FILE_EXTENTIONS: [&str; 1] = ["rb"];
// TODO Specify paths to exclude in configuration file.
const EXCLUDE_PATHS: [&str; 1] = ["vendor/bundle"];

/// Finds `**.rb` files except for specific paths.
fn find_ruby_files<P: AsRef<Path>>(path: P, paths: &mut Vec<PathBuf>) -> Result<(), io::Error> {
    for entry in fs::read_dir(path)? {
        let entry: DirEntry = entry?;

        if EXCLUDE_PATHS.iter().any(|p: &&str|entry.path().to_string_lossy().contains(p)) {
            continue;
        }

        if entry.file_type()?.is_dir() {
            find_ruby_files(entry.path(), paths)?;
        }

        // IF file extention is "rb", add file to list
        match entry.path().extension() {
            None => {}
            Some(extention) => {
                if RUBY_FILE_EXTENTIONS.iter().any(|e: &&str| e == &extention) {
                    paths.push(entry.path());
                }
            }
        }
    }

    Ok(())
}

/// Aggregates & returns disabled cops.
fn search(paths: Vec<PathBuf>) -> HashMap<String, i32> {
    let mut result: HashMap<String, i32> = HashMap::new();
    let len: usize = paths.len();
    let (tx, rx) = mpsc::channel::<Vec<String>>();

    for path in paths {
        let tx: mpsc::Sender<Vec<String>> = tx.clone();

        thread::spawn(move || match fs::read_to_string(path) {
            Ok(content) => {
                let copss = find_disabled_copss(&content);
                tx.send(copss).unwrap();
            }
            Err(err) => {
                eprintln!("{}", err);
                tx.send(Vec::new()).unwrap();
            }
        });
    }

    let mut n = 0;
    while n < len {
        n += 1;

        let copss: Vec<String> = rx.recv().unwrap();
        for c in copss {
            *result.entry(c).or_insert(0) += 1;
        }
    }

    result
}

/// Returns disabled cops without duplicates in the content.
fn find_disabled_copss(content: &str) -> Vec<String> {
    let mut copsersions: Vec<String> = Vec::new();
    let pattern: &str = "rubocop:disable";
    let regex: Regex = Regex::new(r"rubocop:disable( (.)*/[^ ]*)*").unwrap();

    for line in content.lines() {
        if line.contains(pattern) {
            // Delete spaces & "#" & "rubocop:disable"
            let mut comment = regex.captures(line)
                                            .unwrap()
                                            .get(0)
                                            .unwrap()
                                            .as_str()
                                            .replace(pattern, "");
            comment.retain(|c: char| c != '#' && c != ' ');

            let cops: Vec<&str> = comment.split(',').collect();
            for cop in cops {
                copsersions.push(cop.to_string());
            }
        }
    }

    copsersions.sort();
    copsersions.dedup();
    copsersions
}

pub fn run(path: String) -> Result<Vec<(String, i32)>, io::Error> {
    let mut paths = Vec::new();
    match find_ruby_files(path, paths.as_mut()) {
        Ok(_) => {
            if paths.is_empty() {
                eprintln!("A ruby file is not found");
                return Ok(Vec::new());
            }
        }
        Err(error) => return Err(error),
    };

    let result: HashMap<String, i32> = search(paths);
    let mut sorted_result: Vec<(String, i32)> = result.into_iter().collect();
    sorted_result.sort_by(|x: &(String, i32), y: &(String, i32)| y.1.cmp(&x.1));

    Ok(sorted_result)
}

fn main() {
    match run(RdgrepArgs::from_args().path) {
        Ok(result) => {
            for r in result {
                println!("{:?}", r);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::vec;

    #[test]
    fn test_find_ruby_files() {
        let mut paths = Vec::new();
        find_ruby_files("./testdata/ok/", paths.as_mut()).unwrap();

        paths.sort();

        assert_eq!(
            paths,
            vec![
                PathBuf::from("./testdata/ok/a.rb"),
                PathBuf::from("./testdata/ok/b.rb"),
                PathBuf::from("./testdata/ok/hoge/c.rb")
            ]
        )
    }

    #[test]
    fn test_find_ruby_files_empty() {
        let mut paths = Vec::new();
        find_ruby_files("./testdata/empty/", paths.as_mut()).unwrap();

        paths.sort();

        assert!(paths.is_empty())
    }

    #[test]
    fn test_find_disabled_copss() {
        let content = "# rubocop:disable Metrics/AbcSize, Metrics/BlockLength\nputs 'R2-D2'# rubocop:disable Metrics/AbcSize, Metrics/BlockNesting";
        let result = find_disabled_copss(content);

        assert_eq!(
            result,
            vec![
                "Metrics/AbcSize",
                "Metrics/BlockLength",
                "Metrics/BlockNesting"
            ]
        );
    }

    #[test]
    fn test_find_disabled_copss_empty() {
        let content = "A long time ago in a galaxy far, far away…";
        let result = find_disabled_copss(content);

        assert!(result.is_empty());
    }

    #[test]
    fn test_run() {
        let result = run("./testdata/ok/".to_string()).unwrap();
        assert_eq!(
            result,
            vec![
                ("Style/AccessModifierDeclarations".to_string(), 3),
                ("Style/Alias".to_string(), 2),
                ("Style/AccessorGrouping".to_string(), 1),
            ]
        )
    }

    #[test]
    fn test_run_empty() {
        let result = run("./testdata/empty/".to_string()).unwrap();
        assert_eq!(result, vec![])
    }

    #[test]
    fn test_run_ruby_files_does_not_exists() {
        let result = run("./testdata/ruby_files_does_not_exists/".to_string()).unwrap();
        assert_eq!(result, vec![])
    }
}
