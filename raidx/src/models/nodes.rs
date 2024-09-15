use std::{sync::mpsc::channel, thread};

use crate::{
    models::utils::error::RDatabaseError, peers::server::RServer, protocol::message::{RMessage, RMessageTrait}, schema::nodes::{self, all_columns}, utils::configs::RConfigNode
};

use diesel::{associations::HasTable, prelude::*};
use log::error;
use uuid::Uuid;
use websocket::{ClientBuilder, OwnedMessage};

#[derive(Queryable, Selectable, Insertable, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = nodes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RNode {
    pub uid: String,

    pub host: String,
    pub port: i32,

    pub local: bool,
}

impl RNode {
    pub fn get_by_host_and_port(
        conn: &mut SqliteConnection,
        search_host: String,
        search_port: i32,
    ) -> Option<RNode> {
        use crate::schema::nodes::dsl::*;

        let result = nodes::table()
            .select(all_columns)
            .filter(host.eq(search_host).and(port.eq(search_port)))
            .first::<RNode>(conn);

        if result.is_ok() {
            return Some(result.unwrap());
        } else {
            return None;
        }
    }

    pub fn get_all(conn: &mut SqliteConnection) -> Result<Vec<RNode>, RDatabaseError> {
        use crate::schema::nodes::dsl::*;

        let results = nodes::table().select(all_columns).load::<RNode>(conn);

        if results.is_ok() {
            return Ok(results.unwrap());
        } else {
            return Err(RDatabaseError::DieselResult(results.unwrap_err()));
        }
    }

    pub fn get_by_local_flag(conn: &mut SqliteConnection, flag: bool) -> Option<RNode> {
        use crate::schema::nodes::dsl::*;

        let result = nodes::table()
            .select(all_columns)
            .filter(local.eq(flag))
            .first::<RNode>(conn);

        if result.is_ok() {
            return Some(result.unwrap());
        } else {
            return None;
        }
    }

    pub fn get_local(conn: &mut SqliteConnection) -> Option<RNode> {
        return RNode::get_by_local_flag(conn, true);
    }

    pub fn get_local_or_create(
        conn: &mut SqliteConnection,
        data_host: String,
        data_port: i32,
    ) -> Option<RNode> {
        let node = RNode::get_by_local_flag(conn, true);

        if node.is_some() {
            return node;
        } else {
            let result = RNode::create_local(conn, data_host, data_port);

            if result.is_ok() {
                return Some(result.unwrap());
            } else {
                return None;
            }
        }
    }

    pub fn get_others(conn: &mut SqliteConnection) -> Option<Vec<RNode>> {
        use crate::schema::nodes::dsl::*;

        let results = nodes::table()
            .select(all_columns)
            .filter(local.eq(false))
            .load::<RNode>(conn);

        if results.is_ok() {
            return Some(results.unwrap());
        } else {
            return None;
        }
    }

    pub fn get_by_uid(
        conn: &mut SqliteConnection,
        search_uid: String,
    ) -> Result<RNode, RDatabaseError> {
        use crate::schema::nodes::dsl::*;

        let result = nodes::table()
            .select(all_columns)
            .filter(uid.eq(search_uid))
            .first(conn);

        if result.is_ok() {
            return Ok(result.unwrap());
        } else {
            return Err(RDatabaseError::DieselResult(result.unwrap_err()));
        }
    }

    pub fn create(
        conn: &mut SqliteConnection,
        data_host: String,
        data_port: i32,
        data_local: bool,
    ) -> Result<RNode, RDatabaseError> {
        let node = RNode {
            local: data_local,
            uid: Uuid::new_v4().to_string(),
            host: data_host,
            port: data_port,
        };

        let result = diesel::insert_into(nodes::table)
            .values(&node)
            .execute(conn);

        if result.is_ok() {
            return RNode::get_by_uid(conn, node.uid);
        } else {
            return Err(RDatabaseError::DieselResult(result.unwrap_err()));
        }
    }

    pub fn create_local(
        conn: &mut SqliteConnection,
        data_host: String,
        data_port: i32,
    ) -> Result<RNode, RDatabaseError> {
        return RNode::create(conn, data_host, data_port, true);
    }

    pub fn create_other(
        conn: &mut SqliteConnection,
        data_host: String,
        data_port: i32,
    ) -> Result<RNode, RDatabaseError> {
        return RNode::create(conn, data_host, data_port, false);
    }

    pub fn request_uid(server: RConfigNode) {
        let connection_url = RServer::connection_url(server);
        let client = ClientBuilder::new(connection_url.as_str())
            .unwrap()
            .add_protocol("rust-websocket")
            .connect_insecure()
            .unwrap();

        let request = RMessage {
            _type: crate::protocol::message::RMessageType::UidRequest,
            data: None,
        };

        let data = request.to_slice();

        if data.is_ok() {
            let data = data.unwrap();
            let request = OwnedMessage::Binary(data);

            
        } else {
            error!("Error to convert request");
        }
    }

    pub fn connection_url(self, ssl: bool) -> String {
        let host = self.host;
        let port = self.port;
        
        let protocol = if ssl {
            "wss"
        } else {
            "ws"
        };

        
        return format!("{}://{}:{}", protocol, host, port);
    }
}
