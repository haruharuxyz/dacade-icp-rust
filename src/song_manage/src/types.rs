use std::borrow::Cow;

use candid::{Encode, Decode};
use ic_stable_structures::{Storable, BoundedStorable};


#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Song {
   pub id: u64,
   pub file_name: String,
   pub mime_type: String,
   pub title: String,
   pub singer: String,
   pub genre: String,
   pub duration: u64,
   pub release_date: String,
   pub updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Song {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Song {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct SongPayload {
   pub file_name: String,
   pub mime_type: String,
   pub title: String,
   pub singer: String,
   pub genre: String,
   pub duration: u64,
   pub release_date: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct SongUpdate {
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub title: Option<String>,
    pub singer: Option<String>,
    pub genre: Option<String>,
    pub duration: Option<u64>,
    pub release_date: Option<String>,
}
#[derive(candid::CandidType, Deserialize, Serialize)]
pub enum Error {
    NotFound { msg: String },
    UploadFail { msg: String },
    UpdateFail { msg: String },
}