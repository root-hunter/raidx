use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::utils::error::RDatabaseError;
use crate::protocol::message::{RMessage, RMessageType};
use crate::schema::messages_outgoing::{self, all_columns};
use diesel;
use diesel::{associations::HasTable, prelude::*};

use super::messages::RMessageQueue;

#[derive(Queryable, Selectable, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = messages_outgoing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RMessageOutgoing {
    pub id: i32,
    pub uid: String,
    pub message_type: String,
    pub data: Option<Vec<u8>>,
    pub to: String,
    pub created_at: i32,
}

#[derive(Insertable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = messages_outgoing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RNewMessageOutgoing {
    pub uid: String,
    pub message_type: String,
    pub data: Option<Vec<u8>>,
    pub to: String,
    pub created_at: i32,
}

impl RMessageQueue<RMessageOutgoing> for RMessageOutgoing {
    fn push(
        conn: &mut SqliteConnection,
        node_uid: String,
        message: RMessage,
    ) -> Result<RMessageOutgoing, RDatabaseError> {
        let uid = uuid::Uuid::new_v4();
        let uid = uid.to_string();
        let to = node_uid;

        let message_type = message._type.to_string();
        let data = message.data;

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let created_at = created_at as i32;

        let message = RNewMessageOutgoing {
            uid,
            to,
            message_type,
            data,
            created_at,
        };

        let result = diesel::insert_into(messages_outgoing::table)
            .values(&message)
            .returning(all_columns)
            .load::<RMessageOutgoing>(conn);

        if result.is_ok() {
            let result = result.unwrap();
            let message = result.first();

            if let Some(message) = message {
                return Ok(message.clone());
            } else {
                return Err(RDatabaseError::EntryNotInsert);
            }
        } else {
            return Err(RDatabaseError::DieselResult(result.err().unwrap()));
        }
    }

    fn last(conn: &mut SqliteConnection) -> Option<RMessageOutgoing> {
        use crate::schema::messages_incoming::dsl::*;

        let result = messages_incoming::table()
            .select(messages_incoming::all_columns())
            .order_by(created_at.desc())
            .first::<RMessageOutgoing>(conn);

        if result.is_ok() {
            let result = result.unwrap();
            return Some(result);
        } else {
            return None;
        }
    }

    fn delete_by_id(conn: &mut SqliteConnection, search_id: i32) -> Result<(), RDatabaseError> {
        use crate::schema::messages_incoming::dsl::*;

        let result = diesel::delete(messages_incoming::table())
            .filter(id.eq(search_id))
            .execute(conn);

        if result.is_ok() {
            return Ok(());
        } else {
            return Err(RDatabaseError::EntryNotDeleted);
        }
    }

    fn delete(&self, conn: &mut SqliteConnection) -> Result<(), RDatabaseError> {
        return RMessageOutgoing::delete_by_id(conn, self.id);
    }

    fn pop(conn: &mut SqliteConnection) -> Result<RMessageOutgoing, RDatabaseError> {
        let message = RMessageOutgoing::last(conn);

        if let Some(message) = message {
            let result = message.delete(conn);

            if result.is_ok() {
                return Ok(message);
            } else {
                return Err(result.unwrap_err());
            }
        } else {
            return Err(RDatabaseError::EntryNotExists);
        }
    }

    fn last_n(conn: &mut SqliteConnection, n: usize) -> Option<Vec<RMessageOutgoing>> {
        use crate::schema::messages_outgoing::dsl::*;

        let result = messages_outgoing::table()
            .select(messages_outgoing::all_columns())
            .order_by(created_at.desc())
            .limit(n as i64)
            .load::<RMessageOutgoing>(conn);

        if result.is_ok() {
            let result = result.unwrap();
            return Some(result);
        } else {
            return None;
        }
    }

    fn to_message(&self) -> Option<RMessage> {
        let message_type = serde_json::from_str(self.message_type.as_str());

        if message_type.is_ok() {
            return Some(RMessage {
                _type: message_type.unwrap(),
                data: self.data.clone(),
            });
        } else {
            return None;
        }
    }
}
