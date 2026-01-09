use anyhow::{self, Ok, Result};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use std::sync::LazyLock;
use walkdir::{DirEntry, WalkDir};

pub static DEFAULT_FILTERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "tid:".to_string(),
        "pid:".to_string(),
        "cpu usage".to_string(),
    ]
});

static BASE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(
        "D:/project/日照港卸料小车项目/code/unload_modeling/build/bin/RelWithDebInfo/log/",
    )
});

#[derive(Parser)]
pub struct CheckLineArgs {
    /// 文件路径
    #[arg(short, long)]
    pub path: PathBuf,

    /// 需要过滤的关键字
    #[arg(short, long)]
    pub filters: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct RemoveLineArgs {
    /// 文件路径
    #[arg(short, long)]
    pub path: PathBuf,

    /// 需要过滤的关键字
    #[arg(short, long)]
    pub filters: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct RemoveFileArgs {}

pub fn process_check_line(args: CheckLineArgs) -> Result<()> {
    let path = if args.path.is_absolute() {
        args.path
    } else {
        BASE_DIR.join(&args.path)
    };

    let filters = args.filters.unwrap_or(DEFAULT_FILTERS.to_vec());

    if path.is_dir() {
        check_log_dir_cpu_mem_infos(path, &filters);
    } else {
        check_log_file_cpu_mem_info(path, &filters)?;
    }

    Ok(())
}

fn check_log_dir_cpu_mem_infos<P: AsRef<Path>>(dir: P, filters: &[String]) {
    let entries = get_entries(dir);

    entries.par_iter().for_each(|e| {
        let file_path = e.path();
        if let Err(e) = check_log_file_cpu_mem_info(file_path, filters) {
            println!("❌check line failed, path {:?}, reason: {}", file_path, e);
        }
    });
}

fn check_log_file_cpu_mem_info<P: AsRef<Path>>(path: P, filters: &[String]) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let lines = content
        .lines()
        .filter(|&s| contains_keyword(s, filters))
        .collect::<Vec<_>>();

    println!(
        "file: {}, keyword lines: {}",
        path.as_ref().display(),
        lines.len()
    );

    Ok(())
}

pub fn process_remove_line(args: RemoveLineArgs) -> Result<()> {
    let path = if args.path.is_absolute() {
        args.path
    } else {
        BASE_DIR.join(&args.path)
    };

    let filters = args.filters.unwrap_or(DEFAULT_FILTERS.to_vec());

    if path.is_dir() {
        remove_log_dir_cpu_mem_infos(&path, &filters);
    } else {
        remove_log_file_cpu_mem_info(&path, &filters)?;
    }

    Ok(())
}

fn remove_log_dir_cpu_mem_infos<P: AsRef<Path>>(dir: P, filters: &[String]) {
    let entries = get_entries(dir);

    entries.par_iter().for_each(|e| {
        let file_path = e.path();
        if let Err(e) = remove_log_file_cpu_mem_info(file_path, filters) {
            println!("❌remove line failed, path {:?}, reason: {}", file_path, e);
        }
    });
}

fn get_entries<P: AsRef<Path>>(dir: P) -> Vec<DirEntry> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|s| !s.contains("_filtered"))
        })
        .collect::<Vec<_>>()
}

fn remove_log_file_cpu_mem_info<P: AsRef<Path>>(path: P, filters: &[String]) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let lines = content
        .lines()
        .filter(|&s| filter_keyword(s, filters))
        .map(|s| format!("{s}\n"))
        .collect::<String>();

    let path = path.as_ref();
    let stem = path.file_stem().unwrap_or_default();
    let ext = path.extension().unwrap_or_default();
    let new_path = path.with_file_name(format!(
        "{}_filtered{}{}",
        stem.display(),
        if ext.is_empty() { "" } else { "." },
        ext.display(),
    ));

    fs::write(new_path, lines)?;
    println!("write file after remove lines, path: {:?}", path.display());

    Ok(())
}

fn contains_keyword(line: &str, filters: &[String]) -> bool {
    filters.iter().any(|s| line.contains(s))
}

fn filter_keyword(line: &str, filters: &[String]) -> bool {
    filters.iter().all(|s| !line.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_keyword() {
        let wrong_line1 = "[2026-01-06 10:22:50.306] [info] [Global]  tid: 17916, start: 0x7ff93b051b70, (thread 17916 not found), create time: 72130383";
        let wrong_line2 = "[2026-01-06 10:29:10.765] [info] [Global]  cpu usage: 5.83%, memory usage: 0.35%, total: 65301.08MB, used: 230.32MB";
        let wrong_line3 =
            "[2026-01-06 10:29:10.792] [info] [Global]  pid: 12992, total threads: 59";

        let right_line1 =
            "[2026-01-06 10:29:09.814] [info] [ModelServer]  generateAllGltfModel called";
        let right_line2 =
            "[2026-01-06 10:29:10.765] [error] [Global]  exception callback: ERRCODE_MSOPTIMEOUT";
        let right_line3 =
            "[2026-01-06 11:37:24.511] [info] [ModelServer]  GET:/api/model/path from 172.24.25.2";

        let filters = &[
            "tid:".to_string(),
            "pid:".to_string(),
            "cpu usage".to_string(),
        ];

        assert_eq!(filter_keyword(wrong_line1, filters), false);
        assert_eq!(filter_keyword(wrong_line2, filters), false);
        assert_eq!(filter_keyword(wrong_line3, filters), false);
        assert_eq!(filter_keyword(right_line1, filters), true);
        assert_eq!(filter_keyword(right_line2, filters), true);
        assert_eq!(filter_keyword(right_line3, filters), true);

        assert_eq!(
            filter_keyword(wrong_line1, filters),
            !contains_keyword(wrong_line1, filters)
        );
        assert_eq!(
            filter_keyword(wrong_line2, filters),
            !contains_keyword(wrong_line2, filters)
        );
        assert_eq!(
            filter_keyword(wrong_line3, filters),
            !contains_keyword(wrong_line3, filters)
        );
        assert_eq!(
            filter_keyword(right_line1, filters),
            !contains_keyword(right_line1, filters)
        );
        assert_eq!(
            filter_keyword(right_line2, filters),
            !contains_keyword(right_line2, filters)
        );
        assert_eq!(
            filter_keyword(right_line3, filters),
            !contains_keyword(right_line3, filters)
        );
    }
}
