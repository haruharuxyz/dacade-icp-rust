# ICP CRUD Song Management Rust Canister

## Features
1. Add singer info
2. Update singer info
3. Upload new song
4. Get song info by song's ID
5. Get all songs info
6. Update song's info
7. Update song's file name
8. Update song's mime type
9. Update song's title
10. Update song's singer name
11. Update song's genre
12. Update song's duration
13. Update song's release date
14. Delete song by song's ID
15. Get all singers
16. Get singer info by singer's ID

## Deploy Canister

```bash
dfx start --background --clean
npm run gen-deploy
```

## Commands

### Add new singer
```bash
dfx canister call song_manage add_singer '(
  record {
    name = "Taylor Swift";
  }
)'
```

### Update singer info
```bash
dfx canister call song_manage update_singer '(
  0,
  record {
    name = "Taylor Swift new";
  }
)'
```

### Get all singers
```bash
dfx canister call song_manage get_all_singers
```

### Get singer info by singer's ID
```bash
dfx canister call song_manage get_singer '(0)'
```

### Upload new song
```bash
dfx canister call song_manage upload_song '(
  record {
    singer_id = 0;
  	file_name = "Dacade";
  	mime_type = "mp4";
    title = "Title";
    genre = "Pop";
    duration = 126;
    release_date = "11/12/2023";
  }
)'
```

### Get all songs info
```bash
dfx canister call song_manage get_all_songs
```

### Get song info by song's ID
```bash
dfx canister call song_manage get_song '(1)'
```

### Update song info
```bash
dfx canister call song_manage update_song '(1, record {
    singer_id = 0;
  	file_name = "Dacade";
  	mime_type = "mp4";
    title = "Title Updated";
    genre = "Pop";
    duration = 126;
    release_date = "11/12/2023";
})'
```

### Update song's file name
```bash
dfx canister call song_manage update_song_file_name '(1, "Filename updated")'
```

### Update song's mime type
```bash
dfx canister call song_manage update_song_mime_type '(1, "wmv")'
```

### Update song's title
```bash
dfx canister call song_manage update_song_title '(1, "Title Updated 2")'
```

### Update song's singer name
```bash
dfx canister call song_manage update_song_singer '(1, "Not Taylor Swift")'
```

### Update song's genre
```bash
dfx canister call song_manage update_song_genre '(1, "Hiphop")'
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
