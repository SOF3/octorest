#![cfg_attr(feature = "internal-docsrs", feature(doc_cfg))]

mod client;
pub use client::Client;

/// An error at the transport layer
#[derive(Debug, snafu::Snafu)]
pub enum TransportError {
    #[snafu(display("A network layer error occurred: {}", err))]
    /// A network layer error occurred
    Reqwest { err: reqwest::Error },
    #[snafu(display("Parameters could not be encoded: {}", err))]
    /// The parameters were invalid
    Encode { err: serde_json::Error },
    #[snafu(display("GitHub returned invalid data: {}", err))]
    /// GitHub returned invalid data
    Decode { err: serde_json::Error },
    #[snafu(display("GitHub returned unexpected status code: {}", status))]
    /// GitHub returned unexpected status code
    UnexpectedStatus { status: u16 },
}

include!(concat!(env!("OUT_DIR"), "/out.rs"));
