// mod fs_pack;
// mod user;
// mod module;

#[macro_use]
extern crate clap;

use clap::Parser;
use command_ext::{BinUtil, Cargo, CommandExt, Qemu};
// use module::build_module;
use once_cell::sync::Lazy;
use std::{
    path::{Path, PathBuf},
};

const TARGET_ARCH: &str = "riscv64gc-unknown-none-elf";

static PROJECT: Lazy<&'static Path> =
    Lazy::new(|| Path::new(std::env!("CARGO_MANIFEST_DIR")).parent().unwrap());

static TARGET: Lazy<PathBuf> = Lazy::new(|| PROJECT.join("target").join(TARGET_ARCH));

#[derive(Parser)]
#[clap(name = "rCore-Tutorial")]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Make(BuildArgs),
    Qemu(QemuArgs),
}

fn main() {
    use Commands::*;
    match Cli::parse().command {
        Make(args) => {
            let _ = args.make();
        }
        Qemu(args) => args.run(),
    }
}

#[derive(Args, Default)]
struct BuildArgs {
    /// Character.
    #[clap(short, long)]
    module: Option<String>,
    /// features
    #[clap(short, long)]
    features: Option<String>,
    /// features
    #[clap(long)]
    log: Option<String>,
    /// Build in debug mode.
    #[clap(long)]
    release: bool,
}

impl BuildArgs {
    fn make(&self) -> PathBuf {
        // build_module(false);
        // user::build_for(false);
        
        let package = self.module.as_ref().unwrap();
        // 生成
        let mut build = Cargo::build();
        build
            .package(package)
            .optional(&self.features, |cargo, features| {
                cargo.features(false, features.split_whitespace());
            })
            .optional(&self.log, |cargo, log| {
                cargo.env("LOG", log);
            })
            .conditional(self.release, |cargo| {
                cargo.release();
            })
            .target(TARGET_ARCH);
        build.invoke();
        TARGET
            .join(if self.release { "release" } else { "debug" })
            .join(package)
    }
}


#[derive(Args)]
struct QemuArgs {
    #[clap(flatten)]
    build: BuildArgs,
    /// Path of executable qemu-system-x.
    #[clap(long)]
    qemu_dir: Option<String>,
    /// Number of hart (SMP for Symmetrical Multiple Processor).
    #[clap(long)]
    smp: Option<u8>,
    /// Port for gdb to connect. If set, qemu will block and wait gdb to connect.
    #[clap(long)]
    gdb: Option<u16>,
}

impl QemuArgs {
    fn run(self) {
        let elf = self.build.make();
        if let Some(p) = &self.qemu_dir {
            Qemu::search_at(p);
        }
        let mut qemu = Qemu::system("riscv64");
        qemu.args(&["-machine", "virt"])
            .arg("-nographic")
            .arg("-bios")
            .arg(PROJECT.join("rustsbi-qemu.bin"))
            .arg("-kernel")
            .arg(objcopy(elf, true))
            .args(&["-smp", &self.smp.unwrap_or(1).to_string()])
            .args(&["-serial", "mon:stdio"]);
            // .args(&[
            //     "-drive",
            //     format!(
            //         "file={},if=none,format=raw,id=x0",
            //         TARGET
            //             .join(if self.build.release {
            //                 "release"
            //             } else {
            //                 "debug"
            //             })
            //             .join("fs.img")
            //             .into_os_string()
            //             .into_string()
            //             .unwrap()
            //     )
            //     .as_str(),
            // ])
            // .args(&[
            //     "-device",
            //     "virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0",
            // ]);
        qemu.optional(&self.gdb, |qemu, gdb| {
            qemu.args(&["-S", "-gdb", &format!("tcp::{gdb}")]);
        })
        .invoke();
    }
}

fn objcopy(elf: impl AsRef<Path>, binary: bool) -> PathBuf {
    let elf = elf.as_ref();
    let bin = elf.with_extension("bin");
    BinUtil::objcopy()
        .arg(elf)
        .arg("--strip-all")
        .conditional(binary, |binutil| {
            binutil.args(["-O", "binary"]);
        })
        .arg(&bin)
        .invoke();
    bin
}
