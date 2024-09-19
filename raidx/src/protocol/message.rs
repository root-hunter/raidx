extern crate strum_macros;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use websocket::OwnedMessage;

use crate::models::{files::{NewRFile, RFile}, nodes::RNode};

#[derive(Serialize, Deserialize, Debug, Clone, strum_macros::Display)]
pub enum RMessageType {
    OK,
    Error,
    SyncFiles,
    FileAdded,
    UidRequest,
    UidResponse
}

#[derive(Debug)]
pub enum RContentKind {
    OK(RMOk),
    Error(RMError),
    SyncFiles(RMSyncFiles),
    FileAdded(RMFileAdded),
    UidRequest(RMUidRequest),
    UidResponse(RMUidRespose),
}

pub trait RMessageTrait<T> {
    fn from_slice(data: Vec<u8>) -> Result<T, serde_json::Error>;
    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMessage {
    pub _type: RMessageType,
    pub data: Option<Vec<u8>>
}

impl RMessageTrait<RMessage> for RMessage {
    fn from_slice(data: Vec<u8>) -> Result<RMessage, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMOk;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMError {
    pub text: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMSyncFiles {
    pub files: Vec<NewRFile>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMFileAdded {
    pub file: RFile
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMUidRequest;

impl RMessageTrait<RMUidRequest> for RMUidRequest {
    fn from_slice(data: Vec<u8>) -> Result<RMUidRequest, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RMUidRespose {
    pub uid: String
}

impl RMessageTrait<RMUidRespose> for RMUidRespose {
    fn from_slice(data: Vec<u8>) -> Result<RMUidRespose, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}



impl RMessage {
    pub fn to_ws_message(&self) -> Result<OwnedMessage, serde_json::Error> {
        let message = self.to_slice();

        if let Ok(message) = message {
            return Ok(OwnedMessage::Binary(message));
        } else {
            return Err(message.unwrap_err());
        }
    }

    pub fn has_data(self) -> bool {
        return self.data.is_some();
    }
 
    pub fn get_content(self, conn: &mut SqliteConnection) -> RContentKind {
        return match self._type {
            RMessageType::OK => RContentKind::OK(RMOk{}),
            RMessageType::FileAdded => {
                if self.clone().has_data() {
                    let data = self.data.unwrap();
                    let file = serde_json::from_slice(data.as_slice());

                    if file.is_ok() {
                        let content = RContentKind::FileAdded(RMFileAdded{
                            file: file.unwrap()
                        });

                        return content;
                    } else {
                        return RContentKind::Error(RMError { text: format!("{}", file.unwrap_err()) });
                    }
                } else {
                    return RContentKind::Error(RMError { text: format!("data can't be None") })
                }
            },
            RMessageType::UidRequest => RContentKind::UidRequest(RMUidRequest{}),
            RMessageType::UidResponse => {
                let node = RNode::get_local(conn);

                if node.is_some() {
                    let node = node.unwrap();
                    
                    return RContentKind::UidResponse(RMUidRespose { uid: node.uid });
                } else {
                    return RContentKind::Error(RMError { text: format!("error to fetch local node from db") });
                }
            },
            RMessageType::Error => RContentKind::Error(RMError{
                text: String::from("dada")
            }),
            RMessageType::SyncFiles => {

                if self.data.is_some() {
                    let data = self.data.unwrap();

                    let content = serde_json::from_slice(data.as_slice());

                    if content.is_ok() {
                        return RContentKind::SyncFiles(content.unwrap());
                    } else {
                        return RContentKind::Error(RMError { text: format!("{}", content.unwrap_err()) })
                    }
                } else {
                    return RContentKind::Error(RMError { text: format!("data can't be None") })
                }
            },
        };
    }
}