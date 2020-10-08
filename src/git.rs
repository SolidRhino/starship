use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};

use crate::utils;

#[derive(Default, Debug, PartialEq)]
pub struct GitStatus {
    untracked: u8,
    added: u8,
    modified: u8,
    renamed: u8,
    deleted: u8,
    stashed: u8,
    unmerged: u8,
    ahead: u8,
    behind: u8,
    diverged: u8,
}

#[derive(Debug)]
pub struct Repository {
    git_dir: PathBuf,
    root_dir: PathBuf,
    branch: OnceCell<String>,
    status: OnceCell<GitStatus>
}

impl Repository {
    pub fn discover(path: &Path) -> Self {

    }

    pub fn git_status(path: &Path) -> Result<GitStatus> {
        // TODO: Don't bother running "git status" if no ".git" in parent directories
        let output = utils::exec_cmd("git", &["status", "--porcelain"])?.stdout;
        parse_porcelain_output(output)
    }
}

/// Parse git status values from `git status --porcelain`
///
/// Example porcelain output:
/// ```code
///  M src/prompt.rs
///  M src/main.rs
/// ```
fn parse_porcelain_output<S: Into<String>>(porcelain: S) -> Result<GitStatus> {
    let porcelain_str = porcelain.into();
    let porcelain_lines = porcelain_str.lines();
    let mut vcs_status: GitStatus = Default::default();

    porcelain_lines.for_each(|line| {
        let mut characters = line.chars();

        // Extract the first two letter of each line
        let letter_codes = (
            characters.next().unwrap_or(' '),
            characters.next().unwrap_or(' '),
        );

        if letter_codes.0 == letter_codes.1 {
            increment_git_status(&mut vcs_status, letter_codes.0);
        } else {
            increment_git_status(&mut vcs_status, letter_codes.0);
            increment_git_status(&mut vcs_status, letter_codes.1);
        }
    });

    Ok(vcs_status)
}

/// Update the cumulative git status, given the "short format" letter of a file's status
/// https://git-scm.com/docs/git-status#_short_format
fn increment_git_status(vcs_status: &mut GitStatus, letter: char) {
    match letter {
        'A' => vcs_status.added += 1,
        'M' => vcs_status.modified += 1,
        'D' => vcs_status.deleted += 1,
        'R' => vcs_status.renamed += 1,
        'C' => vcs_status.added += 1,
        'U' => vcs_status.modified += 1,
        '?' => vcs_status.untracked += 1,
        _ => (),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_porcelain_output() -> Result<()> {
        let output = parse_porcelain_output("")?;

        let expected: GitStatus = Default::default();
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_parse_porcelain_output() -> Result<()> {
        let output = parse_porcelain_output(
            "M src/prompt.rs
MM src/main.rs
A src/formatter.rs
? README.md",
        )?;

        let expected = GitStatus {
            modified: 2,
            added: 1,
            untracked: 1,
            ..Default::default()
        };
        assert_eq!(output, expected);
        Ok(())
    }
}