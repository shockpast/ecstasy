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
