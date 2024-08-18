use crate::tag_summary::TagSummary;
use serde::Serialize;
use std::collections::HashMap;

/// A struct that contains aggregated tag and question data for a site
#[derive(Serialize)]
pub struct SiteSummary {
    /// Site name
    #[serde(skip_serializing)]
    pub name: String,
    /// N° of questions in this site
    pub questions: u32,
    /// Total word count for every question in this site
    pub words: u32,
    /// Summaries of each tag in this site
    pub tags: HashMap<String, TagSummary>,
}

impl SiteSummary {
    /// Creates a new SiteSummary with the given site name and adds given tags.
    pub fn new(site: &str, tags: Vec<TagSummary>) -> SiteSummary {
        // TODO: sacarle el vec a esta función
        let mut summary = SiteSummary {
            name: String::from(site),
            questions: 0,
            words: 0,
            tags: HashMap::new(),
        };

        for tag in tags {
            summary.words += tag.words;
            summary.questions += tag.questions;
            summary.add_tag(tag);
        }

        summary
    }

    /// Creates an empty SiteSummary
    pub fn empty() -> SiteSummary {
        SiteSummary {
            name: "Empty".to_string(),
            questions: 0,
            words: 0,
            tags: HashMap::new(),
        }
    }

    /// Combines two summaries, aggregating word and question counts and combining tags accordingly
    pub fn combine(&self, other: &SiteSummary) -> SiteSummary {
        let mut tags = self.tags.clone();
        for (tag_name, tag) in &other.tags {
            if let Some(my_tag) = tags.get_mut(tag_name) {
                *my_tag = my_tag.combine(tag);
            } else {
                tags.insert(tag_name.to_string(), tag.clone());
            }
        }
        SiteSummary {
            name: "Combined".to_string(),
            questions: self.questions + other.questions,
            words: self.words + other.words,
            tags,
        }
    }

    /// Adds a tag to the summary, combining it with an existing tag if it already exists.
    ///
    /// Does NOT update question and word counts, that has to be done using `add_question`.
    /// This is because a question can have multiple tags, so it could count the same question multiple times
    pub fn add_tag(&mut self, tag: TagSummary) {
        if let Some(existing_tag) = self.tags.get_mut(&tag.name) {
            *existing_tag = existing_tag.combine(&tag);
        } else {
            self.tags.insert(tag.name.to_string(), tag);
        }
    }

    /// Updates question and word counts of the site according to a new question of the given wordcount
    pub fn add_question(&mut self, word_count: u32) {
        self.questions += 1;
        self.words += word_count;
    }

    /// Returns chattiness score (word count / question count).
    /// Returns 0 if there are no questions
    pub fn chattiness(&self) -> f32 {
        if self.questions == 0 {
            return 0.0;
        }
        self.words as f32 / self.questions as f32
    }

    /// Returns the n tags with the highest chattiness score in the site
    pub fn n_chattiest(&self, n: usize) -> Vec<TagSummary> {
        let mut tags: Vec<TagSummary> = self.tags.values().cloned().collect();
        tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        tags.reverse();
        tags.truncate(n);
        tags
    }
}

/// Clone implementation for SiteSummary
impl Clone for SiteSummary {
    fn clone(&self) -> Self {
        SiteSummary {
            name: self.name.clone(),
            questions: self.questions,
            words: self.words,
            tags: self.tags.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::tag_summary::TagSummary;
    use super::SiteSummary;

    fn test_site1() -> SiteSummary {
        let tag1 = TagSummary::new("tag1".to_string(), 10, 100);
        let tag2 = TagSummary::new("tag2".to_string(), 20, 150);
        let tags = vec![tag1, tag2];
        SiteSummary::new(&"site1".to_string(), tags)
    }

    fn test_site2() -> SiteSummary {
        let tag1 = TagSummary::new("tag3".to_string(), 20, 100);
        let tag2 = TagSummary::new("tag4".to_string(), 30, 200);
        let tags = vec![tag1, tag2];
        SiteSummary::new(&"site2".to_string(), tags)
    }

    fn test_site3() -> SiteSummary {
        let tag1 = TagSummary::new("tag2".to_string(), 10, 150);
        let tag2 = TagSummary::new("tag3".to_string(), 30, 200);
        let tags = vec![tag1, tag2];
        SiteSummary::new(&"site3".to_string(), tags)
    }

    #[test]
    fn stats_initialize_correctly() {
        let site = test_site1();
        assert_eq!(site.questions, 30);
        assert_eq!(site.words, 250);
        assert_eq!(site.tags.len(), 2);
    }

    #[test]
    fn can_combine_sites() {
        let site1 = test_site1();
        let site2 = test_site2();
        let combined = site1.combine(&site2);
        assert_eq!(combined.questions, 80);
        assert_eq!(combined.words, 550);
        assert_eq!(combined.tags.len(), 4);
    }

    #[test]
    fn overlapping_tags_get_combined() {
        let site1 = test_site1();
        let site3 = test_site3();
        let combined = site1.combine(&site3);
        assert_eq!(combined.questions, 70);
        assert_eq!(combined.words, 600);
        assert_eq!(combined.tags.len(), 3);
    }

    #[test]
    fn n_chattiest_works() {
        let site = test_site1().combine(&test_site2());
        let chattiest = site.n_chattiest(2);
        assert_eq!(chattiest.len(), 2);
        assert_eq!(chattiest[0].name, "tag1");
        assert_eq!(chattiest[1].name, "tag2");
    }
}
