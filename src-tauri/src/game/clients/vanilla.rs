use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::io::{stdout, AsyncWriteExt};

use crate::{create_client, create_manifest, game::client::{GameClient, GameClientDetails, GameClientType}, utils::http};

create_manifest!(VanillaManifest {
    // minecraft: vanilla_manifest::MinecraftManifest
});

create_client!(VanillaClient {});

#[async_trait]
impl GameClient for VanillaClient {
    fn new(handle: AppHandle, details: GameClientDetails) -> Self {
        Self {
            handle: handle.clone(),
            details
        }
    }

    async fn get_all_version_urls() -> Result<HashMap<String, String>, Box<dyn Error>> {
        const URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let request = http::create_client().get(URL).send().await?;

        let json = request.json::<Value>().await?;

        let versions = json.get("versions").ok_or("No versions object")?
            .as_array().ok_or("Invalid versions object")?; 

        let mut map = HashMap::new();

        for version in versions {
            let version = version.as_object().ok_or("Invalid version object")?;

            let id = version.get("id").ok_or("No id object")?
                .as_str().ok_or("Invalid id object")?;
            let url = version.get("url").ok_or("No url object")?
                .as_str().ok_or("Invalid url object")?;

            map.insert(id.to_string(), url.to_string());
        }

        Ok(map)
    }
    
    async fn get_minecraft_manifest(version_url: String) -> Result<vanilla_manifest::MinecraftManifest, Box<dyn Error>> {
        let request = http::create_client().get(version_url).send().await?;

        match request.json::<vanilla_manifest::MinecraftManifest>().await {
            Ok(manifest) => Ok(manifest),
            Err(e) => Err(e.into())
        }
    }

    fn get_handle(&self) -> &AppHandle {
        &self.handle
    }

    fn get_details(&self) -> &GameClientDetails {
        &self.details
    }

    fn get_client(&self) -> &GameClientType {
        &self.get_details().client_type
    }

    async fn setup(&self) -> Result<(), Box<dyn Error>> {
        let libraries_dir = self.get_handle().path().app_config_dir()?.join("libraries");
        let versions = Self::get_all_version_urls().await?;
        let version = versions.get(&self.get_details().version).ok_or("Version not found")?.to_owned();
        let manifest = Self::get_minecraft_manifest(version).await?;

        let libraries = vanilla_impl::setup_libraries(&manifest.libraries, &libraries_dir).await?;
        println!("{:#?}", libraries);
        // vanilla_impl::setup_natives().await?;
        // vanilla_impl::setup_assets().await?;
        Ok(())
    }

    async fn launch(&self) -> Result<(), Box<dyn Error>> {
        println!("Launching Vanilla client");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Vanilla client launched");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Vanilla client exited");
        stdout().flush().await?; // why...
        Ok(())
    }
}

pub mod vanilla_impl {
    use std::{error::Error, fs, path::PathBuf};

    use serde_json::{Map, Value};
    use tauri::AppHandle;

    use crate::utils::http::download_file;

    use super::vanilla_manifest::{self, Library};

    pub async fn setup_libraries(libraries: &Vec<Library>, libraries_folder: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
        let mut natives_ret: Vec<&Library> = vec![];
        let mut libraries_ret: Vec<String> = vec![];

        for library in libraries {
            // let library: &Map<String, Value> = library.as_object().unwrap();
            
            // if let Some(_) = library.get("natives") {
            //     natives_ret.push(library);
            //     continue;
            // }
            
            // let artifact = library.get("downloads").ok_or("No downloads object")?
            //     .as_object().ok_or("Invalid downloads object")?
            //     .get("artifact").ok_or("No artifact object")?
            //     .as_object().ok_or("Invalid artifact object")?;

            // let path = artifact.get("path").ok_or("No path object")?
            //     .as_str().ok_or("Invalid path object")?;
            // let url = artifact.get("url").ok_or("No url object")?
            //     .as_str().ok_or("Invalid url object")?;

            if let Some(_) = library.natives {
                natives_ret.push(library);
                continue;
            }

            let artifact = library.downloads.artifact.clone().ok_or("No artifact object")?;
            let path = artifact.path;
            let url = artifact.url;
            
            // TODO: Add checks for rules + platform
            
            let dest = libraries_folder.join(path);
            fs::create_dir_all(dest.parent().ok_or("Couldn't get library parent")?)?;

            if !dest.exists() {
                download_file(url.as_str(), &dest).await?;
            }

            libraries_ret.push(dest.to_str().ok_or("Couldn't get library path")?.to_string());
        }

        Ok(libraries_ret)
    }

    pub async fn setup_natives() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn setup_assets() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub mod vanilla_manifest {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MinecraftManifest {
        pub asset_index: AssetIndex,
        pub assets: String,
        pub compliance_level: i64,
        pub downloads: Downloads,
        pub id: String,
        pub java_version: JavaVersion,
        pub libraries: Vec<Library>,
        pub logging: Logging,
        pub main_class: String,
        pub minecraft_arguments: String,
        pub minimum_launcher_version: i64,
        pub release_time: String,
        pub time: String,

        #[serde(rename = "type")]
        pub type_field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AssetIndex {
        pub id: String,
        pub sha1: String,
        pub size: i64,
        pub total_size: i64,
        pub url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Downloads {
        pub client: Client,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Client {
        pub sha1: String,
        pub size: i64,
        pub url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct JavaVersion {
        pub component: String,
        pub major_version: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Library {
        pub downloads: Downloads2,
        pub name: String,
        #[serde(default)]
        pub rules: Vec<Rule>,
        pub extract: Option<Extract>,
        pub natives: Option<Natives>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Downloads2 {
        pub artifact: Option<Artifact>,
        pub classifiers: Option<Classifiers>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Artifact {
        pub path: String,
        pub sha1: String,
        pub size: i64,
        pub url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Classifiers {
        #[serde(rename = "natives-windows-32")]
        pub natives_windows_32: Option<NativeLibrary>,
        #[serde(rename = "natives-windows-64")]
        pub natives_windows_64: Option<NativeLibrary>,
        #[serde(rename = "natives-osx")]
        pub natives_osx: Option<NativeLibrary>,
        #[serde(rename = "natives-linux")]
        pub natives_linux: Option<NativeLibrary>,
        #[serde(rename = "natives-windows")]
        pub natives_windows: Option<NativeLibrary>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NativeLibrary {
        pub path: String,
        pub sha1: String,
        pub size: i64,
        pub url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Rule {
        pub action: String,
        pub os: Option<Os>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Os {
        pub name: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Extract {
        pub exclude: Vec<String>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Natives {
        pub windows: String,
        pub linux: Option<String>,
        pub osx: Option<String>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Logging {
        pub client: Client2,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Client2 {
        pub argument: String,
        pub file: File,
        #[serde(rename = "type")]
        pub type_field: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct File {
        pub id: String,
        pub sha1: String,
        pub size: i64,
        pub url: String,
    }

}