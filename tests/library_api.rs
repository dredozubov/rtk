use rtk::core::filter::{get_filter, FilterLevel, Language};
use rtk::pipe_cmd::{apply, resolve_filter_raw};

#[test]
fn exposes_core_filter_module() {
    let filter = get_filter(FilterLevel::Minimal);
    let output = filter.filter("// hidden\nfn main() {}\n", &Language::Rust);

    assert_eq!(output, "fn main() {}");
}

#[test]
fn exposes_pipe_filter_resolver() {
    let filter = resolve_filter_raw("grep").expect("grep filter should be exported");
    let output = filter("src/main.rs:42:fn main() {}\n");

    assert!(output.contains("1 matches"));
}

#[test]
fn apply_compresses_recognized_dialects() {
    let mut input = String::from("diff --git a/x.rs b/x.rs\nindex 111..222 100644\n--- a/x.rs\n+++ b/x.rs\n@@ -1 +1 @@\n-old\n+new\n");
    for _ in 0..20 {
        input.push_str("diff --git a/filler.rs b/filler.rs\nindex 111..222 100644\n--- a/filler.rs\n+++ b/filler.rs\n@@ -1 +1 @@\n context\n");
    }
    let filtered = apply("git-diff", &input).expect("real diff should compress");
    assert!(filtered.len() < input.len());
    assert!(filtered.contains("+new"));
}

#[test]
fn apply_rejects_unrecognized_dialects_instead_of_lying() {
    // Plain-text go test output is not the go-test parser's dialect;
    // the old behavior answered "Go test: No tests found" for it.
    let plain_go_test = "=== RUN   TestAdd\n--- FAIL: TestAdd (0.00s)\nFAIL\nFAIL\texample.com/pkg\t0.01s\n";
    assert_eq!(apply("go-test", plain_go_test), None);

    // Off-format prettier input must not become an all-clean verdict.
    assert_eq!(apply("prettier", "src/main.rs\n"), None);

    // Bare tsc config errors are not the located forms the tsc parser
    // counts; they must not become "compilation completed".
    assert_eq!(apply("tsc", "error TS2688: Cannot find type definition file for 'x'.\n"), None);

    // Non-diff text is not git-diff material.
    assert_eq!(apply("git-diff", "hello world\n"), None);
}

#[test]
fn apply_rejects_output_that_does_not_shrink() {
    // Small grep inputs reformat larger than raw; never_worse reverts
    // and apply declines rather than charge the difference.
    let small = "a.rs:1:x\nb.rs:2:y\nc.rs:3:z\n";
    assert_eq!(apply("grep", small), None);
}

#[test]
fn apply_recognizes_the_parsers_tsc_dialects() {
    let pretty = include_str!("fixtures/tsc_pretty_raw.txt");
    let filtered = apply("tsc", pretty).expect("located tsc errors should compress");
    assert!(filtered.contains("TS2322"));
}
