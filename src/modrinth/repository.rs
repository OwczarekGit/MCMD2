use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::{client, download_file, DownloadStatus, ModLoader, ModStatus, Repository};
use crate::mc_mod::MinecraftMod;
use crate::modrinth::connection::API_URL;
use crate::modrinth::data::{ModrinthProject, ModrinthReleases, ModrinthResponse};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ModrinthRepository;

impl ModrinthRepository {
    pub async fn get_releases(&self, mod_identifier: &str) -> Vec<ModrinthReleases> {
        let client = client();
        let url = &format!("{}project/{}/version", &API_URL, mod_identifier);
        let url = reqwest::Url::parse(url).expect("To parse correctly.");
        let request = client.get(url).build().unwrap();

        client.execute(request)
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

#[async_trait]
impl Repository for ModrinthRepository {
    async fn search_mods(&self, name: &str, version: &str, mod_loader: ModLoader) -> Vec<MinecraftMod> {
        let client = client();
        let Ok(url) = reqwest::Url::parse_with_params(
            &(API_URL.to_string() + "search"),
            &[
                ("query", name),
                ("limit", "20"),
                ("facets", format!("[[\"project_type:mod\"],[\"versions:{version}\"],[\"categories:{}\"]]", String::from(mod_loader)).as_str())
            ]
        ) else {
            return vec![];
        };

        let request = client.get(url).build().unwrap();
        let mods: ModrinthResponse = client.execute(request)
            .await
            .unwrap()
            .json()
            .await
            .expect("The mods to be parsed correctly.");

        let mods = mods.hits;

        mods.iter().map(|m|
            MinecraftMod {
                corresponding_file: None,
                mod_identifier: m.project_id.clone(),
                name: m.title.clone(),
                status: ModStatus::Normal,
            }
        ).collect()
    }

    async fn download_mod(&self, mod_identifier: &str, version: &str, loader: &ModLoader, location: &PathBuf) -> DownloadStatus {
        let releases: Vec<ModrinthReleases> = self.get_releases(mod_identifier)
            .await
            .into_iter()
            .filter(|m| m.fits_requirements(version, *loader))
            .collect();

        if releases.is_empty() {
            return DownloadStatus::Error;
        }

        let release = releases.first().expect("The release to be found");

        let Some(release) = release.files.first() else {
            return DownloadStatus::Error;
        };

        let mut location = location.clone();
        location.push(release.filename.clone());

        download_file(
            &release.url,
            location.to_str().unwrap()
        ).await
    }

    async fn open(&self, mod_identifier: &str) {
        let _ = open::that_detached(format!("https://modrinth.com/project/{}", mod_identifier));
    }

    async fn resolve_dependencies(&self, mod_identifier: &str, version: &str, loader: &ModLoader) -> Vec<MinecraftMod> {
        let releases: Vec<ModrinthReleases> = self.get_releases(mod_identifier)
            .await
            .into_iter()
            .filter(|m| m.fits_requirements(version, *loader))
            .collect();

        if releases.is_empty() {
            return vec![];
        }

        let release = releases.first().expect("The release to be found");

        let client = client();

        let mut results = vec![];

        for dependency in release.dependencies.iter() {
            if let Some(project_id) = &dependency.project_id {
                let url = format!("{API_URL}project/{}", project_id.clone());
                let url = reqwest::Url::parse(&url).expect("To parse correctly.");
                let request = client.get(url).build().unwrap();
                let result: ModrinthProject = client.execute(request)
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                if dependency.dependency_type.to_lowercase().eq("required") {
                    results.push(MinecraftMod::from(result));
                }
            }
        }

        results
    }
}
