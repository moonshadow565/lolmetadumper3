use mmap::MemoryMap;
use std::fs;
use std::path::Path;

mod macho;
mod stubs;

pub fn map_image<P: AsRef<Path>>(path: P) -> anyhow::Result<MemoryMap> {
    // Read macho
    let data = fs::read(path)?;
    let macho = macho::MachOImage::new(&data)?;
    let map = macho.map_image(&data)?;
    let image = unsafe { &mut *std::ptr::slice_from_raw_parts_mut(map.data(), map.len()) };
    macho.resolve_imports(image, stubs::resolve);
    macho.run_mod_init(image);
    Ok(map)
}
