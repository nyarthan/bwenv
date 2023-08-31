use std::{
    env,
    io::{self, Read, Write},
    process::{self, Command, Stdio},
};

use bitwarden::secrets_manager::secrets::SecretIdentifiersByProjectRequest;
use bitwarden::{
    auth::request::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    secrets_manager::secrets::SecretGetRequest,
    Client,
};
use clap::Parser;
use uuid::Uuid;

mod cli;
mod config;

use crate::{cli::{Args, CLI}, config::get_config};

const BW_IDENTITY_URL: &str = "https://identity.bitwarden.com";
const BW_API_URL: &str = "https://api.bitwarden.com";
const BW_USER_AGENT: &str = "Bitwarden Rust-SDK";
const BW_DEVICE_TYPE: DeviceType = DeviceType::SDK;

fn get_profile_from_env(env_var_names: &Vec<String>) -> Option<String> {
    let mut existing_env_vars = Vec::new();

    for env_var_name in env_var_names {
        if let Ok(env_var_value) = env::var(env_var_name) {
            existing_env_vars.push(env_var_value);
        }
    }

    existing_env_vars.first().map(|s| s.to_string())
}

fn evaluate_config(config: &config::Config) -> [String; 1] {
    let env_var_names = config.environment.as_ref().unwrap();
    let env_profile = get_profile_from_env(env_var_names)
        .expect("please provide a profile via environment variables");

    let profile = config.profiles.get(&env_profile).expect(&format!(
        "Profile '{}' not found in config file",
        env_profile
    ));

    let project = &config.project;

    let project = profile.project.as_ref().unwrap_or_else(|| {
        project
            .as_ref()
            .expect("please provide a project via environment variables or config file")
    });

    [project.to_string()]
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = CLI::new();
    let (program, program_args) = cli.get_program();

    let config = get_config().expect("could not find config file");
    let [project] = evaluate_config(&config);

    let mut cmd = Command::new(program);

    cmd.args(program_args);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Ok(mut child) = cmd.spawn() {
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let mut buffer = [0; 1024];

        // Create separate threads to handle stdout and stderr
        let stdout_thread = std::thread::spawn(move || loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = String::from_utf8(buffer[0..n].to_vec()) {
                        print!("{}", s);
                        io::stdout().flush().expect("Failed to flush stdout");
                    }
                }
                Err(err) => {
                    eprintln!("Error reading child process stdout: {:?}", err);
                    break;
                }
            }
        });

        let stderr_thread = std::thread::spawn(move || loop {
            match stderr.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = String::from_utf8(buffer[0..n].to_vec()) {
                        eprint!("{}", s);
                        io::stderr().flush().expect("Failed to flush stderr");
                    }
                }
                Err(err) => {
                    eprintln!("Error reading child process stderr: {:?}", err);
                    break;
                }
            }
        });

        // Wait for the child process to finish and close the threads
        let _ = child.wait();
        stdout_thread.join().expect("stdout thread panicked");
        stderr_thread.join().expect("stderr thread panicked");
    }

    cmd.env("TEST_ENV", "BLAH");

    process::exit(0);

    let mut bw_client = Client::new(Some(ClientSettings {
        identity_url: BW_IDENTITY_URL.to_string(),
        api_url: BW_API_URL.to_string(),
        user_agent: BW_USER_AGENT.to_string(),
        device_type: BW_DEVICE_TYPE,
    }));

    bw_client
        .access_token_login(&AccessTokenLoginRequest {
            access_token: env::var("BWS_ACCESS_TOKEN").unwrap(),
        })
        .await
        .unwrap();

    let secrets_by_project_request = SecretIdentifiersByProjectRequest {
        project_id: Uuid::parse_str(project.as_str()).unwrap(),
    };

    let secret_identifiers = bw_client
        .secrets()
        .list_by_project(&secrets_by_project_request)
        .await
        .unwrap();

    let mut secrets = Vec::new();

    for secret_identifier in secret_identifiers.data {
        let secret_get_request = SecretGetRequest {
            id: secret_identifier.id,
        };

        let secret = bw_client.secrets().get(&secret_get_request).await.unwrap();

        secrets.push([secret.key, secret.value]);
    }

    println!("Secrets: {:#?}", secrets);
}
