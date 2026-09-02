#[path = "prompts/example.rs"]
mod example;

use std::path::Path;

use promptex_rs::config::{claude, ConfigUnit};

pub fn units(_project_root: &Path) -> Vec<ConfigUnit> {
    vec![ConfigUnit::new("./prompts", vec![claude()])
        .name("ex-minimal-project-rs")
        .out_dir(".")
        .evaluate(example::declare)]
}
