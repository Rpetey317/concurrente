use crate::site_summary::SiteSummary;
use crate::tag_summary::TagSummary;
use serde::Serialize;
use std::collections::HashMap;

const N_CHATTY: usize = 10;

/// A struct containing the two global summaries (chattiest sites and tags).
/// Made only to make use of serde api for formatting and printing
#[derive(Serialize)]
struct Totals {
    /// 10 chattiest (word count / question count) sites
    chatty_sites: Vec<String>,
    /// 10 chattiest (word count / question count) tags (by aggregate word & question count over all sites)
    chatty_tags: Vec<String>,
}

/// A struct containing the full summary of the data as asked in the assignment specifications.
#[derive(Serialize)]
pub struct FullSummary {
    /// Student ID
    padron: u32,
    /// Summary of each site
    sites: HashMap<String, SiteSummary>,
    /// Summary of each tag aggregated through all sites
    tags: HashMap<String, TagSummary>,
    /// Global summaries (chattiest sites and tags)
    totals: Totals,
}

impl FullSummary {
    /// Creates a summary from the given, analyzed data.
    ///
    /// # Arguments
    /// `padron` - Student ID
    /// `sites` - Summary of each individual site
    /// `total` - Aggregated summary of all sites, used to get the global chattiest tags
    pub fn new(
        padron: u32,
        sites: HashMap<String, SiteSummary>,
        total: &SiteSummary,
    ) -> FullSummary {
        let tags = total.tags.clone();

        let _chatty_tags = total.n_chattiest(N_CHATTY);
        let chatty_tags = _chatty_tags.iter().map(|tag| tag.name.clone()).collect();

        let mut sites_vec = sites
            .iter()
            .map(|(name, site)| (name.clone(), site.chattiness()))
            .collect::<Vec<_>>();
        sites_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let chatty_sites = sites_vec
            .iter()
            .take(N_CHATTY)
            .map(|(name, _)| name.clone())
            .collect();

        let totals = Totals {
            chatty_sites,
            chatty_tags,
        };

        FullSummary {
            padron,
            sites,
            tags,
            totals,
        }
    }
}

// No tests as this is just for output formatting
