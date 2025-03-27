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

pub async fn add_to_collection(
    collection_list: &Arc<RwLock<osu_db::CollectionList>>,
    name: &str,
    checksum: &String,
) -> bool {
    if is_checksum_in_collection(collection_list, name, checksum).await {
        return false;
    }

    collection_list
        .write()
        .await
        .collections
        .iter_mut()
        .find(|c| c.name.as_ref().unwrap_or(&"".to_string()) == name)
        .unwrap()
        .beatmap_hashes
        .push(Some(checksum.to_string()));

    true
}

async fn is_checksum_in_collection(
    collection_list: &Arc<RwLock<osu_db::CollectionList>>,
    name: &str,
    checksum: &String,
) -> bool {
    collection_list
        .write()
        .await
        .collections
        .iter_mut()
        .find(|c| c.name.as_ref().unwrap_or(&"".to_string()) == name)
        .unwrap()
        .beatmap_hashes
        .iter()
        .any(|c| c.as_ref().unwrap() == checksum)
}
