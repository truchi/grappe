#![allow(unused)]

#[macro_use]
pub mod utils;

pub mod cluster;
pub mod eol;
pub mod line;
pub mod page;
pub mod text;

pub use cluster::Cluster;
pub use eol::Eol;
pub use line::*;
pub use page::*;
pub use text::Text;

const SPACES: &'static str = unsafe { std::str::from_utf8_unchecked(&[b' '; u8::MAX as usize]) };

#[derive(Copy, Clone, Default, Debug)]
pub struct Offset {
    len:   usize,
    chars: usize,
    lines: usize,
}

/// Test function.
pub fn main() {
    let text = "Hello, world\nHow you doing?12345678901234567890\n";
    let mut builder = text::PageBuilder::default();

    let mut str = text;

    while !str.is_empty() {
        let s = builder.push_str(str).unwrap_or("");

        str = s;
        dbg!(str);
    }
}

fn test1() {
    let texts : &[&[&str]]= &[
        &["Hello, world\nHow you doing?", "12345678901234567890\n"],
        &[
            "  Hello, world\n   How you doing?",
            "  34    5678901234567890\n",
        ],
        &[
            "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n",
        ],
        &["11\r2\n"],
        &["     "],
        &["\n", "2\n"],
    ];

    for chunks in texts {
        println!("==============================================");
        println!("==============================================");
        dbg!(chunks);
        println!();
        println!();

        let mut builder = text::PageBuilder::default();
        let mut string = String::new();

        for chunk in *chunks {
            // dbg!(chunk);
            for page in builder.push(chunk) {
                // dbg!(page);
                let p = page.to_string();
                string.push_str(&p);
                dbg!(p);
                // dbg!(page.chunks().collect::<Vec<_>>());
            }
        }

        let page = builder.done();
        let p = page.to_string();
        string.push_str(&p);
        dbg!(p);
        dbg!(page.chunks().collect::<Vec<_>>());

        let expected = chunks.concat();
        println!("expected {:?}", &expected);
        println!("string   {:?}", &string);
        assert!(expected == string);
    }
}
