extern crate websocket;

use std::thread;
use std::thread::sleep;
use std::time::Duration;

use diesel::prelude::*;
use diesel::SqliteConnection;

use glob::glob;

use log::{error, info, warn};

use crate::models::files::{NewRFile, RFile};
use crate::models::nodes::RNode;
use crate::utils::configs::RConfig;

pub fn init_sync(configs: RConfig) {
    let configs = configs.clone();
    let database_url = configs.database.path.clone();

    let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();

    let local_node = RNode::get_local_or_create(&mut conn, "0.0.0.0".to_string(), 4000);

    if local_node.is_some() {
        let local_node = local_node.unwrap();

        let results = RFile::get_all(&mut conn);

        if results.is_ok() {
            let files = results.unwrap();

            for file in files {
                let path = file.abspath();
                let entry = std::path::Path::new(path.as_str());

                if !entry.exists() {
                    RFile::remove_from_uid(&mut conn, &file.uid);
                    info!(target: "START_SYNC", "FILE REMOVED: {:?} ({})", file.abspath(), file.uid);
                } else {
                }
            }
        } else {
            error!(target: "START_SYNC", "not valid result from database");
        }

        let raidx_path = configs.folder_path.clone();

        let raidx_check_pattern = format!("{}/**/*", raidx_path);
        let results = glob(&raidx_check_pattern);

        if results.is_ok() {
            let results = results.unwrap();

            for entry in results {
                let entry = entry.unwrap();

                if entry.is_file() {
                    let file = NewRFile::from_entry(&mut conn, &local_node, &entry);

                    if file.is_ok() {
                        let file = file.unwrap();

                        info!(target: "START_SYNC", "FILE ADDED: {:?} ({})", entry, file.uid);
                    } else {
                        //error!(target: "START_SYNC", "error to add file: {:?}", entry);
                    }
                }
            }
        } else {
            error!(target: "START_SYNC", "not valid RAIDX check pattern: {}", raidx_check_pattern);
        }
    } else {
        error!("Not valid local node");
    }
}

pub fn init(configs: RConfig) {
    init_sync(configs.clone());
    thread::spawn(move || {
        let database_url = configs.database.path.clone();
        let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();
        let local_node = RNode::get_local_or_create(&mut conn, "0.0.0.0".to_string(), 4000);

        if local_node.is_some() {
            let local_node = local_node.unwrap();

            loop {
                let _new_files = sync_files(&configs, &mut conn, &local_node);

                for _node in configs.clone().nodes {}

                sleep(Duration::from_secs(configs.synchronizer.timeout as u64));
            }
        } else {
            error!("Not valid local nde");
        }
    });
}

fn sync_files(configs: &RConfig, conn: &mut SqliteConnection, local_node: &RNode) -> Vec<RFile> {
    let pattern = format!("{}/**/*", configs.folder_path);
    let results = glob(&pattern).unwrap();

    let mut new_files = Vec::<RFile>::new();

    for entry in results {
        if entry.is_ok() {
            let entry = entry.unwrap();

            if entry.is_file() {
                //println!("UID: {:x}\nFOLDER: {}\nfilename: {} ", uid, folder, name);
                let file = NewRFile::from_entry(conn, local_node, &entry);

                if file.is_ok() {
                    let file = file.unwrap();

                    info!("new file was added {}", file.filename);
                    new_files.push(file);
                } else {
                    //warn!("{:?}", file.unwrap_err());
                }
            }
        } else {
            warn!("{:?}", entry.unwrap_err());
        }
    }

    return new_files;
}
