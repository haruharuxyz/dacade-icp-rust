use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, MemoryId, MemoryManager, StableBTreeMap, Storable, VirtualMemory};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Song {
    id: u64,
    file_name: String,
    mime_type: String,
    title: String,
    singer: String,
    genre: String,
    duration: u64,
    release_date: String,
    updated_at: Option<u64>,
}

impl Song {
    fn validate(&self) -> Result<(), Error> {
        if self.file_name.is_empty() || self.mime_type.is_empty() || self.title.is_empty() || self.release_date.is_empty() || self.singer.is_empty() || self.duration == 0 {
            return Err(Error::UploadFail {
                msg: String::from("Invalid input data"),
            });
        }
        Ok(())
    }
}

impl Storable for Song {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

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

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SongPayload {
    file_name: String,
    mime_type: String,
    title: String,
    singer: String,
    genre: String,
    duration: u64,
    release_date: String,
}

#[ic_cdk::query]
fn get_song(id: u64) -> Result<Song, Error> {
    _get_song(&id).ok_or_else(|| Error::NotFound {
        msg: format!("A song with id={} not found", id),
    })
}

#[ic_cdk::update]
fn upload_song(song: SongPayload) -> Result<Song, Error> {
    let id = ID_COUNTER
        .with(|counter| counter.borrow_mut().get_then_increment())
        .expect("Cannot increment id counter");

    let mut new_song = Song {
        id,
        file_name: song.file_name.clone(),
        mime_type: song.mime_type.clone(),
        title: song.title.clone(),
        genre: song.genre.clone(),
        duration: song.duration,
        release_date: song.release_date.clone(),
        singer: song.singer.clone(),
        updated_at: Some(time()),
    };

    new_song.validate()?;
    do_insert(&new_song);
    Ok(new_song)
}

#[ic_cdk::update]
fn update_song(id: u64, payload: SongPayload) -> Result<Song, Error> {
    let mut updated_song = STORAGE.with(|service| service.borrow().get(&id).cloned());

    if let Some(ref mut song) = updated_song {
        song.file_name = payload.file_name.clone();
        song.mime_type = payload.mime_type.clone();
        song.title = payload.title.clone();
        song.genre = payload.genre.clone();
        song.duration = payload.duration;
        song.release_date = payload.release_date.clone();
        song.singer = payload.singer.clone();
        song.updated_at = Some(time());

        song.validate()?;
        do_insert(song);
        Ok(song.clone())
    } else {
        Err(Error::NotFound {
            msg: format!("Couldn't update a song with id={}. Song not found", id),
        })
    }
}

#[ic_cdk::update]
fn delete_song(id: u64) -> Result<Song, Error> {
    STORAGE.with(|service| service.borrow_mut().remove(&id).ok_or_else(|| Error::NotFound {
        msg: format!("Couldn't delete a song with id={}. Song not found", id),
    }))
}

// ... (other update methods follow a similar pattern)

fn do_insert(song: &Song) {
    STORAGE.with(|service| service.borrow_mut().insert(song.id, song.clone()));
}

fn _get_song(id: &u64) -> Option<Song> {
    STORAGE.with(|service| service.borrow().get(id).cloned())
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    UploadFail { msg: String },
    UpdateFail { msg: String },
}

ic_cdk::export_candid!();
