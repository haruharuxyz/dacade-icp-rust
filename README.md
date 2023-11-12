# ICP CRUD Song Management Rust Canister

## Features
1. Upload new song
2. Get song info by song's ID
3. Update song's info
4. Update song's file name
5. Update song's mime type
6. Update song's title
7. Update song's singer name
8. Update song's genre
9. Update song's duration
10. Update song's release date
11. Delete song by song's ID

## Deploy Canister

```bash
dfx start --background --clean
npm run gen-deploy
```

## Commands

### Upload new song
```bash
dfx canister call song_manage upload_song '(
  record {
  	file_name = "Dacade";
  	mime_type = "mp4";
    title = "Title";
    singer = "Taylor Swift";
    genre = "Pop";
    duration = 126;
    release_date = "11/12/2023";
  }
)'
```

### Get song info by song's ID
```bash
dfx canister call song_manage get_song '(0)'
```

### Update song info
```bash
dfx canister call song_manage update_song '(0, record {
  	file_name = "Dacade";
  	mime_type = "mp4";
    title = "Title Updated";
    singer = "Taylor Swift";
    genre = "Pop";
    duration = 126;
    release_date = "11/12/2023";
})'
```

### Update song's file name
```bash
dfx canister call song_manage update_song_file_name '(0, "Filename updated")'
```

### Update song's mime type
```bash
dfx canister call song_manage update_song_mime_type '(0, "wmv")'
```

### Update song's title
```bash
dfx canister call song_manage update_song_title '(0, "Title Updated 2")'
```

### Update song's singer name
```bash
dfx canister call song_manage update_song_singer '(0, "Not Taylor Swift")'
```

### Update song's genre
```bash
dfx canister call song_manage update_song_genre '(0, "Hiphop")'
```

### Update song's duration
```bash
dfx canister call song_manage update_song_duration '(0, 420)'
```

### Update song's release date
```bash
dfx canister call song_manage update_song_release_date '(0, "12/12/2023")'
```

### Delete song by song's ID
```bash
dfx canister call song_manage delete_song '(0)'
```