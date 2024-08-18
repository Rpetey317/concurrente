use crate::site_summary::SiteSummary;
use crate::tag_summary::TagSummary;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Auxiliary struct to parse jsonl files
#[derive(Deserialize)]
struct Line {
    texts: Vec<String>,
    tags: Vec<String>,
}

/// Parses a jsonl file containing site data and returns a SiteSummary with the given name
///
/// May fail if the file cannot be opened
pub fn parse_file(filename: &str, site_name: &str) -> io::Result<SiteSummary> {
    let path = Path::new(filename);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut summary = SiteSummary::new(site_name, Vec::new());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Ok(line) = serde_json::from_str::<Line>(&line) {
            let wc = line.texts[0].split_whitespace().count() as u32
                + line.texts[1].split_whitespace().count() as u32;

            summary.add_question(wc);

            for tag in line.tags {
                let new_tag = TagSummary::new(tag, 1, wc);
                summary.add_tag(new_tag);
            }
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod test {
    use super::parse_file;

    #[test]
    fn simplest_site_works() {
        let summary_result = parse_file("test_data/site1.jsonl", &"test".to_string());
        let summary = match summary_result {
            Ok(s) => s,
            Err(e) => panic!("Failed to parse file: {}", e),
        };
        assert_eq!(summary.questions, 1);
        assert_eq!(summary.words, 6);
        assert_eq!(summary.tags.len(), 1);
        assert_eq!(summary.chattiness(), 6.0)
    }

    #[test]
    fn two_tags_work() {
        let summary_result = parse_file("test_data/site2.jsonl", &"test".to_string());
        let summary = match summary_result {
            Ok(s) => s,
            Err(e) => panic!("Failed to parse file: {}", e),
        };
        assert_eq!(summary.questions, 1);
        assert_eq!(summary.words, 6);
        assert_eq!(summary.tags.len(), 2);
        assert_eq!(summary.chattiness(), 6.0)
    }

    #[test]
    fn two_questions_work() {
        let summary_result = parse_file("test_data/site3.jsonl", &"test".to_string());
        let summary = match summary_result {
            Ok(s) => s,
            Err(e) => panic!("Failed to parse file: {}", e),
        };
        assert_eq!(summary.questions, 2);
        assert_eq!(summary.words, 13);
        assert_eq!(summary.tags.len(), 3);
        assert_eq!(summary.chattiness(), 6.5)
    }

    #[test]
    fn small_site_test() {
        let summary_result = parse_file("test_data/site4.jsonl", &"test".to_string());
        let summary = match summary_result {
            Ok(s) => s,
            Err(e) => panic!("Failed to parse file: {}", e),
        };
        assert_eq!(summary.questions, 15);
        assert_eq!(summary.words, 150);
        assert_eq!(summary.tags.len(), 11);
        assert_eq!(summary.chattiness(), 10.0)
    }
}
