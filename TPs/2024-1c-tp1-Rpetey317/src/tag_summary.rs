use serde::Serialize;
use std::cmp::Ordering;

/// A strcut that contains word and question count for a given tag
#[derive(Serialize)]
pub struct TagSummary {
    /// Tag name. Necessary to make sure only similar tags are combined
    #[serde(skip_serializing)]
    pub name: String,
    /// N° of questions this tag appears in
    pub questions: u32,
    /// Total word count for every question this tag appears in
    pub words: u32,
}

impl TagSummary {
    /// Creates a new TagSummary with the given name, question count and word count
    pub fn new(name: String, question_count: u32, word_count: u32) -> TagSummary {
        TagSummary {
            name,
            questions: question_count,
            words: word_count,
        }
    }

    /// Combines two summaries into one, adding word and question counts.
    /// Fails if the tags have different names
    pub fn combine(&self, other: &TagSummary) -> TagSummary {
        assert!(self.name == other.name, "Cannot combine two different tags");
        TagSummary {
            name: self.name.clone(),
            questions: self.questions + other.questions,
            words: self.words + other.words,
        }
    }

    /// Returns chattiness score (word count / question count).
    /// Returns 0 if there are no questions
    pub fn chattiness(&self) -> f32 {
        if self.questions == 0 {
            return 0.0;
        }
        self.words as f32 / self.questions as f32
    }
}

/// Clone implementation for TagSummary
impl Clone for TagSummary {
    fn clone(&self) -> Self {
        TagSummary {
            name: self.name.clone(),
            questions: self.questions,
            words: self.words,
        }
    }
}

// TODO: chequear si puedo sacar eq y ord
impl PartialEq for TagSummary {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for TagSummary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.chattiness()).partial_cmp(&other.chattiness())
    }
}

#[cfg(test)]
mod test {
    use super::TagSummary;

    fn test_tag1() -> TagSummary {
        TagSummary::new("tag1".to_string(), 10, 100)
    }

    fn test_tag1_alt() -> TagSummary {
        TagSummary::new("tag1".to_string(), 10, 200)
    }

    fn test_tag2() -> TagSummary {
        TagSummary::new("tag2".to_string(), 20, 150)
    }

    #[test]
    fn chattiness_works() {
        let tag = test_tag1();
        assert_eq!(tag.chattiness(), 10.0);
    }

    #[test]
    fn can_combine_similar_tags() {
        let tag1 = test_tag1();
        let tag2 = test_tag1_alt();
        let combined = tag1.combine(&tag2);
        assert_eq!(combined.questions, 20);
        assert_eq!(combined.words, 300);
        assert_eq!(combined.chattiness(), 15.0);
    }

    #[test]
    #[should_panic]
    fn cannot_combine_different_tags() {
        let tag1 = test_tag1();
        let tag2 = test_tag2();
        let _ = tag1.combine(&tag2);
    }
}
