#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]

use std::{env, fmt, fs, io, path::Path, path::PathBuf};

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

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dev")]
    {
        pretty_env_logger::init();
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

    let pkg_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("api.github.com.json");
    let json_path = if pkg_path.exists() {
        pkg_path
    } else {
        todo!("Download latest version of https://github.com/github/rest-api-description/blob/main/descriptions/api.github.com/api.github.com.json")
        /*cfg_if! {
            if #[cfg(feature = "online")] {
                let client = reqwest::blocking::Client::new();
                let data = client.get("https://api.github.com/repos/octokit/routes/releases/latest")
                    .send().map_err(err!("Error fetching latest routes"))?
                    .error_for_status().map_err(err!("Error fetching latest routes"))?
                    .json::<ReleaseData>().map_err(err!("Error parsing routes release data"))?;

                let json_path = out_dir.join(&format!("{}.json", &data.tag_name));
                if !json_path.exists() {
                    log::info!("Downloading new routes version {}", &data.tag_name);
                    let url = match data.assets.iter().filter(|asset| &asset.name == "api.github.com.json")
                        .map(|asset| &asset.browser_download_url)
                        .next() {
                            Some(url) => url,
                            None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Latest octokit/routes release does not contain api.github.com.json"))),
                        };
                    let mut file = fs::File::create(url).map_err(Box::new)?;
                    let _ = client.get(url) // we don't care about number of bytes written
                        .send().map_err(Box::new)?
                        .error_for_status().map_err(Box::new)?
                        .copy_to(&mut file).map_err(Box::new)?;
                }
                json_path
            } else {
                return Err(Box::new(io::Error::new(io::ErrorKind::Other, "`online` feature not enabled, but no packaged api.github.com.json is available")));
            }
        }*/
    };

    let out_path = out_dir.join("out.rs");
    let mut out = fs::File::create(&out_path)?;

    let json_data = fs::read_to_string(&json_path)?;

    let index = task("Parsing api.github.com.json", || schema::parse(&json_data))?;
    let ts_out = task("Generating code", || gen::gen(&index));

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
    use std::io::Write;

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

fn task<R>(name: &(impl fmt::Display + ?Sized), f: impl FnOnce() -> R) -> R {
    use std::time::Instant;

    println!("[Phase start] {}", name);
    let start = Instant::now();
    let r = f();
    let end = Instant::now();
    println!("[Phase end] {} (spent {:?})", name, end - start);
    r
}

fn id<T>(t: T) -> T {
    t
}
