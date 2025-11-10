use crate::parser::ast::{Expr, Literal, Operation};
use crate::code_gen::builder::{CodeGenContext};
use crate::code_gen::statements::{codegen_function};
use inkwell::AddressSpace;
use inkwell::values::*;                             // Gave up on adding stuff, literally just
                                                    // importing all of it.

// Using: LLVM 15.0, 15.0+ aren't fully supported with Inkwell right now.
// If compiling from source, make sure you have LLVM 15.0 installed and as your primary version.

fn build_float_mod<'ctx>(ctx: &mut CodeGenContext<'ctx>, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>) -> FloatValue<'ctx> {
    let f64_type = ctx.context.f64_type();
    let fmod_fn = ctx.module.get_function("fmod").unwrap_or_else(|| {
        let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        ctx.module.add_function("fmod", fn_type, None)
    });

    let call_site = ctx.builder.build_call(fmod_fn, &[lhs.into(), rhs.into()], "fmodtmp").expect("call failed");
    let result: BasicValueEnum = unsafe {
        // Turn the private ValueKind into a public BasicValueEnum (unsafe)
        std::mem::transmute::<_, BasicValueEnum>(call_site.try_as_basic_value())
    };

    result.into_float_value()
}

// Translate various expression types -> LLVM types.
// +, /, -, *, %
pub fn codegen_expressions<'ctx>(ctx: &mut CodeGenContext<'ctx>, expr: &Expr) -> BasicValueEnum<'ctx> {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => ctx.context.f64_type().const_float(*n).into(),
            Literal::Bool(b) => ctx.context.bool_type().const_int(*b as u64, false).into(),
            Literal::Nil => ctx.context.i8_type().ptr_type(AddressSpace::from(0u16)).const_null().into(),
            Literal::String(s) => {
                let array = ctx.context.const_string(s.as_bytes(), true);
                let global = ctx.module.add_global(array.get_type(), None, "str");

                global.set_initializer(&array);
                global.as_pointer_value().into()
            }

            _ => unimplemented!(),
        },

        Expr::Unary { op, right } => {
            match op {
                Operation::Negate => {
                    let r = codegen_expressions(ctx, right).into_float_value();

                    ctx.builder.build_float_neg(r, "negtmp").expect("negation failed").into() // Flips x -> -x
                },

                Operation::Not => {
                    let r = codegen_expressions(ctx, right).into_int_value();

                    ctx.builder.build_not(r, "nottmp").expect("temporary not failed").into() // Flips true -> false, false -> true
                },

                _ => unimplemented!(),
            }
        },

        Expr::Assign { name, value } => {
            let val = codegen_expressions(ctx, value);
            let var_pointer = ctx.variables.get(name).expect("Variable not declared");

            ctx.builder.build_store(*var_pointer, val);
            val
        },

        Expr::FCall { callee, args } => {
            println!("Calling function: {}", callee);

            let parts: Vec<&str> = callee.split('.').collect();
            let func: FunctionValue<'ctx> = if parts.len() == 1 {
                ctx.module.get_function(parts[0]).expect("Function not defined")
            } else {
                let module_name = parts[..parts.len() - 1].join(".");
                let func_name = parts.last().unwrap();
                println!("Looking up function '{}' in module '{}'", func_name, module_name);

                let decl_ref = {
                    let module_val = ctx.modules.get(&module_name).expect("Module not found");
                    module_val.get_function(func_name).expect("Function not defined in module").clone()
                };

                codegen_function(ctx, &decl_ref).expect("Failed to codegen included function")
            };

            println!("Function resolved: {:?}", func);

            let arg_vals: Vec<BasicValueEnum> = args.iter().map(|arg| codegen_expressions(ctx, arg)).collect();
            println!("Arguments: {:?}", arg_vals);

            let metadata_args: Vec<BasicMetadataValueEnum> = arg_vals.iter().map(|v| (*v).into()).collect();
            let call_site_result = ctx.builder.build_call(func, &metadata_args, "calltmp");
            let call_site = call_site_result.expect("call failed");
            let ret_val: BasicValueEnum = unsafe {
                std::mem::transmute::<_, BasicValueEnum>(call_site.try_as_basic_value())
            };

            println!("Call returned: {:?}", ret_val);
            ret_val
        }

        Expr::Variable(var_name) => {
            let var_pointer = ctx.variables.get(var_name).expect("Variable not declared");

            ctx.builder.build_load(ctx.f64_type, *var_pointer, "loadtmp").expect("build load failed").into()
        }, 

        Expr::Binary { left, op, right } => {
            let l = codegen_expressions(ctx, left).into_float_value();
            let r = codegen_expressions(ctx, right).into_float_value();
            
            match op {
                Operation::Add => ctx.builder.build_float_add(l, r, "addtmp").expect("add failed").into(),
                Operation::Subtract => ctx.builder.build_float_sub(l, r, "subtmp").expect("sub failed").into(),
                Operation::Multiply => ctx.builder.build_float_mul(l, r, "multmp").expect("multiplication failed").into(),
                Operation::Divide => ctx.builder.build_float_div(l, r, "divtmp").expect("div failed").into(),
                Operation::Modulo => build_float_mod(ctx, l, r).into(),
             
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}
