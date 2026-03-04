/**
 * a12kdump, gio 2026 (https://github.com/gio3k)
 */
use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::Path,
};

use clap::Parser;

use crate::{
    rddm::RddmDump,
    tlv::{raw, CrashDump},
};

pub mod rddm;
pub mod tlv;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 't', long, default_value = "coredump")]
    input_type: String,

    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    silent: bool,
}

fn main() {
    let args = Args::parse();

    let output_path = Path::new(&args.output);

    if !args.silent {
        println!("gio's ath12k dump dumper");
        println!(
            "input type: '{}', input: '{}', output: '{}'",
            args.input_type, args.input, args.output
        );
    }

    // Open the file, check the magic
    let mut input_file = File::open(&args.input).expect("Failed to open input file");
    let has_coredump_magic = CrashDump::check_magic(&mut input_file);

    // Make sure we're handling input_type correctly
    match args.input_type.as_str() {
        "coredump" if !has_coredump_magic => {
            println!(
                "input_type is 'coredump' but a valid coredump wasn't provided (magic wasn't right)"
            );
            println!("if you're trying to dump data from an RDDM dump, use '--input-type=rddm'");
            return;
        }
        "coredump" => {}
        "rddm" if has_coredump_magic && !args.silent => {
            println!("input_type is 'rddm' but the input seems to be a valid coredump!");
            println!("this is probably wrong! will try to handle the input as RDDM data anyways...")
        }
        "rddm" => {}
        x => {
            println!("unknown input_type '{}'", x);
            println!("try 'coredump' or 'rddm'");
            return;
        }
    }

    // Move back to the start of the file
    input_file
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek back to the file start");

    // Handle coredump if applicable
    if args.input_type == "coredump" {
        let crash_dump = CrashDump::parse(&mut input_file).expect("Failed to parse crash dump");

        // First pass of the sections, let's dump them
        for section in crash_dump.sections() {
            if !args.silent {
                println!(
                    "[section] type: {:?}, offset: {}, len: {}",
                    section.section_type(),
                    section.offset(),
                    section.len()
                );
            }

            input_file
                .seek(std::io::SeekFrom::Start(section.offset()))
                .expect(
                    format!(
                        "Failed to seek to the crash dump section at {}",
                        section.offset()
                    )
                    .as_str(),
                );

            let output_file_path = output_path.join(format!(
                "coredump-{:?}-{}.bin",
                section.section_type(),
                section.len()
            ));
            let mut output_file =
                File::create(output_file_path).expect("Failed to create output file");

            // Read section data into a buffer
            let mut section_data_buffer = vec![0u8; section.len() as usize];
            input_file
                .read_exact(&mut section_data_buffer)
                .expect("Failed to read section data");

            // Write buffer to the output file
            output_file
                .write_all(&section_data_buffer)
                .expect("Failed to write output file");

            if !args.silent {
                println!("[section] dumped {:?}", section.section_type());
            }
        }

        // Second pass, we just want to handle the RDDM section
        for section in crash_dump.sections() {
            if section.section_type() != raw::FwCrashDumpType::RddmData {
                continue;
            }

            input_file
                .seek(std::io::SeekFrom::Start(section.offset()))
                .expect(
                    format!(
                        "Failed to seek to the crash dump section at {}",
                        section.offset()
                    )
                    .as_str(),
                );
        }
    }

    // We should be at the RDDM section now
    let start_offset = input_file
        .stream_position()
        .expect("Failed to get offset in file");

    if !args.silent {
        println!("[rddm] dumping RDDM data at offset {}", start_offset);
    }

    let rddm_dump = RddmDump::parse(&mut input_file).expect("Failed to parse RDDM data");
    if !args.silent {
        println!("[rddm] header size = {}", rddm_dump.header_size());
    }

    for entry in rddm_dump.entries() {
        if !args.silent {
            println!(
                "[rddm entry] entry offset = {}, entry len = {}, name = {}",
                entry.offset(),
                entry.len(),
                entry.entry_name()
            );
        }

        input_file
            .seek(std::io::SeekFrom::Start(entry.offset()))
            .expect(format!("Failed to seek to the entry at {}", entry.offset()).as_str());

        let output_file_path = output_path.join(format!(
            "rddm-{}-{}.bin",
            entry.entry_file_name(),
            entry.len()
        ));
        let mut output_file = File::create(output_file_path).expect("Failed to create output file");

        // Read entry data into a buffer
        let mut entry_data_buffer = vec![0u8; entry.len() as usize];
        input_file
            .read_exact(&mut entry_data_buffer)
            .expect("Failed to read entry data");

        // Write buffer to the output file
        output_file
            .write_all(&entry_data_buffer)
            .expect("Failed to write output file");

        if !args.silent {
            println!("[rddm entry] dumped {}", entry.entry_file_name());
        }
    }

    if !args.silent {
        println!("done!");
    }
}
