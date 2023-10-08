use crate::location::Location;
use crate::error::{Result, Error};
use super::engine::SyncEngine;
use super::syncable::{Syncable, Diff};
use super::REMOTE_ALREADY_EXIST;


/// Synchronization engine that uses git internally.
pub struct GitSyncEngine {
    /// Repository handle.
    repo: git2::Repository,

    /// Path to repository's home.
    repo_path: std::path::PathBuf,

    /// Default git configuration.
    config: git2::Config,
}


impl GitSyncEngine {
    pub fn create<L: Location>(loc: &L, remote: Option<&str>) -> Result<Self> {
        //
        // Init or clone repository
        //

        let repo_path = Self::sync_repo_path(loc);
        match remote {
            Some(remote) => {
                git2::Repository::clone(remote, repo_path)?
            }
            None => {
                git2::Repository::init(repo_path)?
            }
        };

        //
        // Now I can just open repository and build engine
        //

        Self::open(loc)
    }

    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        let repo_path = Self::sync_repo_path(loc);
        Ok(GitSyncEngine { 
            repo: git2::Repository::open(&repo_path)?,
            repo_path: repo_path,
            config: git2::Config::open_default()?,
        })
    }
}


impl SyncEngine for GitSyncEngine {
    fn perform_sync<S: Syncable>(&self, current_instance: &str, syncable: &S) -> Result<()> {
        //
        // Get all changes from remote, create diffs and merge remote ones
        //

        self.pull_remote()?;

        let local_diff = syncable.diff_since(chrono::Utc::now())?;
        let remote_diffs = Vec::new();  // TODO

        syncable.merge_diffs(remote_diffs)?;

        //
        // Create file and serialize diff into it
        //

        let local_diff_path = self.sync_instance_path(current_instance);
        let mut local_diff_file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&local_diff_path)?;

        local_diff.write_into(&mut local_diff_file)?;

        //
        // Now commit new version and push to remote
        //

        self.commit_files([local_diff_path].iter(), current_instance)?;
        self.push_remote()
    }

    fn add_remote(&self, _remote: &str) -> Result<()> {
        let remotes_present = self.repo
            .remotes()
            .map(|remotes| !remotes.is_empty())?;

        if remotes_present {
            return Err(Error::from_message(REMOTE_ALREADY_EXIST));
        }

        Ok(())
    }
}


impl GitSyncEngine {
    fn pull_remote(&self) -> Result<()> {
        // TODO
        Ok(())
    }

    fn push_remote(&self) -> Result<()> {
        // TODO
        Ok(())
    }

    fn commit_files<T, I>(&self, pathspecs: I, current_instance: &str) -> Result<()> 
    where
        T: git2::IntoCString,
        I: Iterator<Item = T>
    {
        let reference = "HEAD";

        //
        // Let's stage our changes
        //

        let tree = self.repo
            .index()
            .and_then(|mut index| {
                index.add_all(pathspecs, git2::IndexAddOption::DEFAULT, None)?;
                index.write()?;
                index.write_tree()
            })?;
        
        let tree = self.repo
            .find_tree(tree)?;

        //
        // Create commit changes and author
        //

        let name = self.config.get_str("name")?;
        let email = self.config.get_str("email")?;
        let signature = git2::Signature::now(name, email)?;

        let message = format!("Updates from instance {}", current_instance);

        //
        // Now let's find out parent commit and perform commit
        //

        let head = self.repo
            .refname_to_id(reference)
            .and_then(|oid| self.repo.find_commit(oid))
            .ok();

        let mut parents = Vec::new();
        if let Some(head) = head.as_ref() {
            parents.push(head);
        }

        self.repo.commit(Some(reference), &signature, 
            &signature, &message, &tree, &parents)?;

        Ok(())
    }
}


impl GitSyncEngine {
    fn sync_folder<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join("sync")
    }

    fn sync_repo_path<L: Location>(loc: &L) -> std::path::PathBuf {
        Self::sync_folder(loc)
            .join("repository")
    }

    fn sync_instance_path(&self, instance: &str) -> std::path::PathBuf {
        self.repo_path
            .join(instance)
    }
}
