/**
 * a12kdump, gio 2026 (https://github.com/gio3k)
 */
pub mod raw {
    #[derive(Debug)]
    #[repr(C, packed)]
    // 8 bytes
    pub struct Header {
        unk0: u32,
        pub header_size: u32,
    }

    #[derive(Debug)]
    #[repr(C, packed)]
    // 64 bytes
    pub struct FileEntry {
        pub unk0: u64,

        // +8
        pub unk1: u64,
        // pub offset: u64,

        // +16
        pub len: u64,

        // +24
        pub entry_name: [u8; 20],

        // +44
        pub entry_file_name: [u8; 20],
    }
}

pub struct RddmDumpEntry {
    raw_entry: raw::FileEntry,
    pub(crate) offset: u64,
}

impl RddmDumpEntry {
    pub fn entry_name(&self) -> &str {
        std::str::from_utf8(&self.raw_entry.entry_name)
            .expect("Failed to get entry name")
            .trim_end_matches('\0')
    }

    pub fn entry_file_name(&self) -> &str {
        std::str::from_utf8(&self.raw_entry.entry_file_name)
            .expect("Failed to get entry file name")
            .trim_end_matches('\0')
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn len(&self) -> u64 {
        self.raw_entry.len
    }
}
pub struct RddmDump {
    header: raw::Header,
    entries: Vec<RddmDumpEntry>,
}

impl RddmDump {
    pub fn raw_header(&self) -> &raw::Header {
        &self.header
    }

    pub fn entries(&self) -> &[RddmDumpEntry] {
        &self.entries
    }

    pub fn header_size(&self) -> u32 {
        self.header.header_size
    }

    pub fn parse<R: std::io::Read + std::io::Seek>(data: &mut R) -> Option<Self> {
        let start_offset = data.stream_position().ok()?;

        // Read the dump header from the file
        let mut x = [0u8; std::mem::size_of::<raw::Header>()];
        data.read_exact(&mut x[..]).ok()?;
        let raw_header: raw::Header = unsafe { std::mem::transmute(x) };

        // Read the file entries until we reach the end of the header
        let mut entries = Vec::<RddmDumpEntry>::new();
        let mut current_entry_data_offset = start_offset + raw_header.header_size as u64;
        loop {
            if data.stream_position().ok()? >= start_offset + raw_header.header_size as u64 {
                break;
            }

            // Read the entry header
            let mut x = [0u8; std::mem::size_of::<raw::FileEntry>()];
            data.read_exact(&mut x[..]).ok()?;
            let raw_entry: raw::FileEntry = unsafe { std::mem::transmute(x) };

            // Push entry, increment data offset
            let entry_len = raw_entry.len;

            entries.push(RddmDumpEntry {
                raw_entry,
                offset: current_entry_data_offset,
            });

            current_entry_data_offset += entry_len;
        }

        Some(Self {
            header: raw_header,
            entries,
        })
    }
}
