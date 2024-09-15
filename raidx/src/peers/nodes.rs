use std::collections::HashMap;
use std::iter::Map;
use std::net::TcpStream;

use diesel::prelude::*;
use diesel::SqliteConnection;
use log::{info, warn, error};
use websocket::futures::Stream;

use websocket::sync::Client;
use websocket::ClientBuilder;
use websocket::OwnedMessage;

use websocket::futures::Future;
use websocket::WebSocketError;

use crate::protocol::message::RMessage;
use crate::{models::nodes::RNode, utils::configs::RConfig};
pub struct RNodesPool {
    pub nodes: HashMap<String, Client<TcpStream>>
}

impl RNodesPool {
    pub fn load_connections(conn: &mut SqliteConnection) -> Option<Self> {
        let mut pool = RNodesPool{
            nodes: HashMap::new()
        };
        let result = RNode::get_others(conn);

        if result.is_some() {
            let result = result.unwrap();

            for _node in result {
                let connection_url = _node.clone().connection_url(false);
                let client = ClientBuilder::new(&connection_url)
                    .expect("Client error")
                    .add_protocol("rust-websocket")
                    .async_connect_insecure();
            }

            return Some(pool);
        } else {
            warn!("can't get other nodes from db");

            return None;
        }
    }
}

pub fn init(configs: RConfig) {
    let nodes = configs.nodes;

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
