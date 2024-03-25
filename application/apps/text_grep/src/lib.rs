pub mod buffer;
use crate::buffer::{CancallableMinBuffered, REDUX_MIN_BUFFER_SPACE, REDUX_READER_CAPACITY};
use buf_redux::BufReader;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{sinks::UTF8, Searcher};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error, Clone)]
pub enum GrepError {
    #[error("File '{0}' is not a text file")]
    NotATextFile(String),
    #[error("Error reading file: {0}")]
    FileReadError(String),
    #[error("Error processing file: {0}")]
    FileProcessingError(String),
    #[error("Operation cancelled")]
    OperationCancelled,
    #[error("Error building regex: {0}")]
    BuilingRegExError(grep_regex::Error),
    #[error("Error building regex: {0}")]
    RegExError(regex::Error),
    #[error("IO error: {0}")]
    IOError(String),
}

impl From<grep_regex::Error> for GrepError {
    fn from(e: grep_regex::Error) -> Self {
        Self::BuilingRegExError(e)
    }
}

impl From<regex::Error> for GrepError {
    fn from(e: regex::Error) -> Self {
        Self::RegExError(e)
    }
}

impl From<io::Error> for GrepError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub pattern_counts: HashMap<String, usize>,
    pub error_message: Option<String>,
}

fn get_matcher(patterns: &[&str]) -> Result<RegexMatcher, GrepError> {
    Ok(RegexMatcherBuilder::new().build(
        &patterns
            .iter()
            .map(|pattern| regex::escape(pattern))
            .collect::<Vec<String>>()
            .join("|"),
    )?)
}

fn get_patterns_as_regs(patterns: &[&str], _case_sensitive: bool) -> Result<Vec<Regex>, GrepError> {
    // TODO: consider "case_sensitive" flag
    let mut regs: Vec<Regex> = Vec::new();
    for pattern in patterns.iter() {
        regs.push(Regex::new(&regex::escape(pattern))?);
    }
    Ok(regs)
}

fn process_file(
    file_path: &PathBuf,
    matcher: &RegexMatcher,
    patterns: &[Regex],
    cancel_token: &CancellationToken,
) -> Result<SearchResult, GrepError> {
    if !is_text_file(file_path) {
        return Ok(SearchResult {
            file_path: file_path.to_string_lossy().into_owned(),
            pattern_counts: HashMap::new(),
            error_message: Some(format!("File '{}' is not a text file", file_path.display())),
        });
    }
    let mut pattern_counts = HashMap::new();
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(REDUX_READER_CAPACITY, file).set_policy(
        CancallableMinBuffered((REDUX_MIN_BUFFER_SPACE, cancel_token.clone())),
    );
    let mut searcher = Searcher::new();
    searcher
        .search_reader(
            matcher,
            reader,
            UTF8(|_, line| {
                for pattern in patterns {
                    let count_entry = pattern_counts.entry((*pattern).to_string()).or_insert(0);
                    *count_entry += pattern.captures_iter(line).count();
                }
                Ok(true)
            }),
        )
        .map_err(|e| GrepError::FileProcessingError(format!("Error processing file: {}", e)))?;

    Ok(SearchResult {
        file_path: file_path.to_string_lossy().into_owned(),
        pattern_counts,
        error_message: None,
    })
}

fn is_text_file(_file_path: &Path) -> bool {
    true
}

pub async fn count_occurrences(
    patterns: &[&str],
    file_paths: &[&PathBuf],
    case_sensitive: bool,
    cancel_token: CancellationToken,
) -> Result<Vec<Result<SearchResult, GrepError>>, GrepError> {
    let mut results = Vec::new();
    let matcher = get_matcher(patterns)?;
    let regs = get_patterns_as_regs(patterns, case_sensitive)?;
    for file_path in file_paths {
        if cancel_token.is_cancelled() {
            return Err(GrepError::OperationCancelled); // Return early if cancellation requested
        }
        results.push(process_file(file_path, &matcher, &regs, &cancel_token));
    }
    Ok(results)
}
