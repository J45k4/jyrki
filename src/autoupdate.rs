use std::env::consts::OS;
use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;

use flate2::read::GzDecoder;
use reqwest::Client;
use tar::Archive;

pub async fn force_update_newest_version() {
    let client = Client::new();
    let res = client
        .get("https://api.github.com/repos/j45k4/puppycoder/releases?per_page=10")
        .header("User-Agent", "puppycoder")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    
    log::info!("res: {}", res);
    let res: serde_json::Value = serde_json::from_str(&res).unwrap();
    log::info!("res: {:#?}", res);

	let releases = res.as_array().unwrap();

	let asset = releases.iter().find_map(|release| {
		let assets = release["assets"].as_array().unwrap();
		assets.iter().find(|asset| {
			if OS == "windows" {
				asset["name"].as_str().unwrap().contains("windows")
			} else if OS == "linux" {
				asset["name"].as_str().unwrap().contains("linux")
			} else if OS == "macos" {
				asset["name"].as_str().unwrap().contains("macos")
			} else {
				false
			}
		})
	}).unwrap();

	let asset_file_name = asset["name"].as_str().unwrap();
	log::info!("asset: {:#?}", asset);

    let browser_download_url = asset["browser_download_url"].as_str().unwrap();
    log::info!("browser_download_url: {}", browser_download_url);

    // Download the file content
    let file_res = client
        .get(browser_download_url)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

	log::info!("File successfully downloaded");

    // Save the file to disk
    let mut file = File::create(asset_file_name).unwrap();
    file.write_all(&file_res).unwrap();
	let file = File::open(asset_file_name).unwrap();
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
	let temp_file_path = Path::new("./puppycoder");
	let mut bin_file = File::create(temp_file_path).unwrap();
    for entry in archive.entries().unwrap() {
		let mut f = entry.unwrap();
		log::info!("entry: {:?}", f.path().unwrap());
		// let mut file = File::create(f.path().unwrap()).unwrap();
		io::copy(&mut f, &mut bin_file).unwrap();
	}

	log::info!("File successfully extracted");
	std::fs::remove_file(asset_file_name).unwrap();

	let current_exe = std::env::current_exe().unwrap();
	log::info!("current_exe: {:?}", current_exe);
    #[cfg(target_family = "unix")]
    {
        std::fs::set_permissions(&temp_file_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    std::fs::rename(&temp_file_path, &current_exe).unwrap(); 
	log::info!("File successfully moved");
	std::process::Command::new(current_exe).spawn().unwrap();
	log::info!("stopping old version");
	exit(0);
}