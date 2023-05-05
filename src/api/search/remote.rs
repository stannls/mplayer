use crate::api::search::wrapper;
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::entity::recording::{Recording, RecordingSearchQuery};
use musicbrainz_rs::entity::release::{self, Release};
use musicbrainz_rs::entity::release_group::{ReleaseGroup, ReleaseGroupSearchQuery};
use musicbrainz_rs::prelude::*;
use reqwest::Error;

pub async fn search_artists(query: String) -> Result<Vec<wrapper::ArtistWrapper>, Error> {
    let q = ArtistSearchQuery::query_builder().artist(&query).build();

    let res = Artist::search(q)
        .with_releases()
        .with_release_relations()
        .with_releases_and_discids()
        .with_release_groups()
        .with_artist_relations()
        .with_recordings()
        .with_recording_relations()
        .execute()
        .await?
        .entities
        .into_iter()
        .map(|f| wrapper::ArtistWrapper::new(f, vec![]))
        .collect();
    Ok(res)
}

pub async fn search_songs(query: String) -> Result<Vec<wrapper::SongWrapper>, Error> {
    let q = RecordingSearchQuery::query_builder()
        .recording(&query)
        .build();
    let res = Recording::search(q)
        .with_isrcs()
        .with_artists()
        .with_releases()
        .execute()
        .await?
        .entities
        .into_iter()
        .filter(|f| matches!(f.releases.as_ref(), Option::Some(_)))
        .map(|f| wrapper::SongWrapper::new(f.to_owned(), f.releases.unwrap().get(0).unwrap().title.to_owned()))
        .collect();
    Ok(res)
}

pub async fn search_albums(query: String) -> Result<Vec<wrapper::ReleaseGroupWrapper>, Error> {
    let q = ReleaseGroupSearchQuery::query_builder()
        .release(&query)
        .build();
    let res = ReleaseGroup::search(q)
        .with_release_group_relations()
        .with_releases()
        .with_annotations()
        .with_series_relations()
        .execute()
        .await?
        .entities
        .into_iter()
        .map(|f| wrapper::ReleaseGroupWrapper::new(f))
        .collect();
    Ok(res)
}

pub async fn album_from_release_group(release_group: ReleaseGroup) -> release::Release {
    let id = release_group
        .releases
        .unwrap()
        .get(0)
        .unwrap()
        .id
        .to_owned();
    release::Release::fetch()
        .id(id.as_str())
        .with_annotations()
        .with_recording_level_relations()
        .with_recordings()
        .with_artist_credits()
        .execute()
        .await
        .unwrap()
}

pub async fn album_from_release_group_id(release_group_id: String) -> release::Release{
    Release::browse()
        .by_release_group(&release_group_id)
        .with_annotations()
        .with_recording_level_relations()
        .with_recordings()
        .with_artist_credits()
        .execute()
        .await
        .unwrap()
        .entities
        .get(0)
        .unwrap()
        .clone()
}

pub async fn unique_releases(artist_id: String) -> Vec<ReleaseGroup> {
    let mut release_groups = ReleaseGroup::browse()
        .by_artist(&artist_id)
        .execute()
        .await
        .unwrap()
        .entities
        .into_iter()
        .filter(|f| matches!(f.first_release_date, Option::Some(_)))
        .collect::<Vec<ReleaseGroup>>();
    release_groups.sort_by(|a, b| b.first_release_date.unwrap().cmp(&a.first_release_date.unwrap()));
    release_groups
}
