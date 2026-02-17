use anyhow::{Context, Result};
use std::process::Command;

pub struct GitOps {
    workdir: String,
}

impl GitOps {
    pub fn new(workdir: &str) -> Self {
        Self {
            workdir: workdir.to_string(),
        }
    }

    fn git(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.workdir);
        cmd
    }

    pub fn stage_file(&self, path: &str) -> Result<()> {
        let output = self.git()
            .args(["add", "--", path])
            .output()
            .context("Failed to run git add")?;
        if !output.status.success() {
            anyhow::bail!("git add failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let output = self.git()
            .args(["reset", "HEAD", "--", path])
            .output()
            .context("Failed to run git reset")?;
        if !output.status.success() {
            anyhow::bail!("git reset failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let output = self.git()
            .args(["add", "-A"])
            .output()
            .context("Failed to run git add -A")?;
        if !output.status.success() {
            anyhow::bail!("git add -A failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let output = self.git()
            .args(["commit", "-m", message])
            .output()
            .context("Failed to run git commit")?;
        if !output.status.success() {
            anyhow::bail!("git commit failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn push(&self) -> Result<String> {
        let output = self.git()
            .args(["push"])
            .output()
            .context("Failed to run git push")?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        if !output.status.success() {
            anyhow::bail!("git push failed: {}", combined.trim());
        }
        Ok(combined)
    }

    pub fn pull(&self) -> Result<String> {
        let output = self.git()
            .args(["pull"])
            .output()
            .context("Failed to run git pull")?;
        if !output.status.success() {
            anyhow::bail!("git pull failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn discard_file(&self, path: &str) -> Result<()> {
        let output = self.git()
            .args(["checkout", "--", path])
            .output()
            .context("Failed to run git checkout")?;
        if !output.status.success() {
            anyhow::bail!("git checkout failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn stash(&self) -> Result<String> {
        let output = self.git()
            .args(["stash"])
            .output()
            .context("Failed to run git stash")?;
        if !output.status.success() {
            anyhow::bail!("git stash failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn stash_pop(&self) -> Result<String> {
        let output = self.git()
            .args(["stash", "pop"])
            .output()
            .context("Failed to run git stash pop")?;
        if !output.status.success() {
            anyhow::bail!("git stash pop failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn branch_list(&self) -> Result<Vec<String>> {
        let output = self.git()
            .args(["branch", "--format=%(refname:short)"])
            .output()
            .context("Failed to run git branch")?;
        if !output.status.success() {
            anyhow::bail!("git branch failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    pub fn create_branch(&self, name: &str) -> Result<()> {
        let output = self.git()
            .args(["checkout", "-b", name])
            .output()
            .context("Failed to run git checkout -b")?;
        if !output.status.success() {
            anyhow::bail!("git checkout -b failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let output = self.git()
            .args(["checkout", name])
            .output()
            .context("Failed to run git checkout")?;
        if !output.status.success() {
            anyhow::bail!("git checkout failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }
}
