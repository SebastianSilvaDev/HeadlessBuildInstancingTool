use clap::Parser;
use firebase_rs::*;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Debug, Serialize, Deserialize)]
struct GlobalConfigs {
    firebase_url: String,
    backend_url: String,
    user_prefix: String,
    path_to_auth: String,
    test_psw: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser)]
    executable_name: String,
    #[clap(short = 'i', long, value_parser, default_value = "20")]
    number_of_instances: i32,
    #[clap(long = "nullRHI")]
    null_rhi: bool,
    #[clap(short = 's', long, value_parser, default_value = "20")]
    time_to_stop: u64,
    #[clap(long = "wait-time", value_parser)]
    time_between_executions: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StressTestVariables {
    #[serde(rename(serialize = "user-number"))]
    user_number: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginData {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenData {
    token: String,
}

pub fn execute(exe: &str, args: &[&str]) -> Child {
    let command = Command::new(exe).args(args).spawn().expect("cant spawn");
    return command;
}

async fn get_number(firebase: &Firebase) -> i32 {
    let number_responded = firebase.at("user-number").get::<i32>().await;

    if number_responded.is_ok() {
        let n = number_responded.ok().unwrap();
        println!("Value: {n}");
        return n;
    }
    let _error = number_responded.err().unwrap().to_string();
    panic!("{_error}");
}

async fn set_next_number_of_users(
    firebase: &Firebase,
    current_number_of_users: i32,
    users_to_spawn: i32,
) {
    let next_number_of_users = current_number_of_users + users_to_spawn;

    let new_number = StressTestVariables {
        user_number: next_number_of_users,
    };

    let update_response = firebase.update(&new_number).await;

    if update_response.is_err() {
        let _error = update_response.err().unwrap().to_string();
        panic!("{_error}");
    }
}

async fn get_user_token(
    username: String,
    password: String,
    configurations: &GlobalConfigs,
) -> Result<TokenData, ()> {
    let client = reqwest::Client::new();
    let mut login_backend_url = String::from(&configurations.backend_url);
    login_backend_url.push_str("api-token-auth/");

    let login_data = LoginData {
        username: username,
        password: password,
    };

    let response = client
        .post(login_backend_url)
        .form(&login_data)
        .send()
        .await;
    if response.is_err() {
        let err = response.err().unwrap();
        panic!("{err}");
    }

    let body = response.ok().unwrap().json::<TokenData>().await;

    if body.is_err() {
        panic!("No Token");
    }

    let token_data = body.ok().unwrap();
    return Ok(token_data);
}

fn safe_auth_json(token_data: TokenData, configurations: &GlobalConfigs) {
    let mut path = String::from("./");
    path.push_str(&configurations.path_to_auth);
    path.push_str("/auth.json");
    println!("{path}");
    let _result = std::fs::write(path, serde_json::to_string_pretty(&token_data).unwrap());
    if _result.is_err() {
        panic!("Could't safe file");
    }
}

async fn update_user_token(
    current_user_number: i32,
    configurations: &GlobalConfigs,
) -> Result<(), ()> {
    let mut username: String = String::from(&configurations.user_prefix);
    username.push_str(&current_user_number.to_string());
    println!("{username}");
    let token_data = get_user_token(
        username,
        String::from(&configurations.test_psw),
        &configurations,
    )
    .await?;
    safe_auth_json(token_data, &configurations);
    Ok(())
}

fn create_default_config_file() {
    let default_config = GlobalConfigs {
        firebase_url: String::from("default"),
        backend_url: String::from("default"),
        user_prefix: String::from("default"),
        path_to_auth: String::from("default"),
        test_psw: String::from("default")
    };
    let new_config = std::fs::File::create("./config.yml").expect("Error Creating File");
    serde_yaml::to_writer(new_config, &default_config).expect("Couldn't write on new file");
    println!("Created config file")
}

#[async_std::main]
async fn main() -> Result<(), ()> {
    let config_file_result =
        std::fs::File::open("config.yml");
    if config_file_result.is_err() {
        create_default_config_file();
        println!("Default Config Created, please fill up");
        return Err(());
    }
    let configs_result: Result<GlobalConfigs, serde_yaml::Error> =
        serde_yaml::from_reader(config_file_result.ok().unwrap());

    if configs_result.is_err() {
        create_default_config_file();
        println!("No valid yaml to parse");
        return Err(());
    }

    let configs = configs_result.ok().unwrap();

    let args: Args = Args::parse();

    let executable_name = args.executable_name;
    let mut path: String = String::from("./");
    path.push_str(&executable_name);
    path.push_str(".exe");
    if !(std::path::Path::new(&path).exists()) {
        println!("Path does not exists!");
        return Err(());
    }

    let numer_times: i32 = args.number_of_instances;

    let firebase = &Firebase::new(&configs.firebase_url)
        .unwrap()
        .at("stress-test-variables");

    let initial_number_of_user = get_number(firebase).await;

    set_next_number_of_users(firebase, initial_number_of_user, numer_times).await;

    let mut execution_args: Vec<&str> = Vec::new();
    if args.null_rhi {
        execution_args.push("-nullRHI");
    }
    println!("Runing {numer_times} times");
    let mut children: Vec<Child> = Vec::new();
    for _i in 1..(numer_times + 1) {
        update_user_token(initial_number_of_user + _i, &configs).await?;

        let child_result = execute(&path, &execution_args);
        children.push(child_result);
        println!("Process {_i} Spawned!");
        if args.time_between_executions.is_some() {
            std::thread::sleep(std::time::Duration::from_secs(
                args.time_between_executions.unwrap(),
            ));
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

    Ok(())
}
