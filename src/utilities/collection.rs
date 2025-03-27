use std::sync::Arc;

use tokio::sync::RwLock;

use crate::collector::Collection;

pub fn format_collection_name(fmt: &str, collection: &Collection) -> String {
    fmt.replace("{collection_author}", &collection.uploader.username)
        .replace("{collection_title}", &collection.name)
        .replace("{collection_id}", collection.id.to_string().as_str())
}

pub async fn create_collection(
    collection_list: Arc<RwLock<osu_db::CollectionList>>,
    name: &str,
    path: &String,
) {
    let collection_exists = collection_list
        .read()
        .await
        .collections
        .iter()
        .any(|collection| collection.name.as_ref().unwrap_or(&"".to_string()) == name);
    if collection_exists {
        return;
    }

    collection_list
        .write()
        .await
        .collections
        .push(osu_db::collection::Collection {
            name: Some(name.to_string()),
            beatmap_hashes: vec![],
        });

    collection_list.read().await.to_file(path).unwrap();
}
