#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod byte_order;
mod dbc;
mod error;
mod message;
mod nodes;
mod parser;
mod receivers;
mod signal;
mod version;

pub use byte_order::ByteOrder;

pub use dbc::Dbc;
#[cfg(feature = "std")]
pub use dbc::DbcBuilder;

pub use error::{Error, Result};

pub use message::Message;
#[cfg(feature = "std")]
pub use message::MessageBuilder;

pub use nodes::Nodes;
#[cfg(feature = "std")]
pub use nodes::NodesBuilder;

pub(crate) use parser::Parser;

pub use receivers::Receivers;

pub use signal::Signal;
#[cfg(feature = "std")]
pub use signal::SignalBuilder;

pub use version::Version;
#[cfg(feature = "std")]
pub use version::VersionBuilder;

#[cfg(feature = "std")]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "std"), allow(dead_code))]
const DBC_KEYWORDS: &[&str] = &[
    "VECTOR__INDEPENDENT_SIG_MSG",
    "VECTOR__XXX",
    "BA_DEF_DEF_REL_",
    "BA_DEF_SGTYPE_",
    "SIGTYPE_VALTYPE_",
    "ENVVAR_DATA_",
    "SIG_TYPE_REF_",
    "NS_DESC_",
    "BA_DEF_REL_",
    "BA_SGTYPE_",
    "SGTYPE_VAL_",
    "VAL_TABLE_",
    "SIG_GROUP_",
    "SIG_VALTYPE_",
    "BO_TX_BU_",
    "BU_SG_REL_",
    "BU_EV_REL_",
    "BU_BO_REL_",
    "SG_MUL_VAL_",
    "BA_DEF_DEF_",
    "BA_DEF_",
    "BA_REL_",
    "CAT_DEF_",
    "EV_DATA_",
    "BA_",
    "VAL_",
    "CM_",
    "CAT_",
    "NS_",
    "BS_",
    "BU_",
    "BO_",
    "SG_",
    "EV_",
    "VERSION",
    "FILTER",
];
