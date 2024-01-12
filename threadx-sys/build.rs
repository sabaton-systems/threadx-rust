// Build script for Building threadx and to create the bindings

use std::io::{Write, BufRead};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::env;
use std::sync::{Arc, Mutex};
use bindgen::Builder;
use bindgen::callbacks::ParseCallbacks;
use cmake::Config;

fn main() {

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let tx_user_file = env::var("TX_USER_FILE").ok();
    let src_path = out_dir.join("threadx"); // source code of threadx is vendored here
    let threadx_gh = "https://github.com/eclipse-threadx/threadx.git";
    let threadx_tag = "v6.4.0_rel";

    let tx_user_file_path = 
    if let Some(tx_user_file) = tx_user_file {
        let tx_user_file = PathBuf::from(tx_user_file).canonicalize().expect("Unable to find TX_USER_FILE");
        println!("cargo:info=Using TX_USER_FILE: {}", tx_user_file.display());
        println!("cargo:rerun-if-changed={}",tx_user_file.display());
        Some(tx_user_file)
    } else {
        println!("cargo:info=No TX_USER_FILE specified, using defaults");
        None
    };
    
    // Clone threadx
    std::process::Command::new("git")
        .arg("clone")
        .arg(threadx_gh)
        .current_dir(out_dir.clone())
        .output()
    .expect("Failed to fetch git submodules!");

    // checkout tag
    std::process::Command::new("git")
        .arg("checkout")
        .arg(threadx_tag)
        .current_dir(src_path.clone())
        .output()
        .expect("Unable to checkout threadx tag");


    let target = env::var("TARGET").expect("TARGET is not set");

    /*
    # target = "thumbv6m-none-eabi"    # Cortex-M0 and Cortex-M0+
    # target = "thumbv7m-none-eabi"    # Cortex-M3
    # target = "thumbv7em-none-eabi"   # Cortex-M4 and Cortex-M7 (no FPU)
    # target = "thumbv7em-none-eabihf" # Cortex-M4F and Cortex-M7F (with FPU)
    # target = "thumbv8m.base-none-eabi"   # Cortex-M23
    # target = "thumbv8m.main-none-eabi"   # Cortex-M33 (no FPU)
    # target = "thumbv8m.main-none-eabihf" # Cortex-M33 (with FPU)
     */

    let build_commands = out_dir.join("build_commands.txt");

    // We create a wrapper script to capture the commands passed to the compiler
    let launcher_script = format!(r#"
    #!/bin/sh
    #echo "Wrapper $@"
    set -e
    echo "$@" >> {}
    exec $@
    "#, out_dir.join("build_commands.txt").display());

    let compiler_wrapper_path = out_dir.join("compiler_wrapper.sh");

    let _ = std::fs::remove_file(compiler_wrapper_path.as_path());

    let mut file = std::fs::OpenOptions::new().write(true).mode(0o700).create_new(true).open(compiler_wrapper_path.as_path()).expect("Unable to open wrapper");
    file.write_all(launcher_script.as_bytes()).expect("Unable to write wrapper");
    file.flush().expect("Unable to flush wrapper");
    drop(file);


    let toolchain_file = match target.as_str() {
        "thumbv6m-none-eabi" => "cmake/cortex_m0.cmake",
        "thumbv7m-none-eabi" => "cmake/cortex_m3.cmake",
        "thumbv7em-none-eabi" => "cmake/cortex_m4.cmake",
        "thumbv7em-none-eabihf" => "cmake/cortex_m7.cmake",
        "thumbv8m.base-none-eabi" => "cmake/cortex_m23.cmake",
        _ => {
            println!("cargo:error=Unsupported cortex M target: {}", target);
            panic!("Unsupported cortex M target: {}", target);
            
        }
    };

    // Build threadx
    let mut cfg = Config::new(src_path.to_owned());

        cfg.define("CMAKE_TOOLCHAIN_FILE", toolchain_file)
        .generator("Ninja")
        .build_target("threadx")
        .env("CMAKE_C_COMPILER_LAUNCHER", compiler_wrapper_path.as_path())
        .env("CMAKE_CXX_COMPILER_LAUNCHER", compiler_wrapper_path.as_path());

    if tx_user_file_path.is_some() {
        cfg.define("TX_USER_FILE", tx_user_file_path.unwrap().to_str().unwrap());
    };

    let dst= cfg.build().join("build");

    println!("cargo:info=threadx build completed and output at {}", dst.display());

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=threadx");

    // Parse the build_commands.txt file to find the include directories and other compiler flags
    let build_commands = std::fs::OpenOptions::new().read(true).open(build_commands.as_path()).expect("Unable to open build_commands.txt");
    let build_commands = std::io::BufReader::new(build_commands).lines();
    let mut include_dirs = Vec::new();
    let mut defines =  Vec::new();
    let mut compiler = None;

    for line in build_commands {
        if let Ok(line) = line {

            if compiler.is_none() {
                // get the compiler from the first line
                let compiler_cmd = line.split(" ").take(1).next().unwrap();
                compiler = Some(compiler_cmd.to_string());
            }

            for cmd in line.split(" ") {
                if cmd.starts_with("-I") {
                    let include_dir = cmd.trim_start_matches("-I");
                    include_dirs.push(include_dir.to_string());
                } else if cmd.starts_with("-D") {
                    let define = cmd.trim_start_matches("-D");
                    defines.push(define.to_string());
                }
            }
        }
    }

    include_dirs.sort();
    include_dirs.dedup();
    defines.sort();
    defines.dedup();

    let threadx_api_path = src_path.join("common/inc/tx_api.h");
    let bindings_path = out_dir.join("generated.rs");
    let mut bindings = bindgen::Builder::default()
        .header(threadx_api_path.to_str().unwrap())
        .use_core()
        .layout_tests(false)
        .allowlist_function("_tx.*")
        .allowlist_recursively(true);
    for include_dir in include_dirs {
        bindings = bindings.clang_arg(format!("-I{}", include_dir));
    }
    for define in defines {
        bindings = bindings.clang_arg(format!("-D{}", define));
    }
    
    let int_macros = Arc::new(Mutex::new(Vec::new()));

    let mut bindings = configure_builder(bindings, int_macros.clone());

    // Get the standard include paths from the compiler
    // Create an empty file to pass to the compiler
    std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(out_dir.join("empty.c")).expect("Unable to create empty.c");
    let output = Command::new(compiler.unwrap())
        .arg("-xc")
        .arg("-E")
        .arg("-v")
        .arg(out_dir.join("empty.c"))
        .output()
        .expect("Unable to run compiler");

    let output = String::from_utf8(output.stderr).expect("Unable to parse compiler output");
    
    let mut inside_include_dirs = false;
    let mut compiler_include_dirs = Vec::new();
    for line in output.lines() {
        if line.contains("search starts here:") {
            inside_include_dirs = true;
            continue;
        }
        if line.contains("End of search list.") {
            //inside_include_dirs = false;
            break;
        }
        if inside_include_dirs {
            compiler_include_dirs.push(line.trim().to_string());
        }
    }

    println!("Found compiler include dirs: {:?}", compiler_include_dirs);

    for include_dir in compiler_include_dirs {
        bindings = bindings.clang_arg(format!("-I{}", include_dir));
    }

    let bindings = bindings
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(bindings_path.clone())
        .expect("Couldn't write bindings");

    // now generate the int macros
    let mut out_file = std::fs::OpenOptions::new().create(false).append(true).write(true).open(&bindings_path).expect("Unable to open bindings file");

    writeln!(out_file,"// Constants extracted from TX_API.H and TX_PORT.H with overridden values").expect("Unable to write int macros");
    for (name, value) in int_macros.lock().unwrap().iter() {
        writeln!(out_file, "pub const {} : UINT = {};", name, value).expect("Unable to write int macro");
    }

    // Copy the file to src/generated.rs to keep the documentation build happy
    std::fs::copy(PathBuf::from(bindings_path), PathBuf::from("src/generated.rs")).unwrap();
}

// Configure the builder
fn configure_builder( builder : Builder, int_macros: Arc<Mutex<Vec<(String,String)>>>) -> Builder {
    builder
    .fit_macro_constants(false)
    .parse_callbacks(Box::new(Callbacks{int_macros}))

}


#[derive(Debug)]
struct Callbacks{
    int_macros: Arc<Mutex<Vec<(String,String)>>>,
}

impl ParseCallbacks for Callbacks {
    // fn will_parse_macro(&self, _name: &str) -> bindgen::callbacks::MacroParsingBehavior {
    //     println!("MACRO: {}", _name);
    //     bindgen::callbacks::MacroParsingBehavior::Default
    // }

    // fn generated_name_override(
    //     &self,
    //     _item_info: bindgen::callbacks::ItemInfo<'_>,
    // ) -> Option<String> {
    //     None
    // }

    // fn generated_link_name_override(
    //     &self,
    //     _item_info: bindgen::callbacks::ItemInfo<'_>,
    // ) -> Option<String> {
    //     None
    // }

    fn int_macro(&self, _name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        //println!("Int Macro: {}={}", _name, _value);
        if _name.starts_with("TX_") {
            self.int_macros.lock().unwrap().push((_name.to_string(), _value.to_string()));
        }
        Some(bindgen::callbacks::IntKind::U32)
    }

    // fn str_macro(&self, _name: &str, _value: &[u8]) { 
    //     println!("STR MACRO: {}={}", _name, String::from_utf8_lossy(_value))
    // }

    // fn func_macro(&self, _name: &str, _value: &[&[u8]]) {
    //     println!("FUNC MACRO: {}", _name);
    //     for v in _value {
    //         println!("   Value:{}",String::from_utf8_lossy(v))
    //     }
    // }

    // fn enum_variant_behavior(
    //     &self,
    //     _enum_name: Option<&str>,
    //     _original_variant_name: &str,
    //     _variant_value: bindgen::callbacks::EnumVariantValue,
    // ) -> Option<bindgen::callbacks::EnumVariantCustomBehavior> {
    //     None
    // }

    // fn enum_variant_name(
    //     &self,
    //     _enum_name: Option<&str>,
    //     _original_variant_name: &str,
    //     _variant_value: bindgen::callbacks::EnumVariantValue,
    // ) -> Option<String> {
    //     None
    // }

    // fn item_name(&self, _original_item_name: &str) -> Option<String> {
    //     None
    // }

    // fn header_file(&self, _filename: &str) {}

    // fn include_file(&self, _filename: &str) {}

    // fn read_env_var(&self, _key: &str) {}

    // fn blocklisted_type_implements_trait(
    //     &self,
    //     _name: &str,
    //     _derive_trait: bindgen::callbacks::DeriveTrait,
    // ) -> Option<bindgen::callbacks::ImplementsTrait> {
    //     None
    // }

    // fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
    //     vec![]
    // }

    // fn process_comment(&self, _comment: &str) -> Option<String> {
    //     None
    // }

    // fn field_visibility(
    //     &self,
    //     _info: bindgen::callbacks::FieldInfo<'_>,
    // ) -> Option<bindgen::FieldVisibilityKind> {
    //     None
    // }
}
