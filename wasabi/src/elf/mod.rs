extern crate alloc;

pub mod loader;

use alloc::vec::Vec;
use core::mem::size_of;

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub os_abi: u8,
    pub _padding: [u8; 7],
    pub elf_type: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}
impl Elf64Header {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < size_of::<Self>() {
            return None;
        }

        let header = unsafe { *(data.as_ptr() as *const Self) };

        // Validate ELF magic number
        if header.magic != ELF_MAGIC {
            return None;
        }

        // Validate ELF class (64-bit)
        if header.class != 2 {
            return None;
        }

        // Validate machine (x86-64)
        if header.machine != 0x3E {
            return None;
        }

        Some(header)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,   // セグメントタイプ
    pub p_flags: u32,  // フラグ (R/W/X)
    pub p_offset: u64, // ファイル内オフセット
    pub p_vaddr: u64,  // 仮想アドレス
    pub p_paddr: u64,  // 物理アドレス（通常無視）
    pub p_filesz: u64, // ファイル内サイズ
    pub p_memsz: u64,  // メモリ上サイズ
    pub p_align: u64,  // アライメント
}
impl Elf64ProgramHeader {
    pub fn parse_all(data: &[u8], header: &Elf64Header) -> Vec<Self> {
        let mut headers = Vec::new();
        let offset = header.phoff as usize;
        let size = header.phentsize as usize;

        for i in 0..header.phnum as usize {
            let start = offset + i * size;
            if start + size <= data.len() {
                let ph = unsafe { (data[start..].as_ptr() as *const Self).read_unaligned() };
                headers.push(ph);
            }
        }

        headers
    }
}

// セグメントタイプ
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;

// フラグ
pub const PF_X: u32 = 1; // 実行可能
pub const PF_W: u32 = 2; // 書き込み可能
pub const PF_R: u32 = 4; // 読み込み可能
