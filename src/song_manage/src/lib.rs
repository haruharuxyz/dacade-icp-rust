#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
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
    match _get_song(&id) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("a song with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn upload_song(song: SongPayload) -> Result<Song, Error> {
    if song.file_name.is_empty() {
        return Err(Error::UploadFail {
            msg: String::from("Invalid file name"),
        });
    };
    if song.mime_type.is_empty() {
        return Err(Error::UploadFail {
            msg: String::from("Invalid mime type"),
        });
    };
    if song.title.is_empty() {
        return Err(Error::UploadFail {
            msg: String::from("Invalid title"),
        });
    };
    if song.release_date.is_empty() {
        return Err(Error::UploadFail {
            msg: String::from("Invalid release_date"),
        });
    };
    if song.singer.is_empty() {
        return Err(Error::UploadFail {
            msg: String::from("Invalid singer"),
        });
    };
    if song.duration == 0 {
        return Err(Error::UploadFail {
            msg: String::from("Invalid song duration"),
        });
    };
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let song = Song {
        id,
        file_name: song.file_name,
        mime_type: song.mime_type,
        title: song.title,
        genre: song.genre,
        duration: song.duration,
        release_date: song.release_date,
        singer: song.singer,
        updated_at: Some(time()),
    };
    do_insert(&song);
    Ok(song)
}

#[ic_cdk::update]
fn update_song(id: u64, payload: SongPayload) -> Result<Song, Error> {
    if payload.file_name.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid file name"),
        });
    };
    if payload.mime_type.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid mime type"),
        });
    };
    if payload.title.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid title"),
        });
    };
    if payload.release_date.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid release_date"),
        });
    };
    if payload.singer.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid singer"),
        });
    };
    if payload.duration == 0 {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid song duration"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.file_name = payload.file_name;
            song.mime_type = payload.mime_type;
            song.title = payload.title;
            song.genre = payload.genre;
            song.duration = payload.duration;
            song.release_date = payload.release_date;
            song.singer = payload.singer;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_file_name(id: u64, file_name: String) -> Result<Song, Error> {
    if file_name.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid file name"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.file_name = file_name;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_mime_type(id: u64, mime_type: String) -> Result<Song, Error> {
    if mime_type.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid mime type"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.mime_type = mime_type;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_title(id: u64, title: String) -> Result<Song, Error> {
    if title.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid title"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.title = title;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_genre(id: u64, genre: String) -> Result<Song, Error> {
    if genre.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid genre"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.genre = genre;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_singer(id: u64, singer: String) -> Result<Song, Error> {
    if singer.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid singer"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.singer = singer;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_duration(id: u64, duration: u64) -> Result<Song, Error> {
    if duration == 0 {
        return Err(Error::UploadFail {
            msg: String::from("Invalid song duration"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.duration = duration;
            song.updated_at = Some(time());
            do_insert(&song);
            Ok(song)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a song with id={}. song not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_song_release_date(id: u64, release_date: String) -> Result<Song, Error> {
    if release_date.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid release date"),
        });
    };
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.release_date = release_date;
            song.updated_at = Some(time());
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

#[ic_cdk::update]
fn delete_song(id: u64) -> Result<Song, Error> {
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

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    UploadFail { msg: String },
    UpdateFail { msg: String },
}

// generate candid
ic_cdk::export_candid!();
