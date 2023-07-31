mod cpu;
mod tui;

use cpu::CPU;
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

    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    if args.tui {
        let _ = tui::tui_start();
    } else {
        println!("Starting CHIP-8 emulator...");
        let mut cpu = CPU::new();

        // Open file on arg 1
        let mut file = File::open(args.file).expect("File not found");
        let mut binary: Vec<u8> = Vec::new();
        file.read_to_end(&mut binary).expect("Error reading file");
        cpu.load_bin(binary, false);

        for y in 0..32 {
            for x in 0..64 {
                if cpu.read_vbuf(x, y) {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
        cpu.run();
        return;

    }

}

   /*
    let args: Vec<String> = std::env::args().collect();
   */
