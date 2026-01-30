use std::{fs, path::Path};

use anyhow::{Ok, Result, anyhow};
use clap::{Parser, Subcommand};
use rust_xlsxwriter::workbook::Workbook;
use subcommand::{
    BaseDirArgs, CheckLineArgs, RemoveFileArgs, RemoveLineArgs, get_base_dir, process_check_line,
    process_remove_file, process_remove_line, set_base_dir,
};

mod subcommand;

#[derive(Parser)]
#[command(name = "lp", version, about = "简单日志处理工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// 子命令集合
#[derive(Subcommand)]
enum Commands {
    /// 设置要操作文件的根路径
    #[command(name = "sbd", alias = "set_bd")]
    SetBaseDir(BaseDirArgs),

    /// 获取当前的根路径
    #[command(name = "gbd", alias = "get_bd")]
    GetBaseDir,

    /// 检查日志内容
    #[command(name = "cl", alias = "cl_ln")]
    CheckLine(CheckLineArgs),

    /// 移除日志中带有特定信息的内容
    #[command(name = "rl", alias = "rm_ln")]
    RemoveLine(RemoveLineArgs),

    /// 删除文件
    #[command(name = "rf", alias = "rm_f")]
    RemoveFile(RemoveFileArgs),
}

fn main() -> Result<()> {
    // // let path = "E:/project/select_direction/1234 - 副本.log";
    // let path = "E:/project/select_direction/23.log";
    // split_log_to_excel(path)?;

    let args = Cli::parse();
    match args.command {
        Commands::SetBaseDir(args) => {
            set_base_dir(args)?;
        }
        Commands::GetBaseDir => {
            println!("{}", get_base_dir()?.path.display());
        }
        Commands::CheckLine(args) => {
            process_check_line(args)?;
        }
        Commands::RemoveLine(args) => {
            process_remove_line(args)?;
        }
        Commands::RemoveFile(args) => {
            process_remove_file(args)?;
        }
    }

    Ok(())
}

fn split_log_to_excel<P: AsRef<Path>>(path: P) -> Result<()> {
    let line = fs::read_to_string(&path)?;
    let mut east_str = String::new();
    let mut west_str = String::new();

    let mut east_data = Vec::new();
    let mut west_data = Vec::new();

    for line in line.lines() {
        if line.contains("East") {
            east_str.push_str(line);
            east_str.push('\n');

            east_data.push(line);
        } else if line.contains("West") {
            west_str.push_str(line);
            west_str.push('\n');

            west_data.push(line);
        }
    }

    fs::write("east.log", east_str)?;
    fs::write("west.log", west_str)?;

    write_to_xlsx(&east_data, "east.xlsx")?;
    write_to_xlsx(&west_data, "west.xlsx")?;

    Ok(())
}

fn write_to_xlsx<P: AsRef<Path>>(lines: &[&str], path: P) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

    for (row, &line) in lines.iter().enumerate() {
        let mut parts = line.split(']');
        let mut time = parts
            .next()
            .ok_or_else(|| anyhow!("line should contain time"))?;
        if time.starts_with('[') {
            time = time.strip_prefix('[').unwrap();
        }

        let other = parts
            .next()
            .ok_or_else(|| anyhow!("line should contain other Lines"))?;
        let other_parts = other.split_whitespace();

        let row = row as u32;
        ws.write_string(row, 0, time)?;
        for (col, part) in other_parts.enumerate() {
            ws.write_string(row, (col + 1) as u16, part)?;
        }
    }

    wb.save(path)?;

    Ok(())
}
