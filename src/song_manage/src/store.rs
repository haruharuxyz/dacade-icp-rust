use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{ Cell, DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

use crate::types::Song;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

   pub static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

   pub static STORAGE: RefCell<StableBTreeMap<u64, Song, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

}

// helper method to perform insert.
pub fn do_insert(song: &Song) {
    STORAGE.with(|service| service.borrow_mut().insert(song.id, song.clone()));
}

// a helper method to get a song by id
pub fn _get_song(id: &u64) -> Option<Song> {
    STORAGE.with(|service| service.borrow().get(id))
}