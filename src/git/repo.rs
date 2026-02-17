use anyhow::{Context, Result};
use git2::{DiffOptions, Repository, StatusOptions};

use super::diff::{DiffHunk, DiffLine, DiffLineKind, FileDiff};
use super::status::{FileStatus, FileStatusKind, StageState};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open(path: &str) -> Result<Self> {
        let repo = Repository::discover(path)
            .context("Failed to find git repository")?;
        Ok(Self { repo })
    }

    pub fn workdir(&self) -> Option<&std::path::Path> {
        self.repo.workdir()
    }

    pub fn status_list(&self) -> Result<Vec<FileStatus>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_unmodified(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut result = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            let index_status = if status.contains(git2::Status::INDEX_NEW) {
                Some(FileStatusKind::New)
            } else if status.contains(git2::Status::INDEX_MODIFIED) {
                Some(FileStatusKind::Modified)
            } else if status.contains(git2::Status::INDEX_DELETED) {
                Some(FileStatusKind::Deleted)
            } else if status.contains(git2::Status::INDEX_RENAMED) {
                Some(FileStatusKind::Renamed)
            } else if status.contains(git2::Status::INDEX_TYPECHANGE) {
                Some(FileStatusKind::Typechange)
            } else {
                None
            };

            let worktree_status = if status.contains(git2::Status::WT_NEW) {
                Some(FileStatusKind::Untracked)
            } else if status.contains(git2::Status::WT_MODIFIED) {
                Some(FileStatusKind::Modified)
            } else if status.contains(git2::Status::WT_DELETED) {
                Some(FileStatusKind::Deleted)
            } else if status.contains(git2::Status::WT_RENAMED) {
                Some(FileStatusKind::Renamed)
            } else if status.contains(git2::Status::WT_TYPECHANGE) {
                Some(FileStatusKind::Typechange)
            } else {
                None
            };

            let kind = worktree_status.clone()
                .or(index_status.clone())
                .unwrap_or(FileStatusKind::Modified);

            let stage_state = match (&index_status, &worktree_status) {
                (Some(_), Some(_)) => StageState::Partial,
                (Some(_), None) => StageState::Staged,
                _ => StageState::Unstaged,
            };

            if status.contains(git2::Status::CONFLICTED) {
                result.push(FileStatus {
                    path,
                    kind: FileStatusKind::Conflicted,
                    stage_state: StageState::Unstaged,
                    index_status: None,
                    worktree_status: Some(FileStatusKind::Conflicted),
                });
            } else {
                result.push(FileStatus {
                    path,
                    kind,
                    stage_state,
                    index_status,
                    worktree_status,
                });
            }
        }

        Ok(result)
    }

    pub fn diff_file(&self, path: &str, staged: bool) -> Result<FileDiff> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);

        let diff = if staged {
            let head_tree = self.repo.head()
                .ok()
                .and_then(|h| h.peel_to_tree().ok());
            self.repo.diff_tree_to_index(
                head_tree.as_ref(),
                None,
                Some(&mut diff_opts),
            )?
        } else {
            self.repo.diff_index_to_workdir(None, Some(&mut diff_opts))?
        };

        let mut hunks = Vec::new();
        let mut current_lines: Vec<DiffLine> = Vec::new();
        let mut current_header = String::new();

        diff.print(git2::DiffFormat::Patch, |_delta, hunk, line| {
            match line.origin() {
                'H' | 'F' => {}
                _ => {
                    if let Some(hunk) = hunk {
                        let header = String::from_utf8_lossy(hunk.header()).to_string();
                        if header != current_header && !current_header.is_empty() {
                            hunks.push(DiffHunk {
                                header: current_header.clone(),
                                lines: std::mem::take(&mut current_lines),
                            });
                        }
                        if header != current_header {
                            current_header = header.clone();
                            current_lines.push(DiffLine {
                                kind: DiffLineKind::HunkHeader,
                                content: header,
                                old_lineno: None,
                                new_lineno: None,
                            });
                        }
                    }

                    let content = String::from_utf8_lossy(line.content()).to_string();
                    let kind = match line.origin() {
                        '+' | '>' => DiffLineKind::Addition,
                        '-' | '<' => DiffLineKind::Deletion,
                        _ => DiffLineKind::Context,
                    };

                    current_lines.push(DiffLine {
                        kind,
                        content,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                    });
                }
            }
            true
        })?;

        if !current_lines.is_empty() {
            hunks.push(DiffHunk {
                header: current_header,
                lines: current_lines,
            });
        }

        Ok(FileDiff {
            path: path.to_string(),
            hunks,
        })
    }

    pub fn branch_name(&self) -> Result<String> {
        let head = self.repo.head()?;
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }
}
