use regex::Regex;
use std::path::{Path, PathBuf};
use std::{fs, io};

struct LowC {
    tags: Vec<(Regex, String)>,
}

impl LowC {
    fn new() -> Self {
        Self { tags: Vec::new() }
    }

    fn add_simple_tag(&mut self, args: &[&str], replacement: &str) {
        let pattern = self.create_tag_pattern(args, false);
        self.add_custom_tag(&pattern, replacement)
    }

    fn add_empty_tag(&mut self, args: &[&str], replacement: &str) {
        let pattern = self.create_tag_pattern(args, true);
        self.add_custom_tag(&pattern, replacement)
    }

    fn add_custom_tag(&mut self, pattern: &str, replacement: &str) {
        let regex = Regex::new(pattern).expect("Invalid regex pattern");
        self.tags.push((regex, String::from(replacement)))
    }

    fn create_tag_pattern(&self, args: &[&str], empty: bool) -> String {
        let (name, attrs) = args.split_first().expect("No tag name provided!");
        let mut pattern = format!(r#"<{name}"#);

        for attr in attrs {
            pattern.push_str(&format!(r#"\s+{}="([^"]*?)""#, attr));
        }

        let pat = if empty {
            format!(r#"\s*/?>"#)
        } else {
            format!(r#"\s*>([^<]?)</{name}\s*>"#)
        };

        pattern.push_str(&pat);
        pattern
    }

    fn transform(&self, mut input: String) -> String {
        input = self.append_header(input);
        input = self.replace_custom_tags(input);
        input
    }

    fn append_header(&self, mut input: String) -> String {
        let header = r#"
            <meta charset="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        "#;

        if let Some(index) = input.find("<head>") {
            input.insert_str(index + "<head>".len(), header);
        }

        input
    }

    fn replace_custom_tags(&self, mut input: String) -> String {
        for (ref regex, replacement) in &self.tags {
            input = regex.replace_all(&input, replacement).to_string();
        }

        input
    }
}

fn main() {
    let mut lowc = LowC::new();

    lowc.add_simple_tag(&["d"], "<strong><dfn>$1</dfn></strong>");
    lowc.add_simple_tag(&["t"], "<strong><cite>$1</cite></strong>");
    lowc.add_simple_tag(&["link", "href"], "<a href=\"$1\">$2</a>");

    lowc.add_empty_tag(
        &["tube", "watch"],
        "<iframe src=\"https://yewtu.be/embed/$1\"></iframe>",
    );
    lowc.add_empty_tag(&["picture", "src"], "<figure><img src=\"$1\"></figure>");
    lowc.add_empty_tag(
        &["picture", "src", "caption"],
        "<figure><img src=\"$1\" alt=\"$2\"><figcaption>$2</figcaption></figure>",
    );

    let argv: Vec<String> = std::env::args().collect();

    let input: String = if argv.len() <= 1 {
        from_stdin().expect("Error reading from stdin")
    } else {
        let path = Path::new(&argv[0]);
        let source = &argv[1];

        match source.as_str() {
            "-h" | "--help" => {
                let binary = Path::new(path)
                    .file_name()
                    .expect("What the actual fuck is going on??")
                    .to_string_lossy();
                println!("Usage: {binary} [path]\n");
                return;
            }
            "-" => from_stdin().expect("Error reading from stdin"),
            _ => from_file(source).expect("Error reading from file"),
        }
    };

    let valid_html = lowc.transform(input);
    println!("{}", valid_html);
}

fn from_file(path: &str) -> Result<String, io::Error> {
    let canonicalized_path = PathBuf::from(path).canonicalize()?;
    fs::read_to_string(canonicalized_path)
}

fn from_stdin() -> Result<String, io::Error> {
    io::read_to_string(io::stdin())
}
