use anyhow::{Context, Result};
use git2::{BranchType, Repository};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub ahead: usize,
    pub behind: usize,
}

pub struct BranchManager;

impl BranchManager {
    pub fn list_branches(repo_path: &Path) -> Result<Vec<BranchInfo>> {
        let repo = Repository::open(repo_path).context("Failed to open git repo")?;

        let mut result = Vec::new();

        for item in repo.branches(None)? {
            let (branch, branch_type) = item?;
            let name = branch.name()?.unwrap_or("").to_string();
            let is_head = branch.is_head();

            // Ahead/behind calculation requires tracking upstream
            let (ahead, behind) = if let Ok(upstream) = branch.upstream() {
                // Determine graph ahead/behind
                let local_oid = branch.get().target();
                let upstream_oid = upstream.get().target();

                if let (Some(l), Some(u)) = (local_oid, upstream_oid) {
                    if let Ok((a, b)) = repo.graph_ahead_behind(l, u) {
                        (a, b)
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            result.push(BranchInfo {
                name,
                is_current: is_head,
                is_remote: matches!(branch_type, BranchType::Remote),
                ahead,
                behind,
            });
        }

        Ok(result)
    }

    pub fn create_branch(repo_path: &Path, name: &str) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        let commit = repo.head()?.peel_to_commit()?;
        repo.branch(name, &commit, false)?;
        Ok(())
    }

    pub fn checkout(repo_path: &Path, name: &str) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        // set_head expects refs/heads/...
        // If name is just "main", we need to find the ref.
        // Usually "refs/heads/name".
        let refname = format!("refs/heads/{}", name);
        repo.set_head(&refname)?;
        repo.checkout_head(None)?;
        Ok(())
    }

    pub fn delete_branch(repo_path: &Path, name: &str) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        let mut branch = repo.find_branch(name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }
}
