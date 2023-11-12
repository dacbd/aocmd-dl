
use std::option::Option;

use clap::Parser;
use chrono::{Utc, FixedOffset, Datelike};
use scraper::{Html, Selector};


#[derive(Debug)]
struct ParseDayError {
    details: String
}
impl ParseDayError {
    fn new(msg: &str) -> ParseDayError {
        ParseDayError{details: msg.to_string()}
    }
}
impl std::error::Error for ParseDayError {
    fn description(&self) -> &str {
        &self.details
    }
}
impl std::fmt::Display for ParseDayError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ParseDayError: {}", self.details)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    token: Option<String>,

    #[arg(short, long, default_value = "latest")]
    day: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (year, day) = convert_day_to_url(args.day)?;
    println!("WARNING: this will overwrite README.md file in current directory, hopefully a `git restore README.md` will save you.");


    let base_url = format!("https://adventofcode.com/{}/day/{}", year, day);
    let client = reqwest::Client::new();
    let mut request_builder = client.get(base_url.clone())
        .header("User-Agent", "https://github.com/dacbd/aocmd-dl");
    match args.token {
        Some(token) => {
            request_builder = request_builder.header("Cookie", format!("session={}", token));
        },
        None => {
            println!("No token provided, only public puzzles (part 1) will be available");
            println!("See link to get your token:");
        }
    }

    let request = request_builder.build()?;
    println!("Loading puzzle from {}", base_url);
    let response = client.execute(request).await?;
    let body = response.text().await?;
    println!("Parsing document");
    let document = Html::parse_document(&body);

    let mut markdown_buffer = String::new();
    let article_selector = Selector::parse("article")?;

    println!("Converting to markdown");
    for article in document.select(&article_selector) {
        markdown_buffer.push_str(html2md::parse_html(article.inner_html().as_str()).as_str() );
        markdown_buffer.push_str("\n\n\n");
    }
    println!("Writing to README.md");
    std::fs::write("README.md", markdown_buffer)?;
    Ok(())
}

fn convert_day_to_url(day: String) -> Result<(String, String), ParseDayError> {
    let eastern_time = FixedOffset::east_opt(5 * 3600).unwrap();
    let now = Utc::now().with_timezone(&eastern_time);
    let last_day = String::from("25");
    if day == "latest" {
        let (year, month, day) = (now.year(), now.month(), now.day());
        if month == 12 {
            if day > 25 {
                return Ok((year.to_string(), last_day));
            }
            return Ok((year.to_string(), day.to_string()));
        }
    
        return Ok(((year - 1).to_string(), last_day));
    }
    if !(day.len() == 6 || day.len() == 7) {
        return Err(ParseDayError::new(format!("Invalid day format: {}, expect length to be 6 or 7 characters, ex 2020/1 or 2015/25", day).as_str()));
    }
    match day.split_once("/") {
        Some((year, day)) => {
            if year.len() != 4 {
                return Err(ParseDayError::new("Invalid year, expect length to be 4 characters, ex 2020"));
            }
            if !(day.len() == 1 || day.len() == 2) {
                return Err(ParseDayError::new("Invalid day, expect length to be 1 or 2 characters, ex 1 or 25"));
            }
            return Ok((year.to_string(), day.to_string()));
        },
        None => {
            return Err(ParseDayError::new("Invalid day format, expect format to be year/day, ex 2020/1 or 2015/25"));
        }
    }
}
