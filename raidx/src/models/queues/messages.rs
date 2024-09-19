use diesel::SqliteConnection;
use crate::protocol::message::RMessage;
use crate::models::utils::error::RDatabaseError;

pub trait RMessageQueue<T> {
    fn push(conn: &mut SqliteConnection, node_uid: String, message: RMessage) -> Result<T, RDatabaseError>;
    fn last(conn: &mut SqliteConnection) -> Option<T>;
    fn last_n(conn: &mut SqliteConnection, n: usize) -> Option<Vec<T>>;
    fn delete_by_id(conn: &mut SqliteConnection, id: i32) -> Result<(), RDatabaseError>;
    fn delete(&self, conn: &mut SqliteConnection) -> Result<(), RDatabaseError>;
    fn pop(conn: &mut SqliteConnection) -> Result<T, RDatabaseError>;
    fn to_message(&self) -> Option<RMessage>;
}