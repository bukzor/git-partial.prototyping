//! Tests for path matching logic.

use std::path::{Path, PathBuf};

use crate::prepare::path_matches;

#[test]
fn exact_file_match() {
    let entry = Path::new("src/main.rs");
    let requested = vec![PathBuf::from("src/main.rs")];
    assert!(path_matches(entry, &requested));
}

#[test]
fn directory_contains_file() {
    let entry = Path::new("src/main.rs");
    let requested = vec![PathBuf::from("src")];
    assert!(path_matches(entry, &requested));
}

#[test]
fn nested_directory_contains_file() {
    let entry = Path::new("src/foo/bar/baz.rs");
    let requested = vec![PathBuf::from("src/foo")];
    assert!(path_matches(entry, &requested));
}

#[test]
fn no_match_different_file() {
    let entry = Path::new("src/main.rs");
    let requested = vec![PathBuf::from("src/lib.rs")];
    assert!(!path_matches(entry, &requested));
}

#[test]
fn no_match_sibling_directory() {
    let entry = Path::new("src/main.rs");
    let requested = vec![PathBuf::from("tests")];
    assert!(!path_matches(entry, &requested));
}

#[test]
fn no_false_prefix_match() {
    // "src/foo" should NOT match "src/foobar/baz.rs"
    let entry = Path::new("src/foobar/baz.rs");
    let requested = vec![PathBuf::from("src/foo")];
    assert!(!path_matches(entry, &requested));
}

#[test]
fn multiple_requested_paths() {
    let entry = Path::new("tests/test_foo.rs");
    let requested = vec![PathBuf::from("src"), PathBuf::from("tests")];
    assert!(path_matches(entry, &requested));
}

#[test]
fn root_matches_everything() {
    let entry = Path::new("src/deeply/nested/file.rs");
    let requested = vec![PathBuf::from("")];
    assert!(path_matches(entry, &requested));
}
