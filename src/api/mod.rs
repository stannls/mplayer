pub mod download;
pub mod fs;
pub mod search;
use dyn_clone::DynClone;

pub trait Artist: DynClone {
    fn get_albums(&self) -> Vec<Box<dyn Album>>;
    fn get_name(&self) -> String;
}

dyn_clone::clone_trait_object!(Artist);

pub trait Song: DynClone + Send {
    fn get_title(&self) -> String;
    fn get_length(&self) -> String;
    fn get_disambiguation(&self) -> Option<String>;
    fn get_artist_name(&self) -> String;
    fn get_number(&self) -> Option<String>;
}

dyn_clone::clone_trait_object!(Song);

pub trait Album: DynClone {
    fn get_name(&self) -> String;
    fn get_release_date(&self) -> String;
    fn get_songs(&self) -> Vec<Box<dyn Song>>;
}

dyn_clone::clone_trait_object!(Album);
