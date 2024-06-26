//! This module contains the build related functions

use crate::hasher::Hasher;
use crate::parser::{BuildConfig, OSConfig, TargetConfig};
use crate::utils::features::cfg_feat;
use crate::utils::log::{log, LogLevel};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

static BUILD_DIR: &str = "ruxgo_bld";
static BIN_DIR: &str = "ruxgo_bld/bin";
#[cfg(target_os = "windows")]
static OBJ_DIR: &str = "ruxgo_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str = "ruxgo_bld/obj_linux";

// ruxlibc info and ld script
lazy_static! {
    static ref RUXLIBC_INC: String = {
        let path1 = "../ruxos/ulib/ruxlibc/include";
        let path2 = "../../../ulib/ruxlibc/include";
        if Path::new(path1).exists() {
            String::from(path1)
        } else {
            String::from(path2)
        }
    };
    static ref LD_SCRIPT: String = {
        let path1 = "../ruxos/modules/ruxhal";
        let path2 = "../../../modules/ruxhal";
        if Path::new(path1).exists() {
            String::from(path1)
        } else {
            String::from(path2)
        }
    };
}
static RUXLIBC_BIN: &str = "ruxgo_bld/bin/libc.a";
static RUXLIBC_RUST_LIB: &str = "libruxlibc.a";

// ruxmusl info
static RUXMUSL_INC: &str = "ruxgo_bld/ruxmusl/install/include";
static RUXMUSL_BIN: &str = "ruxgo_bld/ruxmusl/install/lib/libc.a";
static RUXMUSL_RUST_LIB: &str = "libruxmusl.a";

/// Represents a target
pub struct Target<'a> {
    srcs: Vec<Src>,
    build_config: &'a BuildConfig,
    target_config: &'a TargetConfig,
    os_config: &'a OSConfig,
    dependant_includes: HashMap<String, Vec<String>>,
    pub bin_path: String,
    pub elf_path: String,
    hash_file_path: String,
    path_hash: HashMap<String, String>,
    dependant_libs: Vec<Target<'a>>,
}

/// Represents a source file (A single C or Cpp file)
#[derive(Debug)]
struct Src {
    path: String,
    name: String,
    obj_name: String,
    bin_path: String, // consider change to obj_path
    dependant_includes: Vec<String>,
}

impl<'a> Target<'a> {
    /// Creates a new target
    /// # Arguments
    /// * `build_config` - Build config
    /// * `target_config` - Target config
    /// * `targets` - All targets
    pub fn new(
        build_config: &'a BuildConfig,
        os_config: &'a OSConfig,
        target_config: &'a TargetConfig,
        targets: &'a Vec<TargetConfig>,
    ) -> Self {
        let srcs = Vec::new();
        let dependant_includes: HashMap<String, Vec<String>> = HashMap::new();
        let mut bin_path = format!("{}/{}", BIN_DIR, target_config.name);
        let mut elf_path = String::new();
        #[cfg(target_os = "windows")]
        match target_config.typ.as_str() {
            "exe" => bin_path.push_str(".exe"),
            "dll" => bin_path.push_str(".dll"),
            "static" => bin_path.push_str(".lib"),
            _ => (),
        }
        #[cfg(target_os = "linux")]
        match target_config.typ.as_str() {
            "exe" => {
                elf_path = format!("{}.elf", bin_path);
                bin_path.push_str(".bin");
            }
            "dll" => bin_path.push_str(".so"),
            "static" => bin_path.push_str(".a"),
            "object" => bin_path.push_str(".o"),
            _ => (),
        }
        #[cfg(target_os = "windows")]
        let hash_file_path = format!("ruxgo_bld/{}.win32.hash", &target_config.name);
        #[cfg(target_os = "linux")]
        let hash_file_path = format!("ruxgo_bld/{}.linux.hash", &target_config.name);
        let path_hash = Hasher::load_hashes_from_file(&hash_file_path);
        let mut dependant_libs = Vec::new();

        // add dependant libs
        for dependant_lib in &target_config.deps {
            for target in targets {
                if target.name == *dependant_lib {
                    dependant_libs.push(Target::new(build_config, os_config, target, targets));
                }
            }
        }

        // check types of the dependant libs
        for dep_lib in &dependant_libs {
            if dep_lib.target_config.typ != "dll"
                && dep_lib.target_config.typ != "static"
                && dep_lib.target_config.typ != "object"
            {
                log(
                    LogLevel::Error,
                    "Can add only dll, static or object libs as dependant libs",
                );
                log(
                    LogLevel::Error,
                    &format!(
                        "Target: {} is not a dll, static or object library",
                        dep_lib.target_config.name
                    ),
                );
                log(
                    LogLevel::Error,
                    &format!(
                        "Target: {} is a {}",
                        dep_lib.target_config.name, dep_lib.target_config.typ
                    ),
                );
                std::process::exit(1);
            }
            log(
                LogLevel::Info,
                &format!("Adding dependant lib: {}", dep_lib.target_config.name),
            );
            if dep_lib.target_config.typ == "dll" && !dep_lib.target_config.name.starts_with("lib")
            {
                log(
                    LogLevel::Error,
                    "Dependant dll lib name must start with lib",
                );
                log(
                    LogLevel::Error,
                    &format!(
                        "Target: {} does not start with lib",
                        dep_lib.target_config.name
                    ),
                );
                std::process::exit(1);
            }
        }
        if target_config.deps.len() > dependant_libs.len() {
            log(LogLevel::Error, "Dependant libs not found!");
            log(
                LogLevel::Error,
                &format!("Dependant libs: {:?}", target_config.deps),
            );
            log(
                LogLevel::Error,
                &format!(
                    "Found libs: {:?}",
                    targets
                        .iter()
                        .map(|x| {
                            if x.typ == "dll" || x.typ == "static" || x.typ == "object" {
                                x.name.clone()
                            } else {
                                "".to_string()
                            }
                        })
                        .collect::<Vec<String>>()
                        .into_iter()
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<String>>()
                ),
            );
            std::process::exit(1);
        }
        let mut target = Target::<'a> {
            srcs,
            build_config,
            target_config,
            os_config,
            dependant_includes,
            bin_path,
            elf_path,
            path_hash,
            hash_file_path,
            dependant_libs,
        };
        target.get_srcs(&target_config.src);
        target
    }

    /// Builds the target
    /// # Arguments
    /// * `gen_cc` - Generate compile_commands.json
    /// * `relink` - Determine whether to re-link
    pub fn build(&mut self, gen_cc: bool, relink: bool) {
        let mut to_link: bool = false;

        // if the source file needs to be build, then to link
        let mut link_causer: Vec<&str> = Vec::new();
        let mut srcs_needed = 0;
        let total_srcs = self.srcs.len();
        let mut src_ccs = Vec::new();
        for src in &self.srcs {
            let (to_build, _) = src.to_build(&self.path_hash);
            if to_build {
                to_link = true;
                link_causer.push(&src.path);
                srcs_needed += 1;
            }
            if gen_cc {
                src_ccs.push(self.gen_cc(src));
            }
        }

        // if the source file is empty and dependant_libs is not empty, then to link
        if self.srcs.is_empty() && !self.dependant_libs.is_empty() {
            to_link = true;
        }

        // if the os config changes, then to link
        if relink {
            to_link = true
        }

        if gen_cc {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open("./compile_commands.json")
                .unwrap();
            for src_cc in src_ccs {
                if let Err(e) = writeln!(file, "{},", src_cc) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }

        // log output when to link
        if to_link {
            log(
                LogLevel::Log,
                &format!("Compiling Target: {}", &self.target_config.name),
            );
            if srcs_needed > 0 {
                log(
                    LogLevel::Log,
                    &format!(
                        "\t {} of {} source files have to be compiled",
                        srcs_needed, total_srcs
                    ),
                );
            }
            if self.srcs.is_empty() && !self.dependant_libs.is_empty() {
                for dep_lib in &self.dependant_libs {
                    log(
                        LogLevel::Log,
                        &format!("\t {} have to be linked", dep_lib.bin_path),
                    );
                }
            }
            if !Path::new(OBJ_DIR).exists() {
                fs::create_dir(OBJ_DIR).unwrap_or_else(|why| {
                    log(
                        LogLevel::Error,
                        &format!("Couldn't create obj dir: {}", why),
                    );
                    std::process::exit(1);
                });
            }
        } else {
            log(
                LogLevel::Log,
                &format!("Target: {} is up to date", &self.target_config.name),
            );
            return;
        }

        // parallel built
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new(srcs_needed as u64)));
        let num_complete = Arc::new(Mutex::new(0));
        let src_hash_to_update = Arc::new(Mutex::new(Vec::new()));
        let warns = Arc::new(Mutex::new(Vec::new()));
        self.srcs.par_iter().for_each(|src| {
            let (to_build, _message) = src.to_build(&self.path_hash);
            //log(LogLevel::Debug, &format!("{} => {}", src.path, to_build));
            if to_build {
                let warn = src.build(
                    self.build_config,
                    self.os_config,
                    self.target_config,
                    &self.dependant_libs,
                );
                if let Some(warn) = warn {
                    warns.lock().unwrap().push(warn);
                }
                src_hash_to_update.lock().unwrap().push(src);
                log(LogLevel::Info, &format!("Compiled: {}", src.path));
                // If the RUXGO_LOG_LEVEL is not "Info" or "Debug", update the compilation progress bar
                let log_level = std::env::var("RUXGO_LOG_LEVEL").unwrap_or("".to_string());
                if !(log_level == "Info" || log_level == "Debug") {
                    let mut num_complete = num_complete.lock().unwrap();
                    *num_complete += 1;
                    let progress_bar = progress_bar.lock().unwrap();
                    let template = format!(
                        "    {}{}",
                        "Compiling :".cyan(),
                        "[{bar:40.}] {pos}/{len} ({percent}%) {msg}[{elapsed_precise}] "
                    );
                    progress_bar.set_style(
                        ProgressStyle::with_template(&template)
                            .unwrap()
                            .progress_chars("=>-"),
                    );
                    progress_bar.inc(1);
                }
            }
        });
        let warns = warns.lock().unwrap();
        if warns.len() > 0 {
            log(LogLevel::Warn, "Warnings emitted during build:");
            for warn in warns.iter() {
                log(LogLevel::Warn, &format!("\t{}", warn));
            }
        }
        for src in src_hash_to_update.lock().unwrap().iter() {
            Hasher::save_hash(&src.path, &mut self.path_hash);
        }

        // links the target
        if to_link {
            for src in link_causer {
                log(LogLevel::Info, &format!("\tLinking file: {}", &src));
            }
            for src in &self.srcs {
                for include in &src.dependant_includes {
                    Hasher::save_hash(include, &mut self.path_hash);
                }
            }
            Hasher::save_hashes_to_file(&self.hash_file_path, &self.path_hash);
            self.link(&self.dependant_libs);
        }
    }

    /// Links the dependant libs(or targets)
    /// # Arguments
    /// * `dep_targets` - The targets that this target depends on
    pub fn link(&self, dep_targets: &Vec<Target>) {
        let mut objs = Vec::new();
        if !Path::new(BIN_DIR).exists() {
            fs::create_dir_all(BIN_DIR).unwrap_or_else(|why| {
                log(
                    LogLevel::Error,
                    &format!("Couldn't create build dir: {}", why),
                );
                std::process::exit(1);
            })
        }
        for src in &self.srcs {
            objs.push(&src.obj_name);
        }
        let mut cmd = String::new();
        let mut cmd_bin = String::new();
        if self.target_config.typ == "dll" {
            cmd = self.link_dll(objs, dep_targets);
        } else if self.target_config.typ == "static" {
            cmd = self.link_static(objs);
        } else if self.target_config.typ == "object" {
            cmd = self.link_object(objs, dep_targets);
        } else if self.target_config.typ == "exe" {
            (cmd, cmd_bin) = self.link_exe(objs, dep_targets);
        }

        log(
            LogLevel::Log,
            &format!("Linking target: {}", &self.target_config.name),
        );
        log(LogLevel::Info, &format!("  Command: {}", &cmd));
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("failed to execute process");
        if output.status.success() {
            log(LogLevel::Log, "Linking successful");
            Hasher::save_hashes_to_file(&self.hash_file_path, &self.path_hash); // ? check if repeated
        } else {
            log(LogLevel::Error, "Linking failed");
            log(LogLevel::Error, &format!(" Command: {}", &cmd));
            log(
                LogLevel::Error,
                &format!("  Error: {}", String::from_utf8_lossy(&output.stderr)),
            );
            std::process::exit(1);
        }
        if !cmd_bin.is_empty() {
            let output_bin = Command::new("sh")
                .arg("-c")
                .arg(&cmd_bin)
                .output()
                .expect("failed to execute process");
            if output_bin.status.success() {
                log(LogLevel::Info, &format!(" Bin_path: {}", &self.bin_path));
                log(LogLevel::Info, &format!(" Elf_path: {}", &self.elf_path));
            } else {
                log(LogLevel::Error, "  Rust-objcopy failed");
                log(LogLevel::Error, &format!(" Command: {}", &cmd_bin));
                log(
                    LogLevel::Error,
                    &format!("  Error: {}", String::from_utf8_lossy(&output_bin.stderr)),
                );
                std::process::exit(1);
            }
        }
    }

    /// Links the dll targets
    fn link_dll(&self, objs: Vec<&String>, dep_targets: &Vec<Target>) -> String {
        let mut cmd = String::new();
        if !self.target_config.linker.is_empty() {
            cmd.push_str(&self.target_config.linker);
        } else {
            cmd.push_str(&self.build_config.compiler.read().unwrap());
        }
        cmd.push_str(" -shared");
        cmd.push_str(" -o ");
        cmd.push_str(&self.bin_path);
        for obj in objs {
            cmd.push(' ');
            cmd.push_str(obj);
        }
        cmd.push(' ');

        // link other dependant libraries
        for dep_target in dep_targets {
            dep_target
                .target_config
                .include_dir
                .iter()
                .for_each(|include| {
                    cmd.push_str(" -I");
                    cmd.push_str(include);
                });
            cmd.push(' ');
            let lib_name = dep_target.target_config.name.clone();
            let lib_name = lib_name.replace("lib", "-l");
            cmd.push_str(&lib_name);
            cmd.push(' ');
        }

        // add -L library search path
        if !self.dependant_libs.is_empty() {
            cmd.push_str(" -L");
            cmd.push_str(BIN_DIR);
            cmd.push_str(" -Wl,-rpath,\'$ORIGIN\' "); // '$ORIGIN' represents the directory path where the executable is located
            cmd.push(' ');
        }

        // add ldflags
        cmd.push_str(&self.target_config.ldflags);

        cmd
    }

    /// Links the static targets
    fn link_static(&self, objs: Vec<&String>) -> String {
        let mut cmd = String::new();
        cmd.push_str(&self.target_config.archive);
        cmd.push(' ');
        cmd.push_str(&self.target_config.ldflags);
        cmd.push(' ');
        cmd.push_str(&self.bin_path);
        for obj in objs {
            cmd.push(' ');
            cmd.push_str(obj);
        }

        cmd
    }

    /// Links the object targets
    fn link_object(&self, objs: Vec<&String>, dep_targets: &Vec<Target>) -> String {
        let mut cmd = String::new();
        if !self.target_config.linker.is_empty() {
            cmd.push_str(&self.target_config.linker);
        } else {
            cmd.push_str(&self.build_config.compiler.read().unwrap());
        }
        cmd.push(' ');
        cmd.push_str(&self.target_config.ldflags);
        cmd.push_str(" -o ");
        cmd.push_str(&self.bin_path);
        for obj in objs {
            cmd.push(' ');
            cmd.push_str(obj);
        }
        // link other dependant libraries
        for dep_target in dep_targets {
            cmd.push(' ');
            cmd.push_str(&dep_target.bin_path);
        }

        cmd
    }

    /// Links the executable targets
    fn link_exe(&self, objs: Vec<&String>, dep_targets: &Vec<Target>) -> (String, String) {
        let mut cmd = String::new();
        let mut cmd_bin = String::new();
        if !self.target_config.linker.is_empty() {
            cmd.push_str(&self.target_config.linker);
        } else {
            cmd.push_str(&self.build_config.compiler.read().unwrap());
        }
        cmd.push(' ');

        // consider os config
        if !self.os_config.name.is_empty() {
            // add os_ldflags and target_config.ldflags
            let mut ldflags = String::new();
            let mut os_ldflags = String::new();
            os_ldflags.push_str("-nostdlib -static -no-pie --gc-sections");
            let ld_script = format!(
                "{}/linker_{}.lds",
                LD_SCRIPT.as_str(),
                self.os_config.platform.name
            );
            os_ldflags.push_str(&format!(" -T{}", &ld_script));
            if self.os_config.platform.arch == *"x86_64" {
                os_ldflags.push_str(" --no-relax");
            }
            ldflags.push_str(&os_ldflags);
            ldflags.push(' ');
            ldflags.push_str(&self.target_config.ldflags);
            cmd.push_str(&ldflags);

            // link ulib and os
            if self.os_config.ulib == "ruxlibc" {
                cmd.push(' ');
                cmd.push_str(RUXLIBC_BIN);
                cmd.push(' ');
                let mode = if !self.os_config.platform.mode.is_empty() {
                    &self.os_config.platform.mode
                } else {
                    "debug"
                };
                cmd.push_str(&format!(
                    "{}/target/{}/{}/{}",
                    BUILD_DIR, &self.os_config.platform.target, mode, RUXLIBC_RUST_LIB
                ));
            } else if self.os_config.ulib == "ruxmusl" {
                cmd.push(' ');
                cmd.push_str(RUXMUSL_BIN);
                cmd.push(' ');
                let mode = if !self.os_config.platform.mode.is_empty() {
                    &self.os_config.platform.mode
                } else {
                    "debug"
                };
                cmd.push_str(&format!(
                    "{}/target/{}/{}/{}",
                    BUILD_DIR, &self.os_config.platform.target, mode, RUXMUSL_RUST_LIB
                ));
            }

            // link other obj
            for obj in objs {
                cmd.push(' ');
                cmd.push_str(obj);
            }

            // link other dependant libraries
            for dep_target in dep_targets {
                cmd.push(' ');
                cmd.push_str(&dep_target.bin_path);
            }
            cmd.push_str(" -o ");
            cmd.push_str(&self.elf_path);

            // generate a bin file
            cmd_bin.push_str(&format!(
                "rust-objcopy --binary-architecture={}",
                &self.os_config.platform.arch
            ));
            cmd_bin.push(' ');
            cmd_bin.push_str(&self.elf_path);
            cmd_bin.push_str(" --strip-all -O binary ");
            cmd_bin.push_str(&self.bin_path);
        } else {
            cmd.push_str(" -o ");
            cmd.push_str(&self.bin_path);
            for obj in objs {
                cmd.push(' ');
                cmd.push_str(obj);
            }
            cmd.push(' ');
            // link other dependant libraries
            for dep_target in dep_targets {
                if dep_target.target_config.typ == "object"
                    || dep_target.target_config.typ == "static"
                {
                    cmd.push_str(&dep_target.bin_path);
                    cmd.push(' ');
                } else if dep_target.target_config.typ == "dll" {
                    dep_target
                        .target_config
                        .include_dir
                        .iter()
                        .for_each(|include| {
                            cmd.push_str(" -I");
                            cmd.push_str(include);
                        });
                    cmd.push(' ');
                    let lib_name = dep_target.target_config.name.clone();
                    let lib_name = lib_name.replace("lib", "-l");
                    cmd.push_str(&lib_name);
                    cmd.push(' ');
                    // added -L library search path
                    cmd.push_str(" -L");
                    cmd.push_str(BIN_DIR);
                    cmd.push_str(" -Wl,-rpath,\'$ORIGIN\' "); // '$ORIGIN' represents the directory path where the executable is located
                    cmd.push(' ');
                }
            }
            cmd.push_str(&self.target_config.ldflags);
        }

        (cmd, cmd_bin)
    }

    /// Generates the compile_commands.json file for a src
    fn gen_cc(&self, src: &Src) -> String {
        let mut cc = String::new();
        cc.push_str("{\n"); // Json start
        if *self.build_config.compiler.read().unwrap() == "clang++"
            || *self.build_config.compiler.read().unwrap() == "g++"
        {
            cc.push_str("\t\"command\": \"c++");
        } else if *self.build_config.compiler.read().unwrap() == "clang"
            || *self.build_config.compiler.read().unwrap() == "gcc"
        {
            cc.push_str("\t\"command\": \"cc");
        } else {
            log(
                LogLevel::Error,
                &format!(
                    "Compiler: {} is not supported",
                    &self.build_config.compiler.read().unwrap()
                ),
            );
            log(
                LogLevel::Error,
                "Supported compilers: clang++, g++, clang, gcc",
            );
            std::process::exit(1);
        }
        cc.push_str(" -c -o ");
        cc.push_str(&src.obj_name);
        self.target_config.include_dir.iter().for_each(|include| {
            cc.push_str(" -I");
            cc.push_str(include);
        });

        for lib in &self.dependant_libs {
            lib.target_config.include_dir.iter().for_each(|include| {
                cc.push_str(" -I");
                cc.push_str(include);
            });
        }

        cc.push(' ');
        let cflags = &self.target_config.cflags;

        let subcmds = cflags.split('`').collect::<Vec<&str>>();
        // Take even entries are non-subcmds and odd entries are subcmds
        let (subcmds, non_subcmds): (Vec<String>, String) = subcmds.iter().enumerate().fold(
            (Vec::new(), String::new()),
            |(mut subcmds, mut non_subcmds), (i, subcmd)| {
                if i % 2 != 0 {
                    subcmds.push(subcmd.to_string());
                } else {
                    non_subcmds.push_str(subcmd);
                    non_subcmds.push(' ');
                }
                (subcmds, non_subcmds)
            },
        );

        cc.push_str(&non_subcmds);

        for subcmd in subcmds {
            let cmd_output = Command::new("sh")
                .arg("-c")
                .arg(&subcmd)
                .output()
                .expect("failed to execute process");
            if cmd_output.status.success() {
                let stdout = String::from_utf8_lossy(&cmd_output.stdout);
                let stdout = stdout.replace('\n', " ");
                cc.push_str(&stdout);
            } else {
                let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                log(
                    LogLevel::Error,
                    &format!("Failed to execute subcmd: {}", &subcmd),
                );
                log(LogLevel::Error, &format!("  Stderr: {}", stderr));
                std::process::exit(1);
            }
        }

        #[cfg(target_os = "linux")]
        if self.target_config.typ == "dll" {
            cc.push_str("-fPIC ");
        }

        cc.push_str(&src.path);
        cc.push_str("\",\n"); // Json end
                              // other info: "directory","file"
        let mut dirent = String::new();
        dirent.push_str("\t\"directory\": \"");
        dirent.push_str(
            &std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/"),
        );
        dirent.push_str("\",\n");
        let dirent = dirent.replace('/', "\\\\").replace("\\\\.\\\\", "\\\\"); // aim to Windows
        cc.push_str(&dirent);
        let mut fileent = String::new();
        fileent.push_str("\t\"file\": \"");
        fileent.push_str(
            &std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/"),
        );
        fileent.push('/');
        fileent.push_str(&src.path);
        fileent.push('\"');
        let fileent = fileent.replace('/', "\\\\").replace("\\\\.\\\\", "\\\\");
        cc.push_str(&fileent);

        cc.push_str("\n}");
        #[cfg(target_os = "linux")]
        return cc.replace("\\\\", "/");
        #[cfg(target_os = "windows")]
        return cc;
    }

    /// Recursively gets all the source files in the given root path
    /// # Notes
    /// The source is first filtered through the `src_only` and `src_exclude` fields
    fn get_srcs(&mut self, root_path: &str) {
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let path_str = path.to_str().unwrap_or_default();
            #[cfg(target_os = "windows")]
            let path_str = path_str.replace('\\', "/");
            if self.should_exclude(path_str) {
                continue;
            }
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if (ext == "cpp" || ext == "c") && self.should_include(path_str) {
                        self.add_src(path_str.to_owned());
                    }
                }
            }
        }
    }

    /// Exclusion logic: Check if the path is in src_exclude
    fn should_exclude(&self, path: &str) -> bool {
        self.target_config
            .src_exclude
            .iter()
            .any(|excluded| path.contains(excluded))
    }

    /// Inclusion logic: Apply src_only logic only to files
    fn should_include(&self, path: &str) -> bool {
        if self.target_config.src_only.is_empty() {
            return true;
        }
        self.target_config
            .src_only
            .iter()
            .any(|included| path.contains(included))
    }

    /// Adds a source file to the target's srcs field
    fn add_src(&mut self, path: String) {
        let name = Target::get_src_name(&path);
        let obj_name = self.get_src_obj_name(&name);
        let dependant_includes = self.get_dependant_includes(&path);
        let bin_path = self.bin_path.clone();
        self.srcs
            .push(Src::new(path, name, obj_name, bin_path, dependant_includes));
    }

    /// Returns the file name without the extension from the path
    fn get_src_name(path: &str) -> String {
        let path_buf = PathBuf::from(path);
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let name = file_name.split('.').next().unwrap();
        name.to_string()
    }

    /// Returns the object file name corresponding to the source file
    fn get_src_obj_name(&self, src_name: &str) -> String {
        let mut obj_name = String::new();
        obj_name.push_str(OBJ_DIR);
        obj_name.push('/');
        obj_name.push_str(&self.target_config.name);
        obj_name.push('-');
        obj_name.push_str(src_name);
        obj_name.push_str(".o");
        obj_name
    }

    /// Returns a vector of .h or .hpp files the given C/C++ depends on
    fn get_dependant_includes(&mut self, path: &str) -> Vec<String> {
        let mut result = HashSet::new();
        // Use the stack to handle recursive paths
        let mut to_process = vec![path.to_string()];
        let include_substrings: HashSet<String> = self
            .get_include_substrings(path)
            .unwrap_or_default()
            .into_iter()
            .collect();
        if include_substrings.is_empty() {
            return Vec::new();
        }
        while let Some(current_path) = to_process.pop() {
            if !result.insert(current_path.clone()) {
                // If this path has already been processed, skip it
                continue;
            }
            if !self.dependant_includes.contains_key(&current_path) {
                self.dependant_includes
                    .insert(current_path.clone(), Vec::new());
            }

            for include_dir in &self.target_config.include_dir {
                for entry in WalkDir::new(include_dir).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file()
                        && include_substrings
                            .iter()
                            .any(|substring| path.ends_with(substring))
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if result.insert(path_str.clone()) {
                            self.dependant_includes
                                .get_mut(&current_path)
                                .unwrap()
                                .push(path_str.clone());
                            to_process.push(path_str);
                        }
                    }
                }
            }
        }

        result.into_iter().collect()
    }

    /// Returns a list of substrings that contain "#include \"" in the source file
    fn get_include_substrings(&self, path: &str) -> Option<Vec<String>> {
        let file = std::fs::File::open(path);
        if file.is_err() {
            // If the software is self-developed, enable this debug option
            //log(LogLevel::Debug, &format!("Failed to get include substrings for file: {}", path));
            return None;
        }
        let mut file = file.unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        let lines = buf.lines();
        let mut include_substrings = Vec::new();
        for line in lines {
            if line.starts_with("#include \"") {
                let include_path = line.split('\"').nth(1).unwrap().to_owned();
                include_substrings.push(include_path);
            }
        }
        Some(include_substrings)
    }
}

impl Src {
    /// Creates a new source file
    fn new(
        path: String,
        name: String,
        obj_name: String,
        bin_path: String,
        dependant_includes: Vec<String>,
    ) -> Self {
        Self {
            path,
            name,
            obj_name,
            bin_path,
            dependant_includes,
        }
    }

    /// Determines whether the object file needs to be rebuilt
    fn to_build(&self, path_hash: &HashMap<String, String>) -> (bool, String) {
        if !Path::new(&self.bin_path).exists() {
            let result = (true, format!("\tBinary does not exist: {}", &self.bin_path));
            return result;
        }

        if Hasher::is_file_changed(&self.path, path_hash) {
            let result = (true, format!("\tSource file has changed: {}", &self.path));
            return result;
        }
        for dependant_include in &self.dependant_includes {
            if Hasher::is_file_changed(&dependant_include.clone(), path_hash) {
                let result = (
                    true,
                    format!(
                        "\tSource file: {} depends on changed include file: {}",
                        &self.path, &dependant_include
                    ),
                );
                return result;
            }
        }

        (
            false,
            format!("Source file: {} does not need to be built", &self.path),
        )
    }

    /// Builds the source files
    fn build(
        &self,
        build_config: &BuildConfig,
        os_config: &OSConfig,
        target_config: &TargetConfig,
        dependant_libs: &Vec<Target>,
    ) -> Option<String> {
        let mut cmd = String::new();
        cmd.push_str(&build_config.compiler.read().unwrap());
        // If os exist
        let mut os_cflags = String::new();
        if !os_config.name.is_empty() {
            os_cflags.push_str("-nostdinc -fno-builtin -ffreestanding -Wall");
            if os_config.ulib == "ruxlibc" {
                os_cflags.push_str(" -isystem");
                os_cflags.push_str(RUXLIBC_INC.as_str());
                let (_, lib_feats) = cfg_feat(os_config);
                // generate the preprocessing macro definition
                for lib_feat in lib_feats {
                    let processed_lib_feat = lib_feat.to_uppercase().replace('-', "_");
                    os_cflags.push_str(&format!(" -DRUX_CONFIG_{}", &processed_lib_feat));
                }
                os_cflags.push_str(&format!(
                    " -DRUX_CONFIG_{}",
                    os_config.platform.log.to_uppercase()
                ));
            } else if os_config.ulib == "ruxmusl" {
                os_cflags.push_str(" -isystem");
                os_cflags.push_str(RUXMUSL_INC);
            }
            if os_config.platform.mode == "release" {
                os_cflags.push_str(" -O3");
            }
            if os_config.platform.arch == "riscv64" {
                os_cflags.push_str(" -march=rv64gc -mabi=lp64d -mcmodel=medany");
            }
            if !os_config.features.contains(&"fp_simd".to_string()) {
                if os_config.platform.arch == *"x86_64".to_string() {
                    os_cflags.push_str(" -mno-sse");
                } else if os_config.platform.arch == *"aarch64".to_string() {
                    os_cflags.push_str(" -mgeneral-regs-only");
                }
            }
        }

        // Add cflags
        let mut cflags = String::new();
        if !os_cflags.is_empty() {
            cflags.push_str(&os_cflags);
            cflags.push(' ');
        }
        cflags.push_str(&target_config.cflags);
        cmd.push(' ');
        cmd.push_str(&cflags);
        target_config.include_dir.iter().for_each(|include| {
            cmd.push_str(" -I");
            cmd.push_str(include);
        });
        cmd.push_str(" -o ");
        cmd.push_str(&self.obj_name);

        // consider some includes in other depandant_libs
        for dependant_lib in dependant_libs {
            dependant_lib
                .target_config
                .include_dir
                .iter()
                .for_each(|include| {
                    cmd.push_str(" -I");
                    cmd.push_str(include);
                });
        }

        cmd.push_str(" -c ");
        cmd.push_str(&self.path);

        if target_config.typ == "dll" {
            cmd.push_str(" -fPIC");
        }

        log(LogLevel::Info, &format!("Building: {}", &self.name));
        log(LogLevel::Info, &format!("  Command: {}", &cmd));
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("failed to execute process");
        if output.status.success() {
            log(LogLevel::Info, &format!("  Success: {}", &self.name));
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.len() > 0 {
                log(LogLevel::Info, &format!("  Stdout: {}", stdout));
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.len() > 0 {
                return Some(stderr.to_string());
            }
            None
        } else {
            log(LogLevel::Error, &format!("  Command: {}", &cmd));
            log(
                LogLevel::Error,
                &format!("  Stdout: {}", String::from_utf8_lossy(&output.stdout)),
            );
            log(
                LogLevel::Error,
                &format!("  Stderr: {}", String::from_utf8_lossy(&output.stderr)),
            );
            std::process::exit(1);
        }
    }
}
