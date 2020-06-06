use std::{env, fs, io, path::Path};

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::quote;

macro_rules! err {
    ($format:literal $(, $($tt:tt)*)?) => {{
        |err| ::std::io::Error::new(::std::io::ErrorKind::Other,
            format!(concat!($format, ": {}"), $($($tt)*, )? err))
    }}
}

mod schema;

pub fn main() -> io::Result<()> {
    let path = Path::new(&env::var("OUT_DIR").expect("defined by cargo")).join("out.rs");
    let mut f = fs::File::create(path)?;
    write_token_stream(run()?, &mut f, 0, &mut true)?;
    Ok(())
}

fn run() -> io::Result<TokenStream> {
    let index = schema::parse()?;

    for path_item in index.paths().get().values() {
        for oper in path_item.get().values() {
            let oper = oper.get();
            for param in oper.parameters() {
                schema(param.schema());
            }
            if let Some(body) = oper.request_body() {
                for mt in body.content().values() {
                    schema(mt.schema());
                }
            }
            for resp in oper.responses().get().values() {
                for mt in resp.content().values() {
                    schema(mt.schema());
                }
            }
        }
    }
    fn schema(schema: &schema::Schema) {
        // println!("{}", serde_json::to_string(schema).unwrap());
    }

    Ok(quote! {
        pub fn hello_world() {}
    })
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
            TokenTree::Ident(ident) => write!(f, " {} ", ident)?,
            TokenTree::Punct(punct) => {
                match punct.spacing() {
                    Spacing::Alone => write!(f, " {} ", punct.as_char())?,
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
