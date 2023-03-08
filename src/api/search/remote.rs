use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::artist::{ArtistSearchQuery, Artist};
use musicbrainz_rs::entity::recording::{RecordingSearchQuery, Recording};
use musicbrainz_rs::entity::release::{ReleaseSearchQuery, Release};
use reqwest::Error;

pub async fn search_artists(query: &str) -> Result<Vec<Artist>, Error>{
    let q = ArtistSearchQuery::query_builder()
            .artist(query)
            .build();

    let res = Artist::search(q).execute().await?.entities;
    Ok(res)
}

pub async fn search_albums(query: &str) -> Result<Vec<Recording>, Error>{
    let q = RecordingSearchQuery::query_builder()
        .recording(query)
        .build();
    let res = Recording::search(q).execute().await?.entities;
    Ok(res)
}

pub async fn search_titles(query: &str) -> Result<Vec<Release>, Error>{
    let q = ReleaseSearchQuery::query_builder()
        .release(query)
        .build();
    let res = Release::search(q).execute().await?.entities;
    Ok(res)
}
