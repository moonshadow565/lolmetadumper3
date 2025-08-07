use regex::bytes::Regex;
use serde_json::json;
use std::env;
#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
mod loader;
#[allow(dead_code)]
mod meta;
#[allow(dead_code)]
mod meta_dump;

type MetaVector = meta::RiotVector<&'static meta::Class>;

const PATTERN_CLASSES: &str = r"(?s-u)\x48\x8D\x3D(....)\x48?\x89\xDE\xE8....\x48\x83\xC4\x08\x5B\x5D\xFF\x60\x10";

#[allow(dead_code)]
const PATTERN_VERSION: &str = r"(?s-u)\x00Releases/(\d+(\.\d+)+)\x00";

fn find_version(data: &[u8]) -> Option<String> {
    Regex::new(PATTERN_VERSION)
        .expect("Bad regex PATTERN_VERSION!")
        .captures(data)
        .and_then(|captures| captures.get(1))
        .map(|x| { String::from_utf8_lossy(x.as_bytes()) }.to_string())
}

fn find_classes(data: &[u8]) -> &MetaVector {
    Regex::new(PATTERN_CLASSES)
        .expect("Bad regex PATTERN_CLASSES!")
        .captures(data)
        .and_then(|captures| captures.get(1))
        .map(|x| unsafe {
            let base = data.as_ptr().offset(x.end() as _);
            let rel = x.as_bytes().as_ptr().cast::<i32>().read_unaligned();
            base.offset(rel as _)
        })
        .map(|x| x as *const MetaVector)
        .and_then(|x| unsafe { x.as_ref() })
        .expect("Failed to find PATTERN_CLASSES!")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Gib path to executable!");

    eprintln!("Mapping image...");
    let map = loader::map_image(path)?;
    let data = unsafe { &*std::ptr::slice_from_raw_parts(map.data(), map.len()) };
    eprintln!("Mapped at: {:#x}", data.as_ptr() as usize);

    eprintln!("Extracting version info...");
    let version = find_version(data);
    eprintln!("Found version: {:?}", version);

    eprintln!("Finding metaclasses...");
    let classes = find_classes(data);
    eprintln!(
        "Found classes at {:#x} len {:#x}",
        classes as *const _ as usize,
        classes.slice().len()
    );

    eprintln!("Processing classes...");
    let meta_info = json!({
        "version": version.unwrap_or_else(|| "unknown".to_string()),
        "classes": meta_dump::dump_class_list(data.as_ptr() as usize, classes.slice()),
    });

    eprintln!("Serializing classes...");
    serde_json::to_writer_pretty(std::io::stdout(), &meta_info).expect("Failed to serialize json!");

    eprintln!("Done!");
    Ok(())
}
