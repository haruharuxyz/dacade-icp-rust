use crate::types::{SongPayload, Error};

pub fn validate_song_payload(song: &SongPayload) -> Result<(), Error> {
    let fields = [
        ("file_name", &song.file_name),
        ("mime_type", &song.mime_type),
        ("title", &song.title),
        ("release_date", &song.release_date),
        ("singer", &song.singer),
    ];

    for (field_name, field_value) in &fields {
        if field_value.is_empty() {
            return Err(Error::UploadFail {
                msg: format!("Invalid {}", field_name),
            });
        }
    }

    if song.duration == 0 {
        return Err(Error::UploadFail {
            msg: String::from("Invalid song duration"),
        });
    }

    Ok(())
}