mod cpu;
mod tui;

use cpu::CPU;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "false")]
    tui: bool,

    #[arg(short, long, default_value = "false")]
    debug: bool,


    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    // Open file on arg 1
    let mut file = File::open(args.file).expect("File not found");
    let mut binary: Vec<u8> = Vec::new();
    file.read_to_end(&mut binary).expect("Error reading file");

    if args.tui {
        let _ = tui::tui_start(binary, args.debug);
    } else {
        println!("Starting CHIP-8 emulator...");
        let mut cpu = CPU::new(args.debug);

        cpu.load_bin(binary, false);

        let rows = cpu.get_registers().into_iter().enumerate().map(|(idx, x)| format!("V{:X}:{:X}",idx, x)).collect::<Vec<String>>();

        println!("{:?}", rows);

        cpu.run();
        return;

    }

}

   /*
    let args: Vec<String> = std::env::args().collect();
   */
