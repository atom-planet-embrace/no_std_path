//! Cross-platform path manipulation.
//!
//! This module provides two types, [`PathBuf`] and [`Path`] (akin to [`String`]
//! and [`str`]), for working with paths abstractly. These types are thin wrappers
//! around [`OsString`] and [`OsStr`] respectively, meaning that they work directly
//! on strings according to the local platform's path syntax.
//!
//! Paths can be parsed into [`Component`]s by iterating over the structure
//! returned by the [`components`] method on [`Path`]. [`Component`]s roughly
//! correspond to the substrings between path separators (`/` or `\`). You can
//! reconstruct an equivalent path from components with the [`push`] method on
//! [`PathBuf`]; note that the paths may differ syntactically by the
//! normalization described in the documentation for the [`components`] method.
//!
//! ## Feature `std`
//!
//! When the `std` feature is enabled, all types are re-exported directly from
//! [`std::path`] and [`std::ffi`], giving full platform support and filesystem
//! methods. When `std` is disabled (the default), lightweight `no_std`
//! implementations are provided instead.
//!
//! [`components`]: Path::components
//! [`push`]: PathBuf::push

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

// ── std feature: re-export from std ──────────────────────────────────────────

#[cfg(feature = "std")]
pub use std::ffi::{OsStr, OsString};

#[cfg(feature = "std")]
pub mod ffi {
    //! Re-exports of [`std::ffi::OsStr`] and [`std::ffi::OsString`].
    pub use std::ffi::{OsStr, OsString};
}

#[cfg(feature = "std")]
pub use std::path::{
    Ancestors, Component, Components, Display, Iter, Path, PathBuf, Prefix, PrefixComponent,
    StripPrefixError, MAIN_SEPARATOR, MAIN_SEPARATOR_STR, is_separator,
};

// ── no_std: use our own implementations ──────────────────────────────────────

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub mod ffi;

#[cfg(not(feature = "std"))]
mod sys_path;

#[cfg(not(feature = "std"))]
mod path;

#[cfg(not(feature = "std"))]
pub use crate::ffi::{OsStr, OsString};

#[cfg(not(feature = "std"))]
pub use crate::path::{
    Ancestors, Component, Components, Display, Iter, Path, PathBuf, Prefix,
    PrefixComponent, StripPrefixError, MAIN_SEPARATOR, MAIN_SEPARATOR_STR, is_separator,
};

// ── Implementation sentinel ──────────────────────────────────────────────────

/// `true` when using the custom `no_std` implementation,
/// `false` when re-exporting from `std::path`.
#[cfg(not(feature = "std"))]
pub const NO_STD_IMPL: bool = true;

/// `true` when using the custom `no_std` implementation,
/// `false` when re-exporting from `std::path`.
#[cfg(feature = "std")]
pub const NO_STD_IMPL: bool = false;
