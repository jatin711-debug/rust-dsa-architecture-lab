//! Streaming text search: CLI arguments, BufRead, Results, and borrowed lines.

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn search<R: BufRead, W: Write>(reader: R, needle: &str, output: &mut W) -> io::Result<usize> {
    let mut matches = 0;
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.contains(needle) {
            writeln!(output, "{}:{}", index + 1, line)?;
            matches += 1;
        }
    }
    Ok(matches)
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let needle = args.next().ok_or("usage: mini_grep <pattern> <file>")?;
    let path = args.next().ok_or("usage: mini_grep <pattern> <file>")?;
    if needle.is_empty() || args.next().is_some() {
        return Err("usage: mini_grep <nonempty-pattern> <file>".into());
    }
    let file = File::open(path)?;
    search(BufReader::new(file), &needle, &mut io::stdout())?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_line_by_line() {
        let input = "rust\nhello\nrusty rust\n";
        let mut output = Vec::new();
        assert_eq!(search(input.as_bytes(), "rust", &mut output).unwrap(), 2);
        assert_eq!(String::from_utf8(output).unwrap(), "1:rust\n3:rusty rust\n");
    }
}
