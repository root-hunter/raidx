use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};

use crate::models::{files::NewRFile, nodes::RNode};
pub trait RChannelMessage<T> {
    fn from_slice(data: Vec<u8>) -> Result<T, serde_json::Error>;
    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RMessage {
    pub request_type: RMessageType,
    pub data: Option<Vec<u8>>
}

impl RChannelMessage<RMessage> for RMessage {
    fn from_slice(data: Vec<u8>) -> Result<RMessage, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RMOk;

#[derive(Serialize, Deserialize, Debug)]
pub struct RMError {
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RMSyncFiles {
    pub files: Vec<NewRFile>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RMessageType {
    OK,
    Error,
    SyncFiles,
    UidRequest,
    UidResponse
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RMUidRequest;

impl RChannelMessage<RMUidRequest> for RMUidRequest {
    fn from_slice(data: Vec<u8>) -> Result<RMUidRequest, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RMUidRespose {
    pub uid: String
}

impl RChannelMessage<RMUidRespose> for RMUidRespose {
    fn from_slice(data: Vec<u8>) -> Result<RMUidRespose, serde_json::Error> {
        return serde_json::from_slice(data.as_slice());
    }

    fn to_slice(&self) -> Result<Vec<u8>, serde_json::error::Error>{
        return serde_json::to_vec(self);
    }
}

#[derive(Debug)]
pub enum RContentKind {
    OK(RMOk),
    Error(RMError),
    SyncFiles(RMSyncFiles),
    UidRequest(RMUidRequest),
    UidResponse(RMUidRespose),
}

impl RMessage {
   
 
    pub fn get_content(self, conn: &mut SqliteConnection) -> RContentKind {
        return match self.request_type {
            RMessageType::OK => RContentKind::OK(RMOk{}),
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