//! Архитектурные подсистемы x86 (32-бит).

pub mod io;
pub mod entry;
pub mod idt;
pub mod pic;
pub mod paging;
pub mod cpu;
pub mod tss;
pub mod pit;
pub mod font;