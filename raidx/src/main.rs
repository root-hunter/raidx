use std::path::PathBuf;

use clap::value_parser;
use env_logger;
use log::{info, warn};
use raidx::{peers, utils::configs::RConfig};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let matches = clap::Command::new("raidx")
        .author("Antonio Ricciardi")
        .about("Directory sharing protocol")
        .version("0.1.0")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("deamon")
                .about("RAIDX deamon")
                .subcommand_required(true)
                .arg(
                    clap::Arg::new("configs")
                        .long("configs")
                        .short('C')
                        .help("Config file path (.json)")
                        .action(clap::ArgAction::Set)
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .subcommand(clap::Command::new("start").about("Start RAIDX deamon"))
        )
        .get_matches();

        match matches.subcommand() {
            Some(("deamon", data)) => {
                let mut configs_path = data.get_raw("configs").unwrap();
                let configs_path = configs_path.next().unwrap().to_str().unwrap().to_string();
                
                match data.subcommand() {
                    Some(("start", _)) => {
                        let configs = RConfig::load_or_create_default(
                            configs_path.clone(),
                            "".to_string()
                        );
                    
                        if let Ok(configs) = configs {
                            peers::watcher::init(configs.clone());
                            peers::synchronizer::init(configs.clone());
                            peers::nodes::init(configs.clone());
                        } else {
                            panic!("Not valid configs file!");
                        }
                    }
                    _ => {
                        warn!("Not valid command");
                    }
                }
            },
            _ => {
                warn!("Not valid command");
            }
        }
}