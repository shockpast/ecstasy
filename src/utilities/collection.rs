use crate::collector::Collection;

pub fn format_collection_name(fmt: &String, collection: &Collection) -> String {
  fmt
    .replace("{collection_author}", &collection.uploader.username)
    .replace("{collection_title}", &collection.name)
    .replace("{collection_id}", collection.id.to_string().as_str())
}

pub fn get_or_create_collection<'a>(collection_list: &'a mut osu_db::collection::CollectionList, name: &str, path: Option<String>) -> &'a mut osu_db::collection::Collection {
  let index = collection_list
    .collections
    .iter()
    .position(|c| c.name.as_deref() == Some(name));

  if let Some(i) = index {
    return &mut collection_list.collections[i];
  }
  
  collection_list.collections.push(osu_db::collection::Collection {
    name: Some(name.to_string()),
    beatmap_hashes: vec![]
  });

  if path.is_some() {
    collection_list.to_file(path.unwrap()).unwrap();
  }

  collection_list.collections.last_mut().unwrap()
}