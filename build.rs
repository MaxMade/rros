use std::fs;
use std::io::{Read, Write};
use std::path;

extern crate yaml_rust;

#[derive(Debug)]
struct Config {
    pub name: String,
    pub value: String,
    pub ty: String,
    pub description: String,
}

fn parse_config() -> Vec<Config> {
    /* Read config file */
    let mut config_file = fs::File::open(path::Path::new("config.yaml")).unwrap();
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer).unwrap();

    /* Parse YAML */
    let mut config_options = Vec::new();
    let config = yaml_rust::YamlLoader::load_from_str(config_buffer.as_str())
        .unwrap()
        .pop()
        .unwrap();
    for (key, values) in config.as_hash().unwrap().iter() {
        /* Get name of config option */
        let name = key.as_str().unwrap().to_string();

        /* Process config option */
        let value = if let Some(value) = values["value"].as_str() {
            value.to_string()
        } else if let Some(value) = values["value"].as_i64() {
            value.to_string()
        } else {
            panic!("Unable to process value of configuration!");
        };
        let ty = values["type"].as_str().unwrap().to_string();
        let description = values["description"].as_str().unwrap().to_string();

        /* Add config option to list */
        let config_option = Config {
            name,
            value,
            ty,
            description,
        };
        config_options.push(config_option);
    }

    return config_options;
}

fn generate_config_rs(configs_options: &[Config]) {
    /* Open output file */
    let mut config_file = fs::File::options()
        .write(true)
        .create(true)
        .open("./src/config.rs")
        .unwrap();

    /* Generate constants */
    for config_option in configs_options {
        let mut var_name = config_option.name.trim().replace("CONFIG_", "");
        var_name.make_ascii_uppercase();

        write!(
            config_file,
            "/// {}\npub const {}: {} = {};\n",
            config_option.description.trim(),
            var_name,
            config_option.ty.trim(),
            config_option.value.trim()
        )
        .unwrap();
    }
}

fn compile_assembly_file(file: &path::Path, configs_options: &[Config]) {
    /* Get input file name as string */
    let input = file.to_str().unwrap();
    assert!(input.ends_with(".S"));

    /* Get output file name as string */
    let output: String = file
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(".S", ".o");

    /* Set dependencies for re-building */
    println!("cargo:rerun-if-changed={}", input);
    println!("cargo:rerun-if-changed=./config.yaml");
    println!("cargo:rerun-if-changed=./src/config.rs");

    /* Construct build command */
    let mut builder = cc::Build::new();

    /* Search suitable C compiler */
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

    /* Add default flags */
    builder.compiler(cc);
    builder.flag("-march=rv64gc");
    builder.flag("-mabi=lp64d");

    /* Add defines */
    for config_option in configs_options {
        let mut name = config_option.name.trim().replace("CONFIG_", "");
        name.make_ascii_uppercase();
        builder.define(&name, Some(config_option.value.as_str()));
    }

    /* Set input file */
    builder.file(file);

    /* Compile file */
    builder.compile(&output);
}

fn main() {
    /* Parse config file and generate src/config.rs */
    let configs_options = parse_config();

    /* Geneate src/config.rs */
    generate_config_rs(&configs_options);

    /* Build ./src/boot/head.S */
    compile_assembly_file(path::Path::new("./src/boot/head.S"), &configs_options);

    /* Build ./src/kernel/trap_entry.S */
    compile_assembly_file(
        path::Path::new("./src/kernel/trap_entry.S"),
        &configs_options,
    );
}
