
use crate::syntax::Var;

// This isn't really working out. Come back later?

enum Basic {
    Const(Var, i64),
    Alias(Var, Var),
    Arith(Var, BinOp, Var, Var),
    Print(Var),
    // Phi(Var, Vec<(BlockRef, Var)>),
    // Only necessary for SSA? But SSA is really useful?
}

enum Control {
    Continue(BlockRef),
    Branch(BlockRef, BlockRef),
}

struct BasicBlock {
    insts: Vec<Basic>,
    control: Control,
}

struct BlockRef(usize);

struct CFG {
    blocks: Vec<BasicBlock>,
    entry: BlockRef,
}


// compilation generates a cfg
// it gets built a block at a time, and each block is built an instruction at a time.
// (With some patching to deal with branch targets?)

// Codegen for the CFG is somewhat more interesting.
// Everything is straightforward, except for the most important part: variable assignments.
// The process of building the CFG means de-nesting expressions, which introduces large numbers of
// temporary local variables. This leads to poor codegen, as each local variable gets its own
// locals slot. A smarter solution would be to do liveness analysis and assign a minimal number of
// slots, but I'm 99% confident that this boils down to computing a minimal coloring of the
// interference graph. I *really* don't want to have to deal with graph coloring register
// allocation.
//
// Maybe that "reverse scan register allocation" thing might help?
// Hmm. It might. 

