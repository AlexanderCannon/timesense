use std::collections::HashMap;
use std::time::Duration;

/// Normalizes an app name by converting to lowercase and removing common suffixes
pub fn normalize_app_name(app_name: &str) -> String {
    let lower = app_name.to_lowercase();
    
    // Remove common suffixes and extensions
    let without_suffix = lower
        .replace(".app", "")
        .replace(".exe", "")
        .replace(".lnk", "")
        .replace(".desktop", "");
    
    // Remove common prefixes
    let without_prefix = without_suffix
        .replace("microsoft ", "")
        .replace("google ", "")
        .replace("apple ", "");
    
    without_prefix.trim().to_string()
}

/// Groups similar application names together based on string similarity
/// Returns a HashMap where keys are the canonical app names and values are tuples of
/// (total duration, list of (app name, duration) pairs)
pub fn group_similar_apps(app_times: &HashMap<String, Duration>) -> HashMap<String, (Duration, Vec<(String, Duration)>)> {
    let mut grouped_apps: HashMap<String, (Duration, Vec<(String, Duration)>)> = HashMap::new();
    
    for (app_name, duration) in app_times {
        let canonical_name = find_canonical_name(app_name, &grouped_apps);
        
        let entry = grouped_apps.entry(canonical_name.clone())
            .or_insert_with(|| (Duration::from_secs(0), Vec::new()));
        
        entry.0 += *duration;
        entry.1.push((app_name.clone(), *duration));
    }
    
    grouped_apps
}

/// Finds the canonical name for an app by checking similarity with existing names
fn find_canonical_name(app_name: &str, grouped_apps: &HashMap<String, (Duration, Vec<(String, Duration)>)>) -> String {
    // If no existing groups, use the app name as is
    if grouped_apps.is_empty() {
        return app_name.to_string();
    }
    
    // Check similarity with existing canonical names
    for canonical_name in grouped_apps.keys() {
        if are_similar_names(app_name, canonical_name) {
            return canonical_name.clone();
        }
    }
    
    // If no similar name found, use the app name as is
    app_name.to_string()
}

/// Determines if two app names are similar enough to be considered the same app
fn are_similar_names(name1: &str, name2: &str) -> bool {
    // Convert to lowercase for case-insensitive comparison
    let name1 = name1.to_lowercase();
    let name2 = name2.to_lowercase();
    
    // Direct match
    if name1 == name2 {
        return true;
    }
    
    // Check if one name contains the other
    if name1.contains(&name2) || name2.contains(&name1) {
        return true;
    }
    
    // Remove common suffixes and check again
    let suffixes = [".app", " (1)", " (2)", " - ", " — "];
    let name1_clean = suffixes.iter().fold(name1.clone(), |s, suffix| s.replace(suffix, ""));
    let name2_clean = suffixes.iter().fold(name2.clone(), |s, suffix| s.replace(suffix, ""));
    
    if name1_clean == name2_clean {
        return true;
    }
    
    // Calculate Levenshtein distance for fuzzy matching
    let distance = levenshtein_distance(&name1_clean, &name2_clean);
    let max_length = std::cmp::max(name1_clean.len(), name2_clean.len());
    
    // If the distance is less than 30% of the max length, consider them similar
    (distance as f32) / (max_length as f32) < 0.3
}

/// Calculates the Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let mut matrix = vec![vec![0; v2.len() + 1]; v1.len() + 1];
    
    // Initialize first row and column
    for i in 0..=v1.len() {
        matrix[i][0] = i;
    }
    for j in 0..=v2.len() {
        matrix[0][j] = j;
    }
    
    // Fill in the rest of the matrix
    for i in 1..=v1.len() {
        for j in 1..=v2.len() {
            let cost = if v1[i-1] == v2[j-1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                matrix[i-1][j] + 1, // deletion
                std::cmp::min(
                    matrix[i][j-1] + 1, // insertion
                    matrix[i-1][j-1] + cost // substitution
                )
            );
        }
    }
    
    matrix[v1.len()][v2.len()]
} 