use std::env;
use std::path::Path;
use crate::config::Config;
use crate::utils::{prompt, exe_dir, ask_yes_no, run_git};
use crate::git_ops::{
    init_config, do_clone, do_commit, do_push, do_push_direct,
    do_update, merge_from_remote, view_log_interactive,
    do_revert, do_reset, checkout_branch,
    apply_git_config, setup_remote,
};

pub fn menu_loop(config_path: &Path) {
    loop {
        // 每次循环重新加载配置，避免未使用变量警告
        let config = Config::load(config_path);
        // 切换目录
        let exe_dir = exe_dir();
        let target_dir = exe_dir.join(&config.dir);
        if target_dir.exists() {
            let _ = env::set_current_dir(&target_dir);
        }

        println!("\n当前目录: {}", env::current_dir().unwrap_or(std::path::PathBuf::from(".")).display());
        println!("====== GitUp 菜单 ======");
        println!("1) 提交并推送");
        println!("2) 创建 / 切换分支");
        println!("3) 克隆仓库");
        println!("4) 从远程仓库合并提交");
        println!("5) 更新源码");
        println!("6) 修改配置");
        println!("7) 撤销指定提交（安全）");
        println!("8) 删除本地提交（回退）");
        println!("9) 查看提交日志（交互式）");
        println!("10) 直接上传（无需提交信息）");
        println!("11) 退出");
        let choice = prompt("请选择", "0");

        match choice.as_str() {
            "1" => {
                let _ = apply_git_config(&config);
                let _ = setup_remote(&config);
                if let Err(e) = do_commit() {
                    eprintln!("❌ 提交失败: {}", e);
                } else {
                    let _ = do_push("是否现在开始推送？", &config);
                }
            }
            "2" => {
                if let Err(e) = checkout_branch(&config.git_branch) {
                    eprintln!("❌ 切换分支失败: {}", e);
                }
            }
            "3" => {
                let mut config_clone = config.clone();
                if let Err(e) = do_clone(&mut config_clone, config_path) {
                    eprintln!("❌ 克隆失败: {}", e);
                }
            }
            "4" => {
                if let Err(e) = merge_from_remote(&config) {
                    eprintln!("❌ 合并失败: {}", e);
                }
            }
            "5" => {
                if let Err(e) = do_update(&config) {
                    eprintln!("❌ 更新失败: {}", e);
                }
            }
            "6" => {
                let mut config_clone = config.clone();
                if let Err(e) = init_config(&mut config_clone, config_path) {
                    eprintln!("❌ 配置失败: {}", e);
                } else {
                    println!("配置已更新");
                }
            }
            "7" => {
                if let Err(e) = do_revert() {
                    eprintln!("❌ 撤销失败: {}", e);
                } else if ask_yes_no("是否现在推送撤销更改？", true) {
                    let _ = do_push("是否现在推送撤销更改？", &config);
                }
            }
            "8" => {
                if let Err(e) = do_reset() {
                    eprintln!("❌ 回退失败: {}", e);
                } else if ask_yes_no("是否要强制推送到远程仓库？", false) {
                    if ask_yes_no("确认强制推送？(yes/NO)", false) {
                        let _ = run_git(&["push", &config.git_remote_name, &config.git_branch, "--force"]);
                    }
                }
            }
            "9" => {
                if let Err(e) = view_log_interactive() {
                    eprintln!("❌ 查看日志失败: {}", e);
                }
            }
            "10" => {
                let _ = apply_git_config(&config);
                let _ = setup_remote(&config);
                if let Err(e) = do_push_direct(&config) {
                    eprintln!("❌ 直接上传失败: {}", e);
                }
            }
            "11" => {
                println!("👋 再见！");
                break;
            }
            _ => println!("❌ 无效选项"),
        }
    }
}