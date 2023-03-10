use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::artist::{ArtistSearchQuery, Artist};
use musicbrainz_rs::entity::recording::{RecordingSearchQuery, Recording};
use musicbrainz_rs::entity::release::{ReleaseSearchQuery, Release};
use crate::api::search::wrapper;
use reqwest::Error;

pub async fn search_artists(query: String) -> Result<Vec<wrapper::Artist>, Error>{
    let q = ArtistSearchQuery::query_builder()
            .artist(&query)
            .build();

    let res = Artist::search(q).execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Artist::new(f))
        .collect();
    Ok(res)
}

pub async fn search_albums(query: String) -> Result<Vec<wrapper::Recording>, Error>{
    let q = RecordingSearchQuery::query_builder()
        .recording(&query)
        .build();
    let res = Recording::search(q).execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Recording::new(f))
        .collect();
    Ok(res)
}

pub async fn search_titles(query: String) -> Result<Vec<wrapper::Release>, Error>{
    let q = ReleaseSearchQuery::query_builder()
        .release(&query)
        .build();
    let res = Release::search(q).execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Release::new(f))
        .collect();
    Ok(res)
}
