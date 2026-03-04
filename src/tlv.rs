/**
 * a12kdump, gio 2026 (https://github.com/gio3k)
 * note: "crash dump" and "core dump" are referring to the same thing in this tool
 * the kernel calls it a core dump, the driver calls it a crash dump / core dump
 */
use std::{
    io::{Read, Seek},
    mem::transmute,
};

pub mod raw {
    // ath12k_fw_crash_dump_type
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(C)]
    pub enum FwCrashDumpType {
        PagingData,
        RddmData,
        RemoteMemData,
        PageableData,
        M3Dump,
        None,
        MloGlobalData,
        __MAX__,
    }

    // ath12k_tlv_dump_data
    #[derive(Debug)]
    #[repr(C, packed)]
    pub struct TlvDumpData {
        // __le32 type
        pub crash_dump_type: FwCrashDumpType,
        pub len: u32,
    }

    // ath12k_dump_file_data
    #[derive(Debug)]
    #[repr(C, packed)]
    pub struct DumpFileData {
        pub df_magic: [u8; 16],
        pub len: u32,
        pub version: u32,
        pub chip_id: u32,
        pub qrtr_id: u32,
        pub bus_id: u32,
        pub guid: [u8; 16],
        pub tv_sec: u64,
        pub tv_nsec: u64,
        pub unused: [u8; 128],
    }
}

const CRASH_DUMP_MAGIC: &[u8; 16] = b"ATH12K-FW-DUMP\0\0";

pub struct CrashDumpSection {
    section_type: raw::FwCrashDumpType,
    offset: u64,
    len: u32,
}

impl CrashDumpSection {
    pub fn section_type(&self) -> raw::FwCrashDumpType {
        self.section_type
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn parse<R: Read + Seek>(data: &mut R) -> Option<Self> {
        let mut x = [0u8; std::mem::size_of::<raw::TlvDumpData>()];
        data.read_exact(&mut x[..]).ok()?;
        let raw_header: raw::TlvDumpData = unsafe { transmute(x) };

        Some(CrashDumpSection {
            section_type: raw_header.crash_dump_type,

            // offset: start of the section's data
            // we've already read the header, so we're at the data now
            offset: data.seek(std::io::SeekFrom::Current(0)).ok()?,
            len: raw_header.len,
        })
    }
}

pub struct CrashDump {
    raw_header: raw::DumpFileData,
    sections: Vec<CrashDumpSection>,
}

impl CrashDump {
    pub fn raw_header(&self) -> &raw::DumpFileData {
        &self.raw_header
    }

    pub fn sections(&self) -> &[CrashDumpSection] {
        &self.sections
    }

    pub fn check_magic<R: Read>(data: &mut R) -> bool {
        let mut x = [0u8; std::mem::size_of::<raw::DumpFileData>()];
        if data.read_exact(&mut x[..]).is_ok() {
            let raw_header: raw::DumpFileData = unsafe { std::mem::transmute(x) };
            raw_header.df_magic == *CRASH_DUMP_MAGIC
        } else {
            false
        }
    }

    pub fn parse<R: Read + Seek>(data: &mut R) -> Option<Self> {
        // Read the file header first
        let mut x = [0u8; std::mem::size_of::<raw::DumpFileData>()];
        data.read_exact(&mut x[..]).ok()?;
        let raw_header: raw::DumpFileData = unsafe { std::mem::transmute(x) };

        // Check the file magic
        if raw_header.df_magic != *CRASH_DUMP_MAGIC {
            return None;
        }

        let mut sections = Vec::new();
        while let Some(section) = CrashDumpSection::parse(data) {
            data.seek(std::io::SeekFrom::Current(section.len() as i64))
                .ok()?;
            sections.push(section);
        }

        Some(Self {
            raw_header,
            sections,
        })
    }
}
