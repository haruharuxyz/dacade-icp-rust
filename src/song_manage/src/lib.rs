#[macro_use]
extern crate serde;
mod store;
mod types;
mod utils;
use store::*;
use types::*;
use utils::*;

use ic_cdk::api::time;

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
    validate_song_payload(&song)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value =counter.borrow().get();
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
fn update_song(id: u64, update: SongUpdate) -> Result<Song, Error> {
    let updates = vec![
        ("file_name", update.file_name),
        ("mime_type", update.mime_type),
        ("title", update.title),
        ("genre", update.genre),
        ("singer", update.singer),
        ("release_date", update.release_date),
    ];

    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut song) => {
            for (field_name, field_value) in updates {
                if let Some(value) = field_value {
                    if value.is_empty() {
                        return Err(Error::UpdateFail {
                            msg: format!("Invalid {}", field_name),
                        });
                    }
                    match field_name {
                        "file_name" => song.file_name = value,
                        "mime_type" => song.mime_type = value,
                        "title" => song.title = value,
                        "genre" => song.genre = value,
                        "singer" => song.singer = value,
                        "release_date" => song.release_date = value,
                        _ => (),
                    }
                }
            }
            if let Some(duration) = update.duration {
                if duration == 0 {
                    return Err(Error::UploadFail {
                        msg: String::from("Invalid song duration"),
                    });
                }
                song.duration = duration;
            }
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
fn delete_song(id: u64) -> Result<Song, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a song with id={}. song not found.", id),
        }),
    }
}

// generate candid
ic_cdk::export_candid!();