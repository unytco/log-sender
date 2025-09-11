#![deny(missing_docs)]
//! hc-reporter

use base64::prelude::*;

use std::io::Result;

pub mod client;

pub mod config;

pub mod crypto;
use crypto::*;

#[cfg(test)]
mod test;
