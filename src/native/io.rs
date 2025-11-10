/*

    Rewritten I/O library for AdaN

    Includes:
        Terminal:
            - io.out(message: &str);

*/

use inkwell::values::BasicValueEnum;
use crate::code_gen::builder::CodeGenContext;

pub fn out<'ctx>(ctx: &mut CodeGenContext<'ctx>, args: Vec<BasicValueEnum<'ctx>>) -> BasicValueEnum<'ctx> {
    println!("{:?}", args[0]);
    args[0]
}

pub fn register_native<'ctx>(ctx: &mut CodeGenContext<'ctx>) {
    ctx.register_native_fn("io", "out", out);
}