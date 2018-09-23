use std::collections::*;

use addressspace::*;

use std::collections::btree_map::Values;
use std::marker::PhantomData;


pub trait Instruction {
    fn address(&self) -> u64;
    fn bytes(&self) -> Vec<u8>;
    fn nexts(&self) -> Vec<u64>;
}

pub trait InstructionFactory {
    type Ins : Instruction;
    fn decode<AS : AddressSpace>(space : &AS, address : u64) -> Option<Self::Ins>;
}


struct HintInstruction {
    
}

#[derive(Debug, Clone)]
pub struct AsmSpace<Ins : Instruction, InsFactory : InstructionFactory<Ins=Ins>> {
    space : BTreeMap<u64, Ins>,
    phantom: PhantomData<InsFactory>,
}

impl<Ins: Instruction, InsFactory: InstructionFactory<Ins=Ins>> Default for AsmSpace<Ins, InsFactory> {
    fn default() -> Self {
        AsmSpace {
            space : BTreeMap::new(),
            phantom: PhantomData
        }
    }
}

impl<Ins: Instruction, InsFactory : InstructionFactory<Ins=Ins>> AsmSpace<Ins, InsFactory> {
    pub fn insert(&mut self, address: u64, ins: Ins) -> Option<Ins> {
        self.space.insert(address, ins)
    }

    pub fn get(&self, address : u64) -> Option<&Ins> {
        self.space.get(&address)
    }
    
    pub fn instructions(&self) -> Values<u64, Ins> {
        self.space.values()
    }

    pub fn disassemble_all<AS: AddressSpace>(space: &AS, entrypoint: u64) -> Self {
        let mut queue = vec![entrypoint];
        let mut seen = BTreeSet::new();

        let mut asm = AsmSpace::default();
        while let Some(addr) = queue.pop() {
            if let Some(ins) = InsFactory::decode(space, addr) {
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
