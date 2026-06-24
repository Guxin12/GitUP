mod config;
mod utils;
mod git_ops;
mod menu;

use std::env;
use config::Config;
use utils::{exe_dir, prompt};               // 只保留实际使用的函数
use git_ops::{init_config, do_clone, apply_git_config, setup_remote, checkout_branch};
use menu::menu_loop;

fn main() {
    let exe_dir = exe_dir();
    let config_path = exe_dir.join("git保存内容.ini");
    println!("脚本位置: {}", exe_dir.display());

    let mut config = Config::load(&config_path);

    if !config.is_complete() {
        println!("⚠️ 尚未配置或配置不完整");
        println!("请选择初始化方式:");
        println!("1) 初始化配置（手动输入所有信息）");
        println!("2) 克隆仓库（从远程克隆并自动配置）");
        let choice = prompt("请选择 (1/2)", "1");
        let result = match choice.as_str() {
            "1" => init_config(&mut config, &config_path),
            "2" => do_clone(&mut config, &config_path),
            _ => {
                eprintln!("❌ 无效选项，退出");
                return;
            }
        };
        if let Err(e) = result {
            eprintln!("❌ 初始化失败: {}", e);
            return;
        }
        config = Config::load(&config_path);
        if !config.is_complete() {
            eprintln!("❌ 配置仍不完整，请重新运行程序配置");
            return;
        }
    }

    if !config.dir.is_empty() {
        let target_dir = exe_dir.join(&config.dir);
        if target_dir.exists() {
            if let Err(e) = env::set_current_dir(&target_dir) {
                eprintln!("❌ 无法切换到内核目录 {}: {}", target_dir.display(), e);
                return;
            }
            println!("当前工作目录: {}", target_dir.display());
        } else {
            println!("⚠️ 内核目录不存在: {}", target_dir.display());
            println!("请重新配置");
            if let Err(e) = init_config(&mut config, &config_path) {
                eprintln!("❌ 重新配置失败: {}", e);
                return;
            }
            if let Err(e) = env::set_current_dir(exe_dir.join(&config.dir)) {
                eprintln!("❌ 无法切换到新目录: {}", e);
                return;
            }
        }
    }

    if let Err(e) = apply_git_config(&config) {
        eprintln!("❌ 应用 Git 配置失败: {}", e);
    }
    if let Err(e) = setup_remote(&config) {
        eprintln!("⚠️ 设置远程仓库失败: {}", e);
    }

    if !config.git_branch.is_empty() {
        if let Err(e) = checkout_branch(&config.git_branch) {
            eprintln!("⚠️ 分支操作失败: {}", e);
        }
    }

    menu_loop(&config_path);
}