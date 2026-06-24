use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

pub fn exe_dir() -> PathBuf {
    let exe_path = env::current_exe().expect("无法获取可执行文件路径");
    exe_path.parent().expect("无法获取可执行文件目录").to_path_buf()
}

pub fn prompt(prompt: &str, default: &str) -> String {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
    let trimmed = input.trim();
    if trimmed.is_empty() { default.to_string() } else { trimmed.to_string() }
}

pub fn ask_yes_no(prompt: &str, default: bool) -> bool {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
    let trimmed = input.trim();
    if trimmed.is_empty() { return default; }
    let lower = trimmed.to_lowercase();
    lower == "y" || lower == "yes"
}

pub fn run_git(args: &[&str]) -> Result<(), String> {
    let status = Command::new("git").args(args).status()
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;
    if !status.success() {
        return Err(format!("git 命令执行失败: {:?}", args));
    }
    Ok(())
}

pub fn run_git_output(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git").args(args).output()
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;
    if !output.status.success() {
        return Err(format!("git 命令执行失败: {:?}", args));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("输出不是有效 UTF-8: {}", e))
}

pub fn get_default_branch() -> String {
    if let Ok(out) = run_git_output(&["symbolic-ref", "--short", "HEAD"]) {
        let branch = out.trim();
        if !branch.is_empty() { return branch.to_string(); }
    }
    "main".to_string()
}

pub fn is_git_repo() -> bool {
    std::path::Path::new(".git").exists()
}