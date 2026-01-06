extern crate alloc;

use crate::elf::Elf64Header;
use crate::elf::Elf64ProgramHeader;
use crate::elf::PT_LOAD;
use crate::info;
use crate::result::Result;
use crate::x86::PT;
use crate::x86::with_current_page_table;
use crate::x86::PageAttr;
use crate::x86::PAGE_SIZE;
use crate::x86::ATTR_MASK;
use alloc::boxed::Box;
use core::mem::zeroed;
use core::ptr::copy_nonoverlapping;

pub fn load_elf(data: &[u8]) -> Result<u64> {
    // ELFヘッダの解析
    let header = Elf64Header::parse(data).ok_or("Invalid ELF header")?;

    unsafe {
        with_current_page_table(|pml4| {
            for ph in Elf64ProgramHeader::parse_all(data, &header) {
                if ph.p_type != PT_LOAD {
                    continue; // PT_LOADセグメント以外はスキップ
                }

                info!(
                    "Loading segment: vaddr={:#x}, memsz={:#x}, filesz={:#x}, offset={:#x}",
                    ph.p_vaddr, ph.p_memsz, ph.p_filesz, ph.p_offset
                );

                // マッピング

                // ページ数の計算
                let vaddr_start = ph.p_vaddr & !ATTR_MASK;
                let vaddr_end = (ph.p_vaddr + ph.p_memsz + ATTR_MASK) & !ATTR_MASK;
                let size_to_map = (vaddr_end - vaddr_start) as usize;
                let pages_count = size_to_map / PAGE_SIZE;

                for i in 0..pages_count {
                    // ゼロ初期化されたページを確保
                    let frame: Box<PT> = Box::new(zeroed());
                    let phys_addr = Box::into_raw(frame) as u64;

                    // 1ページ(4KB)ずつマッピング
                    // 確保した個別のフレームごとにマップ
                    pml4.create_mapping(
                        vaddr_start + (i * PAGE_SIZE) as u64,
                        vaddr_start + ((i + 1) * PAGE_SIZE) as u64,
                        phys_addr,
                        PageAttr::ReadWriteKernel,
                    )
                    .expect("Failed to map page");
                }

                // データのコピー
                let dest = ph.p_vaddr as *mut u8;
                let src_offset = ph.p_offset as usize;
                let filesz = ph.p_filesz as usize;

                // ファイルの部分コピー
                copy_nonoverlapping(data[src_offset..].as_ptr(), dest, filesz);

                // BSS部分のゼロクリア
                if ph.p_memsz > ph.p_filesz {
                    core::ptr::write_bytes(
                        dest.add(filesz),
                        0,
                        (ph.p_memsz - ph.p_filesz) as usize,
                    );
                }
            }
        });
    }
    Ok(header.entry)
}
