use core::panic;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use site_summary::SiteSummary;
use std::collections::HashMap;
use std::env;
use std::fs;

mod full_summary;
mod parser;
mod site_summary;
mod tag_summary;

/// Main function. Reads all jsonl files in the data directory, parses them and outputs a full summary to stdout.
///
/// Takes an optional argument for the number of threads to use. Defaults to 4 on error or if not provided.
///
/// Prints info messages to stderr for the sake of leaving just the final summary in stdout.
fn main() {
    // get number of threads from command line
    let args: Vec<String> = env::args().collect();
    let n_threads = args
        .get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(4);

    // set number of threads
    let thread_pool_result = ThreadPoolBuilder::new()
        .num_threads(n_threads)
        .build_global();
    if let Err(e) = thread_pool_result {
        panic!("Failed to set number of threads: {}", e);
    }

    // get list of jsonl files in data directory
    let mut sites: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("data") {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.ends_with(".jsonl") {
                    sites.push(file_name_str.to_string());
                }
            }
        }
    } else {
        panic!("Failed to read data directory. Perhaps you forgot to run download_data.sh?");
    }

    // process files
    let summaries: HashMap<String, SiteSummary> = sites
        .par_iter()
        .map(|site| {
            eprintln!("Processing {}", site);
            (
                site.clone(),
                parser::parse_file(&format!("data/{}", site), site).unwrap_or(SiteSummary::empty()),
            )
        })
        .collect();

    // get aggregated summary
    let total = summaries.clone().into_par_iter().reduce(
        || (String::new(), SiteSummary::empty()),
        |a: (String, SiteSummary), b: (String, SiteSummary)| {
            let combined = a.1.combine(&b.1);
            (combined.name.clone(), combined)
        },
    );

    // generate output
    let ans = full_summary::FullSummary::new(109442, summaries, &total.1);
    let json_result = serde_json::to_string_pretty(&ans);
    match json_result {
        Ok(j) => println!("{}", j),
        Err(e) => panic!("Failed to serialize output: {}", e),
    };
}
