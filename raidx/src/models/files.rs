use std::{os::unix::fs::MetadataExt, time::{SystemTime, UNIX_EPOCH}};

use crate::schema::files::{self, all_columns};

use diesel::{associations::HasTable, prelude::*};
use sha1::{Digest, Sha1};

use super::{utils::error::RDatabaseError, nodes::RNode};

#[derive(Queryable, Selectable, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RFile {
    pub id: i32,
    pub uid: String,
    pub node: String,

    pub folder: String,
    pub filename: String,
    pub size: i32,

    pub status: String,
    pub sync: bool,

    pub created_at: i32,
    pub modified_at: i32,

    pub updated_at: i32,
}

impl RFile {
    pub fn get_abspath(folder: String, filename: String) -> String {
        let path = std::path::Path::new(folder.as_str());
        return path.join(filename).to_str().unwrap().to_string();
    }

    pub fn remove_from_uid(conn: &mut SqliteConnection, search_uid: &String) -> usize {
        use crate::schema::files::dsl::*;

        let statement = diesel::delete(files::table().filter(uid.eq(search_uid)));
        let result = statement.execute(conn).unwrap();

        return result;
    }

    pub fn from_entry(conn: &mut SqliteConnection, entry: &std::path::Path) -> Option<Self> {
        let entry_uid = RFile::calc_uid(entry);
        let file = RFile::get_by_uid(conn, entry_uid);

        if file.is_some() {
            let file = file.unwrap();
            return Some(file);
        } else {
            return None;
        }
    }

    pub fn from_new_rfile(conn: &mut SqliteConnection, file: NewRFile) -> Option<Self> {
        return RFile::get_by_uid(conn, file.uid);
    }

    pub fn get_by_uid(conn: &mut SqliteConnection, file_uid: String) -> Option<Self> {
        use crate::schema::files::dsl::*;

        let result = files::table()
            .select(files::all_columns())
            .filter(uid.eq(file_uid))
            .first::<RFile>(conn);

        if result.is_ok() {
            let result = result.unwrap();
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn get_all(conn: &mut SqliteConnection) -> Result<Vec<Self>, ()> {
        use crate::schema::files::dsl::*;
        let result = files::table().select(all_columns).load::<RFile>(conn);
        if result.is_ok() {
            return Ok(result.unwrap());
        } else {
            return Err(());
        }
    }

    pub fn calc_uid(entry: &std::path::Path) -> String {
        let mut hasher = Sha1::new();
        let data = entry.as_os_str().to_str().unwrap().as_bytes();
        hasher.update(data);

        let digest = format!("{:X}", hasher.finalize());
        return digest;
    }

    pub fn abspath(&self) -> String {
        return RFile::get_abspath(self.folder.clone(), self.filename.clone());
    }

    pub fn check_sync(&mut self, conn: &mut SqliteConnection) -> Result<usize, RDatabaseError> {
        use crate::schema::files::dsl::*;

        let abspath = self.abspath();
        let path = std::path::Path::new(abspath.as_str());
        let path_exists = path.exists();

        let result = match path_exists {
            true => diesel::update(files)
            .set(sync.eq(true))
            .execute(conn),
            false => diesel::update(files)
            .set(sync.eq(false))
            .execute(conn)
        };

        if result.is_ok() {
            self.sync = path_exists;
            return Ok(result.unwrap());
        } else {
            return Err(RDatabaseError::DieselResult(result.unwrap_err()));
        }
    }

    pub fn refresh(&mut self, conn: &mut SqliteConnection) -> Result<&mut Self, RDatabaseError> {
        let result = RFile::get_by_uid(conn, self.uid.clone());

        if result.is_some() {
            let result = result.unwrap();
            
            self.id = result.id;
            self.uid = result.uid;
            self.node = result.node;

            self.folder = result.folder;
            self.filename = result.filename;
            self.size = result.size;

            self.status = result.status;
            self.sync = result.sync;

            self.created_at = result.created_at;
            self.modified_at = result.modified_at;
            self.updated_at = result.updated_at;

            return Ok(self);
        } else {
            return Err(RDatabaseError::EntryNotExists);
        }
    }
}


#[derive(Insertable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewRFile {
    pub uid: String,
    pub node: String,

    pub folder: String,
    pub filename: String,
    pub size: i32,

    pub status: String,

    pub created_at: i32,
    pub modified_at: i32,

    pub updated_at: i32,
}

impl NewRFile {
    pub fn from_entry(
        conn: &mut SqliteConnection,
        node: &RNode,
        entry: &std::path::Path,
    ) -> Result<RFile, RDatabaseError> {
        let uid = RFile::calc_uid(entry);
        let folder = entry.parent().unwrap().to_str().unwrap().to_string();
        let filename = entry.file_name().unwrap().to_str().unwrap().to_string();

        let metadata = entry.metadata().unwrap();
        let size = metadata.size();
        let created_at = metadata
            .created()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let modified_at = metadata
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let file = NewRFile {
            uid: uid,
            node: node.uid.clone(),
            folder: folder,
            filename: filename,
            size: size as i32,
            status: String::from("READY"),
            created_at: created_at as i32,
            modified_at: modified_at as i32,
            updated_at: updated_at as i32
        };

        return file.save(conn);
    }

    pub fn to_rfile(self, conn: &mut SqliteConnection) -> Option<RFile> {
        return RFile::from_new_rfile(conn, self);
    }

    pub fn save(self, conn: &mut SqliteConnection) -> Result<RFile, RDatabaseError> {
        let result = diesel::insert_into(files::table)
            .values(&self)
            .execute(conn);

        if result.is_ok() {
            let file = self.to_rfile(conn);

            if file.is_some() {
                return Ok(file.unwrap());
            } else {
                return Err(RDatabaseError::EntryNotExists);
            }
        } else {
            return Err(RDatabaseError::DieselResult(result.err().unwrap()));
        }
    }
}
