use rtk::core::filter::{get_filter, FilterLevel, Language};
use rtk::pipe_cmd::resolve_filter;

#[test]
fn exposes_core_filter_module() {
    let filter = get_filter(FilterLevel::Minimal);
    let output = filter.filter("// hidden\nfn main() {}\n", &Language::Rust);

    assert_eq!(output, "fn main() {}");
}

#[test]
fn exposes_pipe_filter_resolver() {
    let filter = resolve_filter("grep").expect("grep filter should be exported");
    let output = filter("src/main.rs:42:fn main() {}\n");

    assert!(output.contains("1 matches"));
}
