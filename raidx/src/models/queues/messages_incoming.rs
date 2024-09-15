use std::time::{SystemTime, UNIX_EPOCH};

use diesel;
use diesel::{associations::HasTable, prelude::*};
use crate::models::utils::error::RDatabaseError;
use crate::protocol::message::RMessage;
use crate::schema::messages_incoming::{self, all_columns};

use super::messages::RMessageQueue;

#[derive(Queryable, Selectable, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = messages_incoming)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RMessagesIncoming {
    pub id: i32,
    pub uid: String,
    pub message_type: String,
    pub data: Option<Vec<u8>>,
    pub from: String,
    pub created_at: i32,
}

#[derive(Insertable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = messages_incoming)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RNewMessagesIncoming {
    pub uid: String,
    pub message_type: String,
    pub data: Option<Vec<u8>>,
    pub from: String,
    pub created_at: i32,
}

impl RMessageQueue<RMessagesIncoming> for RMessagesIncoming {
    fn push(conn: &mut SqliteConnection, node_uid: String, message: RMessage) -> Result<RMessagesIncoming, RDatabaseError>{
        let uid = uuid::Uuid::new_v4();
        let uid = uid.to_string();
        let from = node_uid;

        let message_type = message._type.to_string();
        let data = message.data;
        
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let created_at = created_at as i32;

        let message = RNewMessagesIncoming{
            uid,
            from,
            message_type,
            data,
            created_at
        };

        let result = diesel::insert_into(messages_incoming::table)
            .values(&message)
            .returning(all_columns)
            .load::<RMessagesIncoming>(conn);

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

    fn last(conn: &mut SqliteConnection) -> Option<RMessagesIncoming> {
        use crate::schema::messages_incoming::dsl::*;

        let result = messages_incoming::table()
            .select(messages_incoming::all_columns())
            .order_by(created_at.desc())
            .first::<RMessagesIncoming>(conn);

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
        return RMessagesIncoming::delete_by_id(conn, self.id);
    }

    fn pop(conn: &mut SqliteConnection) -> Result<RMessagesIncoming, RDatabaseError> {
        let message = RMessagesIncoming::last(conn);

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
}