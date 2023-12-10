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
    singer_id: u64,
    file_name: String,
    mime_type: String,
    title: String,
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

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Singer {
    id: u64,
    name: String,
    updated_at: Option<u64>,
}

impl Storable for Singer {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Singer {
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

    static SONG_STORAGE: RefCell<StableBTreeMap<u64, Song, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static SINGER_STORAGE: RefCell<StableBTreeMap<u64, Singer, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SongPayload {
    singer_id: u64,
    file_name: String,
    mime_type: String,
    title: String,
    genre: String,
    duration: u64,
    release_date: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SingerPayload {
    name: String,
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

#[ic_cdk::query]
fn get_all_songs() -> Result<Vec<Song>, Error> {
    let songs_map: Vec<(u64, Song)> =
        SONG_STORAGE.with(|service| service.borrow().iter().collect());
    let songs: Vec<Song> = songs_map.into_iter().map(|(_, song)| song).collect();

    if !songs.is_empty() {
        Ok(songs)
    } else {
        Err(Error::NotFound {
            msg: "No songs found.".to_string(),
        })
    }
}

#[ic_cdk::query]
fn get_singer(id: u64) -> Result<Singer, Error> {
    match _get_singer(&id) {
        Some(singer) => Ok(singer),
        None => Err(Error::NotFound {
            msg: format!("a singer with id={} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_all_singers() -> Result<Vec<Singer>, Error> {
    let singers_map: Vec<(u64, Singer)> =
        SINGER_STORAGE.with(|service| service.borrow().iter().collect());
    let singers: Vec<Singer> = singers_map.into_iter().map(|(_, singer)| singer).collect();

    if !singers.is_empty() {
        Ok(singers)
    } else {
        Err(Error::NotFound {
            msg: "No singers found.".to_string(),
        })
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
        singer_id: song.singer_id,
        file_name: song.file_name,
        mime_type: song.mime_type,
        title: song.title,
        genre: song.genre,
        duration: song.duration,
        release_date: song.release_date,
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
    if payload.duration == 0 {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid song duration"),
        });
    };
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.singer_id = payload.singer_id;
            song.file_name = payload.file_name;
            song.mime_type = payload.mime_type;
            song.title = payload.title;
            song.genre = payload.genre;
            song.duration = payload.duration;
            song.release_date = payload.release_date;
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
fn update_song_singer(id: u64, singer_id: u64) -> Result<Song, Error> {
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            song.singer_id = singer_id;
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
    match SONG_STORAGE.with(|service| service.borrow().get(&id)) {
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
    SONG_STORAGE.with(|service| service.borrow_mut().insert(song.id, song.clone()));
}

fn do_insert_singer(singer: &Singer) {
    SINGER_STORAGE.with(|service| service.borrow_mut().insert(singer.id, singer.clone()));
}

#[ic_cdk::update]
fn delete_song(id: u64) -> Result<Song, Error> {
    match SONG_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a song with id={}. song not found.", id),
        }),
    }
}

// a helper method to get a song by id
fn _get_song(id: &u64) -> Option<Song> {
    SONG_STORAGE.with(|service| service.borrow().get(id))
}

fn _get_singer(id: &u64) -> Option<Singer> {
    SINGER_STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn add_singer(singer: SingerPayload) -> Result<Singer, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let singer = Singer {
        id,
        name: singer.name,
        updated_at: Some(time()),
    };
    do_insert_singer(&singer);
    Ok(singer)
}

#[ic_cdk::update]
fn update_singer(id: u64, payload: SingerPayload) -> Result<Singer, Error> {
    let singer_option: Option<Singer> = SINGER_STORAGE.with(|service| service.borrow().get(&id));

    match singer_option {
        Some(mut singer) => {
            singer.name = payload.name;
            singer.updated_at = Some(time());
            do_insert_singer(&singer);
            Ok(singer)
        }
        None => Err(Error::NotFound {
            msg: format!("Singer with id={} not found.", id),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    UploadFail { msg: String },
    UpdateFail { msg: String },
}

// generate candid
ic_cdk::export_candid!();
