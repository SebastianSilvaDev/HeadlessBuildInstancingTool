use std::process::{Child, Command};
use sysinfo::{ProcessExt, System, SystemExt};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser)]
    executable_name: String,
    #[clap(short='i', long, value_parser, default_value="20")]
    number_of_instances: i32,
    #[clap(long="nullRHI")]
    null_rhi: bool,
    #[clap(short='s', long, value_parser, default_value="20")]
    time_to_stop: u64,
    #[clap(long="wait-time", value_parser)]
    time_between_executions: Option<u64>
}

pub fn execute(exe: &str, args: &[&str]) -> Child {
    let command = Command::new(exe).args(args).spawn().expect("cant spawn");
    return command;
}

fn main() {
    let args: Args = Args::parse();

    let executable_name = args.executable_name;
    let mut path: String = String::from("./");
    path.push_str(&executable_name);
    path.push_str(".exe");
    if !(std::path::Path::new(&path).exists()) {
        println!("Path does not exists!");
        return;
    }
    let numer_times: i32 = args.number_of_instances;
    let mut execution_args: Vec<&str> = Vec::new();
    if args.null_rhi {
        execution_args.push("-nullRHI");
    }
    println!("Runing {numer_times} times");
    let mut children: Vec<Child> = Vec::new();
    for _i in 1..(numer_times + 1) {
        let child_result = execute(&path, &execution_args);
        children.push(child_result);
        println!("Process {_i} Spawned!");
        if args.time_between_executions.is_some() {
            std::thread::sleep(std::time::Duration::from_secs(args.time_between_executions.unwrap()));
        }
    }
    println!("Finish spawning!");
    let sleep_time: u64 = args.time_to_stop;
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
