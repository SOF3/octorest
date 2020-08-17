#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]

use std::{env, error::Error, fmt, fs, io, path::Path, path::PathBuf};

use cfg_if::cfg_if;
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

macro_rules! err {
    ($format:literal $(, $($tt:tt)*)?) => {{
        |err| ::std::io::Error::new(::std::io::ErrorKind::Other,
            format!(concat!($format, ": {}"), $($($tt)*, )? err))
    }}
}

mod gen;
mod idents;
mod schema;

pub fn main() -> Result<(), Box<dyn Error>> {
    cfg_if! {
        if #[cfg(feature = "dev")] {
            pretty_env_logger::init();
        }
    }

    #[derive(serde::Deserialize)]
    struct ReleaseData {
        tag_name: String,
        assets: Vec<Asset>,
    }
    #[derive(serde::Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
    }

    let out_dir = PathBuf::from(&env::var("OUT_DIR").expect("defined by cargo"));

    let json_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("api.github.com.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    if !json_path.exists() || cfg!(feature = "latest") {
        cfg_if! {
            if #[cfg(feature = "online")] {
                let url = "https://github.com/github/rest-api-description/raw/main/descriptions/api.github.com/api.github.com.json";
                task::<Result<_, Box<dyn Error>>, _, _>("Downloading api.github.com.json online", || {
                    Ok(reqwest::blocking::get(url)?
                        .copy_to(&mut fs::File::create(&json_path)?)?)
                })?;
            } else {
                // impossible path if feature="latest"
                return Err(Box::new(io::Error::new(io::ErrorKind::Other, "`online` feature not enabled, but no packaged api.github.com.json is available")));
            }
        }
    };

    let out_path = out_dir.join("out.rs");
    let mut out = fs::File::create(&out_path)?;

    let json_data = fs::read_to_string(&json_path)?;

    let index: schema::Index = task("Parsing api.github.com.json", || schema::parse(&json_data))?;
    let ts_out = task("Generating code", move || gen::gen(&index));

    task(&format_args!("Writing to {}", out_path.display()), || {
        write_token_stream(ts_out, &mut out, 0, &mut true)
    })?;
    Ok(())
}

fn write_token_stream(
    ts: TokenStream,
    f: &mut fs::File,
    indent: usize,
    start_of_line: &mut bool,
) -> io::Result<()> {
    use io::Write;

    for token in ts {
        if *start_of_line {
            write!(f, "{}", "    ".repeat(indent))?;
            *start_of_line = false;
        }
        match token {
            TokenTree::Literal(literal) => write!(f, "{}", literal)?,
            TokenTree::Ident(ident) => write!(f, "{} ", ident)?,
            TokenTree::Punct(punct) => {
                match punct.spacing() {
                    Spacing::Alone => write!(f, "{} ", punct.as_char())?,
                    Spacing::Joint => write!(f, "{}", punct.as_char())?,
                }
                if punct.as_char() == ';' {
                    writeln!(f)?;
                    *start_of_line = true;
                }
            }
            TokenTree::Group(group) => match group.delimiter() {
                Delimiter::Parenthesis => {
                    write!(f, "(")?;
                    write_token_stream(group.stream(), f, indent + 2, start_of_line)?;
                    write!(f, ")")?;
                }
                Delimiter::Bracket => {
                    write!(f, "[")?;
                    write_token_stream(group.stream(), f, indent + 2, start_of_line)?;
                    write!(f, "]")?;
                }
                Delimiter::Brace => {
                    writeln!(f, "{{")?;
                    *start_of_line = true;
                    write_token_stream(group.stream(), f, indent + 1, start_of_line)?;
                    writeln!(f)?;
                    write!(
                        f,
                        "{}}}",
                        if *start_of_line {
                            "    ".repeat(indent)
                        } else {
                            String::new()
                        }
                    )?;
                    if *start_of_line {
                        writeln!(f)?;
                    }
                }
                Delimiter::None => write_token_stream(group.stream(), f, indent, start_of_line)?,
            },
        }
    }
    Ok(())
}

fn task<R, S, F>(name: &S, f: F) -> R
where
    S: fmt::Display + ?Sized,
    F: FnOnce() -> R,
{
    use std::time::Instant;

    log::info!("[Phase start] {}", name);
    let start = Instant::now();
    let r = f();
    let end = Instant::now();
    log::info!("[Phase end] {} (spent {:?})", name, end - start);
    r
}

fn id<T>(t: T) -> T {
    t
}
