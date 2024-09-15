#[derive(Debug)]
pub enum RDatabaseError {
    EntryNotExists,
    EntryNotInsert,
    EntryNotDeleted,
    DieselResult(diesel::result::Error)
}
