extern crate websocket;

use std::thread;

use diesel::prelude::*;
use diesel::SqliteConnection;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

use log::{error, info, warn};

use crate::models::files::{NewRFile, RFile};
use crate::models::nodes::RNode;
use crate::models::queues::messages::RMessageQueue;
use crate::models::queues::messages_outgoing::RMessageOutgoing;
use crate::protocol::message::RMessage;
use crate::protocol::message::RMessageType;
use crate::utils::configs::RConfig;

pub fn init(configs: RConfig) {
    thread::spawn(move || {
        info!("SIZE NEW FILE: {}", std::mem::size_of::<NewRFile>());

        let raidx_path = configs.folder_path.clone();
        let database_url = configs.database.path.clone();

        info!("start to watch folder: {}", raidx_path);

        if let Err(error) = watch(database_url, raidx_path) {
            println!("{error:?}");
        }
    });
}


fn watch<P: AsRef<Path>>(database_url: String, path: P) -> notify::Result<()> {
    let mut conn =
        SqliteConnection::establish(database_url.as_str()).unwrap();
    let local_node = RNode::get_local_or_create(&mut conn, "0.0.0.0".to_string(), 4000);

    if local_node.is_some() {
        let local_node = local_node.unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    
        for res in rx {
            match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Modify(notify::event::ModifyKind::Data(
                        notify::event::DataChange::Any,
                    )) => {
                        info!("event{:?}", event);
    
                        let paths = event.paths;
                        let _from: &str = paths[0].to_str().unwrap();
    
                    }
                    notify::EventKind::Create(notify::event::CreateKind::File) => {
                        let paths = event.paths;
                        let file_path = paths.get(0).unwrap();
                        let entry = std::path::Path::new(file_path.to_str().unwrap());
    
                        let file = NewRFile::from_entry(&mut conn, &local_node, &entry);
    
                        if file.is_ok() {
                            let file = file.unwrap();
    
                            info!("DEAMON: new file was added {}", file.filename);
                            let data = serde_json::to_vec(&file);

                            if data.is_ok() {
                                let message = RMessage{
                                    _type: RMessageType::FileAdded,
                                    data: Some(data.unwrap()) 
                                };

                                let nodes = RNode::get_others(&mut conn);
                                if nodes.is_some() {
                                    for node in nodes.unwrap() {
                                        let message = RMessageOutgoing::push(&mut conn, node.uid, message.clone());
                                        
                                        info!("new message outgoing: {:?}", message);
                                    }
                                } else {
                                    warn!("can't get nodes");
                                }
                            } else {
                                warn!("error creating new file message");
                            }
                        } else {
                            warn!("error adding new file to db");
                        }
                    },
                    notify::EventKind::Remove(notify::event::RemoveKind::File) => {
                        let paths = event.paths;
                        let file_path = paths.get(0).unwrap();
                        let entry = std::path::Path::new(file_path.to_str().unwrap());
    
                        let file = RFile::from_entry(&mut conn, entry);
    
                        if file.is_some() {
                            let file = file.unwrap();
    
                            RFile::remove_from_uid(&mut conn, &file.uid);
    
                            info!("file removed: {}", entry.as_os_str().to_str().unwrap());
                        } else {
                            warn!("not file found: {}", entry.as_os_str().to_str().unwrap());
                        }
                    },
                    notify::EventKind::Remove(notify::event::RemoveKind::Folder) => {
                        println!("REMOVE FOLDER");
                    }
                    e => {
    
                        println!("OTHER {:?}", e);
                    }
                },
                Err(error) => println!("Error: {error:?}"),
            }
        }
    } else {
        error!("Not valid local node");
    }
  
    Ok(())
}