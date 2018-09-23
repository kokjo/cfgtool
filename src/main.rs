#![allow(dead_code)]

extern crate capstone;
extern crate dot;
extern crate goblin;

pub mod addressspace;
pub mod basicblock;
pub mod instruction;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::fmt::*;

use addressspace::*;
use basicblock::*;
use instruction::*;

fn example<AS:AddressSpace>(space: AS, entrypoint:u64) -> CFG<X86Instruction> {
    let asmspace = AsmSpace::disassemble_all(&space, entrypoint);
    let cfg : CFG<X86Instruction> = CFG::from_asmspace(asmspace, entrypoint);

    for bb in cfg.graph.values() {
        println!("{}", bb);
    }

    cfg
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut f = File::open(filename).expect("file not found"); 
    let mut out = File::create(filename.clone() + ".dot").expect("Could not create file");
    let mut code = Vec::new();

    f.read_to_end(&mut code).expect("could not read file");
    
    let space = BlobAddressSpace::default()
        .load(0x1000, &code);
    let cfg = example(space, 0x1000);
    cfg.render_to(&mut out);
}
