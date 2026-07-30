//! Git diff analysis - identifies changed files, functions, and computes change impact.
//!
//! Parses unified diff format to extract:
//! - Changed files and line ranges
//! - Added/modified/deleted functions
//! - Change type classification (new feature, bug fix, refactor)

use regex::Regex;
use std::collections::HashSet;

/// Represents a single file change from a git diff
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub added_lines: Vec<usize>,
    pub removed_lines: Vec<usize>,
    pub changed_functions: Vec<String>,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed(String),  // old_name -> new_name
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub context: String,
    pub added_lines: Vec<usize>,
    pub removed_lines: Vec<usize>,
}

/// Analysis result for a git diff
#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub files_changed: Vec<FileChange>,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub changed_functions: HashSet<String>,
    pub affected_modules: HashSet<String>,
    pub change_category: ChangeCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeCategory {
    NewFeature,
    BugFix,
    Refactor,
    DependencyUpdate,
    Configuration,
    Documentation,
    Unknown,
}

/// Parse a unified diff string into structured change information
pub fn parse_diff(diff_text: &str) -> DiffAnalysis {
    let file_re = Regex::new(r"^diff --git a/(.+) b/(.+)$").unwrap();
    let change_re = Regex::new(r"^(new file mode|deleted file mode|rename from|rename to)").unwrap();
    let hunk_re = Regex::new(r"^@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@ ?(.*)$").unwrap();
    let func_re = Regex::new(r"(?:def|fn|func|function|class)\s+(\w+)").unwrap();

    let mut files = Vec::new();
    let mut current_file: Option<FileChange> = None;
    let mut total_additions = 0;
    let mut total_deletions = 0;
    let mut changed_functions = HashSet::new();
    let mut affected_modules = HashSet::new();

    for line in diff_text.lines() {
        // New file
        if let Some(caps) = file_re.captures(line) {
            if let Some(file) = current_file.take() {
                files.push(file);
            }
            let path = caps.get(1).unwrap().as_str().to_string();
            // Extract module from path
            if let Some(module) = path.split('/').next() {
                affected_modules.insert(module.to_string());
            }
            current_file = Some(FileChange {
                path,
                change_type: ChangeType::Modified,
                added_lines: Vec::new(),
                removed_lines: Vec::new(),
                changed_functions: Vec::new(),
                hunks: Vec::new(),
            });
            continue;
        }

        let file = match current_file.as_mut() {
            Some(f) => f,
            None => continue,
        };

        // Change type classification
        if line.starts_with("new file mode") {
            file.change_type = ChangeType::Added;
        } else if line.starts_with("deleted file mode") {
            file.change_type = ChangeType::Deleted;
        } else if line.starts_with("rename from ") {
            let old = line.strip_prefix("rename from ").unwrap_or("");
            file.change_type = ChangeType::Renamed(old.to_string());
        }

        // Hunk headers
        if let Some(caps) = hunk_re.captures(line) {
            let new_start: usize = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
            let mut hunk = DiffHunk {
                old_start: caps.get(1).unwrap().as_str().parse().unwrap_or(0),
                old_count: caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(1),
                new_start,
                new_count: caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(1),
                context: caps.get(5).map(|m| m.as_str().to_string()).unwrap_or_default(),
                added_lines: Vec::new(),
                removed_lines: Vec::new(),
            };

            let mut line_num = new_start;
            // Hunk already processed; count lines later in the loop
            file.hunks.push(hunk);
        }

        // Count additions and detect function changes
        if line.starts_with('+') && !line.starts_with("+++") {
            total_additions += 1;
            // Check if this line defines a function
            if let Some(caps) = func_re.captures(line) {
                let func_name = caps.get(1).unwrap().as_str().to_string();
                file.changed_functions.push(func_name.clone());
                changed_functions.insert(func_name);
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            total_deletions += 1;
        }
    }

    // Don't forget the last file
    if let Some(file) = current_file {
        files.push(file);
    }

    let change_category = classify_change(&files);

    DiffAnalysis {
        files_changed: files,
        total_additions,
        total_deletions,
        changed_functions,
        affected_modules,
        change_category,
    }
}

/// Classify the type of change based on file patterns
fn classify_change(files: &[FileChange]) -> ChangeCategory {
    if files.is_empty() {
        return ChangeCategory::Unknown;
    }

    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();

    let all_docs = paths.iter().all(|p| p.ends_with(".md") || p.ends_with(".rst") || p.contains("docs/"));
    if all_docs {
        return ChangeCategory::Documentation;
    }

    let all_config = paths.iter().all(|p| {
        p.contains(".yml") || p.contains(".yaml") || p.contains(".toml")
            || p.contains(".json") || p.contains("Dockerfile") || p.contains("Makefile")
    });
    if all_config {
        return ChangeCategory::Configuration;
    }

    let has_dep_files = paths.iter().any(|p| {
        p.contains("Cargo.toml") || p.contains("Cargo.lock")
            || p.contains("package.json") || p.contains("package-lock.json")
            || p.contains("requirements.txt") || p.contains("Pipfile")
            || p.contains("go.mod") || p.contains("go.sum")
    });
    if has_dep_files && files.len() <= 3 {
        return ChangeCategory::DependencyUpdate;
    }

    let has_test = paths.iter().any(|p| p.contains("test") || p.contains("spec") || p.contains("__tests__"));
    if has_test && files.len() == 1 {
        return ChangeCategory::BugFix;
    }

    if files.iter().any(|f| f.change_type == ChangeType::Added) {
        ChangeCategory::NewFeature
    } else {
        ChangeCategory::Refactor
    }
}
