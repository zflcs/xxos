use crate::{fs_pack::easy_fs_pack, PROJECT, TARGET, TARGET_ARCH};
use command_ext::{Cargo, CommandExt};
use serde_derive::Deserialize;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Deserialize)]
struct Apps {
    apps: Cases,
}

#[derive(Deserialize, Default)]
struct Cases {
    pub cases: Option<Vec<String>>,
}

pub struct CasesInfo {
    bins: Vec<PathBuf>,
}

impl Cases {
    fn build(&mut self, release: bool) -> CasesInfo {
        if let Some(names) = &self.cases {
            let cases = names
                .into_iter()
                .enumerate()
                .map(|(_, name)| build_one(name, release))
                .collect();
            CasesInfo {
                bins: cases,
            }
        } else {
            CasesInfo {
                bins: vec![],
            }
        }
    }
}

fn build_one(name: impl AsRef<OsStr>, release: bool) -> PathBuf {
    let name = name.as_ref();
    Cargo::build()
        .package("user_lib")
        .target(TARGET_ARCH)
        .arg("--bin")
        .arg(name)
        .conditional(release, |cargo| {
            cargo.release();
        })
        .invoke();
    let elf = TARGET
        .join(if release { "release" } else { "debug" })
        .join(name);
    elf
}

pub fn build_for(release: bool) {
    let cfg = std::fs::read_to_string(PROJECT.join("user/cases.toml")).unwrap();
    let mut cases = toml::from_str::<Apps>(&cfg).map(|apps| apps.apps)
    .unwrap_or_default();
    let CasesInfo { bins }  = cases.build(release);
    if bins.is_empty() {
        return;
    }
    easy_fs_pack(
        &cases.cases.unwrap(),
        TARGET
            .join(if release { "release" } else { "debug" })
            .into_os_string()
            .into_string()
            .unwrap()
            .as_str(),
    )
    .unwrap();
}
