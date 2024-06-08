use std::{fs, io};
use std::path::{Path, PathBuf};
use regex::Regex;
use toml::Table;
use platform_dirs::AppDirs;

fn get_config(config_file: &str) -> Result<Table, io::Error> {
    let directories = AppDirs::new(Some("lowc"), false).unwrap();
    let config_path = directories.config_dir.join(config_file);
    let toml = fs::read_to_string(config_path)?;

    Ok(parse_config(toml))
}

fn parse_config(source: String) -> Table {
    toml::from_str(&source).expect("Failed to parse TOML")
}

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
        let mut pattern = format!(r#"(?s)<{name}"#);

        for attr in attrs {
            pattern.push_str(&format!(r#"\s+{}="([^"]*?)""#, attr));
        }

        let closing_tag = if empty {
            format!(r#"\s*/?>"#)
        } else {
            format!(r#"\s*>(.*?)</{name}\s*>"#)
        };

        pattern.push_str(&closing_tag);
        pattern
    }

    fn transform(&self, mut input: String) -> String {
        input = self.append_header(input);
        input = self.replace_mentions(input);
        input = self.replace_custom_tags(input);
        input
    }

    fn append_header(&self, mut input: String) -> String {
        let header = r#"
            <meta charset="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <meta http-equiv="X-UA-Compatible" content="IE=edge" />

            <!-- Google, please don't mess with my lovingly handcrafted HTML -->
            <meta name="googlebot" content="notranslate">
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
    
    fn replace_mentions(&self, mut input: String) -> String {
        if let Ok(config) = get_config("mentions.toml") {            
            for (handle, data) in config {
                let site = data["site"].as_str().expect(&format!("Missing 'site' key for {}.", handle));
                let mention = data["mention"].as_bool().unwrap_or(false);

                let pattern = Regex::new(&format!(r"(@{})", handle)).expect("Invalid regex pattern");
                let replacement = format!(r#"<a href="{}" data-handle="{}" data-mention="{}" class="u-in-reply-to">$1</a>"#, site, handle, mention);

                input = pattern.replace_all(&input, replacement).to_string();
            }
        }

        input
    }
}

fn main() {
    let mut lowc = LowC::new();

    lowc.add_simple_tag(&["d"], "<strong><dfn>$1</dfn></strong>");
    lowc.add_simple_tag(&["t"], "<strong><cite>$1</cite></strong>");
    lowc.add_simple_tag(&["link", "href"], "<a href=\"$1\">$2</a>");

    lowc.add_empty_tag(&["tube", "watch"], "<iframe src=\"https://yewtu.be/embed/$1\"></iframe>",);
    lowc.add_empty_tag(&["button", "src"], "<img src=\"$1\" class=\"button\" width=\"88\" height=\"31\" alt=\"Button\">");
    lowc.add_empty_tag(&["picture", "src"], "<figure><img src=\"$1\"></figure>");
    lowc.add_empty_tag(&["picture", "src", "caption"], "<figure><img src=\"$1\" alt=\"$2\"><figcaption>$2</figcaption></figure>");

    let argv: Vec<String> = std::env::args().collect();

    let input: String = if argv.len() <= 1 {
        from_stdin().expect("Error reading from stdin")
    } else {
        let path = &argv[0];
        let source = &argv[1];

        match source.as_str() {
            "-h" | "--help" => {
                usage(path);
                return;
            },
            "-" => from_stdin().expect("Error reading from stdin"),
            _ => from_file(source).expect("Error reading from file"),
        }
    };

    let valid_html = lowc.transform(input);
    println!("{}", valid_html);
}

fn usage(path: &str) {
    let binary = Path::new(path)
        .file_name()
        .expect("What the actual fuck is going on??")
        .to_string_lossy();

    println!("Usage: {binary} [path]\n");
}

fn from_file(path: &str) -> Result<String, io::Error> {
    let canonicalized_path = PathBuf::from(path).canonicalize()?;
    fs::read_to_string(canonicalized_path)
}

fn from_stdin() -> Result<String, io::Error> {
    io::read_to_string(io::stdin())
}
