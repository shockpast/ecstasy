use tokio::fs::read_dir;

pub async fn find_beatmap(path: &str, id: i32) -> Option<bool> {
    let mut entries = read_dir(path).await.unwrap();

    while let Some(entry) = entries.next_entry().await.unwrap() {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if file_name_str.contains(&id.to_string()) {
            return Some(true);
        }
    }

    None
}

#[cfg(target_os = "windows")]
pub fn find_game() -> Result<String, std::io::Error> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_USERS);

    let keys = hkcu
        .enum_keys()
        .map(|k| k.unwrap())
        .filter(|k| k.ends_with("_Classes"))
        .collect::<Vec<_>>();
    let value: String = hkcu
        .open_subkey(format!(
            "{}\\osustable.File.osz\\Shell\\Open\\Command",
            keys.first().unwrap()
        ))?
        .get_value("")?;

    let path = value
        .split(" ")
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .replace("\"", "")
        .replace("\\osu!.exe", "");

    Ok(path)
}

#[cfg(target_os = "linux")]
pub async fn find_game() -> Result<String, std::io::Error> {
    let path = std::env::var("OSU_FOLDER")
        .expect("'OSU_FOLDER' export is not defined (e.g.: '$HOME\\osu')");
    Ok(path)
}
