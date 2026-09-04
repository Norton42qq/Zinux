//! Порт `kernel/system.c` (paging-часть): 4KB-страницы, identity-карта.
//!
//! Карта: 16 низких page-таблиц (64MB identity), LFB-таблицы под экран,
//! пользовательская память 0x200000..0x300000 помечена USER.

use core::arch::asm;
use core::ptr;

use crate::drivers::video::video_get_info;

pub const PAGE_PRESENT: u32 = 0x001;
pub const PAGE_RW: u32 = 0x002;
pub const PAGE_USER: u32 = 0x004;

pub const USER_MEM_START: u32 = 0x200000;
pub const USER_MEM_END: u32 = 0x300000;

#[derive(Copy, Clone)]
#[repr(C, align(4096))]
pub struct PageTable(pub [u32; 1024]);

/// Page directory (1024 PDE), 4KB-выровнен.
pub static mut PAGE_DIRECTORY: PageTable = PageTable([0; 1024]);
/// 16 низких page-таблиц: 64MB identity.
pub static mut LOW_PAGE_TABLES: [PageTable; 16] = [PageTable([0; 1024]); 16];
/// Page-таблицы для линейного фреймбуфера (до 8 x 4MB = 32MB).
pub static mut LFB_PAGE_TABLES: [PageTable; 8] = [PageTable([0; 1024]); 8];

/// Инициализация paging (порт `paging_init`).
pub fn paging_init() {
    unsafe {
        (ptr::addr_of_mut!(PAGE_DIRECTORY) as *mut u8)
            .write_bytes(0u8, core::mem::size_of::<PageTable>());
        (ptr::addr_of_mut!(LOW_PAGE_TABLES) as *mut u8)
            .write_bytes(0u8, core::mem::size_of::<[PageTable; 16]>());
        (ptr::addr_of_mut!(LFB_PAGE_TABLES) as *mut u8)
            .write_bytes(0u8, core::mem::size_of::<[PageTable; 8]>());

        let pd = ptr::addr_of_mut!(PAGE_DIRECTORY) as *mut u32;
        let low = ptr::addr_of_mut!(LOW_PAGE_TABLES) as *mut u32;

        for pde in 0..16u32 {
            let mut pde_flags = PAGE_PRESENT | PAGE_RW;
            let pde_start = pde << 22;
            let pde_end = pde_start + 0x400000u32;
            if pde_start < USER_MEM_END && pde_end > USER_MEM_START {
                pde_flags |= PAGE_USER;
            }
            // адрес таблицы: base of LOW_PAGE_TABLES + pde*4096
            let table_phys = (low as u32) + (pde << 12);
            *pd.add(pde as usize) = table_phys | pde_flags;

            let table = low.add((pde << 10) as usize);
            for pte in 0..1024u32 {
                let phys = pde_start + (pte << 12);
                let mut flags = PAGE_PRESENT | PAGE_RW;
                if phys >= USER_MEM_START && phys < USER_MEM_END {
                    flags |= PAGE_USER;
                }
                *table.add(pte as usize) = phys | flags;
            }
        }

        // Keep LFB mapped after enabling paging
        let vi = video_get_info();
        if !vi.is_null() && (*vi).lfb_addr != 0 {
            let fb_start = (*vi).lfb_addr & 0xFFFFF000;
            let fb_size = (*vi).pitch * (*vi).height as u32;
            let fb_end = ((*vi).lfb_addr + fb_size + 0xFFF) & 0xFFFFF000;
            let start_pde = fb_start >> 22;
            let end_pde = (fb_end - 1) >> 22;
            let mut tidx = 0usize;

            let lfb = ptr::addr_of_mut!(LFB_PAGE_TABLES) as *mut u32;

            let mut pde = start_pde;
            while pde <= end_pde && tidx < 8 {
                let base = pde << 22;
                let lfb_row = lfb.add((tidx << 10) as usize);
                (lfb_row as *mut u8).write_bytes(0u8, 4096);
                for pte in 0..1024u32 {
                    let phys = base + (pte << 12);
                    if phys >= fb_start && phys < fb_end {
                        *lfb_row.add(pte as usize) = phys | PAGE_PRESENT | PAGE_RW;
                    }
                }
                let lfb_table_phys = (lfb as u32) + ((tidx as u32) << 12);
                *pd.add(pde as usize) = lfb_table_phys | PAGE_PRESENT | PAGE_RW;
                pde += 1;
                tidx += 1;
            }
        }

        // Включение paging
        let pd_addr = ptr::addr_of_mut!(PAGE_DIRECTORY) as u32;
        asm!("mov cr3, {0}", in(reg) pd_addr, options(nostack, nomem, preserves_flags));
        let cr0: u32;
        asm!("mov {0}, cr0", out(reg) cr0, options(nostack, nomem, preserves_flags));
        let cr0 = cr0 | 0x8000_0000u32;
        asm!("mov cr0, {0}", in(reg) cr0, options(nostack, nomem));
    }
}