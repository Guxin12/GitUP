use std::env;
use std::fs;
use std::path::Path;
use crate::config::Config;
use crate::utils::{
    prompt, ask_yes_no, run_git, run_git_output,
    get_default_branch, is_git_repo, exe_dir,
};

// ---------- 配置初始化（手动） ----------
pub fn init_config(config: &mut Config, config_path: &Path) -> Result<(), String> {
    println!("=== 初始化 Git 配置 ===");
    let exe_dir = exe_dir();

    println!("当前目录结构（{}）:", exe_dir.display());
    if let Ok(entries) = fs::read_dir(&exe_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    println!("  {}", name);
                }
            }
        }
    }

    let default_dir = &config.dir;
    let dir_input = prompt("请输入内核源码目录名（相对于脚本所在目录）", default_dir);
    if dir_input == "exit" { return Err("用户取消配置".to_string()); }
    let target_dir = exe_dir.join(&dir_input);
    if !target_dir.exists() {
        println!("⚠️ 目录不存在: {}", target_dir.display());
        if ask_yes_no("是否创建此目录？", false) {
            fs::create_dir_all(&target_dir).map_err(|e| format!("创建目录失败: {}", e))?;
            println!("✔ 目录已创建");
        } else {
            return Err("目录不存在，配置终止".to_string());
        }
    }
    config.dir = dir_input;

    env::set_current_dir(&target_dir).map_err(|e| format!("切换到目录失败: {}", e))?;

    if !target_dir.join(".git").exists() {
        if ask_yes_no("当前目录未初始化 Git 仓库，是否初始化？", true) {
            run_git(&["init"])?;
            println!("✔ Git 仓库已初始化");
        } else {
            println!("⚠️ 未初始化 Git 仓库，后续操作可能失败");
        }
    }

    config.git_name = prompt("Git user.name", &config.git_name);
    config.git_email = prompt("Git user.email", &config.git_email);
    config.git_remote_name = prompt("远程仓库名（如 github / upstream）", &config.git_remote_name);
    config.git_remote_repo = prompt("GitHub 仓库（username/repository-name）", &config.git_remote_repo);
    config.git_branch = prompt("默认分支名", &config.git_branch);

    config.save(config_path)?;
    println!("✔ 配置已保存到 {}", config_path.display());
    Ok(())
}

// ---------- 克隆仓库 ----------
pub fn do_clone(config: &mut Config, config_path: &Path) -> Result<(), String> {
    println!("=== 克隆远程仓库 ===");
    let exe_dir = exe_dir();

    let clone_in_script = ask_yes_no("是否在脚本所在目录下克隆？", true);
    let clone_base_dir = if clone_in_script {
        exe_dir.clone()
    } else {
        let custom = prompt("请输入克隆目录的完整路径", "");
        if custom.is_empty() { return Err("克隆目录不能为空".to_string()); }
        let p = Path::new(&custom).to_path_buf();
        if !p.exists() {
            if ask_yes_no("目录不存在，是否创建？", true) {
                fs::create_dir_all(&p).map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                return Err("目录不存在且用户选择不创建".to_string());
            }
        }
        p
    };

    let remote_repo = prompt("请输入远程仓库（格式：username/repository-name）", "");
    if remote_repo.is_empty() { return Err("远程仓库不能为空".to_string()); }
    if !remote_repo.contains('/') { return Err("远程仓库格式错误，应为 username/repository-name".to_string()); }

    let clone_branch = prompt("请输入分支名称（留空则使用默认分支）", "");

    println!("请选择克隆方式:");
    println!("1) 完全拉取（完整克隆所有历史和文件）");
    println!("2) 只拉取最新提交（浅克隆，深度为1，更快）");
    let clone_type = prompt("请选择 (1/2)", "1");

    let repo_name = remote_repo.split('/').last().unwrap_or("repo");
    let clone_url = format!("git@github.com:{}.git", remote_repo);
    let target_dir = clone_base_dir.join(repo_name);

    println!("\n=== 克隆信息 ===");
    println!("仓库地址: {}", clone_url);
    println!("分支: {}", if clone_branch.is_empty() { "默认分支" } else { &clone_branch });
    println!("克隆方式: {}", if clone_type == "1" { "完全拉取" } else { "只拉取最新提交" });
    println!("目标目录: {}", target_dir.display());

    if !ask_yes_no("确认开始克隆？", true) {
        println!("取消克隆操作");
        return Ok(());
    }

    env::set_current_dir(&clone_base_dir).map_err(|e| format!("切换到克隆基目录失败: {}", e))?;
    let mut args = vec!["clone"];
    if !clone_branch.is_empty() {
        args.push("-b");
        args.push(&clone_branch);
    }
    if clone_type == "2" {
        args.push("--depth");
        args.push("1");
    }
    args.push(&clone_url);
    println!("执行: git {}", args.join(" "));
    let status = std::process::Command::new("git").args(&args).status()
        .map_err(|e| format!("执行 git clone 失败: {}", e))?;
    if !status.success() { return Err("git clone 执行失败".to_string()); }
    println!("✔ 仓库克隆成功");

    if !target_dir.exists() {
        return Err(format!("克隆成功但未找到预期的目录: {}", target_dir.display()));
    }

    if ask_yes_no("是否切换到新克隆的仓库并更新配置？", true) {
        let rel_dir = target_dir.strip_prefix(&exe_dir)
            .map(|p| p.to_str().unwrap_or(repo_name))
            .unwrap_or(repo_name)
            .to_string();
        config.dir = rel_dir;
        config.git_remote_repo = remote_repo.clone();
        if !clone_branch.is_empty() {
            config.git_branch = clone_branch.clone();
        } else {
            env::set_current_dir(&target_dir).map_err(|e| format!("切换到克隆目录失败: {}", e))?;
            config.git_branch = get_default_branch();
        }
        config.save(config_path)?;
        println!("配置已更新:");
        println!("  内核目录: {}", config.dir);
        println!("  远程仓库: {}", config.git_remote_repo);
        println!("  分支: {}", config.git_branch);
        env::set_current_dir(&target_dir).map_err(|e| format!("切换到克隆目录失败: {}", e))?;
        println!("已切换到新克隆的仓库: {}", target_dir.display());
    } else {
        println!("仓库已克隆，但未切换目录");
    }
    Ok(())
}

// ---------- Git 配置应用与远程设置 ----------
pub fn apply_git_config(config: &Config) -> Result<(), String> {
    if !config.git_name.is_empty() {
        run_git(&["config", "user.name", &config.git_name])?;
    }
    if !config.git_email.is_empty() {
        run_git(&["config", "user.email", &config.git_email])?;
    }
    Ok(())
}

pub fn setup_remote(config: &Config) -> Result<(), String> {
    if config.git_remote_repo.is_empty() || config.git_remote_name.is_empty() {
        return Err("远程仓库未配置".to_string());
    }
    let url = format!("git@github.com:{}.git", config.git_remote_repo);
    let remotes = run_git_output(&["remote"])?;
    if remotes.lines().any(|r| r == config.git_remote_name) {
        run_git(&["remote", "set-url", &config.git_remote_name, &url])?;
    } else {
        run_git(&["remote", "add", &config.git_remote_name, &url])?;
    }
    Ok(())
}

// ---------- 分支切换/创建 ----------
pub fn checkout_branch(branch: &str) -> Result<(), String> {
    if !is_git_repo() { return Err("不是 Git 仓库".to_string()); }
    let exists = std::process::Command::new("git")
        .args(["show-ref", "--verify", &format!("refs/heads/{}", branch)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if exists {
        run_git(&["checkout", branch])?;
    } else {
        println!("⚠️ 分支 '{}' 不存在，正在创建...", branch);
        run_git(&["checkout", "-b", branch])?;
        println!("✔ 分支 '{}' 已创建", branch);
    }
    Ok(())
}

// ---------- 提交（支持换行） ----------
pub fn do_commit() -> Result<(), String> {
    let subject = prompt("Commit 标题（subject）", "");
    if subject.is_empty() { return Err("提交标题不能为空".to_string()); }
    let body_raw = prompt("Commit 内容（使用 \\ 表示换行，可留空）", "");
    let body = body_raw.replace("\\", "\n");

    run_git(&["add", "."])?;
    if !body.is_empty() {
        run_git(&["commit", "-s", "-m", &subject, "-m", &body])?;
    } else {
        run_git(&["commit", "-s", "-m", &subject])?;
    }
    Ok(())
}

// ---------- 推送（通用） ----------
pub fn do_push(push_message: &str, config: &Config) -> Result<(), String> {
    let force = ask_yes_no("是否启用强制推送？", false);
    if !ask_yes_no(push_message, true) {
        println!("跳过推送");
        return Ok(());
    }
    if force {
        if !ask_yes_no("确认继续？（强制推送 --force-with-lease）", false) {
            return Ok(());
        }
        run_git(&["push", &config.git_remote_name, &config.git_branch, "--force-with-lease"])?;
    } else {
        run_git(&["push", &config.git_remote_name, &config.git_branch])?;
    }
    Ok(())
}

// ---------- 直接上传（无提交信息） ----------
pub fn do_push_direct(config: &Config) -> Result<(), String> {
    println!("=== 直接上传本地提交 ===");
    let status = run_git_output(&["status", "--porcelain"])?;
    if !status.is_empty() {
        println!("⚠️ 检测到未提交的更改");
        if ask_yes_no("是否现在提交更改？", true) {
            do_commit()?;
        } else {
            return Err("未提交更改，无法上传".to_string());
        }
    }
    let remote_ref = format!("{}/{}", config.git_remote_name, config.git_branch);
    let unpushed = run_git_output(&["rev-list", "--count", &format!("{}..HEAD", remote_ref)])
        .unwrap_or_else(|_| "0".to_string());
    let count: i32 = unpushed.trim().parse().unwrap_or(0);
    if count == 0 {
        println!("没有需要上传的本地提交");
        return Ok(());
    }
    println!("有 {} 个本地提交需要上传", count);
    do_push("确认直接上传？", config)
}

// ---------- 更新源码（拉取） ----------
pub fn do_update(config: &Config) -> Result<(), String> {
    println!("=== 更新源码 ===");
    if !is_git_repo() { return Err("当前目录不是 Git 仓库".to_string()); }
    let current_branch = get_default_branch();
    println!("当前分支: {}", current_branch);
    println!("远程仓库: {}", config.git_remote_name);
    println!("远程分支: {}", config.git_branch);

    println!("\n=== 更新前状态 ===");
    println!("本地提交记录（最近5个）:");
    let log = run_git_output(&["log", "--oneline", "-5"])?;
    println!("----------------------------------------");
    println!("{}", log);
    println!("----------------------------------------");

    let status = run_git_output(&["status", "--porcelain"])?;
    let mut stashed = false;
    if !status.is_empty() {
        println!("⚠️ 检测到未提交的更改:");
        println!("{}", status);
        println!("\n请选择处理未提交更改的方式:");
        println!("1) 暂存更改后更新（推荐）");
        println!("2) 丢弃所有更改后更新（危险！）");
        println!("3) 取消更新");
        let choice = prompt("请选择 (1/2/3)", "1");
        match choice.as_str() {
            "1" => {
                run_git(&["stash"])?;
                stashed = true;
                println!("已暂存更改");
            }
            "2" => {
                if ask_yes_no("确认要丢弃所有更改？(yes/NO)", false) {
                    run_git(&["reset", "--hard", "HEAD"])?;
                    println!("已丢弃所有更改");
                } else {
                    return Err("取消更新".to_string());
                }
            }
            _ => return Err("取消更新".to_string()),
        }
    }

    println!("\n请选择更新方式:");
    println!("1) 拉取并合并（git pull）");
    println!("2) 拉取但不自动合并（git fetch + 手动合并）");
    println!("3) 变基方式拉取（git pull --rebase）");
    let update_type = prompt("请选择 (1/2/3) [1]", "1");

    println!("\n正在获取远程更新...");
    run_git(&["fetch", &config.git_remote_name])?;
    println!("远程更新获取成功");

    // 显示新提交
    let remote_ref = format!("{}/{}", config.git_remote_name, config.git_branch);
    let new_log = run_git_output(&["log", "--oneline", &format!("{}..{}", current_branch, remote_ref)])
        .unwrap_or_else(|_| "（无新提交或分支不存在）".to_string());
    println!("\n远程分支 {}/{} 的新提交:", config.git_remote_name, config.git_branch);
    println!("----------------------------------------");
    println!("{}", new_log);
    let count_cmd = run_git_output(&["rev-list", "--count", &format!("{}..{}", current_branch, remote_ref)])
        .unwrap_or_else(|_| "0".to_string());
    println!("共有 {} 个新提交", count_cmd.trim());
    println!("----------------------------------------");

    if !ask_yes_no("是否继续更新？", true) {
        if stashed { run_git(&["stash", "pop"])?; }
        return Ok(());
    }

    match update_type.as_str() {
        "1" => {
            if let Err(e) = run_git(&["pull", &config.git_remote_name, &config.git_branch]) {
                println!("❌ 拉取合并过程中出现冲突");
                if stashed { println!("注意：您有暂存的更改，使用 'git stash pop' 恢复"); }
                return Err(e);
            }
            println!("✔ 拉取并合并成功");
        }
        "2" => {
            println!("执行拉取但不自动合并 (git fetch)...");
            println!("远程更新已获取到本地，您可以使用以下命令手动合并:");
            println!("  git merge {}/{}", config.git_remote_name, config.git_branch);
            println!("或");
            println!("  git rebase {}/{}", config.git_remote_name, config.git_branch);
        }
        "3" => {
            if let Err(e) = run_git(&["pull", "--rebase", &config.git_remote_name, &config.git_branch]) {
                println!("❌ 变基过程中出现冲突");
                println!("请手动解决冲突后继续:");
                println!("  解决冲突后: git add .");
                println!("  继续变基: git rebase --continue");
                println!("  取消变基: git rebase --abort");
                if stashed { println!("注意：您有暂存的更改，使用 'git stash pop' 恢复"); }
                return Err(e);
            }
            println!("✔ 变基拉取成功");
        }
        _ => return Err("无效的更新方式".to_string()),
    }

    if stashed && update_type != "2" {
        if ask_yes_no("是否恢复之前暂存的更改？", true) {
            if let Err(e) = run_git(&["stash", "pop"]) {
                println!("❌ 恢复暂存更改时出现冲突，请手动解决");
                return Err(e);
            }
            println!("✔ 已恢复暂存的更改");
        } else {
            println!("暂存的更改仍然保存在 stash 中");
        }
    }

    // 显示更新后状态
    let new_log = run_git_output(&["log", "--oneline", "-5"])?;
    println!("\n=== 更新后状态 ===");
    println!("最新提交记录:");
    println!("----------------------------------------");
    println!("{}", new_log);
    println!("----------------------------------------");

    let unpushed = run_git_output(&["rev-list", "--count", &format!("{}..HEAD", remote_ref)])
        .unwrap_or_else(|_| "0".to_string());
    let count: i32 = unpushed.trim().parse().unwrap_or(0);
    if count > 0 {
        println!("有 {} 个本地提交未推送到远程", count);
        if ask_yes_no("是否现在推送？", true) {
            do_push("是否现在推送本地提交？", config)?;
        }
    } else {
        println!("本地分支与远程分支一致");
    }
    println!("更新完成");
    Ok(())
}

// ---------- 从远程仓库合并提交（支持拉取全部或最新） ----------
pub fn merge_from_remote(config: &Config) -> Result<(), String> {
    println!("=== 从其他仓库分支合并提交 ===");
    println!("请选择远程仓库类型:");
    println!("1) GitHub 仓库");
    println!("2) 外部链接仓库");
    let remote_type = prompt("请选择 (1/2)，按Enter返回菜单", "");
    if remote_type.is_empty() { return Ok(()); }

    let (remote_url, remote_branch) = match remote_type.as_str() {
        "1" => {
            let repo = prompt("请输入GitHub仓库（格式：username/repository-name，按Enter返回菜单）", "");
            if repo.is_empty() { return Ok(()); }
            let branch = prompt("请输入分支名称（按Enter返回菜单）", "");
            if branch.is_empty() { return Ok(()); }
            (format!("git@github.com:{}.git", repo), branch)
        }
        "2" => {
            let url = prompt("请输入外部仓库链接（按Enter返回菜单）", "");
            if url.is_empty() { return Ok(()); }
            let branch = prompt("请输入分支名称（按Enter返回菜单）", "");
            if branch.is_empty() { return Ok(()); }
            (url, branch)
        }
        _ => return Err("无效的选择".to_string()),
    };

    let temp_remote = format!("temp_merge_remote_{}", std::process::id());
    // 清理可能存在的同名远程
    let _ = run_git(&["remote", "remove", &temp_remote]);
    run_git(&["remote", "add", &temp_remote, &remote_url])?;

    // ---------- 新增：选择拉取方式 ----------
    println!("\n请选择拉取方式:");
    println!("1) 拉取全部提交（完整历史）");
    println!("2) 只拉取最新提交（指定数量）");
    let fetch_mode = prompt("请选择 (1/2)", "1");
    let depth_opt = if fetch_mode == "2" {
        let depth = prompt("请输入要拉取的最新提交数量（例如 50）", "50");
        Some(depth)
    } else {
        None
    };

    // 获取远程分支
    println!("正在获取远程分支 {} 的提交...", remote_branch);
    let mut fetch_args = vec!["fetch", "--no-tags", &temp_remote, &remote_branch];
    if let Some(depth) = &depth_opt {
        fetch_args.push("--depth");
        fetch_args.push(depth);
        println!("（浅拉取，深度为 {}）", depth);
    } else {
        println!("（拉取全部提交）");
    }
    if let Err(e) = run_git(&fetch_args) {
        run_git(&["remote", "remove", &temp_remote])?;
        return Err(format!("无法获取远程分支: {}", e));
    }
    println!("✔ 远程分支获取成功");

    // 选择合并方式
    println!("请选择合并方式:");
    println!("1) 单个提交合并（通过SHA）");
    println!("2) 多个提交合并");
    let merge_type = prompt("请选择 (1/2)", "1");

    if merge_type == "1" {
        let sha = prompt("请输入要合并的提交SHA", "");
        if sha.is_empty() { run_git(&["remote", "remove", &temp_remote])?; return Ok(()); }
        if ask_yes_no("是否合并此提交到本地？", true) {
            if let Err(e) = run_git(&["cherry-pick", "-s", &sha]) {
                println!("❌ 合并冲突，请手动解决");
                run_git(&["remote", "remove", &temp_remote])?;
                return Err(e);
            }
            println!("✔ 提交合并成功");
            if ask_yes_no("是否现在推送合并的提交？", true) {
                do_push("是否现在推送合并的提交？", config)?;
            }
        }
    } else {
        // 多提交合并 - 获取提交列表
        let remote_ref = format!("{}/{}", temp_remote, remote_branch);
        // 使用 --reverse 从旧到新
        let commits_str = run_git_output(&["log", &remote_ref, "--format=%H|%s", "--reverse"])?;
        let mut commits: Vec<(String, String)> = Vec::new();
        for line in commits_str.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                commits.push((parts[0].to_string(), parts[1].to_string()));
            }
        }
        if commits.is_empty() {
            println!("该分支没有提交");
            run_git(&["remote", "remove", &temp_remote])?;
            return Ok(());
        }
        println!("共找到 {} 个提交", commits.len());
        // 显示前10个和后10个
        let show_count = if commits.len() > 20 { 10 } else { commits.len() };
        for (i, (sha, msg)) in commits.iter().take(show_count).enumerate() {
            println!("{}: {} - {}", i+1, &sha[0..8], msg);
        }
        if commits.len() > 20 {
            println!("... 中间省略 ...");
            let start = commits.len() - 10;
            for (i, (sha, msg)) in commits.iter().enumerate().skip(start) {
                println!("{}: {} - {}", i+1, &sha[0..8], msg);
            }
        }

        // 选择起始和结束序号
        let start_idx: usize = prompt("请输入起始序号（1开始）", "1").parse().unwrap_or(1);
        let end_idx: usize = prompt("请输入结束序号（包含）", &commits.len().to_string()).parse().unwrap_or(commits.len());
        if start_idx < 1 || end_idx > commits.len() || start_idx > end_idx {
            run_git(&["remote", "remove", &temp_remote])?;
            return Err("序号无效".to_string());
        }
        let selected: Vec<&str> = commits[start_idx-1..end_idx].iter().map(|(s,_)| s.as_str()).collect();
        if ask_yes_no(&format!("是否合并这 {} 个提交到本地？", selected.len()), true) {
            // 批量 cherry-pick
            let mut args = vec!["cherry-pick", "-s"];
            args.extend(selected);
            if let Err(e) = run_git(&args) {
                println!("❌ 合并过程中出现冲突，请手动解决");
                run_git(&["remote", "remove", &temp_remote])?;
                return Err(e);
            }
            println!("✔ 批量合并成功");
            if ask_yes_no("是否现在推送合并的提交？", true) {
                do_push("是否现在推送合并的提交？", config)?;
            }
        }
    }
    run_git(&["remote", "remove", &temp_remote])?;
    Ok(())
}

// ---------- 交互式查看提交日志（支持自定义每页条数） ----------
pub fn view_log_interactive() -> Result<(), String> {
    println!("=== 交互式提交日志查看 ===");
    if !is_git_repo() { return Err("当前目录不是 Git 仓库".to_string()); }

    // 获取提交总数
    let total = run_git_output(&["rev-list", "--count", "HEAD"])
        .unwrap_or_else(|_| "0".to_string());
    let total: usize = total.trim().parse().unwrap_or(0);
    if total == 0 {
        println!("仓库中没有提交记录");
        return Ok(());
    }

    // 默认每页 20 条，用户可随时修改
    let mut page_size: usize = 20;
    let mut current_page = 1;

    // 获取所有提交信息（SHA|标题|作者|日期|正文）
    let log_output = run_git_output(&[
        "log",
        "--format=%H|%s|%an|%ad|%b",
        "--date=iso",
    ])?;

    let mut commits: Vec<(String, String, String, String, String)> = Vec::new();
    for line in log_output.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() == 5 {
            commits.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
                parts[4].to_string(),
            ));
        }
    }

    // 计算总页数的辅助函数
    let total_pages = |size: usize| (commits.len() + size - 1) / size;

    loop {
        let total_p = total_pages(page_size);
        // 如果当前页超出范围，自动修正到最后一页
        if current_page > total_p && total_p > 0 {
            current_page = total_p;
        }
        let start = (current_page - 1) * page_size;
        let end = std::cmp::min(start + page_size, commits.len());

        println!("\n=== 提交日志（第 {}/{} 页，共 {} 个提交，每页 {} 条，最新在前）===",
            current_page, total_p, commits.len(), page_size);
        println!("序号 | SHA       | 作者       | 日期                 | 提交信息");
        println!("--------------------------------------------------------------------------------");

        for (i, (sha, subject, author, date, _)) in commits.iter().enumerate().skip(start).take(end - start) {
            let num = i + 1;
            let sha_short = &sha[0..8];
            let author_short = if author.len() > 12 { &author[0..12] } else { author };
            let date_short = if date.len() > 16 { &date[0..16] } else { date };
            let subject_short = if subject.len() > 40 { &subject[0..37] } else { subject };
            println!("{:<4} | {:<8} | {:<12} | {:<18} | {}", num, sha_short, author_short, date_short, subject_short);
        }

        println!("\n=== 操作选项 ===");
        println!("  n) 下一页");
        println!("  p) 上一页");
        println!("  g) 跳转到指定页 (1-{})", total_p);
        println!("  s) 设置每页条数 (当前 {})", page_size);
        println!("  v) 查看指定提交的详细信息 (输入序号)");
        println!("  q) 退出查看");
        let choice = prompt("请选择", "");

        match choice.to_lowercase().as_str() {
            "n" => {
                if current_page < total_p {
                    current_page += 1;
                } else {
                    println!("已是最后一页");
                }
            }
            "p" => {
                if current_page > 1 {
                    current_page -= 1;
                } else {
                    println!("已是第一页");
                }
            }
            "g" => {
                let page = prompt(&format!("请输入要跳转的页码 (1-{})", total_p), "1");
                if let Ok(p) = page.parse::<usize>() {
                    if p >= 1 && p <= total_p {
                        current_page = p;
                    } else {
                        println!("页码无效（1-{}）", total_p);
                    }
                } else {
                    println!("无效的页码");
                }
            }
            "s" => {
                let input = prompt("请输入每页条数", &page_size.to_string());
                if let Ok(new_size) = input.parse::<usize>() {
                    if new_size > 0 {
                        page_size = new_size;
                        current_page = 1; // 重置到第一页，避免超出
                        println!("每页条数已设置为 {}", page_size);
                    } else {
                        println!("每页条数必须大于0");
                    }
                } else {
                    println!("请输入有效的数字");
                }
            }
            "v" => {
                let num = prompt("请输入要查看详细信息的提交序号", "");
                if let Ok(idx) = num.parse::<usize>() {
                    if idx >= 1 && idx <= commits.len() {
                        let (sha, subject, author, date, body) = &commits[idx-1];
                        println!("\n=== 提交 #{} 的详细信息 ===", idx);
                        println!("提交哈希: {}", sha);
                        println!("作者: {}", author);
                        println!("日期: {}", date);
                        println!("标题: {}", subject);
                        println!("内容:");
                        if body.is_empty() {
                            println!("  （无正文）");
                        } else {
                            for line in body.lines() {
                                println!("  {}", line);
                            }
                        }
                        println!("--------------------------------------------------------------------------------");
                        prompt("按Enter键继续...", "");
                    } else {
                        println!("序号无效（1-{}）", commits.len());
                    }
                } else {
                    println!("请输入有效的数字");
                }
            }
            "q" => break,
            _ => println!("无效选项，请重新输入"),
        }
    }
    Ok(())
}

// ---------- 撤销指定提交 ----------
pub fn do_revert() -> Result<(), String> {
    println!("最近的提交记录：");
    let log = run_git_output(&["log", "--oneline", "-10"])?;
    println!("----------------------------------------");
    println!("{}", log);
    println!("----------------------------------------");
    let hash = prompt("请输入要撤销的提交哈希值（输入 'cancel' 取消）", "");
    if hash == "cancel" { println!("取消撤销操作"); return Ok(()); }
    if hash.is_empty() { return Err("提交哈希值不能为空".to_string()); }
    let _ = run_git(&["show", "--quiet", &hash])?;
    if ask_yes_no(&format!("确认要撤销提交 {} 吗？", hash), false) {
        if let Err(e) = run_git(&["revert", "--no-edit", &hash]) {
            println!("❌ 撤销过程中出现冲突，请手动解决后提交");
            return Err(e);
        }
        println!("✔ 提交已成功撤销");
        // 注意：推送需要 config，但此函数无 config，可返回成功，推送由调用者处理
    }
    Ok(())
}

// ---------- 删除本地提交（回退） ----------
pub fn do_reset() -> Result<(), String> {
    println!("最近的提交记录：");
    let log = run_git_output(&["log", "--oneline", "-15"])?;
    println!("----------------------------------------");
    println!("{}", log);
    println!("----------------------------------------");
    let hash = prompt("请输入要回退到的提交哈希值（输入 'cancel' 取消）", "");
    if hash == "cancel" { println!("取消回退操作"); return Ok(()); }
    if hash.is_empty() { return Err("提交哈希值不能为空".to_string()); }
    let _ = run_git(&["show", "--quiet", &hash])?;
    println!("请选择回退方式：");
    println!("1) soft - 回退提交但保留更改在工作区");
    println!("2) mixed - 回退提交且将更改放入暂存区（默认）");
    println!("3) hard - 回退提交并丢弃所有更改（危险！）");
    let reset_type = prompt("选择回退方式 (1/2/3) [2]", "2");
    let reset_option = match reset_type.as_str() {
        "1" => "--soft",
        "3" => "--hard",
        _ => "--mixed",
    };
    if reset_option == "--hard" && !ask_yes_no("确认使用 --hard 模式？(yes/NO)", false) {
        return Ok(());
    }
    println!("将要删除的提交（从当前HEAD到{}之间的提交）：", hash);
    let log = run_git_output(&["log", "--oneline", &format!("{}..HEAD", hash)])?;
    println!("----------------------------------------");
    println!("{}", log);
    println!("----------------------------------------");
    if !ask_yes_no(&format!("确认要回退到提交 {} 吗？", hash), false) {
        return Ok(());
    }
    run_git(&["reset", reset_option, &hash])?;
    println!("✔ 已成功回退到提交 {}", hash);
    if reset_option == "--hard" {
        println!("⚠️ 注意：所有更改已被丢弃，无法恢复！");
    }
    // 提示强制推送，但需要 config，此处省略，由调用者处理
    Ok(())
}