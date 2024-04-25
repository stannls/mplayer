pub mod download;
pub mod fs;
pub mod player;
pub mod search;
use std::path::PathBuf;

use dyn_clone::DynClone;

pub trait Artist: DynClone + Deleteable {
    fn get_albums(&self) -> Vec<Box<dyn Album + Send + Sync>>;
    fn get_name(&self) -> String;
    fn is_local(&self) -> bool;
}

dyn_clone::clone_trait_object!(Artist);

pub trait Song: DynClone + Send + Deleteable {
    fn get_title(&self) -> String;
    fn get_length(&self) -> Option<String>;
    fn get_length_secs(&self) -> Option<usize>;
    fn get_disambiguation(&self) -> Option<String>;
    fn get_artist_name(&self) -> String;
    fn get_number(&self) -> Option<String>;
    fn is_local(&self) -> bool;
    fn get_filepath(&self) -> Option<PathBuf>;
    fn get_album_name(&self) -> String;
    fn get_release_date(&self) -> Option<String>;
}

dyn_clone::clone_trait_object!(Song);

pub trait Album: DynClone + Send + Deleteable {
    fn get_name(&self) -> String;
    fn get_artist_name(&self) -> String;
    fn get_release_date(&self) -> String;
    fn get_songs(&self) -> Vec<Box<dyn Song + Send + Sync>>;
    fn is_groups(&self) -> bool;
    fn get_id(&self) -> String;
    fn is_local(&self) -> bool;
}

dyn_clone::clone_trait_object!(Album);

pub trait Deleteable: DynClone {
    fn delete(&self);
}

dyn_clone::clone_trait_object!(Deleteable);
