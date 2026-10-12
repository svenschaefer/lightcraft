use crate::journal::{LOG, SNAPSHOT, VERSION};
use crate::*;

fn folder(catalog: &mut Catalog, parent: Option<AlbumId>) -> AlbumId {
    let id = catalog.alloc_album_id();
    catalog
        .apply(Op::AddAlbum {
            album: Album {
                id,
                name: format!("Folder {}", id.0),
                parent,
                folder: true,
                photos: vec![],
                cover: None,
                smart: None,
                quick: false,
                order: None,
            },
        })
        .unwrap();
    id
}

fn cycle() -> (Catalog, AlbumId, AlbumId) {
    let mut catalog = Catalog::new();
    let a = folder(&mut catalog, None);
    let b = folder(&mut catalog, Some(a));
    let unrelated = folder(&mut catalog, None);
    catalog.albums.get_mut(&a).unwrap().parent = Some(b);
    (catalog, a, unrelated)
}

#[test]
fn cyclic_snapshot_is_refused() {
    let (catalog, _, _) = cycle();
    assert!(matches!(Catalog::from_snapshot(&catalog.to_snapshot()), Err(CatalogError::Corrupt(_))));
}

#[test]
fn move_below_an_existing_unrelated_cycle_fails_without_changing_the_catalog() {
    let (mut catalog, parent, id) = cycle();
    let before = catalog.to_snapshot();
    assert!(matches!(catalog.apply(Op::MoveAlbum { id, parent: Some(parent) }), Err(CatalogError::Invalid(_))));
    assert_eq!(catalog.to_snapshot(), before);
}

#[test]
fn journal_refuses_a_cyclic_snapshot_without_repairing_or_rewriting_files() {
    let (catalog, _, _) = cycle();
    let store = MemStore::new();
    let snapshot = serde_json::to_vec(&serde_json::json!({
        "format": "lightcraft-catalog", "version": VERSION, "seq": 0, "catalog": catalog,
    }))
    .unwrap();
    let log = b"torn record".to_vec();
    store.set(SNAPSHOT, snapshot.clone());
    store.set(LOG, log.clone());
    assert!(matches!(Journal::open(Box::new(store.clone())), Err(CatalogError::Corrupt(_))));
    assert_eq!(store.get(SNAPSHOT), Some(snapshot));
    assert_eq!(store.get(LOG), Some(log));
}

#[test]
fn missing_and_non_folder_parents_are_refused() {
    let mut catalog = Catalog::new();
    let a = folder(&mut catalog, None);
    let b = folder(&mut catalog, Some(a));
    catalog.albums.get_mut(&a).unwrap().folder = false;
    assert!(Catalog::from_snapshot(&catalog.to_snapshot()).is_err());
    catalog.albums.get_mut(&b).unwrap().parent = Some(AlbumId(999));
    assert!(Catalog::from_snapshot(&catalog.to_snapshot()).is_err());
}

#[test]
fn a_deep_valid_hierarchy_loads_without_recursion() {
    let mut catalog = Catalog::new();
    let mut parent = None;
    for _ in 0..4096 {
        parent = Some(folder(&mut catalog, parent));
    }
    let reopened = Catalog::from_snapshot(&catalog.to_snapshot()).unwrap();
    assert_eq!(reopened.to_snapshot(), catalog.to_snapshot());
}
