#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use validator::Validate;
use ic_cdk::api::{time, caller};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Song {
    id: u64,
    owner: String,
    file_name: String,
    mime_type: String,
    title: String,
    singer: String,
    genre: String,
    duration: u64,
    release_date: String,
    updated_at: Option<u64>,
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

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Song, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct SongPayload {
    #[validate(length(min = 3))]
    file_name: String,
    #[validate(length(min = 8))] // most common mime types have a minimum length of 8 e.g audio/aac, text/css
    mime_type: String,
    #[validate(length(min = 1))]
    title: String,
    #[validate(length(min = 1))]
    singer: String,
    genre: String,
    #[validate(range(min = 1))]
    duration: u64,
    release_date: String,
}

// Function to fetch a song stored in the canister
#[ic_cdk::query]
fn get_song(id: u64) -> Result<Song, Error> {
    match _get_song(&id) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("a song with id={} not found", id),
        }),
    }
}

// Function to add a song to the canister
#[ic_cdk::update]
fn upload_song(song: SongPayload) -> Result<Song, Error> {
    // Validates payload
    let check_payload = _check_input(&song);
    // Returns an error if validations failed
    if check_payload.is_err(){
        return Err(check_payload.err().unwrap());
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let song = Song {
        id,
        owner: caller().to_string(),
        file_name: song.file_name,
        mime_type: song.mime_type,
        title: song.title,
        genre: song.genre,
        duration: song.duration,
        release_date: song.release_date,
        singer: song.singer,
        updated_at: Some(time()),
    };
    // save song
    do_insert(&song);
    Ok(song)
}

// Function to update a song in the canister
#[ic_cdk::update]
fn update_song(id: u64, payload: SongPayload) -> Result<Song, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            // Validates payload
            let check_payload = _check_input(&payload);
            // Returns an error if validations failed
            if check_payload.is_err(){
                return Err(check_payload.err().unwrap());
            }
            // Validates whether caller is the owner of the song
            let check_if_owner = _check_if_owner(&song);
            if check_if_owner.is_err() {
                return Err(check_if_owner.err().unwrap())
            }
            // update song with payload's data
            song.file_name = payload.file_name;
            song.mime_type = payload.mime_type;
            song.title = payload.title;
            song.genre = payload.genre;
            song.duration = payload.duration;
            song.release_date = payload.release_date;
            song.singer = payload.singer;
            song.updated_at = Some(time());
            // save song
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

// helper method to perform insert.
fn do_insert(song: &Song) {
    STORAGE.with(|service| service.borrow_mut().insert(song.id, song.clone()));
}

// Function to delete a song
#[ic_cdk::update]
fn delete_song(id: u64) -> Result<Song, Error> {
    let song = _get_song(&id).expect(&format!("couldn't delete a song with id={}. song not found.", id));
    // Validates whether caller is the owner of the song
    let check_if_owner = _check_if_owner(&song);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap())
    }
    // delete song
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a song with id={}. song not found.", id),
        }),
    }
}

// a helper method to get a song by id
fn _get_song(id: &u64) -> Option<Song> {
    STORAGE.with(|service| service.borrow().get(id))
}
// Helper function to check whether the caller is the owner of a song
fn _check_if_owner(song: &Song) -> Result<(), Error> {
    if song.owner.to_string() != caller().to_string(){
        return Err(Error:: NotOwner { msg: format!("Caller={} isn't the owner of the song with id={}", caller(), song.id) })  
    }else{
        Ok(())
    }
}

// Helper function to check the input data of the payload
fn _check_input(payload: &SongPayload) -> Result<(), Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error:: ValidationFail{ content: check_payload.err().unwrap().to_string()})
    }else{
        Ok(())
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    UploadFail { msg: String },
    UpdateFail { msg: String },
    ValidationFail {content: String},
    NotOwner { msg: String}
}

// generate candid
ic_cdk::export_candid!();
