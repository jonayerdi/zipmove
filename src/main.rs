use std::{
    env,
    fs::File,
    io::{Read, Seek, Write},
    mem,
};

use regex::{Captures, Regex};
use zip::{ZipArchive, ZipWriter};

enum StringBuilderItem {
    Literal(String),
    Capture(usize),
}

struct StringBuilder {
    items: Vec<StringBuilderItem>,
}

impl StringBuilder {
    const ESCAPE_CHAR: char = '=';
    fn new(pattern: &str) -> Self {
        let mut items = Vec::new();
        let mut literal = String::new();
        let mut escape = false;
        for c in pattern.chars().into_iter() {
            if escape {
                if c == StringBuilder::ESCAPE_CHAR {
                    literal.push(StringBuilder::ESCAPE_CHAR);
                } else {
                    let capture_index = c.to_digit(10).unwrap() as usize;
                    let mut completed_literal = String::new();
                    mem::swap(&mut literal, &mut completed_literal);
                    items.push(StringBuilderItem::Literal(completed_literal));
                    items.push(StringBuilderItem::Capture(capture_index));
                }
                escape = false;
            } else {
                if c == StringBuilder::ESCAPE_CHAR {
                    escape = true;
                } else {
                    literal.push(c);
                    escape = false;
                }
            }
        }
        if !literal.is_empty() {
            items.push(StringBuilderItem::Literal(literal));
        }
        Self { items }
    }
    fn build(&self, captures: &Captures) -> String {
        self.items
            .iter()
            .map(|i| match i {
                StringBuilderItem::Literal(s) => s.clone(),
                StringBuilderItem::Capture(n) => captures.get(*n).unwrap().as_str().to_string(),
            })
            .collect()
    }
}

fn rename_zipfile(
    src_filename: &Regex,
    dst_filename: &StringBuilder,
    zipfilename: &str,
) -> Option<String> {
    src_filename
        .captures(zipfilename)
        .map(|captures| dst_filename.build(&captures))
}

fn zipmove<R: Read + Seek, W: Write + Seek>(
    mut src: ZipArchive<R>,
    mut dst: ZipWriter<W>,
    src_filenames: &Regex,
    dst_filenames: &StringBuilder,
) {
    for i in 0..src.len() {
        let zipfile = src.by_index_raw(i).unwrap();
        let zipfilename = zipfile.enclosed_name().unwrap().to_str().unwrap();
        if let Some(new_name) = rename_zipfile(src_filenames, dst_filenames, zipfilename) {
            dst.raw_copy_file_rename(zipfile, new_name).unwrap();
        }
    }
    dst.finish().unwrap();
}

fn zipview<R: Read + Seek>(mut src: ZipArchive<R>) {
    for i in 0..src.len() {
        let zipfile = src.by_index_raw(i).unwrap();
        let zipfilename = zipfile.enclosed_name().unwrap().to_str().unwrap();
        println!("{}", zipfilename);
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    match args.len() {
        2 => {
            let mut args_iter = args.into_iter().skip(1);
            let src = args_iter.next().unwrap();
            let src = File::open(&src).unwrap();
            let src = ZipArchive::new(src).unwrap();
            zipview(src);
        },
        5 => {
            let mut args_iter = args.into_iter().skip(1);
            let src = args_iter.next().unwrap();
            let src = File::open(&src).unwrap();
            let src = ZipArchive::new(src).unwrap();
            let dst = args_iter.next().unwrap();
            let dst = File::create(&dst).unwrap();
            let dst = ZipWriter::new(dst);
            let src_filenames = args_iter.next().unwrap();
            let src_filenames = Regex::new(&src_filenames).unwrap();
            let dst_filenames = args_iter.next().unwrap();
            let dst_filenames = StringBuilder::new(&dst_filenames);
            zipmove(src, dst, &src_filenames, &dst_filenames);
        },
        _ => {
            println!("Usage:");
            println!("./zipmove <ZIP>");
            println!("./zipmove <SRC_ZIP> <DST_ZIP> <REGEX_SRC_FILENAMES> <DST_FILENAMES>");
            println!("<DST_FILENAMES> may contain backreferences to <REGEX_SRC_FILENAMES> with {0}1 through {0}9", StringBuilder::ESCAPE_CHAR);
        },
    }    
}
