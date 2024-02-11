use crate::location::Location;
use crate::error::{Result, Error};
use crate::datetime::{Clock, Timestamp, FIRST_AFTER_JANUARY_1970};
use super::engine::SyncEngine;
use super::syncable::Syncable;
use super::{REMOTE_ALREADY_EXIST, MALFORMED_LAST_SYNC_TIMESTAMP, REMOTE_CONFLICT};


/// Name of git's remote for the repository.
const REMOTE_NAME: &str = "origin";

/// Name of reference to update on commit.
const REF_NAME: &str = "HEAD";

/// Name of reference to fetched head.
const FETCH_REF_NAME: &str = "FETCH_HEAD";

/// Branch name.
const BRANCH_NAME: &str = "main";

/// Name of configuration parameter that contains a username.
const CFG_NAME: &str = "user.name";

/// Name of configuration parameter that contains an email.
const CFG_EMAIL: &str = "user.email";

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

    /// Default authenticator
    /// Usually it is used with `config`
    authenticator: auth_git2::GitAuthenticator,
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
                auth_git2::GitAuthenticator::default()
                    .clone_repo(remote, repo_path)?
            }
            None => {
                git2::Repository::init(repo_path)?
            }
        };

        //
        // Create last sync file
        // I write first nonzero timestamp after January 1970 to
        // ensure, that all predefined items will not by
        // synced between instances
        //

        let last_sync_path = Self::sync_last_sync_path(loc);
        let mut file = std::fs::File::create(last_sync_path)?;

        Self::write_last_sync(&mut file, &FIRST_AFTER_JANUARY_1970)?;

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
            authenticator: auth_git2::GitAuthenticator::default(),
        })
    }
}


impl SyncEngine for GitSyncEngine {
    fn perform_sync<S: Syncable>(&self, current_instance: &S::InstanceId, syncable: &S, context: &S::Context) -> Result<()> {
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

        Self::prepare_for_overwrite(&mut last_sync_file)?;
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
        //
        // Fetch remote changes
        //

        let config = self.repo.config()?;
        let mut fetch_options = git2::FetchOptions::default();
        fetch_options.remote_callbacks(self.remote_callbacks(&config));

        self.repo.find_remote(REMOTE_NAME)
            .and_then(|mut remote| remote.fetch(&[BRANCH_NAME], Some(&mut fetch_options), None))?;

        let fetch_head = match self.repo.find_reference(FETCH_REF_NAME) {
            Ok(r) => r,
            _ => return Ok(())  // Pulling an empty repository
        };

        let fetch_commit = self.repo
            .reference_to_annotated_commit(&fetch_head)?;

        //
        // Perform merge analysis
        //

        let (merge_analysis, _) = self.repo
            .merge_analysis(&[&fetch_commit])?;

        if merge_analysis.is_up_to_date() {
            return Ok(());
        }

        if !merge_analysis.is_fast_forward() {
            //
            // Fast-forward is only possible option. If something else
            // is occurred, it is considered to be an error.
            //

            return Err(Error::from_message(REMOTE_CONFLICT));
        }

        //
        // Perform fast-forward
        // Looking up for branch by its reference name is required here to
        // detect pulling into empty repository
        //

        let ref_name = format!("refs/heads/{}", BRANCH_NAME);
        match self.repo.find_reference(&ref_name) {
            Ok(mut branch_ref) => {
                //
                // Actual fast-forward 
                //

                let reflog_msg = format!("Fast-forward: Setting {} to {}", 
                    ref_name, fetch_commit.id());

                branch_ref.set_target(fetch_commit.id(), &reflog_msg)?;
                self.repo.set_head(&ref_name)?;

                self.repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .force()
                ))?;
            },
            Err(_) => {
                //
                // Pulling into empty local repository
                //

                let reflog_msg = format!("Setting {} to {}", 
                    ref_name, fetch_commit.id());

                self.repo.reference(&ref_name, fetch_commit.id(), true, &reflog_msg)?;
                self.repo.set_head(&ref_name)?;

                self.repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force()
                ))?;
            }
        }

        Ok(())
    }

    fn push_remote(&self, branch_ref: &str) -> Result<()> {
        let config = self.repo.config()?;
        let mut push_options = git2::PushOptions::default();
        push_options.remote_callbacks(self.remote_callbacks(&config));

        self.repo.find_remote(REMOTE_NAME)
            .and_then(|mut remote| remote.push(&[branch_ref], Some(&mut push_options)))
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

        let mut config = self.repo.config()?;
        let config = config.snapshot()?;

        let name = config.get_str(CFG_NAME)?;
        let email = config.get_str(CFG_EMAIL)?;
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

    fn remote_callbacks<'a>(&'a self, config: &'a git2::Config) -> git2::RemoteCallbacks {
        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(
            self.authenticator
                .credentials(config)
        );

        callbacks
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

    fn prepare_for_overwrite<S: std::io::Seek>(s: &mut S) -> Result<()> {
        s.rewind()
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
