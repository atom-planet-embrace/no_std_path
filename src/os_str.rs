//! Minimal `OsStr` / `OsString` replacements for `no_std` environments.
//!
//! On Unix, `OsStr` is just `[u8]` and `OsString` is just `Vec<u8>`.
//! We replicate that here with newtype wrappers so the path module can
//! work without pulling in `std::ffi`.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::ops;

/// Borrowed OS string slice (analogous to `std::ffi::OsStr`).
///
/// In this `no_std` crate the inner representation is simply `[u8]`.
#[repr(transparent)]
pub struct OsStr {
    inner: [u8],
}

/// Owned OS string (analogous to `std::ffi::OsString`).
///
/// In this `no_std` crate the inner representation is simply `Vec<u8>`.
#[derive(Clone, Default)]
pub struct OsString {
    inner: Vec<u8>,
}

// ---------------------------------------------------------------------------
// OsStr
// ---------------------------------------------------------------------------

impl OsStr {
    /// Wrap a `&str` as an `&OsStr`.
    #[inline]
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &OsStr {
        s.as_ref()
    }

    /// View the encoded bytes of this OS string slice.
    #[inline]
    pub fn as_encoded_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Create an `&OsStr` from a byte slice **without** any validation.
    ///
    /// # Safety
    /// The caller must guarantee that the byte slice is a valid encoding
    /// for the platform. In this crate every byte sequence is valid.
    #[inline]
    pub unsafe fn from_encoded_bytes_unchecked(bytes: &[u8]) -> &OsStr {
        // SAFETY: OsStr is #[repr(transparent)] over [u8].
        unsafe { &*(bytes as *const [u8] as *const OsStr) }
    }

    /// Create an `&mut OsStr` from a mutable byte slice without validation.
    ///
    /// # Safety
    /// Same as [`from_encoded_bytes_unchecked`].
    #[inline]
    pub unsafe fn from_encoded_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut OsStr {
        unsafe { &mut *(bytes as *mut [u8] as *mut OsStr) }
    }

    /// The length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the slice is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Convert to an owned `OsString`.
    #[inline]
    pub fn to_os_string(&self) -> OsString {
        OsString { inner: self.inner.to_vec() }
    }

    /// Try to convert to a UTF-8 `&str`.
    #[inline]
    pub fn to_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.inner).ok()
    }

    /// Lossy conversion to a UTF-8 string, replacing invalid sequences
    /// with U+FFFD.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.inner)
    }

    /// Return a `Display` helper that prints the string lossily.
    #[inline]
    pub fn display(&self) -> Display<'_> {
        Display { inner: self }
    }

    /// Make all ASCII characters lowercase, in place.
    #[inline]
    pub fn make_ascii_lowercase(&mut self) {
        self.inner.make_ascii_lowercase();
    }

    /// Make all ASCII characters uppercase, in place.
    #[inline]
    pub fn make_ascii_uppercase(&mut self) {
        self.inner.make_ascii_uppercase();
    }
}

// --- trait impls for OsStr -------------------------------------------------

impl PartialEq for OsStr {
    #[inline]
    fn eq(&self, other: &OsStr) -> bool {
        self.inner == other.inner
    }
}

impl Eq for OsStr {}

impl PartialOrd for OsStr {
    #[inline]
    fn partial_cmp(&self, other: &OsStr) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OsStr {
    #[inline]
    fn cmp(&self, other: &OsStr) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl Hash for OsStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl core::fmt::Debug for OsStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Escape non-UTF-8 the same way Debug for str would
        write!(f, "\"")?;
        for chunk in self.inner.utf8_chunks() {
            // Display the valid UTF-8 part
            let valid = chunk.valid();
            for c in valid.chars() {
                write!(f, "{}", c.escape_debug())?;
            }
            // Display the invalid part as hex escapes
            for &b in chunk.invalid() {
                write!(f, "\\x{b:02x}")?;
            }
        }
        write!(f, "\"")
    }
}

impl core::fmt::Display for OsStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let lossy = self.to_string_lossy();
        f.write_str(&lossy)
    }
}

impl Default for &OsStr {
    #[inline]
    fn default() -> Self {
        // SAFETY: Empty slice is valid.
        unsafe { OsStr::from_encoded_bytes_unchecked(&[]) }
    }
}

impl AsRef<OsStr> for str {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        // SAFETY: UTF-8 is always valid.
        unsafe { OsStr::from_encoded_bytes_unchecked(self.as_bytes()) }
    }
}

impl AsRef<OsStr> for String {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_str().as_ref()
    }
}

impl AsRef<OsStr> for OsStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl AsRef<OsStr> for OsString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl alloc::borrow::ToOwned for OsStr {
    type Owned = OsString;

    #[inline]
    fn to_owned(&self) -> OsString {
        self.to_os_string()
    }

    #[inline]
    fn clone_into(&self, target: &mut OsString) {
        target.inner.clear();
        target.inner.extend_from_slice(&self.inner);
    }
}

impl PartialEq<str> for OsStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.inner == *other.as_bytes()
    }
}

impl PartialEq<OsStr> for str {
    #[inline]
    fn eq(&self, other: &OsStr) -> bool {
        *self.as_bytes() == other.inner
    }
}

// ops::Index<ops::RangeFull> for OsStr -> &OsStr
impl ops::Index<ops::RangeFull> for OsStr {
    type Output = OsStr;
    #[inline]
    fn index(&self, _: ops::RangeFull) -> &OsStr {
        self
    }
}

// ---------------------------------------------------------------------------
// OsString
// ---------------------------------------------------------------------------

impl OsString {
    /// Create a new empty `OsString`.
    #[inline]
    pub const fn new() -> OsString {
        OsString { inner: Vec::new() }
    }

    /// Create an `OsString` with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> OsString {
        OsString { inner: Vec::with_capacity(capacity) }
    }

    /// Borrow as an `&OsStr`.
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        // SAFETY: OsStr is repr(transparent) over [u8].
        unsafe { OsStr::from_encoded_bytes_unchecked(&self.inner) }
    }

    /// Borrow as a `&mut OsStr`.
    #[inline]
    pub fn as_mut_os_str(&mut self) -> &mut OsStr {
        unsafe { OsStr::from_encoded_bytes_unchecked_mut(&mut self.inner) }
    }

    /// Append the given `OsStr` to this string.
    #[inline]
    pub fn push<S: AsRef<OsStr>>(&mut self, s: S) {
        self.inner.extend_from_slice(s.as_ref().as_encoded_bytes());
    }

    /// Try to convert into a `String`.
    #[inline]
    pub fn into_string(self) -> Result<String, OsString> {
        String::from_utf8(self.inner).map_err(|e| OsString { inner: e.into_bytes() })
    }

    /// Truncate to `len` bytes.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len);
    }

    /// Reserve capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Reserve capacity for exactly `additional` more bytes.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    /// Try to reserve capacity for at least `additional` more bytes.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// Try to reserve capacity for exactly `additional` more bytes.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    /// Shrink capacity to fit the current length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Shrink capacity to at least `min_capacity`.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    /// Return the current capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Clear the string.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// The length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// View the encoded bytes.
    #[inline]
    pub fn as_encoded_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Consume and leak, returning a `&'a mut OsStr`.
    #[inline]
    pub fn leak<'a>(self) -> &'a mut OsStr {
        let leaked = self.inner.leak();
        // SAFETY: OsStr is repr(transparent) over [u8].
        unsafe { OsStr::from_encoded_bytes_unchecked_mut(leaked) }
    }

    /// Consume into a boxed `OsStr`.
    #[inline]
    pub fn into_boxed_os_str(self) -> Box<OsStr> {
        let boxed: Box<[u8]> = self.inner.into_boxed_slice();
        let raw = Box::into_raw(boxed) as *mut OsStr;
        // SAFETY: OsStr is repr(transparent) over [u8].
        unsafe { Box::from_raw(raw) }
    }

    /// Extend from a raw byte slice without validation.
    ///
    /// # Safety
    /// The caller must ensure the resulting OsString remains valid.
    /// In this crate every byte sequence is valid, so this is always safe
    /// from a validity standpoint, but we keep the signature for API compat.
    #[inline]
    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[u8]) {
        self.inner.extend_from_slice(slice);
    }
}

// --- trait impls for OsString ----------------------------------------------

impl ops::Deref for OsString {
    type Target = OsStr;

    #[inline]
    fn deref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl ops::DerefMut for OsString {
    #[inline]
    fn deref_mut(&mut self) -> &mut OsStr {
        self.as_mut_os_str()
    }
}

impl Borrow<OsStr> for OsString {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl PartialEq for OsString {
    #[inline]
    fn eq(&self, other: &OsString) -> bool {
        self.as_os_str() == other.as_os_str()
    }
}

impl Eq for OsString {}

impl PartialOrd for OsString {
    #[inline]
    fn partial_cmp(&self, other: &OsString) -> Option<core::cmp::Ordering> {
        self.as_os_str().partial_cmp(other.as_os_str())
    }
}

impl Ord for OsString {
    #[inline]
    fn cmp(&self, other: &OsString) -> core::cmp::Ordering {
        self.as_os_str().cmp(other.as_os_str())
    }
}

impl Hash for OsString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_os_str().hash(state);
    }
}

impl core::fmt::Debug for OsString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_os_str(), f)
    }
}

impl core::fmt::Display for OsString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_os_str(), f)
    }
}

impl From<String> for OsString {
    #[inline]
    fn from(s: String) -> OsString {
        OsString { inner: s.into_bytes() }
    }
}

impl From<&str> for OsString {
    #[inline]
    fn from(s: &str) -> OsString {
        OsString { inner: s.as_bytes().to_vec() }
    }
}

impl From<Box<OsStr>> for OsString {
    #[inline]
    fn from(boxed: Box<OsStr>) -> OsString {
        let raw = Box::into_raw(boxed) as *mut [u8];
        let inner = unsafe { Box::from_raw(raw) };
        OsString { inner: inner.into_vec() }
    }
}

impl From<OsString> for alloc::sync::Arc<OsStr> {
    fn from(s: OsString) -> alloc::sync::Arc<OsStr> {
        let arc: alloc::sync::Arc<[u8]> = alloc::sync::Arc::from(s.inner);
        // SAFETY: OsStr is repr(transparent) over [u8].
        unsafe { alloc::sync::Arc::from_raw(alloc::sync::Arc::into_raw(arc) as *const OsStr) }
    }
}

impl From<&OsStr> for alloc::sync::Arc<OsStr> {
    fn from(s: &OsStr) -> alloc::sync::Arc<OsStr> {
        let arc: alloc::sync::Arc<[u8]> = alloc::sync::Arc::from(&s.inner);
        unsafe { alloc::sync::Arc::from_raw(alloc::sync::Arc::into_raw(arc) as *const OsStr) }
    }
}

impl From<OsString> for alloc::rc::Rc<OsStr> {
    fn from(s: OsString) -> alloc::rc::Rc<OsStr> {
        let rc: alloc::rc::Rc<[u8]> = alloc::rc::Rc::from(s.inner);
        unsafe { alloc::rc::Rc::from_raw(alloc::rc::Rc::into_raw(rc) as *const OsStr) }
    }
}

impl From<&OsStr> for alloc::rc::Rc<OsStr> {
    fn from(s: &OsStr) -> alloc::rc::Rc<OsStr> {
        let rc: alloc::rc::Rc<[u8]> = alloc::rc::Rc::from(&s.inner);
        unsafe { alloc::rc::Rc::from_raw(alloc::rc::Rc::into_raw(rc) as *const OsStr) }
    }
}

impl AsRef<crate::Path> for OsStr {
    #[inline]
    fn as_ref(&self) -> &crate::Path {
        crate::Path::new(self)
    }
}

impl AsRef<crate::Path> for OsString {
    #[inline]
    fn as_ref(&self) -> &crate::Path {
        crate::Path::new(self)
    }
}

impl From<crate::PathBuf> for OsString {
    #[inline]
    fn from(path_buf: crate::PathBuf) -> OsString {
        path_buf.into_os_string()
    }
}

impl From<OsString> for crate::PathBuf {
    #[inline]
    fn from(s: OsString) -> crate::PathBuf {
        crate::PathBuf::from_os_string(s)
    }
}

// ---------------------------------------------------------------------------
// Display helper (used by Path::display)
// ---------------------------------------------------------------------------

/// A helper struct for lossy display of an `OsStr`.
pub struct Display<'a> {
    inner: &'a OsStr,
}

impl<'a> core::fmt::Debug for Display<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.inner, f)
    }
}

impl<'a> core::fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let lossy = self.inner.to_string_lossy();
        f.pad(&lossy)
    }
}

