mod ast_gen;
use std::io;
use ast_gen::*;

fn main() -> io::Result<()> {
    let gen = GenAst::new("Expr", "./src");
    gen.gen_ast()
}
