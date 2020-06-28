#![cfg_attr(feature = "internal-docsrs", feature(doc_cfg))]

mod client;
pub use client::Client;

pub mod apis {
    include!(concat!(env!("OUT_DIR"), "/out.rs"));
}

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}
