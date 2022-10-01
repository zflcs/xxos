use crate::{TARGET, TARGET_ARCH};
use command_ext::{Cargo, CommandExt};
use std::path::PathBuf;

pub fn build_module(release: bool) -> PathBuf {
    let package = &String::from("unfi-sche");
    // 生成
    let mut build = Cargo::build();
    build
        .package(package)
        .target(TARGET_ARCH);
    build.invoke();
    let module =  TARGET
        .join(if release { "release" } else { "debug" })
        .join(package);
    
    module
}
