use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;
use crate::code_gen::builder::CodeGenContext;

static LIBRARY_NAME: &str = "io";

fn add_printf_support<'ctx>(ctx: &mut CodeGenContext<'ctx>) -> inkwell::values::FunctionValue<'ctx> {
    if let Some(func) = ctx.module.get_function("printf") {
        func
    } else {
        let i8ptr_type = ctx.context.i8_type().ptr_type(AddressSpace::from(0));
        let printf_type = ctx.context.i32_type().fn_type(&[i8ptr_type.into()], true);
       
        ctx.module.add_function("printf", printf_type, None)
    }
}

pub fn printf<'ctx>(ctx: &mut CodeGenContext<'ctx>, args: Vec<BasicValueEnum<'ctx>>) -> BasicValueEnum<'ctx> {
    let printf_fn = add_printf_support(ctx);
    let fmt_str: PointerValue = match args[0] {
        BasicValueEnum::FloatValue(_) => {
            let s = ctx.context.const_string(b"%f\n\0", true);
            let g = ctx.module.add_global(s.get_type(), None, "fmt_float");
            
            g.set_initializer(&s);
            g.as_pointer_value()
        },
        BasicValueEnum::PointerValue(_) => {
            let s = ctx.context.const_string(b"%s\n\0", true);
            let g = ctx.module.add_global(s.get_type(), None, "fmt_str");
            
            g.set_initializer(&s);
            g.as_pointer_value()
        },
        _ => panic!("Unsupported argument type for out()"),
    };

    ctx.builder.build_call(
        printf_fn,
        &[fmt_str.into(), args[0].into()],
        "call_printf",
    );

    args[0]
}

pub fn register_native<'ctx>(ctx: &mut CodeGenContext<'ctx>) {
    ctx.register_native_fn(LIBRARY_NAME, "printf", printf);

    // println!("Added '{}' to module registry", LIBRARY_NAME);
}

