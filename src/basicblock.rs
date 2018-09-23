use std::collections::*;
use std::borrow::Cow;
use std::io::Write;
use std::fmt::*;
use dot::*;

use instruction::*;

#[derive(Debug, Clone, Default)]
pub struct BasicBlock<Ins> where Ins : Instruction{
    pub address : u64,
    pub insns : Vec<Ins>,
    pub prevs : Vec<u64>
}

impl<Ins : Instruction> BasicBlock<Ins> {
    pub fn new(first_ins : Ins, prevs : & Vec<u64>) -> Self {
        BasicBlock {
            address : first_ins.address(),
            insns : vec![first_ins],
            prevs : prevs.clone()
        }
    }
    pub fn nexts(&self) -> Vec<u64> {
        if let Some(ins) = self.insns.last(){
            ins.nexts().clone()
        } else {
            Vec::new()
        }
    }
    pub fn push_ins(&mut self, ins : Ins) {
        self.insns.push(ins);
    }
}

impl<Ins : Instruction + Display> Display for BasicBlock<Ins> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "0x{:x}:\n", self.address)?;
        for ins in &self.insns {
            write!(f, "  {}\n", ins)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CFG<Ins: Instruction> {
    pub entrypoint : u64,
    pub graph : BTreeMap<u64, BasicBlock<Ins>>
}

impl<Ins: Instruction + Clone> CFG<Ins> {
    pub fn from_asmspace(space : AsmSpace<Ins>, entrypoint : u64) -> Self {
        let mut prevs_map : BTreeMap<u64, Vec<u64>> = BTreeMap::new();

        for ins in space.space.values() {
            for &next in &ins.nexts() {
                let mut prevs = prevs_map.entry(next).or_insert(Vec::new());
                prevs.push(ins.address());
            }
        }

        let mut queue = vec![entrypoint];
        let mut seen = BTreeSet::new();
        let mut graph = BTreeMap::new();

        while let Some(addr) = queue.pop() {
            if let Some(first_ins) = space.space.get(&addr){
                let first_ins_prevs = prevs_map.entry(addr).or_insert(Vec::new()).clone();
                let mut bb = BasicBlock::new(first_ins.clone(), &first_ins_prevs);
                while bb.nexts().len() == 1 {
                    let next = bb.nexts()[0];
                    if let Some(ins) = space.space.get(&next) {
                        if let Some(prevs) = prevs_map.get(&next) {
                            if prevs.len() <= 1 {
                                bb.push_ins(ins.clone());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                for &next in bb.nexts().iter() {
                    if seen.insert(next) {
                        queue.push(next);
                    }
                }
                graph.insert(bb.address, bb);
            }
        }

        CFG {
            entrypoint : entrypoint,
            graph : graph
        }
    }

}
impl<Ins : Instruction + Display> CFG<Ins> {
    pub fn render_to<W: Write>(&self, output: &mut W) {
        render(self, output).unwrap()
    }
}

type Nd = u64;
type Ed = (u64, u64);

impl<'a, Ins: Instruction + Display> GraphWalk<'a, Nd, Ed> for CFG<Ins> {
    fn nodes(&self) -> Nodes<'a, Nd> {
        self.graph.values().map(|bb| bb.address).collect()
    }

    fn edges(&self) -> Edges<'a, Ed> {
        let mut edges : Vec<Ed> = Vec::new();
        for (&address, bb) in self.graph.iter() {
            for &next in bb.nexts().iter() {
                edges.push((address, next));
            }
        }
        edges.iter().cloned().collect()
    }

    fn source(&self, e: &Ed) -> Nd { e.0 }
    fn target(&self, e: &Ed) -> Nd { e.1 }
}

impl<'a, Ins: Instruction + Display> Labeller<'a, Nd, Ed> for CFG<Ins> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("example1").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> Id<'a> {
        Id::new(format!("N{:x}", *n)).unwrap()
    }

    fn node_label(&'a self, n: &Nd) -> LabelText<'a> {
        let bb = self.graph.get(n).unwrap();
        LabelText::escaped(format!("{}", bb).replace("\n", "\\l"))
    }

    fn node_shape(&'a self, _node: &Nd) -> Option<LabelText<'a>> {
        Some(LabelText::label("box"))
    }

}
