extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::{print, println, vga::{self, ColorCode}};
use crate::keyboard::{self, read_char};
use crate::interrupts::uptime_seconds;
use crate::allocator::get_heap_info;
use crate::fs;

pub const ALL_COMMANDS_LIST: &[&str] = &[
    "help", "clear", "reset", "echo", "printf", "cd", "pwd", "ls", "ll", "la", "dir", "vdir",
    "mkdir", "rmdir", "touch", "cat", "tac", "write", "append", "cp", "mv", "rm", "shred",
    "chmod", "chown", "chgrp", "stat", "file", "head", "tail", "wc", "nl", "grep", "egrep", "fgrep",
    "find", "tree", "diff", "cmp", "comm", "sort", "uniq", "tr", "cut", "paste", "sed", "awk",
    "nano", "vim", "vi", "edit", "less", "more",
    "pacman", "yay", "paru", "makepkg", "pacstrap", "arch-chroot", "genfstab", "install", "rustos-install", "archinstall",
    "wm", "startx", "desktop", "compositor",
    "neofetch", "fastfetch", "hyfetch", "pfetch", "onefetch",
    "uname", "arch", "whoami", "id", "groups", "users", "who", "w", "hostname", "hostnamectl", "passwd",
    "date", "timedatectl", "cal", "uptime", "lscpu", "lspci", "lsusb", "lsblk", "lsmod", "modprobe", "insmod", "rmmod",
    "free", "df", "du", "mount", "umount", "fdisk", "parted", "mkfs", "fsck", "sync",
    "ps", "pstree", "top", "htop", "btop", "kill", "killall", "pkill", "nice", "renice", "sleep",
    "systemctl", "journalctl", "dmesg", "loginctl", "udevadm",
    "ip", "ifconfig", "ping", "traceroute", "route", "netstat", "ss", "curl", "wget", "dig", "nslookup", "host",
    "env", "export", "unset", "alias", "unalias", "which", "whereis", "type", "history", "man", "info", "apropos", "whatis", "source",
    "tar", "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz", "zstd", "unzstd", "zip", "unzip",
    "md5sum", "sha1sum", "sha256sum", "sha512sum", "base64", "xxd", "hexdump", "od",
    "basename", "dirname", "realpath", "readlink", "seq", "yes", "true", "false", "test", "expr",
    "tee", "xargs", "envsubst", "timeout", "watch",
    "cowsay", "fortune", "figlet", "lolcat", "cmatrix", "pipes-rs", "cbonsai",
    "sl", "bat", "eza", "rg", "fd", "git", "cargo", "rustc", "python",
    "tty-clock", "nyancat", "asciiquarium", "rice",
    "colors", "mem", "alloc", "calc", "disk", "format",
    "panic", "reboot", "shutdown", "poweroff", "exit",
];

const BUILTIN_COMMANDS: &[&str] = ALL_COMMANDS_LIST;

pub struct ShellState {
    pub cwd: String,
    pub prev_cwd: String,
    pub user: String,
    pub hostname: String,
    pub history: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub aliases: Vec<(String, String)>,
    pub last_status: i32,
}

impl ShellState {
    pub fn new() -> Self {
        let hostname = if let Ok(data) = fs::read_file("/etc/hostname") {
            if let Ok(s) = core::str::from_utf8(&data) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    String::from(trimmed)
                } else {
                    String::from("rustos")
                }
            } else {
                String::from("rustos")
            }
        } else {
            String::from("rustos")
        };

        let mut env_vars = Vec::new();
        env_vars.push((String::from("USER"), String::from("root")));
        env_vars.push((String::from("HOME"), String::from("/root")));
        env_vars.push((String::from("HOSTNAME"), hostname.clone()));
        env_vars.push((String::from("SHELL"), String::from("/bin/rsh")));
        env_vars.push((String::from("PATH"), String::from("/usr/local/sbin:/usr/local/bin:/usr/bin:/bin")));
        env_vars.push((String::from("TERM"), String::from("linux-vga-256color")));
        env_vars.push((String::from("EDITOR"), String::from("nano")));
        env_vars.push((String::from("LANG"), String::from("en_US.UTF-8")));
        env_vars.push((String::from("DISTRO"), String::from("RustOS (Inspired by Arch Linux)")));

        let mut aliases = Vec::new();
        aliases.push((String::from("ll"), String::from("ls -la")));
        aliases.push((String::from("la"), String::from("ls -a")));
        aliases.push((String::from("cls"), String::from("clear")));
        aliases.push((String::from("vi"), String::from("nano")));
        aliases.push((String::from("vim"), String::from("nano")));
        aliases.push((String::from("update"), String::from("pacman -Syu")));

        Self {
            cwd: String::from("/root"),
            prev_cwd: String::from("/root"),
            user: String::from("root"),
            hostname,
            history: Vec::new(),
            env_vars,
            aliases,
            last_status: 0,
        }
    }

    pub fn display_cwd(&self) -> String {
        if self.cwd == "/root" {
            String::from("~")
        } else if self.cwd.starts_with("/root/") {
            format!("~{}", &self.cwd[5..])
        } else {
            self.cwd.clone()
        }
    }

    pub fn get_env(&self, key: &str) -> Option<&str> {
        for (k, v) in &self.env_vars {
            if k == key {
                return Some(v.as_str());
            }
        }
        None
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        for (k, v) in &mut self.env_vars {
            if k == key {
                *v = String::from(val);
                return;
            }
        }
        self.env_vars.push((String::from(key), String::from(val)));
    }

    pub fn unset_env(&mut self, key: &str) {
        self.env_vars.retain(|(k, _)| k != key);
    }
}

pub fn print_prompt(state: &ShellState) {
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("[");
    if state.user == "root" {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
    } else {
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    }
    print!("{}", state.user);
    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    print!("@");
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("{}", state.hostname);
    vga::set_colors(ColorCode::White, ColorCode::Black);
    print!(" {}", state.display_cwd());
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("]");
    if state.user == "root" {
        vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
        print!("# ");
    } else {
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        print!("$ ");
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);
}

fn replace_input_line(current_input: &mut String, new_text: &str) {
    for _ in 0..current_input.chars().count() {
        vga::backspace();
    }
    current_input.clear();
    current_input.push_str(new_text);
    vga::set_colors(ColorCode::White, ColorCode::Black);
    print!("{}", current_input);
}

pub fn run_shell() -> ! {
    let mut state = ShellState::new();

    println!();
    crate::applets::cmd_neofetch(false);

    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    println!(" Welcome to RustOS (x86_64 Bare Metal in Rust)!");
    vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
    println!(" Inspired by Arch Linux: uses pacman & AUR package models, rolling userland.");
    println!(" Tip: Press [TAB] for command/package/file completion, [UP]/[DOWN] for history.");
    println!(" Try: pacman -Syu | pacman -Ss | yay -S cbonsai | neofetch | ls -la | help\n");

    let mut input_buffer = String::new();
    let mut history_index: Option<usize> = None;
    let mut saved_draft = String::new();

    print_prompt(&state);

    loop {
        let c = read_char();
        match c {
            '\n' => {
                println!();
                let cmd_line = input_buffer.trim().to_string();
                input_buffer.clear();
                history_index = None;
                saved_draft.clear();

                if !cmd_line.is_empty() {
                    if state.history.last().map(|s| s.as_str()) != Some(cmd_line.as_str()) {
                        state.history.push(cmd_line.clone());
                    }
                    execute_line(&mut state, &cmd_line);
                }
                print_prompt(&state);
            }
            '\x08' => {
                if !input_buffer.is_empty() {
                    input_buffer.pop();
                    vga::backspace();
                }
            }
            keyboard::CTRL_C => {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("^C");
                input_buffer.clear();
                history_index = None;
                print_prompt(&state);
            }
            keyboard::CTRL_L => {
                vga::clear_screen();
                print_prompt(&state);
                vga::set_colors(ColorCode::White, ColorCode::Black);
                print!("{}", input_buffer);
            }
            keyboard::CTRL_D => {
                if input_buffer.is_empty() {
                    println!("logout");
                }
            }
            keyboard::KEY_UP => {
                if !state.history.is_empty() {
                    let next_idx = match history_index {
                        None => {
                            saved_draft = input_buffer.clone();
                            state.history.len() - 1
                        }
                        Some(0) => 0,
                        Some(idx) => idx - 1,
                    };
                    history_index = Some(next_idx);
                    let entry = state.history[next_idx].clone();
                    replace_input_line(&mut input_buffer, &entry);
                }
            }
            keyboard::KEY_DOWN => {
                if let Some(idx) = history_index {
                    if idx + 1 < state.history.len() {
                        let next_idx = idx + 1;
                        history_index = Some(next_idx);
                        let entry = state.history[next_idx].clone();
                        replace_input_line(&mut input_buffer, &entry);
                    } else {
                        history_index = None;
                        let draft = saved_draft.clone();
                        replace_input_line(&mut input_buffer, &draft);
                    }
                }
            }
            '\t' => {
                handle_tab_completion(&state, &mut input_buffer);
            }
            printable => {
                if printable >= ' ' && printable <= '~' {
                    input_buffer.push(printable);
                    vga::set_colors(ColorCode::White, ColorCode::Black);
                    print!("{}", printable);
                }
            }
        }
    }
}

fn longest_common_prefix(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let first = &items[0];
    let mut prefix_len = first.len();
    for item in items.iter().skip(1) {
        let mut matched = 0;
        for (a, b) in first.chars().zip(item.chars()) {
            if a == b {
                matched += a.len_utf8();
            } else {
                break;
            }
        }
        if matched < prefix_len {
            prefix_len = matched;
        }
    }
    String::from(&first[..prefix_len])
}

fn handle_tab_completion(state: &ShellState, input_buffer: &mut String) {
    let last_space = input_buffer.rfind(' ');
    let (line_prefix, token) = match last_space {
        Some(idx) => (&input_buffer[..=idx], &input_buffer[idx + 1..]),
        None => ("", input_buffer.as_str()),
    };

    let tokens: Vec<&str> = line_prefix.split_whitespace().collect();
    let mut candidates: Vec<String> = Vec::new();
    let file_replacement_prefix: String;

    if tokens.is_empty() && !token.contains('/') {
        for &cmd in BUILTIN_COMMANDS {
            if cmd.starts_with(token) && !candidates.iter().any(|c| c == cmd) {
                candidates.push(String::from(cmd));
            }
        }
        for (alias, _) in &state.aliases {
            if alias.starts_with(token) && !candidates.iter().any(|c| c == alias) {
                candidates.push(alias.clone());
            }
        }
        for bin in crate::pkg::installed_binaries() {
            if bin.starts_with(token) && !candidates.iter().any(|c| c == &bin) {
                candidates.push(bin);
            }
        }
    } else if !tokens.is_empty()
        && (tokens[0] == "pacman" || tokens[0] == "yay" || tokens[0] == "paru")
    {
        if tokens.len() == 1 && token.starts_with('-') {
            let flags = ["-S", "-Syu", "-Ss", "-Si", "-Q", "-Qi", "-Ql", "-R", "-Rs", "--help"];
            for f in flags.iter() {
                if f.starts_with(token) {
                    candidates.push(String::from(*f));
                }
            }
        } else if tokens.len() >= 2 {
            let flag = tokens[1];
            if flag == "-S" || flag == "-Si" || flag == "-Ss" || flag == "-Syu" {
                for pkg in crate::pkg::all_package_names() {
                    if pkg.starts_with(token) {
                        candidates.push(String::from(pkg));
                    }
                }
            } else if flag == "-R" || flag == "-Rs" || flag == "-Qi" || flag == "-Ql" {
                for pkg in crate::pkg::installed_package_names() {
                    if pkg.starts_with(token) {
                        candidates.push(pkg);
                    }
                }
            }
        } else {
            for pkg in crate::pkg::all_package_names() {
                if pkg.starts_with(token) {
                    candidates.push(String::from(pkg));
                }
            }
        }
    } else if tokens.len() == 1 && (tokens[0] == "man" || tokens[0] == "which" || tokens[0] == "type") {
        for &cmd in BUILTIN_COMMANDS {
            if cmd.starts_with(token) {
                candidates.push(String::from(cmd));
            }
        }
    }

    if candidates.is_empty() {
        let expanded_token = if token.starts_with("~/") {
            format!("/root/{}", &token[2..])
        } else if token == "~" {
            String::from("/root")
        } else {
            String::from(token)
        };

        let (search_dir, partial_name, display_prefix) = if let Some(slash_idx) = expanded_token.rfind('/') {
            let dir_part = if slash_idx == 0 {
                String::from("/")
            } else {
                fs::normalize_path(&state.cwd, &expanded_token[..slash_idx])
            };
            let user_prefix = if let Some(orig_slash) = token.rfind('/') {
                String::from(&token[..=orig_slash])
            } else {
                String::new()
            };
            (dir_part, String::from(&expanded_token[slash_idx + 1..]), user_prefix)
        } else {
            (state.cwd.clone(), expanded_token, String::new())
        };

        file_replacement_prefix = display_prefix;
        let entries = fs::list_dir(&search_dir);
        for entry in entries {
            let bname = entry.basename();
            if bname.starts_with(&partial_name) {
                if entry.is_dir == 1 {
                    candidates.push(format!("{}{}/", file_replacement_prefix, bname));
                } else {
                    candidates.push(format!("{}{}", file_replacement_prefix, bname));
                }
            }
        }
    }

    candidates.sort();

    if candidates.is_empty() {
        return;
    } else if candidates.len() == 1 {
        let mut completion = candidates[0].clone();
        if !completion.ends_with('/') {
            completion.push(' ');
        }
        let new_line = format!("{}{}", line_prefix, completion);
        replace_input_line(input_buffer, &new_line);
    } else {
        let lcp = longest_common_prefix(&candidates);
        if lcp.len() > token.len() {
            let new_line = format!("{}{}", line_prefix, lcp);
            replace_input_line(input_buffer, &new_line);
        } else {
            println!();
            for cand in &candidates {
                if cand.ends_with('/') {
                    vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
                } else {
                    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                }
                print!("{:18}", cand);
            }
            println!();
            print_prompt(state);
            vga::set_colors(ColorCode::White, ColorCode::Black);
            print!("{}", input_buffer);
        }
    }
}

fn expand_variables(state: &ShellState, input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '?' {
                result.push_str(&format!("{}", state.last_status));
                i += 2;
                continue;
            }
            let mut var_name = String::new();
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                var_name.push(chars[j]);
                j += 1;
            }
            if !var_name.is_empty() {
                if var_name == "PWD" {
                    result.push_str(&state.cwd);
                } else if let Some(val) = state.get_env(&var_name) {
                    result.push_str(val);
                }
                i = j;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn execute_line(state: &mut ShellState, raw_line: &str) {
    for segment in raw_line.split(';') {
        for sub in segment.split("&&") {
            let trimmed = sub.trim();
            if !trimmed.is_empty() {
                execute_single_or_redirect(state, trimmed);
            }
        }
    }
}

fn execute_single_or_redirect(state: &mut ShellState, cmd_line: &str) {
    let expanded = expand_variables(state, cmd_line);

    if let Some((left, right, append)) = split_redirection(&expanded) {
        let target_path = fs::normalize_path(&state.cwd, right.trim());
        let left_trim = left.trim();
        let text_out = capture_simple_output(state, left_trim);
        let res = if append {
            fs::append_file(&target_path, text_out.as_bytes())
        } else {
            fs::write_file(&target_path, text_out.as_bytes())
        };
        if let Err(e) = res {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("rsh: {}: {}", right.trim(), e);
            state.last_status = 1;
        } else {
            state.last_status = 0;
        }
        return;
    }

    if let Some(pipe_idx) = expanded.find('|') {
        let left = expanded[..pipe_idx].trim();
        let right = expanded[pipe_idx + 1..].trim();
        let piped_text = capture_simple_output(state, left);
        execute_piped_consumer(state, right, &piped_text);
        return;
    }

    execute_command(state, &expanded);
}

fn split_redirection(line: &str) -> Option<(&str, &str, bool)> {
    if let Some(idx) = line.find(">>") {
        return Some((&line[..idx], &line[idx + 2..], true));
    }
    if let Some(idx) = line.find('>') {
        return Some((&line[..idx], &line[idx + 1..], false));
    }
    None
}

fn capture_simple_output(state: &ShellState, cmd_line: &str) -> String {
    let mut parts = cmd_line.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    match cmd {
        "echo" => format!("{}\n", args.join(" ")),
        "pwd" => format!("{}\n", state.cwd),
        "whoami" => format!("{}\n", state.user),
        "hostname" => format!("{}\n", state.hostname),
        "uname" => {
            if args.contains(&"-a") {
                String::from("Linux rustos 6.12.0-arch1-1-rustos #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux\n")
            } else {
                String::from("Linux\n")
            }
        }
        "date" => {
            let dt = crate::rtc::read_rtc();
            format!(
                "{} {} {:2} {:02}:{:02}:{:02} UTC {:04}\n",
                dt.weekday_name(),
                dt.month_name(),
                dt.day,
                dt.hour,
                dt.minute,
                dt.second,
                dt.year
            )
        }
        "fortune" => {
            let idx = (crate::interrupts::ticks() as usize) % 5;
            let quotes = [
                "Talk is cheap. Show me the Rust kernel. - Linus Torvalds (probably)\n",
                "I use RustOS (Arch btw).\n",
                "Memory safety at ring 0: no segfaults, only fearless concurrency.\n",
                "There is no cloud, it's just somebody else's bare-metal x86_64 server.\n",
                "pacman -Syu: Keeping RustOS bleeding-edge since 2026.\n",
            ];
            String::from(quotes[idx])
        }
        "cat" => {
            let mut out = String::new();
            for arg in args {
                let p = fs::normalize_path(&state.cwd, arg);
                if let Ok(data) = fs::read_file(&p) {
                    if let Ok(s) = core::str::from_utf8(&data) {
                        out.push_str(s);
                        if !s.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                }
            }
            out
        }
        "ls" => {
            let entries = fs::list_dir(&state.cwd);
            let mut out = String::new();
            for e in entries {
                out.push_str(e.basename());
                out.push('\n');
            }
            out
        }
        _ => String::new(),
    }
}

fn execute_piped_consumer(state: &mut ShellState, right_cmd: &str, piped_text: &str) {
    let mut parts = right_cmd.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    match cmd {
        "grep" => {
            if let Some(pattern) = args.first() {
                for line in piped_text.lines() {
                    if line.contains(pattern) {
                        vga::set_colors(ColorCode::White, ColorCode::Black);
                        println!("{}", line);
                    }
                }
            }
        }
        "wc" => {
            let lines = piped_text.lines().count();
            let words = piped_text.split_whitespace().count();
            let bytes = piped_text.len();
            println!("{:7} {:7} {:7}", lines, words, bytes);
        }
        "head" => {
            for line in piped_text.lines().take(10) {
                println!("{}", line);
            }
        }
        "tail" => {
            let all: Vec<&str> = piped_text.lines().collect();
            let start = all.len().saturating_sub(10);
            for line in &all[start..] {
                println!("{}", line);
            }
        }
        "cowsay" => {
            let msg = piped_text.trim();
            let w: Vec<&str> = msg.split_whitespace().collect();
            crate::applets::try_run_applet("cowsay", &w, &state.cwd);
        }
        "lolcat" => {
            let msg = piped_text.trim();
            let w: Vec<&str> = msg.split_whitespace().collect();
            crate::applets::try_run_applet("lolcat", &w, &state.cwd);
        }
        "figlet" => {
            let msg = piped_text.trim();
            let w: Vec<&str> = msg.split_whitespace().collect();
            crate::applets::try_run_applet("figlet", &w, &state.cwd);
        }
        _ => {
            execute_command(state, right_cmd);
        }
    }
}

fn execute_command(state: &mut ShellState, cmd_line: &str) {
    let mut raw_parts = cmd_line.split_whitespace();
    let first_word = match raw_parts.next() {
        Some(cmd) => cmd,
        None => return,
    };

    let resolved_line: String;
    let (command, args): (&str, Vec<&str>) = if let Some((_, target)) =
        state.aliases.iter().find(|(k, _)| k == first_word)
    {
        let rest: Vec<&str> = raw_parts.collect();
        if rest.is_empty() {
            resolved_line = target.clone();
        } else {
            resolved_line = format!("{} {}", target, rest.join(" "));
        }
        let mut p = resolved_line.split_whitespace();
        let c = p.next().unwrap_or("");
        (c, p.collect())
    } else {
        (first_word, raw_parts.collect())
    };

    state.last_status = 0;

    match command {
        "help" => cmd_help(),
        "clear" | "reset" => vga::clear_screen(),
        "echo" => cmd_echo(&args),
        "pwd" => {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("{}", state.cwd);
        }
        "cd" => cmd_cd(state, &args),
        "ls" | "dir" => cmd_ls(state, &args),
        "mkdir" => cmd_mkdir(state, &args),
        "rmdir" => cmd_rmdir(state, &args),
        "touch" => cmd_touch(state, &args),
        "cat" => cmd_cat(state, &args),
        "write" => cmd_write(state, &args, false),
        "append" => cmd_write(state, &args, true),
        "cp" => cmd_cp(state, &args),
        "mv" => cmd_mv(state, &args),
        "rm" => cmd_rm(state, &args),
        "chmod" => cmd_chmod(state, &args),
        "stat" | "file" => cmd_stat(state, &args),
        "head" => cmd_head(state, &args),
        "tail" => cmd_tail(state, &args),
        "wc" => cmd_wc(state, &args),
        "grep" => cmd_grep(state, &args),
        "find" => cmd_find(state, &args),
        "tree" => cmd_tree(state, &args),
        "nano" | "vim" | "vi" | "edit" => {
            if args.is_empty() {
                crate::applets::cmd_nano(&state.cwd, &["untitled.txt"]);
            } else {
                crate::applets::cmd_nano(&state.cwd, &args);
            }
        }
        "install" | "rustos-install" | "archinstall" | "archinsttall" => {
            let auto_yes = args.contains(&"-y") || args.contains(&"--yes");
            crate::installer::run_installer(auto_yes);
            println!();
            print_prompt(state);
            return;
        }
        "wm" | "startx" | "staartx" | "desktop" | "desktopp" | "compositor" => {
            crate::compositor::run();
            print_prompt(state);
            return;
        }
        "pacman" => crate::pkg::cmd_pacman(&args),
        "yay" => crate::pkg::cmd_yay_or_paru("yay", &args),
        "paru" => crate::pkg::cmd_yay_or_paru("paru", &args),
        "makepkg" => crate::pkg::cmd_makepkg(&state.cwd, &args),
        "neofetch" | "fastfetch" => crate::applets::cmd_neofetch(false),
        "hyfetch" => crate::applets::cmd_neofetch(true),
        "pfetch" => crate::applets::cmd_pfetch(),
        "onefetch" => crate::applets::cmd_onefetch(),
        "uname" => cmd_uname(&args),
        "arch" => println!("x86_64"),
        "whoami" => println!("{}", state.user),
        "id" => println!("uid=0(root) gid=0(root) groups=0(root),1(bin),2(daemon),10(wheel)"),
        "hostname" => {
            if let Some(new_host) = args.first() {
                state.hostname = String::from(*new_host);
                state.set_env("HOSTNAME", new_host);
                let _ = fs::write_file("/etc/hostname", format!("{}\n", new_host).as_bytes());
            } else {
                println!("{}", state.hostname);
            }
        }
        "hostnamectl" => cmd_hostnamectl(state, &args),
        "passwd" => cmd_passwd(&args),
        "date" => cmd_date(),
        "timedatectl" => cmd_timedatectl(&args),
        "uptime" => cmd_uptime(),
        "lscpu" => cmd_lscpu(),
        "lspci" => cmd_lspci(),
        "lsblk" => cmd_lsblk(),
        "lsmod" => cmd_lsmod(),
        "free" => cmd_free(&args),
        "df" => cmd_df(&args),
        "du" => cmd_du(state, &args),
        "mount" => cmd_mount(),
        "ps" => cmd_ps(),
        "top" | "htop" | "btop" => {
            let _ = crate::applets::try_run_applet("btop", &args, &state.cwd);
        }
        "systemctl" => cmd_systemctl(&args),
        "journalctl" | "dmesg" => cmd_dmesg(),
        "ip" | "ifconfig" => cmd_ip(),
        "ping" => cmd_ping(&args),
        "curl" | "wget" => cmd_curl(&args),
        "env" => {
            for (k, v) in &state.env_vars {
                println!("{}={}", k, v);
            }
        }
        "export" => cmd_export(state, &args),
        "unset" => {
            for k in args {
                state.unset_env(k);
            }
        }
        "alias" => cmd_alias(state, &args),
        "which" | "type" => cmd_which(state, &args),
        "history" => {
            for (i, h) in state.history.iter().enumerate() {
                println!(" {:4}  {}", i + 1, h);
            }
        }
        "man" => cmd_man(&args),
        "source" | "." => cmd_source(state, &args),
        "info" => cmd_info(),
        "colors" => cmd_colors(),
        "mem" => cmd_mem(),
        "alloc" => cmd_alloc(),
        "calc" => cmd_calc(&args),
        "disk" => cmd_disk(),
        "format" => cmd_format(),
        "panic" => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            panic!("Intentional panic requested via shell command.");
        }
        "reboot" => crate::power::reboot(),
        "shutdown" | "poweroff" | "exit" => crate::power::shutdown(),
        other => {
            if crate::arch_commands::dispatch_arch_command(other, &args, &state.cwd) {
                // executed arch command
            } else if !crate::applets::try_run_applet(other, &args, &state.cwd) {
                let candidate = fs::normalize_path(&state.cwd, other);
                if fs::exists(&candidate) {
                    if let Ok(data) = fs::read_file(&candidate) {
                        if let Ok(text) = core::str::from_utf8(&data) {
                            for line in text.lines() {
                                let t = line.trim();
                                if !t.is_empty() && !t.starts_with('#') {
                                    execute_line(state, t);
                                }
                            }
                            return;
                        }
                    }
                }
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("rsh: {}: command not found", other);
                vga::set_colors(ColorCode::LightGray, ColorCode::Black);
                println!("Tip: Search packages with 'pacman -Ss {}' or type 'help'.", other);
                state.last_status = 127;
            }
        }
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);
}

fn resolve_user_path(cwd: &str, raw: &str) -> String {
    if raw == "~" {
        String::from("/root")
    } else if raw.starts_with("~/") {
        fs::normalize_path("/root", &raw[2..])
    } else {
        fs::normalize_path(cwd, raw)
    }
}

fn cmd_cd(state: &mut ShellState, args: &[&str]) {
    let target = match args.first() {
        None | Some(&"~") => String::from("/root"),
        Some(&"-") => {
            println!("{}", state.prev_cwd);
            state.prev_cwd.clone()
        }
        Some(p) => resolve_user_path(&state.cwd, p),
    };

    if fs::is_dir(&target) {
        state.prev_cwd = state.cwd.clone();
        state.cwd = target;
    } else if fs::exists(&target) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("cd: {}: Not a directory", args.first().unwrap_or(&""));
    } else {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("cd: {}: No such file or directory", args.first().unwrap_or(&""));
    }
}

fn cmd_ls(state: &ShellState, args: &[&str]) {
    let mut long_fmt = false;
    let mut show_all = false;
    let mut target_path = state.cwd.clone();

    for &arg in args {
        if arg.starts_with('-') {
            if arg.contains('l') {
                long_fmt = true;
            }
            if arg.contains('a') {
                show_all = true;
            }
        } else {
            target_path = resolve_user_path(&state.cwd, arg);
        }
    }

    if !fs::exists(&target_path) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("ls: cannot access '{}': No such file or directory", target_path);
        return;
    }

    if !fs::is_dir(&target_path) {
        if let Some(meta) = fs::stat(&target_path) {
            if long_fmt {
                println!(
                    "{} 1 root root {:6} Nov 04 19:30 {}",
                    meta.mode_string(),
                    meta.size_bytes,
                    meta.basename()
                );
            } else {
                println!("{}", meta.basename());
            }
        }
        return;
    }

    let entries = fs::list_dir(&target_path);
    if long_fmt {
        vga::set_colors(ColorCode::LightGray, ColorCode::Black);
        println!("total {}", entries.len() * 4);
        if show_all {
            vga::set_colors(ColorCode::LightGray, ColorCode::Black);
            print!("drwxr-xr-x 2 root root   4096 Nov 04 19:30 ");
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            println!(".");
            vga::set_colors(ColorCode::LightGray, ColorCode::Black);
            print!("drwxr-xr-x 2 root root   4096 Nov 04 19:30 ");
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            println!("..");
        }
        for e in entries {
            let bname = e.basename();
            if !show_all && bname.starts_with('.') {
                continue;
            }
            let mode_str = e.mode_string();
            let size_val = if e.is_dir == 1 { 4096 } else { e.size_bytes };
            vga::set_colors(ColorCode::LightGray, ColorCode::Black);
            print!("{} 1 root root {:6} Nov 04 19:30 ", mode_str, size_val);
            if e.is_dir == 1 {
                vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
                println!("{}/", bname);
            } else if e.mode & 0o111 != 0 {
                vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                println!("{}*", bname);
            } else {
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("{}", bname);
            }
        }
    } else {
        if show_all {
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            print!("{:18}{:18}", "./", "../");
        }
        let mut count = if show_all { 2 } else { 0 };
        for e in entries {
            let bname = e.basename();
            if !show_all && bname.starts_with('.') {
                continue;
            }
            if e.is_dir == 1 {
                vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
                let d = format!("{}/", bname);
                print!("{:18}", d);
            } else if e.mode & 0o111 != 0 {
                vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                print!("{:18}", bname);
            } else {
                vga::set_colors(ColorCode::White, ColorCode::Black);
                print!("{:18}", bname);
            }
            count += 1;
            if count % 4 == 0 {
                println!();
            }
        }
        if count % 4 != 0 {
            println!();
        }
    }
}

fn cmd_mkdir(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("mkdir: missing operand");
        return;
    }
    for &arg in args {
        if arg.starts_with('-') {
            continue;
        }
        let path = resolve_user_path(&state.cwd, arg);
        if let Err(e) = fs::mkdir(&path) {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("mkdir: cannot create directory '{}': {}", arg, e);
        }
    }
}

fn cmd_rmdir(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("rmdir: missing operand");
        return;
    }
    for &arg in args {
        let path = resolve_user_path(&state.cwd, arg);
        if let Err(e) = fs::delete(&path, false) {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("rmdir: failed to remove '{}': {}", arg, e);
        }
    }
}

fn cmd_touch(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("touch: missing file operand");
        return;
    }
    for &arg in args {
        let path = resolve_user_path(&state.cwd, arg);
        if let Err(e) = fs::touch(&path) {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("touch: cannot touch '{}': {}", arg, e);
        }
    }
}

fn cmd_cat(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("Usage: cat <file> [file2 ...]");
        return;
    }
    let show_lines = args.contains(&"-n");
    let mut line_num = 1;
    for &arg in args {
        if arg.starts_with('-') {
            continue;
        }
        let path = resolve_user_path(&state.cwd, arg);
        match fs::read_file(&path) {
            Ok(data) => {
                if let Ok(text) = core::str::from_utf8(&data) {
                    if show_lines {
                        for line in text.lines() {
                            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
                            print!("{:6}  ", line_num);
                            vga::set_colors(ColorCode::White, ColorCode::Black);
                            println!("{}", line);
                            line_num += 1;
                        }
                    } else {
                        vga::set_colors(ColorCode::White, ColorCode::Black);
                        print!("{}", text);
                        if !text.ends_with('\n') {
                            println!();
                        }
                    }
                } else {
                    println!("[Binary file: {} bytes]", data.len());
                }
            }
            Err(err) => {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("cat: {}: {}", arg, err);
            }
        }
    }
}

fn cmd_write(state: &ShellState, args: &[&str], append: bool) {
    if args.len() < 2 {
        println!("Usage: {} <file> <text content>", if append { "append" } else { "write" });
        return;
    }
    let path = resolve_user_path(&state.cwd, args[0]);
    let content = format!("{}\n", args[1..].join(" "));
    let res = if append {
        fs::append_file(&path, content.as_bytes())
    } else {
        fs::write_file(&path, content.as_bytes())
    };
    match res {
        Ok(()) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Saved '{}' ({} bytes).", path, content.len());
        }
        Err(e) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error writing '{}': {}", path, e);
        }
    }
}

fn cmd_cp(state: &ShellState, args: &[&str]) {
    if args.len() < 2 {
        println!("cp: missing file operand (Usage: cp <src> <dst>)");
        return;
    }
    let src = resolve_user_path(&state.cwd, args[0]);
    let mut dst = resolve_user_path(&state.cwd, args[1]);
    if fs::is_dir(&dst) {
        let (_, base) = fs::parent_and_basename(&src);
        dst = fs::normalize_path(&dst, &base);
    }
    if let Err(e) = fs::copy_file(&src, &dst) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("cp: cannot copy '{}' to '{}': {}", args[0], args[1], e);
    }
}

fn cmd_mv(state: &ShellState, args: &[&str]) {
    if args.len() < 2 {
        println!("mv: missing file operand (Usage: mv <src> <dst>)");
        return;
    }
    let src = resolve_user_path(&state.cwd, args[0]);
    let mut dst = resolve_user_path(&state.cwd, args[1]);
    if fs::is_dir(&dst) {
        let (_, base) = fs::parent_and_basename(&src);
        dst = fs::normalize_path(&dst, &base);
    }
    if let Err(e) = fs::move_path(&src, &dst) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("mv: cannot move '{}' to '{}': {}", args[0], args[1], e);
    }
}

fn cmd_rm(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("rm: missing operand");
        return;
    }
    let mut recursive = false;
    let mut force = false;
    let mut targets = Vec::new();
    for &arg in args {
        if arg.starts_with('-') {
            if arg.contains('r') || arg.contains('R') {
                recursive = true;
            }
            if arg.contains('f') {
                force = true;
            }
        } else {
            targets.push(arg);
        }
    }
    for t in targets {
        let path = resolve_user_path(&state.cwd, t);
        if let Err(e) = fs::delete(&path, recursive) {
            if !force {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("rm: cannot remove '{}': {}", t, e);
            }
        }
    }
}

fn cmd_chmod(state: &ShellState, args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: chmod <mode|+x> <file>");
        return;
    }
    let mode_arg = args[0];
    let path = resolve_user_path(&state.cwd, args[1]);
    let new_mode = if mode_arg == "+x" || mode_arg == "a+x" || mode_arg == "u+x" {
        0o755
    } else if mode_arg == "-x" {
        0o644
    } else {
        u16::from_str_radix(mode_arg, 8).unwrap_or(0o644)
    };
    if let Err(e) = fs::chmod(&path, new_mode) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("chmod: {}: {}", args[1], e);
    }
}

fn cmd_stat(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("stat: missing operand");
        return;
    }
    for &arg in args {
        let path = resolve_user_path(&state.cwd, arg);
        match fs::stat(&path) {
            Some(meta) => {
                println!("  File: {}", meta.name_str());
                println!(
                    "  Size: {:<10} Blocks: {:<6} Type: {}",
                    meta.size_bytes,
                    (meta.size_bytes + 511) / 512,
                    if meta.is_dir == 1 { "directory" } else { "regular file" }
                );
                println!(
                    "Access: ({:04o}/{})  Uid: (    0/    root)   Gid: (    0/    root)",
                    meta.mode,
                    meta.mode_string()
                );
            }
            None => {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("stat: cannot statx '{}': No such file or directory", arg);
            }
        }
    }
}

fn cmd_head(state: &ShellState, args: &[&str]) {
    let mut lines_count = 10usize;
    let mut file_arg = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            lines_count = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else {
            file_arg = Some(args[i]);
            i += 1;
        }
    }
    let Some(f) = file_arg else {
        println!("Usage: head [-n lines] <file>");
        return;
    };
    let path = resolve_user_path(&state.cwd, f);
    if let Ok(data) = fs::read_file(&path) {
        if let Ok(text) = core::str::from_utf8(&data) {
            for line in text.lines().take(lines_count) {
                println!("{}", line);
            }
        }
    } else {
        println!("head: cannot open '{}' for reading", f);
    }
}

fn cmd_tail(state: &ShellState, args: &[&str]) {
    let mut lines_count = 10usize;
    let mut file_arg = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            lines_count = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else {
            file_arg = Some(args[i]);
            i += 1;
        }
    }
    let Some(f) = file_arg else {
        println!("Usage: tail [-n lines] <file>");
        return;
    };
    let path = resolve_user_path(&state.cwd, f);
    if let Ok(data) = fs::read_file(&path) {
        if let Ok(text) = core::str::from_utf8(&data) {
            let all: Vec<&str> = text.lines().collect();
            let start = all.len().saturating_sub(lines_count);
            for line in &all[start..] {
                println!("{}", line);
            }
        }
    } else {
        println!("tail: cannot open '{}' for reading", f);
    }
}

fn cmd_wc(state: &ShellState, args: &[&str]) {
    if args.is_empty() {
        println!("Usage: wc <file>");
        return;
    }
    for &arg in args {
        if arg.starts_with('-') {
            continue;
        }
        let path = resolve_user_path(&state.cwd, arg);
        if let Ok(data) = fs::read_file(&path) {
            let text = core::str::from_utf8(&data).unwrap_or("");
            let lines = text.lines().count();
            let words = text.split_whitespace().count();
            let bytes = data.len();
            println!("{:6} {:6} {:6} {}", lines, words, bytes, arg);
        } else {
            println!("wc: {}: No such file or directory", arg);
        }
    }
}

fn cmd_grep(state: &ShellState, args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: grep <pattern> <file>");
        return;
    }
    let pattern = args[0];
    for &file in &args[1..] {
        let path = resolve_user_path(&state.cwd, file);
        if let Ok(data) = fs::read_file(&path) {
            if let Ok(text) = core::str::from_utf8(&data) {
                for (idx, line) in text.lines().enumerate() {
                    if line.contains(pattern) {
                        vga::set_colors(ColorCode::LightMagenta, ColorCode::Black);
                        print!("{}:{}:", file, idx + 1);
                        vga::set_colors(ColorCode::White, ColorCode::Black);
                        println!("{}", line);
                    }
                }
            }
        } else {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("grep: {}: No such file or directory", file);
        }
    }
}

fn cmd_find(state: &ShellState, args: &[&str]) {
    let base = if let Some(&first) = args.first() {
        if !first.starts_with('-') {
            resolve_user_path(&state.cwd, first)
        } else {
            state.cwd.clone()
        }
    } else {
        state.cwd.clone()
    };
    let filter = args.last().copied();
    for entry in fs::list_all() {
        let full_path = entry.name_str();
        if full_path.starts_with(&base) || base == "/" {
            if let Some(pat) = filter {
                if pat != "." && !pat.starts_with('/') && !entry.basename().contains(pat.trim_matches('*')) {
                    continue;
                }
            }
            if entry.is_dir == 1 {
                vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            } else {
                vga::set_colors(ColorCode::White, ColorCode::Black);
            }
            println!("{}", full_path);
        }
    }
}

fn print_tree_dir(dir: &str, depth: usize, dirs: &mut usize, files: &mut usize) {
    let entries = fs::list_dir(dir);
    for (i, e) in entries.iter().enumerate() {
        for _ in 0..depth {
            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
            print!("│   ");
        }
        let branch = if i + 1 == entries.len() { "└── " } else { "├── " };
        vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
        print!("{}", branch);
        if e.is_dir == 1 {
            *dirs += 1;
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            println!("{}/", e.basename());
            if depth < 4 {
                print_tree_dir(e.name_str(), depth + 1, dirs, files);
            }
        } else {
            *files += 1;
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("{}", e.basename());
        }
    }
}

fn cmd_tree(state: &ShellState, args: &[&str]) {
    let target = args
        .first()
        .map(|a| resolve_user_path(&state.cwd, a))
        .unwrap_or_else(|| state.cwd.clone());
    vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
    println!("{}", target);
    let mut dirs = 0;
    let mut files = 0;
    print_tree_dir(&target, 0, &mut dirs, &mut files);
    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    println!("\n{} directories, {} files", dirs, files);
}

fn cmd_uname(args: &[&str]) {
    if args.is_empty() || args.contains(&"-s") {
        println!("Linux");
    } else if args.contains(&"-a") {
        println!("Linux rustos 6.12.0-arch1-1-rustos #1 SMP PREEMPT_DYNAMIC Nov 2026 x86_64 RustOS GNU/Linux");
    } else if args.contains(&"-r") {
        println!("6.12.0-arch1-1-rustos");
    } else if args.contains(&"-m") {
        println!("x86_64");
    } else if args.contains(&"-n") {
        println!("rustos");
    } else {
        println!("Linux");
    }
}

fn read_masked_shell(prompt_text: &str) -> String {
    print!("{}", prompt_text);
    let mut input = String::new();
    loop {
        let ch = keyboard::read_char();
        if ch == '\n' || ch == '\r' {
            println!();
            break;
        } else if ch == '\x08' {
            if !input.is_empty() {
                input.pop();
                vga::backspace();
            }
        } else if ch >= ' ' && ch <= '~' {
            input.push(ch);
            print!("*");
        }
    }
    String::from(input.trim())
}

fn cmd_passwd(args: &[&str]) {
    let target_user = if args.len() > 1 { args[1] } else { "root" };
    println!("Changing password for {}.", target_user);

    let p1 = read_masked_shell("New password: ");
    if p1.is_empty() {
        println!("passwd: password unchanged");
        return;
    }

    let p2 = read_masked_shell("Retype new password: ");
    if p1 != p2 {
        println!("passwd: passwords do not match. Password unchanged.");
        return;
    }

    let shadow_path = "/etc/shadow";
    let entry = format!("{}:$6$salt${}:19000:0:99999:7:::\n", target_user, p1);
    let mut new_shadow = String::new();
    if let Ok(content) = crate::fs::read_file(shadow_path) {
        if let Ok(text) = core::str::from_utf8(&content) {
            for line in text.lines() {
                if !line.starts_with(&format!("{}:", target_user)) && !line.is_empty() {
                    new_shadow.push_str(line);
                    new_shadow.push('\n');
                }
            }
        }
    }
    new_shadow.push_str(&entry);
    let _ = crate::fs::write_file(shadow_path, new_shadow.as_bytes());
    println!("passwd: password updated successfully for {}", target_user);
}

fn cmd_hostnamectl(state: &mut ShellState, args: &[&str]) {
    if args.len() >= 2 && args[0] == "set-hostname" {
        state.hostname = String::from(args[1]);
        let _ = fs::write_file("/etc/hostname", format!("{}\n", args[1]).as_bytes());
        println!("Hostname updated to '{}'", args[1]);
        return;
    }
    println!(" Static hostname: {}", state.hostname);
    println!("       Icon name: computer-vm");
    println!("         Chassis: vm");
    println!("Operating System: RustOS (Inspired by Arch Linux)");
    println!("          Kernel: RustOS 0.2.0-rustos");
    println!("    Architecture: x86-64");
}

fn get_active_timezone() -> (&'static str, i32) {
    if let Ok(data) = crate::fs::read_file("/etc/timezone") {
        if let Ok(tz_str) = core::str::from_utf8(&data) {
            let clean = tz_str.trim();
            if let Some(tz) = crate::tzdata::find_timezone(clean) {
                return (tz.name, tz.offset_minutes);
            }
        }
    }
    ("UTC", 0)
}

fn cmd_date() {
    let utc_dt = crate::rtc::read_rtc();
    let (tz_name, offset) = get_active_timezone();
    let dt = crate::tzdata::convert_utc_to_local(&utc_dt, offset);
    let tz_code = if tz_name == "UTC" {
        "UTC"
    } else {
        tz_name.split('/').last().unwrap_or("LOC")
    };
    println!(
        "{} {} {:2} {:02}:{:02}:{:02} {} {:04}",
        dt.weekday_name(),
        dt.month_name(),
        dt.day,
        dt.hour,
        dt.minute,
        dt.second,
        tz_code,
        dt.year
    );
}

fn cmd_timedatectl(args: &[&str]) {
    if args.len() > 1 && args[1] == "list-timezones" {
        println!("Available system timezones (IANA tzdata):");
        for reg in crate::tzdata::REGIONS {
            println!("  [{}]", reg.name);
            for tz in reg.timezones {
                println!("    {} ({})", tz.name, tz.description);
            }
        }
        return;
    }
    if args.len() > 2 && args[1] == "set-timezone" {
        let new_tz = args[2];
        if let Some(tz) = crate::tzdata::find_timezone(new_tz) {
            let _ = crate::fs::write_file("/etc/timezone", format!("{}\n", tz.name).as_bytes());
            let _ = crate::fs::write_file("/etc/localtime", format!("{}\n", tz.name).as_bytes());
            println!("Time zone set to: {} ({})", tz.name, tz.description);
        } else {
            println!("timedatectl: timezone '{}' not recognized. Use 'timedatectl list-timezones' to view list.", new_tz);
        }
        return;
    }

    let utc_dt = crate::rtc::read_rtc();
    let (tz_name, offset) = get_active_timezone();
    let local_dt = crate::tzdata::convert_utc_to_local(&utc_dt, offset);
    let tz_str = crate::tzdata::format_offset_string(offset);

    println!(
        "               Local time: {} {:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        local_dt.weekday_name(),
        local_dt.year,
        local_dt.month,
        local_dt.day,
        local_dt.hour,
        local_dt.minute,
        local_dt.second,
        tz_str
    );
    println!(
        "           Universal time: {} {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        utc_dt.weekday_name(),
        utc_dt.year,
        utc_dt.month,
        utc_dt.day,
        utc_dt.hour,
        utc_dt.minute,
        utc_dt.second
    );
    println!(
        "                 RTC time: {} {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        utc_dt.weekday_name(),
        utc_dt.year,
        utc_dt.month,
        utc_dt.day,
        utc_dt.hour,
        utc_dt.minute,
        utc_dt.second
    );
    println!("                Time zone: {} ({})", tz_name, tz_str);
    println!("System clock synchronized: yes (CMOS RTC 0x70)");
    println!("              NTP service: active (network time emulation)");
    println!("          RTC in local TZ: no");
}

fn cmd_lscpu() {
    let cpu = crate::sysinfo::get_cpu_info();
    println!("Architecture:             x86_64");
    println!("  CPU op-mode(s):         32-bit, 64-bit");
    println!("  Address sizes:          48 bits physical, 48 bits virtual");
    println!("  Byte Order:             Little Endian");
    println!("CPU(s):                   {}", cpu.cores);
    println!("Vendor ID:                {}", cpu.vendor);
    println!("  Model name:             {}", cpu.model_name);
    println!("  CPU family:             {}", cpu.family);
    println!("  Model:                  {}", cpu.model);
    println!("  Stepping:               {}", cpu.stepping);
    println!("  Flags:                  {}", cpu.flags.join(" "));
}

fn cmd_lspci() {
    let devices = crate::sysinfo::scan_pci();
    for dev in devices {
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        print!("{:02x}:{:02x}.{:x} ", dev.bus, dev.slot, dev.func);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        print!("{}: ", dev.class_name());
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        println!("{} {} [{:04x}:{:04x}]", dev.vendor_name(), dev.device_desc(), dev.vendor_id, dev.device_id);
    }
}

fn cmd_lsblk() {
    let (total_kb, _, _) = fs::stats();
    println!("NAME   MAJ:MIN RM  SIZE RO TYPE MOUNTPOINTS");
    println!("sda      8:0    0   {}M  0 disk ", (total_kb / 1024).max(32));
    println!("├─sda1   8:1    0    6M  0 part /boot/efi");
    println!("└─sda2   8:2    0   26M  0 part /");
    println!("sr0     11:0    1   16M  1 rom  /run/archiso/bootmnt");
}

fn cmd_lsmod() {
    println!("Module                  Size  Used by");
    println!("simplefs               32768  1");
    println!("ata_pio                16384  1 simplefs");
    println!("ps2_keyboard           12288  0");
    println!("vga_framebuffer        28672  1");
    println!("cmos_rtc                8192  0");
    println!("pci_bus                16384  0");
    println!("pic8259                 8192  1");
}

fn cmd_free(_args: &[&str]) {
    let (_, heap_size, _) = get_heap_info();
    let total_kb = (heap_size / 1024).max(4096);
    let used_kb = 640;
    let free_kb = total_kb - used_kb;
    println!("               total        used        free      shared  buff/cache   available");
    println!(
        "Mem:        {:7}Ki   {:7}Ki   {:7}Ki         0Ki       256Ki   {:7}Ki",
        total_kb, used_kb, free_kb, free_kb
    );
    println!("Swap:              0Ki         0Ki         0Ki");
}

fn cmd_df(_args: &[&str]) {
    let (total_kb, used_kb, disk_backed) = fs::stats();
    let avail_kb = total_kb.saturating_sub(used_kb);
    let pct = ((used_kb * 100) / total_kb.max(1)).min(100);
    println!("Filesystem     1K-blocks  Used Available Use% Mounted on");
    println!(
        "{:<14} {:9} {:5} {:9} {:3}% /",
        if disk_backed { "/dev/sda2" } else { "tmpfs-ram" },
        total_kb,
        used_kb,
        avail_kb,
        pct
    );
    println!("devtmpfs            2048     0      2048   0% /dev");
}

fn cmd_du(state: &ShellState, args: &[&str]) {
    let target = args
        .first()
        .map(|a| resolve_user_path(&state.cwd, a))
        .unwrap_or_else(|| state.cwd.clone());
    let entries = fs::list_dir(&target);
    let mut sum = 0;
    for e in entries {
        let kb = ((e.size_bytes as usize + 1023) / 1024).max(4);
        sum += kb;
        println!("{:<6}  {}", kb, e.name_str());
    }
    println!("{:<6}  {}", sum.max(4), target);
}

fn cmd_mount() {
    let (_, _, disk_backed) = fs::stats();
    println!(
        "{} on / type simplefs (rw,relatime,sync)",
        if disk_backed { "/dev/sda2" } else { "rootfs" }
    );
    println!("proc on /proc type proc (rw,nosuid,nodev,noexec)");
    println!("sysfs on /sys type sysfs (rw,nosuid,nodev,noexec)");
    println!("devtmpfs on /dev type devtmpfs (rw,nosuid,size=2048k)");
}

fn cmd_ps() {
    let up = uptime_seconds();
    println!("  PID TTY      STAT   TIME COMMAND");
    println!("    1 tty1     Ss     {:02}:{:02} /sbin/init (rustos-kernel)", up / 60, up % 60);
    println!("    2 tty1     S      00:00 [kthreadd-ata-pio]");
    println!("    3 tty1     S      00:00 [irq/1-i8042-kbd]");
    println!("    4 tty1     S      00:00 [pacman-alpm-cache]");
    println!("   10 tty1     Ss     00:00 -rsh (RustOS Arch Shell)");
    println!("   42 tty1     R+     00:00 ps");
}

fn cmd_systemctl(args: &[&str]) {
    if args.is_empty() || args[0] == "status" {
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        print!("● ");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("rustos.host");
        println!("    State: running");
        println!("     Jobs: 0 queued");
        println!("   Failed: 0 units");
        println!("    Since: Boot ({}s ago)", uptime_seconds());
        println!("   CGroup: /");
        println!("           ├─init.scope (1 /sbin/init)");
        println!("           └─system.slice");
        println!("             ├─ata-pio.service");
        println!("             ├─simplefs-mount.service");
        println!("             ├─pacman-keyring.service");
        println!("             └─getty@tty1.service (10 -rsh)");
    } else if args[0] == "reboot" {
        crate::power::reboot();
    } else if args[0] == "poweroff" {
        crate::power::shutdown();
    } else {
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        println!("[  OK  ] systemctl {} completed.", args.join(" "));
    }
}

fn cmd_dmesg() {
    for (ticks, msg) in crate::sysinfo::get_dmesg() {
        let secs = ticks / 100;
        let rem = (ticks % 100) * 10;
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        print!("[{:5}.{:03}] ", secs, rem);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("{}", msg);
    }
}

fn cmd_ip() {
    println!("1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 state UNKNOWN");
    println!("    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00");
    println!("    inet 127.0.0.1/8 scope host lo");
    println!("2: enp0s3: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 state UP");
    println!("    link/ether 08:00:27:c0:ff:ee brd ff:ff:ff:ff:ff:ff");
    println!("    inet 10.0.2.15/24 brd 10.0.2.255 scope global dynamic enp0s3");
}

fn cmd_ping(args: &[&str]) {
    let host = args.first().copied().unwrap_or("archlinux.org");
    println!("PING {} (95.217.163.246) 56(84) bytes of data.", host);
    for seq in 1..=3 {
        println!(
            "64 bytes from {} (95.217.163.246): icmp_seq={} ttl=64 time=0.{} ms",
            host,
            seq,
            35 + seq * 7
        );
        keyboard::sleep_ticks(15);
    }
    println!("--- {} ping statistics ---", host);
    println!("3 packets transmitted, 3 received, 0% packet loss, time 450ms");
}

fn cmd_curl(args: &[&str]) {
    let url = args.first().copied().unwrap_or("https://archlinux.org");
    if url.contains("aur.archlinux.org") {
        println!("{{\"version\":5,\"type\":\"multiinfo\",\"resultcount\":1,\"results\":[{{\"Name\":\"rustos-aur\",\"Version\":\"1.0.0-1\"}}]}}");
    } else {
        println!("<!DOCTYPE html><html><head><title>Arch Linux / RustOS Mirror</title></head>");
        println!("<body><h1>200 OK - Connected from RustOS x86_64 ({})</h1></body></html>", url);
    }
}

fn cmd_export(state: &mut ShellState, args: &[&str]) {
    if args.is_empty() {
        for (k, v) in &state.env_vars {
            println!("declare -x {}=\"{}\"", k, v);
        }
        return;
    }
    let joined = args.join(" ");
    if let Some(eq_idx) = joined.find('=') {
        let key = joined[..eq_idx].trim();
        let val = joined[eq_idx + 1..].trim().trim_matches('"').trim_matches('\'');
        state.set_env(key, val);
    }
}

fn cmd_alias(state: &mut ShellState, args: &[&str]) {
    if args.is_empty() {
        for (k, v) in &state.aliases {
            println!("alias {}='{}'", k, v);
        }
        return;
    }
    let joined = args.join(" ");
    if let Some(eq_idx) = joined.find('=') {
        let key = joined[..eq_idx].trim();
        let val = joined[eq_idx + 1..].trim().trim_matches('\'').trim_matches('"');
        for (k, v) in &mut state.aliases {
            if k == key {
                *v = String::from(val);
                return;
            }
        }
        state.aliases.push((String::from(key), String::from(val)));
    }
}

fn cmd_which(state: &ShellState, args: &[&str]) {
    for &arg in args {
        if let Some((_, target)) = state.aliases.iter().find(|(k, _)| k == arg) {
            println!("{}: aliased to {}", arg, target);
        } else if BUILTIN_COMMANDS.contains(&arg) {
            println!("/usr/bin/{}", arg);
        } else if crate::pkg::installed_binaries().iter().any(|b| b == arg) {
            println!("/usr/bin/{}", arg);
        } else {
            println!("which: no {} in (/usr/local/bin:/usr/bin:/bin)", arg);
        }
    }
}

fn cmd_source(state: &mut ShellState, args: &[&str]) {
    let Some(f) = args.first() else {
        println!("source: filename argument required");
        return;
    };
    let path = resolve_user_path(&state.cwd, f);
    if let Ok(data) = fs::read_file(&path) {
        if let Ok(text) = core::str::from_utf8(&data) {
            for line in text.lines() {
                let t = line.trim();
                if !t.is_empty() && !t.starts_with('#') {
                    execute_line(state, t);
                }
            }
        }
    } else {
        println!("source: {}: No such file or directory", f);
    }
}

fn cmd_man(args: &[&str]) {
    let Some(&topic) = args.first() else {
        println!("What manual page do you want? Example: man pacman, man yay, man ls");
        return;
    };
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("{} (1)                 RustOS / Arch Linux Manual                 {} (1)", topic.to_uppercase(), topic.to_uppercase());
    vga::set_colors(ColorCode::White, ColorCode::Black);
    match topic {
        "pacman" => {
            println!("NAME\n    pacman - Arch Linux package manager utility in RustOS\n");
            println!("SYNOPSIS\n    pacman <operation> [options] [targets]\n");
            println!("OPERATIONS\n    -S <pkg>     Install package(s) from core/extra repositories");
            println!("    -Syu         Synchronize databases and upgrade all packages");
            println!("    -Ss <query>  Search package descriptions in core/extra");
            println!("    -Si <pkg>    Display detailed package information");
            println!("    -Q           List all installed packages");
            println!("    -Qi <pkg>    Show installed package metadata");
            println!("    -Ql <pkg>    List files owned by package");
            println!("    -R <pkg>     Remove installed package");
        }
        "yay" | "paru" => {
            println!("NAME\n    {} - AUR Helper & Pacman Wrapper written in Rust\n", topic);
            println!("SYNOPSIS\n    {} [options] [aur-package]\n", topic);
            println!("DESCRIPTION\n    Searches, downloads PKGBUILDs, compiles, and installs packages");
            println!("    from both official Arch repositories (core/extra) and the AUR.\n");
            println!("EXAMPLES\n    {} -S cbonsai pipes-rs cmatrix", topic);
            println!("    {} -Ss fetch", topic);
            println!("    {}            (Aliases to {} -Syu)", topic, topic);
        }
        "makepkg" => {
            println!("NAME\n    makepkg - make packages compatible for use with pacman\n");
            println!("SYNOPSIS\n    makepkg [-si]\n");
            println!("DESCRIPTION\n    Reads the PKGBUILD file in the current working directory,");
            println!("    compiles the source in Rust, generates a .pkg.tar.zst archive,");
            println!("    and installs it into the local pacman database.");
        }
        _ => {
            println!("NAME\n    {} - RustOS built-in system command or package utility\n", topic);
            println!("DESCRIPTION\n    Part of the RustOS x86_64 userland suite. Run '{} --help' or 'help'.", topic);
        }
    }
}

fn cmd_help() {
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("==========================================================================");
    println!("  RustOS v0.2.0 (Inspired by Arch Linux) - Interactive Shell Reference");
    println!("==========================================================================");

    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!(" [Arch Linux Repository Packages & AUR]");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("  pacman -Syu            Sync Arch mirror databases & check for upgrades");
    println!("  pacman -S <pkg>        Install Arch packages (cmatrix, btop, cowsay, git...)");
    println!("  pacman -Ss <query>     Search packages from Arch repositories");
    println!("  pacman -Q / -Qi / -R   List, inspect, or remove installed packages");
    println!("  yay -S <pkg> / paru    Install from AUR (cbonsai, pipes-rs, hyfetch, tty-clock)");
    println!("  yay -Ss <query>        Search across standard repositories and AUR");
    println!("  makepkg -si            Build & install a custom package from a PKGBUILD file");

    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("\n [File System & Navigation (Tab Completion Enabled)]");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("  ls [-la] [dir]         List directory contents with permissions & colors");
    println!("  cd <dir> / pwd         Change or print current working directory");
    println!("  mkdir / rmdir / touch  Create or remove directories and files");
    println!("  cat / head / tail / wc View or count lines/words/bytes in files");
    println!("  cp / mv / rm [-rf]     Copy, move, or delete files and directories");
    println!("  grep / find / tree     Search text inside files or visualize directory tree");
    println!("  nano <file>            Interactive terminal text editor (:wq save, ESC exit)");

    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("\n [System, Hardware & Arch Eye-Candy]");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("  neofetch / fastfetch   Display Arch/RustOS system info & ASCII art logo");
    println!("  lscpu / lspci / lsblk  Inspect real x86_64 CPUID, PCI bus devices, and disks");
    println!("  free -h / df -h / ps   Check memory usage, mounted filesystems, and processes");
    println!("  htop / btop / top      Interactive real-time CPU & memory monitor");
    println!("  date / timedatectl     Read hardware CMOS Real-Time Clock (UTC)");
    println!("  dmesg / systemctl      View kernel ring buffer logs and system services");
    println!("  rice                   Run an automated Arch Linux terminal ricing showcase");
    println!("  reboot / shutdown      Restart or power off the machine");
}

fn cmd_echo(args: &[&str]) {
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("{}", args.join(" "));
}

fn cmd_info() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("--------------------------------------------------");
    println!(" RustOS - x86_64 Bare-Metal OS in Rust");
    println!("--------------------------------------------------");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" Design       : Inspired by Arch Linux");
    println!(" Architecture : x86_64 (64-bit Long Mode)");
    println!(" Kernel Model : Monolithic / Freestanding no_std Rust");
    println!(" File System  : SimpleFS v2 (Hierarchical FHS + ATA PIO)");
    println!(" Packages     : Uses packages from Arch Linux & AUR (pacman, yay, paru)");
    println!(" Drivers      : VGA Framebuffer, PS/2 Keyboard, ATA PIO, PCI, CMOS RTC, UART");
    println!(" Compiler     : rustc 1.101.0-nightly");
}

fn cmd_uptime() {
    let secs = uptime_seconds();
    let ticks = crate::interrupts::ticks();
    let minutes = secs / 60;
    let remaining_secs = secs % 60;
    let hours = minutes / 60;
    let remaining_mins = minutes % 60;
    let dt = crate::rtc::read_rtc();

    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(
        " {:02}:{:02}:{:02} up {:02}h {:02}m {:02}s, 1 user, load average: 0.04, 0.02, 0.00 ({} ticks)",
        dt.hour,
        dt.minute,
        dt.second,
        hours,
        remaining_mins,
        remaining_secs,
        ticks
    );
}

fn cmd_colors() {
    println!("16-Color Terminal Palette:");
    let colors = [
        (ColorCode::Black, "Black"),
        (ColorCode::Blue, "Blue"),
        (ColorCode::Green, "Green"),
        (ColorCode::Cyan, "Cyan"),
        (ColorCode::Red, "Red"),
        (ColorCode::Magenta, "Magenta"),
        (ColorCode::Brown, "Brown"),
        (ColorCode::LightGray, "LightGray"),
        (ColorCode::DarkGray, "DarkGray"),
        (ColorCode::LightBlue, "LightBlue"),
        (ColorCode::LightGreen, "LightGreen"),
        (ColorCode::LightCyan, "LightCyan"),
        (ColorCode::LightRed, "LightRed"),
        (ColorCode::LightMagenta, "LightMagenta"),
        (ColorCode::Yellow, "Yellow"),
        (ColorCode::White, "White"),
    ];

    for (code, name) in colors.iter() {
        vga::set_colors(*code, ColorCode::Black);
        print!("  [■■■] ");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("{:12} (code {})", name, *code as u8);
    }
}

fn cmd_mem() {
    let (start, size, initialized) = get_heap_info();
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Kernel Heap Status:");
    println!("  Initialized : {}", initialized);
    println!("  Base Address: {:#018x}", start);
    println!("  Total Size  : {} KB ({} MB)", size / 1024, size / (1024 * 1024));
}

fn cmd_alloc() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("Testing dynamic kernel memory allocations:");
    let heap_value = Box::new(0xCAFE_BABE_u64);
    println!("  1. Box allocation: value = {:#x} at {:p}", *heap_value, heap_value);

    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(i * i);
    }
    println!("  2. Vec allocation: 10 squares = {:?}", vec);

    let test_str = format!("RustOS dynamic string allocation ({})", 2026);
    println!("  3. String format: \"{}\"", test_str);
}

fn cmd_calc(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: calc <number> <operator> <number> ...");
        return;
    }
    let full_expr = args.join(" ");
    match simple_eval(&full_expr) {
        Ok(result) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Result: {}", result);
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error: {}", err);
        }
    }
}

fn simple_eval(expr: &str) -> Result<i64, &'static str> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("Empty expression");
    }
    let mut current_val: i64 = tokens[0].parse().map_err(|_| "Invalid number")?;
    let mut i = 1;
    while i < tokens.len() {
        let op = tokens[i];
        if i + 1 >= tokens.len() {
            return Err("Missing operand after operator");
        }
        let next_val: i64 = tokens[i + 1].parse().map_err(|_| "Invalid number")?;
        match op {
            "+" => current_val = current_val.checked_add(next_val).ok_or("Addition overflow")?,
            "-" => current_val = current_val.checked_sub(next_val).ok_or("Subtraction underflow")?,
            "*" | "x" => current_val = current_val.checked_mul(next_val).ok_or("Multiplication overflow")?,
            "/" => {
                if next_val == 0 {
                    return Err("Division by zero");
                }
                current_val /= next_val;
            }
            "%" => {
                if next_val == 0 {
                    return Err("Modulo by zero");
                }
                current_val %= next_val;
            }
            _ => return Err("Unsupported operator. Use +, -, *, /, %"),
        }
        i += 2;
    }
    Ok(current_val)
}

fn cmd_disk() {
    let ata = crate::ata::ATA_DRIVE.lock();
    if let Some(info) = ata.get_info() {
        println!("ATA Hard Drive Information:");
        println!("  Model    : {}", info.model_str());
        println!("  Capacity : {} MB ({} GB)", info.size_mb, info.size_mb / 1024);
        println!("  Sectors  : {} (512 bytes/sector)", info.sectors);
    } else {
        let (total_kb, used_kb, _) = fs::stats();
        println!("Running on in-memory rootfs ({} KB used / {} KB total).", used_kb, total_kb);
    }
}

fn cmd_format() {
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("Formatting SimpleFS v2 and restoring Arch Linux base directory hierarchy...");
    match crate::fs::FS.lock().format() {
        Ok(()) => {
            crate::pkg::init();
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Filesystem formatted and base Arch layout restored.");
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Format error: {}", err);
        }
    }
}
