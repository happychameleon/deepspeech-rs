// based on the https://github.com/tensorflow/rust/blob/master/tensorflow-sys/build.rs 
// and https://github.com/danigm/gettext-rs/blob/master/gettext-sys/build.rs build files

extern crate curl;
extern crate pkg_config;
extern crate zip;

use std::env;
use std::fs::{self, File};
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;

use curl::easy::Easy;
use zip::ZipArchive;

const LIBRARY: &str = "deepspeech";
const LIBRARY_URL: &str = "https://github.com/happychameleon/deepspeech_bin/raw/main/libdeepspeech.zip";

fn main() {
    // Note that pkg_config will print cargo:rustc-link-lib and cargo:rustc-link-search as
    // appropriate if the library is found.
    if pkg_config::probe_library(LIBRARY).is_ok() {
        return;
    }

    install_prebuilt();
}

fn install_prebuilt() {
    let zip_file_name = "libdeepspeech.zip";

    let path_to_zip = env::current_dir().expect("could not get current dir");
    let path_to_zip = PathBuf::from(path_to_zip);
    let out_dir = env::var("OUT_DIR").unwrap();

    let file_name = path_to_zip.join(PathBuf::from(zip_file_name));

    if !file_name.exists() {
        let f = File::create(&file_name).unwrap();
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        let deepspeech_url = LIBRARY_URL;
        easy.follow_location(true).unwrap();
        easy.url(&deepspeech_url).expect("could not open deepspeech_url");
        easy.write_function(move |data| Ok(writer.write(data).unwrap()))
            .unwrap();
        easy.perform().unwrap();

        let response_code = easy.response_code().unwrap();
        if response_code != 200 {
            panic!(
                "Unexpected response code {} for {}",
                response_code, deepspeech_url
            );
        }
    }

    // Extract deepspeech zip
    let output = PathBuf::from(env::var("OUT_DIR").unwrap());

    let unpacked_dir = PathBuf::from(&output).join(PathBuf::from("libdeepspeech"));
    let lib_dir  = unpacked_dir.join("libdeepspeech");

    extract_zip(file_name, &unpacked_dir);
    println!("cargo:rustc-link-lib={}", LIBRARY);

    

    let framework_files = std::fs::read_dir(lib_dir).unwrap();
    
    for library_entry in framework_files.filter_map(Result::ok) {
        let library_full_path = library_entry.path();
        let new_library_full_path = output.join(&library_full_path.file_name().unwrap());
        if new_library_full_path.exists() {
            fs::remove_file(&new_library_full_path).unwrap();
        }
        fs::copy(&library_full_path, &new_library_full_path).unwrap();
    }

    println!("cargo:rustc-link-search={}", out_dir);
}


fn extract_zip<P: AsRef<Path>, P2: AsRef<Path>>(archive_path: P, extract_to: P2) {
    fs::create_dir_all(&extract_to).expect("Failed to create output path for zip archive.");
    let file = File::open(archive_path).expect("Unable to open deepspeech zip archive.");
    let mut archive = ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut zipfile = archive.by_index(i).unwrap();
        let output_path = extract_to.as_ref().join(zipfile.mangled_name());
        if zipfile.name().starts_with("lib") {
            if zipfile.is_dir() {
                fs::create_dir_all(&output_path)
                    .expect("Failed to create output directory when unpacking archive.");
            } else {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(&parent)
                            .expect("Failed to create parent directory for extracted file.");
                    }
                }
                let mut outfile = File::create(&output_path).unwrap();
                io::copy(&mut zipfile, &mut outfile).unwrap();
            }
        }
    }
}