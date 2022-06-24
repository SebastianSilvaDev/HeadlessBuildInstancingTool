use std::process::{Child, Command};
use sysinfo::{ProcessExt, System, SystemExt};

pub fn execute(exe: &str, args: &[&str]) -> Child {
    let command = Command::new(exe).args(args).spawn().expect("cant spawn");
    return command;
}

fn main() {
    let executable_name = std::env::args().nth(1).expect("No executable given!");
    let mut path: String = String::from("./");
    path.push_str(&executable_name);
    path.push_str(".exe");
    if !(std::path::Path::new(&path).exists()) {
        println!("Path does not exists!");
        return;
    }
    let times = std::env::args()
        .nth(2)
        .expect("No variable times has been created");
    let numer_times: i32 = times.parse().unwrap();
    println!("Runing {numer_times} times");
    let mut children: Vec<Child> = Vec::new();
    for _i in 1..(numer_times + 1) {
        let child_result = execute(&path, &["-nullRHI"]);
        children.push(child_result);
        println!("Process {_i} Spawned!");
    }
    println!("Finish spawning!");
    let wait_time_to_stop = std::env::args().nth(3);
    let mut sleep_time: u64 = 20;
    match wait_time_to_stop {
        Some(x) => {
            // Implicit parse to u64
            sleep_time = x.parse().unwrap();
            println!("Time to wait is {sleep_time} seconds");
        },
        None => println!("The time will be 20"),
    }
    std::thread::sleep(std::time::Duration::from_secs(sleep_time));
    println!("Finish Sleeping!");
    while !children.is_empty() {
        let mut child = children.pop().unwrap();
        let child_id = child.id();
        println!("Object with id {child_id}");
        child.kill().expect("Killed!");
    }
    let s = System::new_all();
    for process in s.processes_by_name(&executable_name) {
        println!("Killed process");
        process.kill();
    }
    println!("Have a great day testing!");
}
