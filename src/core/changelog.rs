use crate::error::Result;
use crate::storage::{Transaction, Account, Category, Plan};


/// Simple changelog representation for some items.
pub(crate) struct SimpleChangelog<T> {
    /// Added items.
    pub added: Vec<T>,

    /// Changed items.
    pub changed: Vec<T>,

    /// Removed items.
    pub removed: Vec<T>,
}


impl<T> SimpleChangelog<T> {
    fn new() -> Self {
        SimpleChangelog::<T> {
            added: Vec::new(),
            changed: Vec::new(),
            removed: Vec::new()
        }
    }
}


/// Database changelog representation.
pub(crate) struct Changelog {
    /// Accounts changelog.
    pub accounts: SimpleChangelog<Account>,

    /// Categories changelog.
    pub categories: SimpleChangelog<Category>,

    /// Transactions changelog.
    pub transactions: SimpleChangelog<Transaction>,

    /// Plans changelog.
    pub plans: SimpleChangelog<Plan>,
}


impl Changelog {
    /// Creates a new changelog object from binary representation.
    /// 
    /// * `binary_changelog` - binary changelog representation
    pub(crate) fn new(binary_changelog: &[u8]) -> Result<Self> {
        let result = Changelog {
            accounts: SimpleChangelog::new(),
            categories: SimpleChangelog::new(),
            transactions: SimpleChangelog::new(),
            plans: SimpleChangelog::new()
        };

        let mut changelog = std::io::Cursor::new(binary_changelog);
        while let Some(_record) = Self::read_record(&mut changelog)? {
            todo!("Implement record handling")
        }

        Ok(result)
    }

    /// Appends another changelog to the current one.
    /// 
    /// * `changelog` - a changelog to append
    pub(crate) fn append(&mut self, _changelog: Changelog) -> Result<()> {
        todo!("append new changelog")
    }

    /// Converts current changelog into a binary representation.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        todo!("convert to bytes")
    }
}


impl Changelog {
    fn read_record<Br: std::io::BufRead>(binary_changelog: &mut Br) -> Result<Option<()>> {
        let mut buffer = Vec::new();
        binary_changelog.read_until(0x00, &mut buffer)?;

        Ok(None)
    }
}