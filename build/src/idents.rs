use proc_macro2::{Ident, Span};

pub fn snake(name: &str) -> Ident {
    let name = name
        .replace("+", " plus ")
        .replace("--", " minus ")
        .replace("'", "");
    let mut conv = heck::SnakeCase::to_snake_case(name.as_str());
    fix_kw(&mut conv);
    if (b'0'..=b'9').contains(&conv.as_bytes()[0]) {
        conv = format!(
            "{}{}",
            match conv.as_bytes()[0] {
                b'0' => "zero_",
                b'1' => "one_",
                b'2' => "two_",
                b'3' => "three_",
                b'4' => "four_",
                b'5' => "five_",
                b'6' => "six_",
                b'7' => "seven_",
                b'8' => "eight_",
                b'9' => "nine_",
                _ => unreachable!(),
            },
            &conv[1..]
        );
    }
    Ident::new(&conv, Span::call_site())
}

pub fn pascal(name: &str) -> Ident {
    let name = name
        .replace("+", " plus ")
        .replace("--", " minus ")
        .replace("'", "");
    let mut conv = heck::CamelCase::to_camel_case(name.as_str());
    fix_kw(&mut conv);
    if (b'0'..=b'9').contains(&conv.as_bytes()[0]) {
        conv = format!(
            "{}{}",
            match conv.as_bytes()[0] {
                b'0' => "Zero",
                b'1' => "One",
                b'2' => "Two",
                b'3' => "Three",
                b'4' => "Four",
                b'5' => "Five",
                b'6' => "Six",
                b'7' => "Seven",
                b'8' => "Eight",
                b'9' => "Nine",
                _ => unreachable!(),
            },
            &conv[1..]
        );
    }
    Ident::new(&conv, Span::call_site())
}

fn fix_kw(conv: &mut String) {
    if syn::parse_str::<Ident>(conv).is_err() {
        conv.push('_');
    }
}
