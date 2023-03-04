#![allow(unsafe_code)]

pub use crate::archive::{Archive, Entries};
pub use crate::entry::{Entry, Unpacked};
pub use crate::entry_type::EntryType;
pub use crate::header::GnuExtSparseHeader;
pub use crate::header::{GnuHeader, GnuSparseHeader, Header, HeaderMode, OldHeader, UstarHeader};

mod archive;
mod entry;
mod entry_type;
mod error;
mod ffi;
mod header;
