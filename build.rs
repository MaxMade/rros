use std::fs;
use std::io::{Read, Write};
use std::path;

extern crate yaml_rust;

#[derive(Debug)]
struct ConfigOptions {
    name: String,
    value: String,
    ty: String,
    description: String,
}

#[derive(Debug)]
struct LevelDescription {
    name: String,
    value: usize,
    description: String,
}

impl Eq for LevelDescription {}

impl PartialEq for LevelDescription {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Ord for LevelDescription {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for LevelDescription {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

fn parse_config_yaml() -> Vec<ConfigOptions> {
    // Read config file
    let mut config_file = fs::File::open(path::Path::new("config.yaml")).unwrap();
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer).unwrap();

    // Parse YAML
    let mut config_options = Vec::new();
    let config = yaml_rust::YamlLoader::load_from_str(config_buffer.as_str())
        .unwrap()
        .pop()
        .unwrap();
    for (key, values) in config.as_hash().unwrap().iter() {
        // Get name of config option
        let name = key.as_str().unwrap().to_string();

        // Process config option
        let value = if let Some(value) = values["value"].as_str() {
            value.to_string()
        } else if let Some(value) = values["value"].as_i64() {
            value.to_string()
        } else {
            panic!("Unable to process value of configuration!");
        };
        let ty = values["type"].as_str().unwrap().to_string();
        let description = values["description"].as_str().unwrap().to_string();

        // Add config option to list
        let config_option = ConfigOptions {
            name,
            value,
            ty,
            description,
        };
        config_options.push(config_option);
    }

    return config_options;
}

fn generate_config_rs(configs_options: &[ConfigOptions]) {
    // Open output file
    let mut config_file = fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./src/config.rs")
        .unwrap();

    // Add module documentation
    writeln!(
        config_file,
        "//! Tailor your system simply by using `config.yaml` configuration file.
//!
//! # Caution
//! This file is auto-generated using the `build.rs` script! Do not change any values here, as those
//! might be overwritten by the next invocation of `cargo build`.
",
    )
    .unwrap();

    // Generate constants
    for config_option in configs_options {
        let mut var_name = config_option.name.trim().replace("CONFIG_", "");
        var_name.make_ascii_uppercase();

        writeln!(config_file, "/// {}", config_option.description.trim(),).unwrap();

        writeln!(
            config_file,
            "pub const {}: {} = {};",
            var_name,
            config_option.ty.trim(),
            config_option.value.trim()
        )
        .unwrap();
    }
}

fn parse_level_yaml() -> Vec<LevelDescription> {
    // Read config file
    let mut config_file = fs::File::open(path::Path::new("levels.yaml")).unwrap();
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer).unwrap();

    // Parse YAML
    let mut level_descs = Vec::new();
    let config = yaml_rust::YamlLoader::load_from_str(config_buffer.as_str())
        .unwrap()
        .pop()
        .unwrap();
    for (key, values) in config.as_hash().unwrap().iter() {
        // Get name of config option
        let name = key.as_str().unwrap().to_string();

        // Process config options
        let value = values["value"].as_i64().unwrap() as usize;
        let description = values["description"].as_str().unwrap().to_string();

        // Add config option to list
        let level_desc = LevelDescription {
            name,
            value,
            description,
        };
        level_descs.push(level_desc);
    }

    // Sanity check: Levels must be continous!
    level_descs.sort();
    for (i, level_desc) in level_descs.iter().enumerate() {
        if level_desc.value != i {
            panic!(
                "Found non-continous level value for {}: Got {}, Expected {}",
                level_desc.name, level_desc.value, i
            );
        }
    }
    level_descs.reverse();

    return level_descs;
}

fn generate_level_rs(level_descs: &[LevelDescription]) {
    // Open output file
    let mut level_file = fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./src/sync/level.rs")
        .unwrap();

    // Add module documentation
    writeln!(
        level_file,
        "//! Practical apprach for deadlock prevention: Use lock hierarchies!"
    )
    .unwrap();

    // Draw ascii art
    let text_width = level_descs
        .iter()
        .map(|config_desc| config_desc.name.len())
        .max()
        .unwrap();
    writeln!(level_file, "//! ```ascii").unwrap();
    for (i, level_desc) in level_descs.iter().enumerate() {
        writeln!(level_file, "//! ┌{:─<1$}┐", "", text_width + 7).unwrap();
        writeln!(
            level_file,
            "//! │ Level{: <1$} │",
            level_desc.name, text_width
        )
        .unwrap();
        writeln!(level_file, "//! └{:─<1$}┘", "", text_width + 7).unwrap();

        if i != level_descs.len() - 1 {
            writeln!(level_file, "//! enter │ ▲").unwrap();
            writeln!(level_file, "//!       ▼ │ leave").unwrap();
        }
    }
    writeln!(level_file, "//! ```").unwrap();

    // Add use statements
    writeln!(level_file, "use core::marker::PhantomData;").unwrap();

    // Add Level trait
    writeln!(
        level_file,
        "
/// Trait to abstract a level within the hierarchy.
pub trait Level
where
    Self: Sized,
{{
    /// Type of upper [`Level`] within the hierarchy.
    type HigherLevel: Level;

    /// Type of upper [`Level`] within the hierarchy.
    type LowerLevel: Level;

    /// Create a new `Level` token.
    unsafe fn create() -> Self;

    /// Get an integer-based representation of the level.
    fn level() -> usize;

    /// Change from `HigherLevel` to `LowerLevel` while consuming `HigherLevel`.
    unsafe fn enter(self) -> Self::LowerLevel {{
        assert!(Self::level() > Self::LowerLevel::level());
        Self::LowerLevel::create()
    }}

    /// Change back from `LowerLevel` to `HigherLevel` while consuming `LowerLevel`.
    unsafe fn leave(self) -> Self::HigherLevel {{
        assert!(Self::level() < Self::HigherLevel::level());
        unsafe {{ Self::HigherLevel::create() }}
    }}
}}"
    )
    .unwrap();

    // Add Adapter/AdpaterGuard trait
    writeln!(
        level_file,
        "
/// Trait to allow to \"skip\" layers using convinient adapter.
pub trait Adapter<HigherLevel, LowerLevel, Guard>
where
    Self: Sized,
    HigherLevel: Level,
    LowerLevel: Level,
    Guard: AdapterGuard<HigherLevel, LowerLevel>,
{{
    /// Create a new [`Adapter`].
    fn new() -> Self;

    /// Change from `HigherLevel` to `LowerLevel` while consuming `HigherLevel`.
    fn enter(self, level: HigherLevel) -> (Guard, LowerLevel) {{
        // Consule level
        let _ = level;

        // Sanity check of HigherLevel and LowerLevel
        assert!(HigherLevel::level() > LowerLevel::level());

        // Create guard
        let guard = Guard::new();

        // Create level
        let level = unsafe {{ LowerLevel::create() }};
        (guard, level)
    }}
}}

/// Trait to return form `Adapter::enter`.
pub trait AdapterGuard<HigherLevel, LowerLevel>
where
    Self: Sized,
    HigherLevel: Level,
    LowerLevel: Level,
{{
    /// Create a new [`AdapterGuard`].
    fn new() -> Self;

    /// Change back from `LowerLevel` to `HigherLevel` while consuming `LowerLevel`.
    fn leave(self, level: LowerLevel) -> HigherLevel {{
        // Consule level
        let _ = level;

        // Sanity check of HigherLevel and LowerLevel
        assert!(HigherLevel::level() > LowerLevel::level());

        // Produce level
        unsafe {{ HigherLevel::create() }}
    }}
}}"
    )
    .unwrap();

    // Add struct LevelInitialization
    writeln!(
        level_file,
        "
/// Level Initialization
#[derive(Debug)]
pub struct LevelInitialization {{
    phantom: PhantomData<Self>,
}}

impl Level for LevelInitialization {{
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {{
        Self {{
            phantom: PhantomData,
        }}
    }}

    fn level() -> usize {{
        panic!();
    }}
}}"
    )
    .unwrap();

    // Add struct LevelInvalid
    writeln!(
        level_file,
        "
/// Invalid level to indicate \"end of hierarchy\"
#[derive(Debug)]
pub struct LevelInvalid {{
    phantom: PhantomData<Self>,
}}

impl Level for LevelInvalid {{
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {{
        panic!();
    }}

    fn level() -> usize {{
        panic!()
    }}
}}"
    )
    .unwrap();

    // Add for each entry its struct definition
    for i in 0..level_descs.len() {
        let prev_desc = level_descs
            .get(i.saturating_sub(1))
            .map_or("Invalid", |desc| &desc.name);
        let curr = level_descs.get(i).unwrap();
        let next_desc = level_descs
            .get(i.saturating_add(1))
            .map_or("Invalid", |desc| &desc.name);

        writeln!(
            level_file,
            "
/// {}
#[derive(Debug)]
pub struct Level{} {{
    phantom: PhantomData<Self>,
}}

impl Level for Level{} {{
    type HigherLevel = Level{};

    type LowerLevel = Level{};

    unsafe fn create() -> Self {{
        Self {{
            phantom: PhantomData,
        }}
    }}

    fn level() -> usize {{
        {}
    }}
}}",
            curr.description.trim(),
            curr.name,
            curr.name,
            next_desc,
            prev_desc,
            curr.value
        )
        .unwrap();

        // Add AdapterGuard/AdapterGuard
        for j in i + 1..level_descs.len() {
            let drag = level_descs.get(j).unwrap();
            writeln!(
                level_file,
                "
/// [`Adapter`] for [`Level{}`] to [`Level{}`]
pub struct Adapter{}{} {{
    phantom: PhantomData<Self>,
}}

/// [`AdapterGuard`] for [`Level{}`] to [`Level{}`]
pub struct AdapterGuard{}{} {{
    phantom: PhantomData<Self>,
}}

impl Adapter<Level{}, Level{}, AdapterGuard{}{}> for Adapter{}{} {{
    fn new() -> Self {{
        Self {{
            phantom: PhantomData,
        }}
    }}
}}

impl AdapterGuard<Level{}, Level{}> for AdapterGuard{}{} {{
    fn new() -> Self {{
        Self {{
            phantom: PhantomData,
        }}
    }}
}}",
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
                curr.name,
                drag.name,
            )
            .unwrap();
        }
    }
}

fn compile_assembly_file(file: &path::Path, configs_options: &[ConfigOptions]) {
    // Get input file name as string
    let input = file.to_str().unwrap();
    assert!(input.ends_with(".S"));

    // Get output file name as string
    let output: String = file
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(".S", ".o");

    // Construct build command
    let mut builder = cc::Build::new();

    // Search suitable C compiler
    let mut cc = None;
    let common_ccs = [
        path::Path::new("/bin/riscv64-elf-gcc"),
        path::Path::new("/bin/riscv64-unknown-elf-gcc"),
        path::Path::new("/usr/bin/riscv64-elf-gcc"),
        path::Path::new("/usr/bin/riscv64-unknown-elf-gcc"),
    ];
    for common_cc in common_ccs {
        if common_cc.exists() {
            cc = Some(common_cc);
            break;
        }
    }
    let cc = match cc {
        Some(cc) => cc,
        None => {
            panic!(
                "Unable to find suitable C compiler! Install one of the following binaries: {:?}",
                common_ccs
            );
        }
    };

    // Add default flags
    builder.compiler(cc);
    builder.flag("-march=rv64gc");
    builder.flag("-mabi=lp64d");

    // Add defines
    for config_option in configs_options {
        let mut name = config_option.name.trim().replace("CONFIG_", "");
        name.make_ascii_uppercase();
        builder.define(&name, Some(config_option.value.as_str()));
    }

    // Set input file
    builder.file(file);

    // Compile file
    builder.compile(&output);
}

fn main() {
    // Set dependencies for re-building
    println!("cargo:rerun-if-changed=./config.yaml");
    println!("cargo:rerun-if-changed=./level.yaml");
    println!("cargo:rerun-if-changed=./src/boot/head.S");
    println!("cargo:rerun-if-changed=./src/trap/entry.S");

    // Parse config file
    let configs_options = parse_config_yaml();

    // Geneate src/config.rs
    generate_config_rs(&configs_options);

    // Parse level description file
    let level_descs = parse_level_yaml();

    // Geneate src/sync/level.rs
    generate_level_rs(&level_descs);

    // Build ./src/boot/head.S
    compile_assembly_file(path::Path::new("./src/boot/head.S"), &configs_options);

    // Build ./src/trap/entry.S
    compile_assembly_file(path::Path::new("./src/trap/entry.S"), &configs_options);
}
