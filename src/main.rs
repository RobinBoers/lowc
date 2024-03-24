use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

use regex::Regex;

struct LowC {
    tags: Vec<(Regex, String)>,
}

impl LowC {
    fn new() -> Self {
        Self {
            tags: Vec::new(),
        }
    }

    fn add_simple_tag(&mut self, args: Vec<&str>, replacement: &str) {
        let pattern = self.create_tag_pattern(args, true);
        self.add_custom_tag(&pattern, replacement)
    }

    fn add_empty_tag(&mut self, args: Vec<&str>, replacement: &str) {
        let pattern = self.create_tag_pattern(args, true);
        self.add_custom_tag(&pattern, replacement)
    }

    fn add_custom_tag(&mut self, pattern: &str, replacement: &str) {
        let regex = Regex::new(pattern).expect("Invalid regex pattern");
        self.tags.push((regex, String::from(replacement)))
    }

    fn create_tag_pattern(&self, args: Vec<&str>, empty: bool) -> String {
        let (name, attrs) = args.split_first().expect("No tag name provided!");
        let mut pattern = format!(r#"<{name}"#);

        for attr in attrs {
            pattern.push_str(&format!(r#"\s+{}="([^"]*?)""#, attr));
        }

        if empty {
            pattern.push_str(&format!(r#"\s*/?>"#));
        } else {
            pattern.push_str(&format!(r#"\s*>([^<]?)</{name}\s*>"#));
        }
        
        String::from(pattern)
    }

    fn transform(&self, input: &str) -> String {
        let mut output = String::from(input);

        for (ref regex, replacement) in &self.tags {
            output = regex.replace_all(&output, replacement).to_string();
        }

        output
    }
}

fn main() {
    let input: String;
    let mut lowc = LowC::new();
    
    lowc.add_simple_tag(vec!["d"], "<strong><dfn>$1</dfn></strong>");
    lowc.add_simple_tag(vec!["t"], "<strong><cite>$1</cite></strong>");
    lowc.add_simple_tag(vec!["link", "href"], "<a href=\"$1\">$2</a>");
    
    lowc.add_empty_tag(vec!["tube", "watch"], "<iframe src=\"https://yewtu.be/embed/$1\"></iframe>");
    lowc.add_empty_tag(vec!["picture", "src"], "<figure><img src=\"$1\"></figure>");
    lowc.add_empty_tag(vec!["picture", "src", "caption"], "<figure><img src=\"$1\" alt=\"$2\"><figcaption>$2</figcaption></figure>");

    let argv: Vec<String> = std::env::args().collect();

    if argv.len() > 1 {
        let path = Path::new(&argv[0]);
        let source = &argv[1];

        match source.as_str() {
            "-h" | "--help" => {
                let binary = Path::new(path).file_name().expect("What the actual fuck is going on??").to_string_lossy();
                println!("Usage: {binary} [path]\n");
                process::exit(0);
            }
            "-" => {
                input = from_stdin().expect("Error reading from stdin"); 
            }
            _ => {
                input = from_file(source).expect("Error reading from file");
            }
        }
    } else {
        input = from_stdin().expect("Error reading from stdin");
    }

    let valid_html = lowc.transform(&input);
    println!("{}", valid_html);
}

fn from_file(path: &str) -> Result<String, io::Error> {
    let canonicalized_path = PathBuf::from(path).canonicalize()?;
    fs::read_to_string(canonicalized_path)
}

fn from_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
