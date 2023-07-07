use std::path::PathBuf;
use crate::core::{client, DownloadStatus, ModLoader, ModStatus, Repository};
use async_trait::async_trait;
use crate::curseforge::connection::{API_URL_V1, GAME_ID, X_API_KEY};
use crate::curseforge::data::{CurseForgeGetModResponse, CurseforgeResponse, ModLoaderType};
use crate::mc_mod::MinecraftMod;

#[derive(Default)]
pub struct CurseforgeRepository;

#[async_trait]
impl Repository for CurseforgeRepository {
    async fn search_mods(&self, name: &str, version: &str, mod_loader: ModLoader) -> Vec<MinecraftMod> {
        let client = client();
        let url = format!("{}mods/search?gameId={}", API_URL_V1, GAME_ID);
        let loader = ModLoaderType::from(mod_loader) as u32;
        let Ok(url) = reqwest::Url::parse_with_params(
            &(url),[
                ("gameVersion", version),
                ("modLoaderType", &format!("{}", loader)),
                ("searchFilter", name)
            ]
        ) else {
            return vec![];
        };

        let request = client.get(url)
            .header("x-api-key", X_API_KEY)
            .build()
            .unwrap();


        let mods: CurseforgeResponse = client.execute(request).await
            .unwrap()
            .json()
            .await
            .expect("The mods to be parsed correctly.");

        mods.data
            .into_iter()
            .map(|m| {
                MinecraftMod {
                    status: ModStatus::Normal,
                    name: m.name,
                    mod_identifier: m.id.to_string(),
                    coresponding_file: None,
                }
            }).collect()
    }

    async fn download_mod(&self, mod_identifier: &str, version: &str, mod_loader: &ModLoader, location: &PathBuf) -> DownloadStatus {
        todo!()
    }

    async fn open(&self, mod_identifier: &str) {
        let client = client();
        let url = format!("{API_URL_V1}mods/{mod_identifier}");

        let request = client.get(url)
            .header("x-api-key", X_API_KEY)
            .build()
            .unwrap();

        let m: CurseForgeGetModResponse = client.execute(request).await
            .unwrap()
            .json()
            .await
            .expect("The mod to be found");

        let m = m.data;

        if let Some(url) = m.links.website_url{
            let _ = open::that_detached(url);
        }
    }
}