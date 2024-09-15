extern crate websocket;

use std::thread;

use crate::models::nodes::RNode;
use crate::protocol::message::{RMessageTrait, RMUidRespose};
use crate::utils::configs::{self, RConfig, RConfigNode};

use super::super::protocol::message::{RContentKind, RMessage};

use diesel::SqliteConnection;
use diesel::prelude::*;

use log::warn;
use websocket::sync::Server;
use websocket::OwnedMessage;

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


pub fn init(configs_path: String) {
    let configs_path = configs_path.clone();
    let configs_path = std::path::Path::new(configs_path.as_str());
    let configs = RConfig::load_from_file(configs_path).unwrap();

    let server_addr = format!("{}:{}", configs.clone().server.host, configs.clone().server.port);
    let server = Server::bind(server_addr).unwrap();

    let database_url = configs.clone().database.path;

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            //let mut conn = SqliteConnection::establish(database_url.as_str()).unwrap();
            
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let mut client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let message = OwnedMessage::Text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();
                
                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    },
                    OwnedMessage::Binary(message) => {
                        let data = RMUidRespose{
                            uid: "LSLSLSSLDKKDKDD-DDDDD_DDD-d".to_string()
                        }.to_slice().unwrap();

                        let message = RMessage{
                            _type: crate::protocol::message::RMessageType::UidResponse,
                            data: Some(data)
                        };

                        let message = message.to_slice().unwrap();
                        let message = OwnedMessage::Binary(message);

                        sender.send_message(&message).unwrap();
                        /* let message = RMessage::from_slice(message);

                        if message.is_ok() {
                            let message = message.unwrap();
                            let content = message.get_content(&mut conn);

                            match content {
                                RContentKind::SyncFiles(_content) => {
                                    //let files = content.files;

                                    
                                },
                                RContentKind::UidRequest(_) => {
                                    let node = RNode::get_local(&mut conn);

                                    if node.is_some() {
                                        let node = node.unwrap();

                                        let data = RMUidRespose{
                                            uid: node.uid
                                        }.to_slice().unwrap();

                                        let message = RMessage{
                                            request_type: crate::protocol::message::RMessageType::UidResponse,
                                            data: Some(data)
                                        };

                                        let message = message.to_slice().unwrap();
                                        let message = OwnedMessage::Binary(message);

                                        sender.send_message(&message).unwrap();
                                    } else {
                                        warn!("no local node into the database!");
                                    }
                                },
                                _ => {

                                }
                            }
                        } else {

                        } */
                    },
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}