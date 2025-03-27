use std::sync::Arc;

use tokio::sync::RwLock;

use crate::collector::Collection;

pub fn format_collection_name(fmt: &String, collection: &Collection) -> String {
    fmt.replace("{collection_author}", &collection.uploader.username)
        .replace("{collection_title}", &collection.name)
        .replace("{collection_id}", collection.id.to_string().as_str())
}

pub async fn create_collection<'a>(
    collection_list: Arc<RwLock<osu_db::CollectionList>>,
    name: &str,
    path: String,
) {
    collection_list
        .read()
        .await
        .collections
        .iter()
        .for_each(|collection| {
            if collection.name.as_ref().unwrap_or(&"".to_string()) == name {
                return;
            }
        });
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
