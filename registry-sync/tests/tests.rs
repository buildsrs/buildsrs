use buildsrs_database::*;
use buildsrs_registry_sync::Syncer;
use crates_index::{git::URL, GitIndex};
use gix::{
    actor::Signature,
    diff::object::{
        date::Time,
        tree::{Entry, EntryKind},
        Tree,
    },
    ObjectId,
};
use proptest::{arbitrary::any, strategy::Strategy};
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};
use tempfile::TempDir;
use test_strategy::*;

#[derive(Arbitrary, Debug, Clone)]
pub struct Crate {
    #[strategy("[a-z][a-z0-9_]{3,10}")]
    pub name: String,
    #[strategy(any::<Vec<Version>>()
        .prop_map(|versions|
            versions
            .into_iter()
            .map(|x| (x.version.clone(), x)).collect()
        )
    )]
    #[filter(!#versions.is_empty())]
    pub versions: BTreeMap<String, Version>,
}

impl Crate {
    fn encode(self) -> String {
        let versions: Vec<_> = self
            .versions
            .into_iter()
            .map(|(_name, version)| IndexVersion {
                name: self.name.clone(),
                vers: version.version,
                cksum: version.checksum,
                deps: Default::default(),
                features: Default::default(),
                yanked: version.yanked,
            })
            .map(|row| serde_json::to_string(&row).unwrap())
            .collect();
        versions.join("\n")
    }
}

#[derive(Arbitrary, Debug, Clone)]
pub struct Version {
    #[strategy("[1-9][0-9]{0,2}\\.[1-9][0-9]{0,2}\\.[1-9][0-9]{0,2}")]
    pub version: String,
    pub yanked: bool,
    pub checksum: [u8; 32],
}

#[derive(Serialize)]
pub struct IndexVersion {
    pub name: String,
    pub vers: String,
    pub deps: Vec<()>,
    #[serde(with = "hex")]
    pub cksum: [u8; 32],
    pub features: BTreeMap<(), ()>,
    pub yanked: bool,
}

#[proptest(async = "tokio", cases = 1)]
async fn can_sync(crates: Vec<Crate>) {
    let tempdir = TempDir::new().unwrap();
    let repository = gix::init_bare(tempdir.path()).unwrap();

    let mut partitioned: BTreeMap<String, BTreeMap<String, BTreeMap<String, Crate>>> =
        Default::default();
    for krate in crates.iter() {
        let crates = partitioned.entry(krate.name[0..2].into()).or_default();
        let crates = crates.entry(krate.name[2..4].into()).or_default();
        crates.insert(krate.name.clone(), krate.clone());
    }

    // build fake git repository that looks like a valid crates index
    let tree = Tree {
        entries: partitioned
            .iter()
            .map(|(prefix1, crates)| {
                let tree = Tree {
                    entries: crates
                        .iter()
                        .map(|(prefix2, crates)| {
                            let tree = Tree {
                                entries: crates
                                    .iter()
                                    .map(|(name, krate)| Entry {
                                        mode: EntryKind::Blob.into(),
                                        filename: name.as_str().into(),
                                        oid: repository
                                            .write_blob(krate.clone().encode())
                                            .unwrap()
                                            .into(),
                                    })
                                    .collect(),
                            };

                            Entry {
                                mode: EntryKind::Tree.into(),
                                filename: prefix2.as_str().into(),
                                oid: repository.write_object(tree).unwrap().into(),
                            }
                        })
                        .collect(),
                };

                Entry {
                    mode: EntryKind::Tree.into(),
                    filename: prefix1.as_str().into(),
                    oid: repository.write_object(tree).unwrap().into(),
                }
            })
            .collect(),
    };

    let tree = repository.write_object(tree).unwrap();
    let author = Signature {
        name: "John Doe".into(),
        email: "john.doe@example.com".into(),
        time: Time::now_local_or_utc(),
    };
    repository
        .commit_as(
            &author,
            &author,
            "FETCH_HEAD",
            "Initial commit",
            tree,
            std::iter::empty::<ObjectId>(),
        )
        .unwrap();

    // open git index
    let index = GitIndex::try_with_path(tempdir.path(), URL)
        .unwrap()
        .unwrap();

    // create temporary database
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let database = TempDatabase::create(&host, None).await.unwrap();

    // create syncer
    let syncer = Syncer::new(Arc::new(database.pool().clone()), index);

    // perform sync
    syncer.sync().await.unwrap();

    // verify crates are there
    let handle = database.pool().read().await.unwrap();
    for krate in crates.iter() {
        let _info = handle.crate_info(&krate.name).await.unwrap();
    }

    // cleanup
    drop(syncer);
    drop(handle);
    database.delete().await.unwrap();
}
