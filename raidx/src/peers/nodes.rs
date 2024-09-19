use std::thread;
use std::time::Duration;

use crate::models::queues::messages::RMessageQueue;
use crate::models::queues::messages_outgoing::RMessageOutgoing;
use crate::utils::configs::RConfigNode;
use crate::{models::nodes::RNode, utils::configs::RConfig};
use diesel::prelude::*;
use diesel::SqliteConnection;
use log::{error, info, warn};
use websocket::OwnedMessage;
use websocket::{ClientBuilder, Message};

pub struct RServer;

impl RServer {    
    pub fn connection_url(server: RConfigNode) -> String {
        let host = server.host;
        let port = server.port;
        
        let protocol = if server.ssl {
            "wss"
        } else {
            "ws"
        };

        
        return format!("{}://{}:{}", protocol, host, port);
    }
}

pub fn load_nodes_from_configs(configs: &RConfig) {
    let nodes = configs.clone().nodes;

    let database_url = configs.database.path.clone();
    let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();

    for node_config in nodes {
        let host = node_config.clone().host;
        let port = node_config.clone().port as i32;

        let _node = RNode::get_by_host_and_port(&mut conn, host, port);

        if _node.is_some() {
            let _node = _node.unwrap();

            info!(
                "node already registred: {}:{} ({})",
                _node.host, _node.port, _node.uid
            );
        } else {
            let host = node_config.clone().host;
            let port = node_config.clone().port as i32;

            let _node = RNode::create_other(&mut conn, host, port);

            if _node.is_ok() {
                let _node = _node.unwrap();
                info!(
                    "node registred with success: {}:{} ({})",
                    _node.host, _node.port, _node.uid
                );
            } else {
                warn!(
                    "node not registred: {}:{}",
                    node_config.host, node_config.port
                );
            }
        }
    }
}

pub fn init(configs: RConfig) {
    let configs = configs.clone();

    thread::spawn(move || {
        let database_url = configs.database.path.clone();
        load_nodes_from_configs(&configs);
        let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();
        let nodes = RNode::get_others(&mut conn);
    
        thread::sleep(Duration::from_secs(10));
    
        if let Some(nodes) = nodes {
            for node in nodes {
                let configs = configs.clone();
                thread::spawn(move || {
                    let database_url = configs.database.path.clone();
                    let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();
                    let client = ClientBuilder::new(node.clone().connection_url(false).as_str())
                        .unwrap()
                        .add_protocol("rust-websocket")
                        .connect_insecure();
    
                    if let Ok(client) = client {
                        println!("Successfully connected");
    
                        let (mut receiver, mut sender) = client.split().unwrap();
    
                        let (tx, rx) = std::sync::mpsc::channel();
    
                        let tx_1 = tx.clone();
    
                        let send_loop = thread::spawn(move || {
                            loop {
                                // Send loop
                                let message = match rx.recv() {
                                    Ok(m) => m,
                                    Err(e) => {
                                        println!("Send Loop: {:?}", e);
                                        return;
                                    }
                                };
                                match message {
                                    OwnedMessage::Close(_) => {
                                        let _ = sender.send_message(&message);
                                        // If it's a close message, just send it and then return.
                                        return;
                                    }
                                    _ => (),
                                }
                                // Send the message
                                match sender.send_message(&message) {
                                    Ok(()) => (),
                                    Err(e) => {
                                        println!("Send Loop: {:?}", e);
                                        let _ = sender.send_message(&Message::close());
                                        return;
                                    }
                                }
                            }
                        });
    
                        let receive_loop = thread::spawn(move || {
                            // Receive loop
                            for message in receiver.incoming_messages() {
                                let message = match message {
                                    Ok(m) => m,
                                    Err(e) => {
                                        println!("Receive Loop: {:?}", e);
                                        let _ = tx_1.send(OwnedMessage::Close(None));
                                        return;
                                    }
                                };
                                match message {
                                    OwnedMessage::Close(_) => {
                                        // Got a close message, so send a close message and return
                                        let _ = tx_1.send(OwnedMessage::Close(None));
                                        return;
                                    }
                                    OwnedMessage::Ping(data) => {
                                        match tx_1.send(OwnedMessage::Pong(data)) {
                                            // Send a pong in response
                                            Ok(()) => (),
                                            Err(e) => {
                                                println!("Receive Loop: {:?}", e);
                                                return;
                                            }
                                        }
                                    }
                                    // Say what we received
                                    _ => println!("Receive Loop: {:?}", message),
                                }
                            }
                        });
    
                        loop {
                            //TODO get outcoming message from database
                            let messages = RMessageOutgoing::last_n(&mut conn, 10);
    
                            if let Some(messages) = messages {
                                for message in messages {
                                    if let Some(message) = message.to_message() {
                                        if let Ok(message) = message.to_ws_message() {
                                            match tx.send(message) {
                                                Ok(()) => (),
                                                Err(e) => {
                                                    println!("Main Loop: {:?}", e);
                                                    break;
                                                }
                                            }
                                        } else {
                                            warn!("can't convert RMessage to OwnedMessage")
                                        }
                                    } else {
                                        warn!("can't convert RMessageOutcoming to RMessage")
                                    }
                                }
                            } else {
                                //warn!("can't retrieve outcoming messages from database");
                            }
                        }
                    } else {
                        error!("Not valid client: {}", node.connection_url(false));
                    }
                });
            }
        } else {
            warn!("nodes not found");
        }
    });
}
