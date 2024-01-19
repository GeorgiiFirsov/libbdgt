use crate::location::Location;
use crate::error::{Result, Error};
use crate::datetime::{Clock, Timestamp};
use super::engine::SyncEngine;
use super::syncable::Syncable;
use super::{REMOTE_ALREADY_EXIST, MALFORMED_LAST_SYNC_TIMESTAMP};


/// Name of git's remote for the repository.
const REMOTE_NAME: &str = "origin";

/// Name of reference to update on commit.
const REF_NAME: &str = "HEAD";

/// Branch name.
const BRANCH_NAME: &str = "main";

/// Name of configuration parameter that contains a username.
const CFG_NAME: &str = "name";

/// Name of configuration parameter that contains an email.
const CFG_EMAIL: &str = "email";

/// Synchronization folder.
const SYNC_FORDER: &str = "sync";

/// File that holds last synchronization time.
const LAST_SYNC_FILE: &str = "last-sync";

/// Repository folder.
const SYNC_REPO: &str = "repository";

/// File with last synchronization timestamp.
const TIMESTAMP_FILE: &str = "timestamp";

/// File with last synchronized instance timestamp.
const LAST_INSTANCE_FILE: &str = "instance";

/// File with full changelog.
const CHANGELOG_FILE: &str = "changelog";


/// Synchronization engine that uses git internally.
pub struct GitSyncEngine {
    /// Repository handle.
    repo: git2::Repository,

    /// Path to repository's home.
    repo_path: std::path::PathBuf,

    /// Path to last sync timestamp file.
    last_sync_path: std::path::PathBuf,

    /// Default git configuration.
    config: git2::Config,
}


impl GitSyncEngine {
    pub fn create<L: Location>(loc: &L, remote: Option<&str>) -> Result<Self> {
        //
        // Check is root location exists and create it if necessary.
        // Sync folder should be created manually
        //

        loc.create_if_absent()?;
        std::fs::create_dir(Self::sync_folder(loc))?;

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
        // Create last sync file
        //

        let last_sync_path = Self::sync_last_sync_path(loc);
        let january_1970 = Timestamp::from_timestamp(0, 0)
            .expect("Zero is a valid timestamp");

        let mut file = std::fs::File::create(last_sync_path)?;
        Self::write_last_sync(&mut file, &january_1970)?;

        //
        // Now I can just open repository and build engine
        //

        Self::open(loc)
    }

    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        let repo_path = Self::sync_repo_path(loc);
        let last_sync_path = Self::sync_last_sync_path(loc);

        Ok(GitSyncEngine { 
            repo: git2::Repository::open(&repo_path)?,
            repo_path: repo_path,
            last_sync_path: last_sync_path,
            config: git2::Config::open_default()?,
        })
    }
}


impl SyncEngine for GitSyncEngine {
    fn perform_sync<S: Syncable>(&self, current_instance: &str, syncable: &S, context: &S::Context) -> Result<()> {
        //
        // Get all changes from remote and open raw files
        //

        self.pull_remote()?;

        let mut timestamp_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.syncable_file_path(TIMESTAMP_FILE))?;

        let mut last_instance_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.syncable_file_path(LAST_INSTANCE_FILE))?;

        let mut changelog_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.syncable_file_path(CHANGELOG_FILE))?;

        //
        // Perform actual synchronization (read last sync timestamp just before and
        // write right after the process)
        //

        let mut last_sync_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.last_sync_path)?;

        syncable.merge_and_export_changes(&mut timestamp_file, &mut last_instance_file, 
            &mut changelog_file, &Self::read_last_sync(&mut last_sync_file)?, context)?;

        Self::write_last_sync(&mut last_sync_file, &Clock::now())?;

        //
        // Now commit new versions of files and push to remote
        //

        let branch_ref = self.commit_files([TIMESTAMP_FILE, LAST_INSTANCE_FILE, CHANGELOG_FILE].iter(), 
            &format!("Updates from {}", current_instance))?;

        self.push_remote(&branch_ref)
    }

    fn add_remote(&self, remote: &str) -> Result<()> {
        if let Ok(_) = self.repo.find_remote(REMOTE_NAME) {
            return Err(Error::from_message(REMOTE_ALREADY_EXIST));
        }

        self.repo
            .remote(REMOTE_NAME, remote)?;

        Ok(())
    }

    fn remove_remote(&self) -> Result<()> {
        self.repo
            .remote_delete(REMOTE_NAME)?;

        Ok(())
    }

    fn change_remote(&self, remote: &str) -> Result<()> {
        self.remove_remote()?;
        self.add_remote(remote)
    }
}


impl GitSyncEngine {
    fn pull_remote(&self) -> Result<()> {
        // TODO
        Ok(())
    }

    fn push_remote(&self, branch_ref: &str) -> Result<()> {
        self.repo.find_remote(REMOTE_NAME)
            .and_then(|mut remote| remote.push(&[branch_ref], None))
            .map_err(Error::from)
    }

    fn commit_files<T, I>(&self, pathspecs: I, message: &str) -> Result<String> 
    where
        T: git2::IntoCString,
        I: Iterator<Item = T>
    {
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

        let name = self.config.get_str(CFG_NAME)?;
        let email = self.config.get_str(CFG_EMAIL)?;
        let signature = git2::Signature::now(name, email)?;

        //
        // Now let's find out parent commit and perform commit
        //

        let head = self.repo
            .refname_to_id(REF_NAME)
            .and_then(|oid| self.repo.find_commit(oid))
            .ok();

        let mut parents = Vec::new();
        if let Some(head) = head.as_ref() {
            parents.push(head);
        }

        let commit = self.repo.commit(Some(REF_NAME), &signature, 
            &signature, &message, &tree, &parents)?;

        //
        // Update branch pointer
        //

        let commit = self.repo.find_commit(commit)?;
        let branch = self.repo.branch(BRANCH_NAME, &commit, true)
            .map(|b| b.into_reference())?;

        let branch_ref = branch.name()
            .expect("Branch MUST have name")
            .to_owned();

        Ok(branch_ref)
    }
}


impl GitSyncEngine {
    fn read_last_sync<R: std::io::Read>(last_sync: &mut R) -> Result<Timestamp> {
        let mut buffer = [0; std::mem::size_of::<i64>()];
        let seconds = match last_sync.read_exact(&mut buffer) {
            Ok(_) => i64::from_le_bytes(buffer),
            _ => 0i64
        };

        Timestamp::from_timestamp(seconds, 0)
            .ok_or(Error::from_message(MALFORMED_LAST_SYNC_TIMESTAMP))
    }

    fn write_last_sync<W: std::io::Write>(last_sync: &mut W, timestamp: &Timestamp) -> Result<()> {
        let timestamp = timestamp
            .timestamp()
            .to_le_bytes();

        last_sync
            .write_all(&timestamp)
            .map_err(Error::from)
    }
}


impl GitSyncEngine {
    fn sync_folder<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join(SYNC_FORDER)
    }

    fn sync_repo_path<L: Location>(loc: &L) -> std::path::PathBuf {
        Self::sync_folder(loc)
            .join(SYNC_REPO)
    }

    fn sync_last_sync_path<L: Location>(loc: &L) -> std::path::PathBuf {
        Self::sync_folder(loc)
            .join(LAST_SYNC_FILE)
    }

    fn syncable_file_path(&self, file: &str) -> std::path::PathBuf {
        self.repo_path
            .join(file)
    }
}
