use anyhow::bail;
use goblin::mach::bind_opcodes::BIND_TYPE_POINTER;
use goblin::mach::constants::*;
use goblin::mach::{load_command::CommandVariant, MachO};
use mmap::{MapOption, MemoryMap};
use scroll::{Pread, Uleb128};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MachORebase {
    pub kind: u8,
    pub vmaddr: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MachOImport {
    pub name: String,
    pub dylib: String,
    pub offset: u64,
    pub size: usize,
    pub address: u64,
    pub addend: i64,
    pub is_lazy: bool,
    pub is_weak: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MachOSection {
    pub sectname: String,
    pub segname: String,
    pub addr: u64,
    pub size: u64,
    pub offset: u32,
    pub align: u32,
    pub reloff: u32,
    pub nreloc: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MachOSegment {
    pub segname: String,
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
    pub maxprot: u32,
    pub initprot: u32,
    pub nsects: u32,
    pub flags: u32,
    pub sections: Vec<MachOSection>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MachOImage {
    pub vmbase: u64,
    pub vmsize: u64,
    pub segments: Vec<MachOSegment>,
    pub imports: Vec<MachOImport>,
    pub rebase: Vec<MachORebase>,
}

impl MachOImage {
    pub fn new(data: &[u8]) -> anyhow::Result<Self> {
        let macho = MachO::parse(data, 0)?;

        // Copy out segments and sections
        let mut vmbase = u64::MAX;
        let mut vmsize = u64::MIN;
        let mut segments = Vec::new();
        for seg in macho.segments.iter() {
            // Get lowest loadable address as base
            if seg.filesize != 0 {
                vmbase = vmbase.min(seg.vmaddr);
                vmsize = vmsize.max(seg.vmaddr + seg.vmsize);
            }
            let mut sections = Vec::new();
            for (sect, _) in seg.sections()?.iter() {
                let size = (seg.vmaddr + seg.vmsize).min(sect.addr + sect.size) - sect.addr;
                sections.push(MachOSection {
                    sectname: sect.name()?.to_owned(),
                    segname: sect.segname()?.to_owned(),
                    addr: sect.addr,
                    size: size,
                    offset: sect.offset,
                    align: sect.align,
                    reloff: sect.reloff,
                    nreloc: sect.nreloc,
                    flags: sect.flags,
                })
            }
            segments.push(MachOSegment {
                segname: seg.name()?.to_owned(),
                vmaddr: seg.vmaddr,
                vmsize: seg.vmsize,
                fileoff: seg.fileoff,
                filesize: seg.filesize,
                maxprot: seg.maxprot,
                initprot: seg.initprot,
                nsects: seg.nsects,
                flags: seg.flags,
                sections: sections,
            });
        }

        // Copy out imports information
        let mut imports = Vec::new();
        for i in macho.imports()? {
            imports.push(MachOImport {
                name: i.name.to_owned(),
                dylib: i.dylib.to_owned(),
                offset: i.offset,
                size: i.size,
                address: i.address,
                addend: i.addend,
                is_lazy: i.is_lazy,
                is_weak: i.is_weak,
            });
        }

        // Goblin does not read 10.6+ relocations :(
        let mut rebase = Vec::new();
        for c in macho.load_commands {
            match c.command {
                CommandVariant::DyldInfo(info) | CommandVariant::DyldInfoOnly(info) => {
                    let mut i = info.rebase_off as usize;
                    let mut kind = 1u8;
                    let mut segment = 0u8;
                    let mut offset = 0u64;
                    while i < (info.rebase_off + info.rebase_size) as usize {
                        let cmd: u8 = data.gread(&mut i)?;
                        let imm = cmd & 0x0F;
                        match cmd & 0xF0 {
                            0x00 => {
                                // REBASE_OPCODE_DONE
                                // NOTE: this opcode is callde "done" but it does not acutally finish anything
                            }
                            0x10 => {
                                // REBASE_OPCODE_SET_TYPE_IMM
                                kind = imm;
                            }
                            0x20 => {
                                // REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB
                                segment = imm;
                                offset = Uleb128::read(&data, &mut i)?;
                            }
                            0x30 => {
                                // REBASE_OPCODE_ADD_ADDR_ULEB
                                offset = offset.wrapping_add(Uleb128::read(&data, &mut i)?);
                            }
                            0x40 => {
                                // REBASE_OPCODE_ADD_ADDR_IMM_SCALED
                                offset = offset
                                    .wrapping_add((imm * std::mem::size_of::<usize>() as u8) as _);
                            }
                            0x50 => {
                                // REBASE_OPCODE_DO_REBASE_IMM_TIMES
                                for _ in 0..imm {
                                    let vmaddr =
                                        segments[segment as usize].vmaddr.wrapping_add(offset);
                                    rebase.push(MachORebase { kind, vmaddr });
                                    offset += std::mem::size_of::<usize>() as u64;
                                }
                            }
                            0x60 => {
                                // REBASE_OPCODE_DO_REBASE_ULEB_TIMES
                                let count = Uleb128::read(&data, &mut i)?;
                                for _ in 0..count {
                                    let vmaddr =
                                        segments[segment as usize].vmaddr.wrapping_add(offset);
                                    rebase.push(MachORebase { kind, vmaddr });
                                    offset += std::mem::size_of::<usize>() as u64;
                                }
                            }
                            0x70 => {
                                // REBASE_OPCODE_DO_REBASE_ADD_ADDR_ULEB
                                let skip = Uleb128::read(&data, &mut i)?;
                                let vmaddr = segments[segment as usize].vmaddr.wrapping_add(offset);
                                rebase.push(MachORebase { kind, vmaddr });
                                offset += std::mem::size_of::<usize>() as u64;
                                offset += skip;
                            }
                            0x80 => {
                                // REBASE_OPCODE_DO_REBASE_ULEB_TIMES_SKIPPING_ULEB
                                let count = Uleb128::read(&data, &mut i)?;
                                let skip = Uleb128::read(&data, &mut i)?;
                                for _ in 0..count {
                                    let vmaddr =
                                        segments[segment as usize].vmaddr.wrapping_add(offset);
                                    rebase.push(MachORebase { kind, vmaddr });
                                    offset += std::mem::size_of::<usize>() as u64;
                                    offset += skip;
                                }
                            }
                            _ => bail!("Unknown opcode: {:x}", cmd),
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            vmbase,
            vmsize,
            segments,
            imports,
            rebase,
        })
    }

    pub fn map_image(&self, data: &[u8]) -> anyhow::Result<MemoryMap> {
        // Try to map at prefered address first.
        let map = MemoryMap::new(
            self.vmsize as _,
            &[
                MapOption::MapReadable,
                MapOption::MapWritable,
                MapOption::MapExecutable,
                MapOption::MapAddr(self.vmbase as _),
            ],
        );

        // If prefered address fails, map anywhere.
        let (map, should_rebase) = match map {
            Ok(map) => (map, false),
            Err(_) => (
                MemoryMap::new(
                    self.vmsize as _,
                    &[
                        MapOption::MapReadable,
                        MapOption::MapWritable,
                        MapOption::MapExecutable,
                    ],
                )?,
                true,
            ),
        };

        // Copy contents
        let image = unsafe { &mut *std::ptr::slice_from_raw_parts_mut(map.data(), map.len()) };
        for segment in self.segments.iter() {
            if segment.filesize == 0 {
                continue;
            }
            if segment.vmsize >= segment.filesize {
                let addr = segment.vmaddr - self.vmbase;
                let src = &data[segment.fileoff as _..][..segment.filesize as _];
                let dst = &mut image[addr as _..][..segment.filesize as _];
                dst.copy_from_slice(src);
            }
            for section in segment.sections.iter() {
                match section.flags & SECTION_TYPE {
                    S_ZEROFILL | S_GB_ZEROFILL | S_THREAD_LOCAL_ZEROFILL => {
                        let addr = section.addr - self.vmbase;
                        let dst = &mut image[addr as _..][..section.size as _];
                        dst.fill(0);
                    }
                    _ => {}
                }
            }
        }

        // Do rebase
        if should_rebase {
            for rebase in self.rebase.iter() {
                if rebase.kind != BIND_TYPE_POINTER {
                    continue;
                }
                let addr = rebase.vmaddr - self.vmbase;
                unsafe {
                    let target = image.as_mut_ptr().offset(addr as _).cast::<usize>();
                    let value = target.read_unaligned();
                    target.write_unaligned(
                        value
                            .wrapping_sub(self.vmbase as usize)
                            .wrapping_add(image.as_ptr() as _),
                    );
                };
            }
        }

        Ok(map)
    }

    pub fn resolve_imports<R: Fn(&str) -> usize>(&self, image: &mut [u8], resolver: R) {
        for import in self.imports.iter() {
            // dont bind weak symbols
            if import.is_weak {
                continue;
            }
            let value = resolver(&import.name);
            if value == 0 {
                continue;
            }
            let addr = import.address - self.vmbase;
            unsafe {
                let target = image.as_mut_ptr().offset(addr as _).cast::<usize>();
                target.write_unaligned(value.wrapping_add(import.addend as _));
            }
        }
    }

    pub fn run_mod_init(&self, image: &[u8]) {
        // Collect all inits.
        let mut init = Vec::<extern "C" fn()>::new();
        for segment in self.segments.iter() {
            for section in segment.sections.iter() {
                if (section.flags & SECTION_TYPE) != S_MOD_INIT_FUNC_POINTERS {
                    continue;
                }
                let mut i = section.addr - self.vmbase;
                let end = i + section.size;
                while i < end {
                    let p = unsafe { *(image.as_ptr().offset(i as _) as *const _) };
                    init.push(p);
                    i += std::mem::size_of::<usize>() as u64;
                }
            }
        }

        // Run initializers
        for f in init {
            f();
        }
    }
}
