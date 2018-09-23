use std::collections::BTreeMap;
use goblin::*;
use goblin::elf::*;

use std::io;
use std::io::prelude::*;
use std::fs::File;

pub trait AddressSpace {
    fn read(&self, address:u64, buffer : &mut [u8]);
}

#[derive(Default)]
pub struct BlobAddressSpace {
    space : BTreeMap<u64, u8>
}

impl BlobAddressSpace {
    pub fn write_byte(&mut self, address : u64, byte : u8) {
        self.space.insert(address, byte);
    }

    pub fn load(mut self, address : u64, data:&[u8]) -> Self {
        for (off, &byte) in data.iter().enumerate() {
            self.write_byte(address+(off as u64), byte);
        }
        self
    }
}

impl AddressSpace for BlobAddressSpace {
    fn read(&self, address:u64, buffer : &mut [u8]) {
        for (off, byte) in buffer.iter_mut().enumerate() {
            *byte = self.space.get(&(address + (off as u64))).cloned().unwrap_or(0);
        }
    }
}

pub struct ElfAddressSpace {
    data : Vec<u8>,
    programheaders : Vec<ProgramHeader>
}

impl ElfAddressSpace {
    pub fn load_from_read<Reader : Read>(f : &mut Reader) -> io::Result<Option<Self>> {
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        Ok(Self::load(data))
    }
    pub fn load(data : Vec<u8>) -> Option<Self> {
        if let Ok(binary) = Elf::parse(&data.clone()) {
            return Some(ElfAddressSpace{
                data : data,
                programheaders : binary.program_headers
            });
        }
        None
    }
}

impl AddressSpace for ElfAddressSpace {
    fn read(&self, address:u64, buffer : &mut [u8]){

    }
}
