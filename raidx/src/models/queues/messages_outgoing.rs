use std::time::{SystemTime, UNIX_EPOCH};

use diesel;
use diesel::{associations::HasTable, prelude::*};
use crate::models::utils::error::RDatabaseError;
use crate::protocol::message::RMessage;
use crate::schema::messages_outgoing::{self, all_columns};

use super::messages::RMessageQueue;

#[derive(Queryable, Selectable, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = messages_outgoing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RMessagesOutgoing {
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
pub struct RNewMessagesOutgoing {
    pub uid: String,
    pub message_type: String,
    pub data: Option<Vec<u8>>,
    pub to: String,
    pub created_at: i32,
}


impl RMessageQueue<RMessagesOutgoing> for RMessagesOutgoing {
    fn push(conn: &mut SqliteConnection, node_uid: String, message: RMessage) -> Result<RMessagesOutgoing, RDatabaseError>{
        let uid = uuid::Uuid::new_v4();
        let uid = uid.to_string();
        let to = node_uid;

        let message_type = message._type.to_string();
        let data = message.data;
        
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let created_at = created_at as i32;

        let message = RNewMessagesOutgoing{
            uid,
            to,
            message_type,
            data,
            created_at
        };

        let result = diesel::insert_into(messages_outgoing::table)
            .values(&message)
            .returning(all_columns)
            .load::<RMessagesOutgoing>(conn);

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

    fn last(conn: &mut SqliteConnection) -> Option<RMessagesOutgoing> {
        use crate::schema::messages_incoming::dsl::*;

        let result = messages_incoming::table()
            .select(messages_incoming::all_columns())
            .order_by(created_at.desc())
            .first::<RMessagesOutgoing>(conn);

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
        return RMessagesOutgoing::delete_by_id(conn, self.id);
    }

    fn pop(conn: &mut SqliteConnection) -> Result<RMessagesOutgoing, RDatabaseError> {
        let message = RMessagesOutgoing::last(conn);

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