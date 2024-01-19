//! # libbdgt
//! 
//! `libbdgt` is a backend library for `bdgt` app.

extern crate dirs;
extern crate git2;
extern crate uuid;
extern crate rand;
extern crate gpgme;
extern crate scrypt;
extern crate chrono;
extern crate typenum;
extern crate aes_gcm;
extern crate rusqlite;

//
// Public modules
//

pub mod datetime;
pub mod location;
pub mod storage;
pub mod crypto;
pub mod error;
pub mod core;
pub mod sync;
