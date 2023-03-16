use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::artist::{ArtistSearchQuery, Artist};
use musicbrainz_rs::entity::recording::{RecordingSearchQuery, Recording};
use musicbrainz_rs::entity::release;
use musicbrainz_rs::entity::release_group::{ReleaseGroupSearchQuery, ReleaseGroup};
use crate::api::search::wrapper;
use reqwest::Error;

pub async fn search_artists(query: String) -> Result<Vec<wrapper::Artist>, Error>{
    let q = ArtistSearchQuery::query_builder()
            .artist(&query)
            .build();

    let res = Artist::search(q)
        .with_releases()
        .with_release_relations()
        .with_releases_and_discids()
        .with_release_groups()
        .with_recordings()
        .with_recording_relations()
        .execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Artist::new(f))
        .collect();
    Ok(res)
}

pub async fn search_songs(query: String) -> Result<Vec<wrapper::Recording>, Error>{
    let q = RecordingSearchQuery::query_builder()
        .recording(&query)
        .build();
    let res = Recording::search(q).execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Recording::new(f))
        .collect();
    Ok(res)
}

pub async fn search_albums(query: String) -> Result<Vec<wrapper::Release>, Error>{
    let q = ReleaseGroupSearchQuery::query_builder()
        .release(&query)
        .build();
    let res = ReleaseGroup::search(q).with_release_group_relations().with_releases().with_annotations().with_series_relations().execute().await?.entities
        .into_iter()
        .map(|f| wrapper::Release::new(f))
        .collect();
    Ok(res)
}

pub async fn album_from_release_group(release_group: ReleaseGroup) -> release::Release {
    let id = release_group.releases.unwrap().get(0).unwrap().id.to_owned();
    release::Release::fetch()
        .id(id.as_str())
        .with_annotations()
        .with_recording_level_relations()
        .with_recordings()
        .execute()
        .await
        .unwrap()
}

pub async fn release_group_by_id(id: String) -> Result<ReleaseGroup, Error> {
    ReleaseGroup::fetch()
        .id(id.as_str())
        .with_release_group_relations()
        .with_releases()
        .with_annotations()
        .with_series_relations()
        .execute()
        .await
}

pub async fn artist_by_id(id: String) -> Artist {
    Artist::fetch()
        .id(id.as_str())
        .with_annotations()
        .with_releases()
        .with_recordings()
        .with_release_groups()
        .with_recording_relations()
        .with_release_relations()
        .with_series_relations()
        .execute()
        .await
        .unwrap()
}
