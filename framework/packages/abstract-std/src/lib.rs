// #![warn(missing_docs)]

// https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html

//! # Abstract Account
//!
//! Abstract Interface is the interface-defining crate to the Abstract smart-contract framework.
//!
//! ## Description
//! This crate provides the key utilities that are required to integrate with or write Abstract contracts.
//!
//! ## Messages
//! All interfacing message structs are defined here so they can be imported.
//! ```no_run
//! use abstract_std::account::ExecuteMsg;
//! ```
//! ### Assets
//! [`cw-asset`](https://crates.io/crates/cw-asset) is used for asset-management.
//! If a message requests a String value for an Asset field then you need to provide the human-readable ans_host key.
//! The full list of supported assets and contracts is given [here](https://github.com/AbstractSDK/scripts/tree/main/resources/ans_host).
//! The contract will handel address retrieval internally.
//!
//! ## State
//! The internal state for each contract is also contained within this crate. This ensures that breaking changes to the internal state are easily spotted.
//! It also allows for tight and low-gas integration between contracts by performing raw queries on these states.
//! A contract's state object can be imported and used like:
//! ```ignore
//! use crate::account::state::ACCOUNT_ID
//! let account_id = ACCOUNT_ID.query(querier, account_address).unwrap();
//! ```
//! The internally stored objects are also contained within this package in [`crate::objects`].
//!
//! ## Names
//! Abstract contract names are used internally and for version management.
//! They are exported for ease of use:
//! ```no_run
//! use abstract_std::ACCOUNT;
//! ```
//! ## Errors
//! An `AbstractError` wraps error throws by `StdError` or `AssetError`. It is also use in the objects to throw errors.
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

/// Result type for Abstract objects
pub type AbstractResult<T> = Result<T, error::AbstractError>;

pub mod base;

pub mod adapter;
pub mod app;
pub mod objects;
pub mod standalone;

mod error;
pub use error::AbstractError;

pub mod account;

mod native;
pub use crate::native::*;

pub mod constants;
pub use constants::*;

pub mod native_addrs;
