use std::collections::*;
use capstone::*;

use addressspace::*;
use capstone::prelude::*;
use std::fmt::*;

pub trait Instruction {
    fn decode<AS : AddressSpace>(space : &AS, address : u64) -> Option<Self>
        where Self : Sized;
    fn address(&self) -> u64;
    fn bytes(&self) -> Vec<u8>;
    fn nexts(&self) -> Vec<u64>;
}

#[derive(Debug, Clone, Default)]
pub struct X86Instruction {
    pub address : u64,
    pub mnemonic : String,
    pub op_str : String,
    pub bytes : Vec<u8>,
    pub nexts : Vec<u64>,
}

impl Instruction for X86Instruction {
    fn decode<AS : AddressSpace>(space: &AS, address: u64) -> Option<Self> {
        X86Instruction::from_addressspace(space, address)
    }
    fn address(&self) -> u64 { self.address }
    fn bytes(&self) -> Vec<u8> { self.bytes.clone() }
    fn nexts(&self) -> Vec<u64> { self.nexts.clone() }
}

impl Display for X86Instruction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {}", self.mnemonic, self.op_str)
    }
}

impl X86Instruction {
    pub fn from_addressspace<AS : AddressSpace>(space : &AS, address : u64) -> Option<Self> {
        if let Ok(mut cs) = Capstone::new()
                .x86().mode(arch::x86::ArchMode::Mode32)
                .detail(false).build() {
            let mut code : [u8; 16] = [0; 16];
            space.read(address, &mut code[0..15]);
            if code[0] == 0 && code[1] == 0 {
                return None
            }
            if let Ok(insns) = cs.disasm_count(&code, address, 1) {
                return insns.iter().next().map(X86Instruction::from_capstone)
            } 
        }
        None
    }

    pub fn from_capstone<'a>(ins : Insn<'a>) -> Self {
        let mnemonic = ins.mnemonic().unwrap().to_string();
        let op_str = ins.op_str().unwrap_or("").to_string();
        let bytes = ins.bytes();

        let mut nexts = vec![ins.address() + (bytes.len() as u64)];
        
        if mnemonic == "ret" { nexts.clear(); }

        if mnemonic == "jmp" { nexts.clear(); }
    
        let branches = vec!["jg", "jne", "je", "ja", "jmp"];
        if branches.contains(&mnemonic.as_ref()) {
            if let Ok(target) = u64::from_str_radix(&op_str[2..], 16) {
                nexts.push(target);
            }
        }

        X86Instruction {
            address : ins.address(),
            mnemonic : mnemonic,
            op_str : op_str,
            bytes : bytes.to_vec(),
            nexts : nexts,
        } 
    }
}

#[derive(Debug, Clone, Default)]
pub struct AsmSpace<Ins : Instruction> {
    pub space : BTreeMap<u64, Ins>
}

impl<Ins: Instruction + Default> AsmSpace<Ins> {
    pub fn insert(&mut self, address: u64, ins: Ins) -> Option<Ins> {
        self.space.insert(address, ins)
    }
    pub fn disassemble_all<AS:AddressSpace>(space: &AS, entrypoint: u64) -> Self {
        let mut queue = vec![entrypoint];
        let mut seen = BTreeSet::new();

        let mut asm = AsmSpace::default();
        while let Some(addr) = queue.pop() {
            if let Some(ins) = Ins::decode(space, addr) {
                for &next in &ins.nexts() {
                    if seen.insert(next) {
                        queue.push(next);
                    }
                }
                asm.insert(addr, ins);
            }
        }
        asm
    }
}
