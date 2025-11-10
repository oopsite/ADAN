use crate::parser::ast::{Expr, Literal, Operation};
use crate::code_gen::builder::CodeGenContext;
use crate::code_gen::statements::codegen_function;
use crate::code_gen::builder::NativeFunc;
use inkwell::values::*;
use inkwell::AddressSpace;

fn build_float_mod<'ctx>(ctx: &mut CodeGenContext<'ctx>, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>) -> Result<FloatValue<'ctx>, String> {
    let f64_type = ctx.context.f64_type();
    let fmod_fn = ctx.module.get_function("fmod").unwrap_or_else(|| {
        let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        ctx.module.add_function("fmod", fn_type, None)
    });

    let call_site = ctx.builder
        .build_call(fmod_fn, &[lhs.into(), rhs.into()], "fmodtmp")
        .map_err(|e| format!("call fmod failed: {:?}", e))?;
    let valkind = unsafe { std::mem::transmute::<_, BasicValueEnum>(call_site.try_as_basic_value()) };
    Ok(valkind.into_float_value())
}

pub fn codegen_expressions<'ctx>(ctx: &mut CodeGenContext<'ctx>, expr: &Expr) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => Ok(ctx.context.f64_type().const_float(*n).into()),
            Literal::Bool(b) => Ok(ctx.context.bool_type().const_int(*b as u64, false).into()),
            Literal::Nil => Ok(ctx.context.ptr_type(AddressSpace::from(0)).const_null().into()),
            Literal::String(s) => {
                let array = ctx.context.const_string(s.as_bytes(), true);
                let global = ctx.module.add_global(array.get_type(), None, "str");
                global.set_initializer(&array);
                Ok(global.as_pointer_value().into())
            }
            _ => Err(format!("Literal kind not implemented: {:?}", lit)),
        },

        Expr::Unary { op, right } => {
            match op {
                Operation::Negate => {
                    let r = codegen_expressions(ctx, right)?.into_float_value();
                    let fv = ctx.builder
                        .build_float_neg(r, "negtmp")
                        .map_err(|e| format!("float neg failed: {:?}", e))?;
                    Ok(fv.into())
                }
                Operation::Not => {
                    let r = codegen_expressions(ctx, right)?.into_int_value();
                    let iv = ctx.builder
                        .build_not(r, "nottmp")
                        .map_err(|e| format!("not op failed: {:?}", e))?;
                    Ok(iv.into())
                }
                _ => Err(format!("Unary op not implemented: {:?}", op)),
            }
        }

        Expr::Assign { name, value } => {
            let val = codegen_expressions(ctx, value)?;
            let var_pointer = ctx.variables
                .get(name)
                .ok_or_else(|| format!("Variable not declared: {}", name))?;
            ctx.builder
                .build_store(*var_pointer, val)
                .map_err(|e| format!("store assign failed: {:?}", e))?;
            Ok(val)
        }

        Expr::FCall { callee, args } => {
            let parts: Vec<&str> = callee.split('.').collect();
            let func_val: Result<BasicValueEnum<'ctx>, String> = if parts.len() == 1 {
                if let Some(NativeFunc::AdanFunction(ref adan_func)) = ctx.modules
                    .get("")
                    .and_then(|m| m.get_function(parts[0]))
                {
                    let llvm_fn = codegen_function(ctx, adan_func)
                        .map_err(|e| format!("codegen ADAN function failed: {:?}", e))?;

                    let arg_vals: Result<Vec<BasicValueEnum<'ctx>>, String> =
                        args.iter().map(|a| codegen_expressions(ctx, a)).collect();
                    let arg_vals = arg_vals?;
                    let metadata_args: Vec<BasicMetadataValueEnum> =
                        arg_vals.iter().map(|v| (*v).into()).collect();

                    let call_site = ctx.builder.build_call(llvm_fn, &metadata_args, "calltmp");
                    let valkind = unsafe {
                        std::mem::transmute::<_, BasicValueEnum>(call_site.try_as_basic_value())
                    };
                    Ok(valkind)
                } else if let Some(NativeFunc::NativeFn(native)) = ctx.modules
                    .get("")
                    .and_then(|m| m.get_function(parts[0]))
                {
                    let arg_vals: Result<Vec<BasicValueEnum<'ctx>>, String> =
                        args.iter().map(|a| codegen_expressions(ctx, a)).collect();
                    let arg_vals = arg_vals?;
                    Ok(native(ctx, arg_vals))
                } else {
                    Err(format!("Function '{}' not found", parts[0]))
                }
            } else {
                let module_name = parts[..parts.len() - 1].join(".");
                let func_name = parts.last().unwrap();
                let module_val = ctx.modules
                    .get(&module_name)
                    .ok_or_else(|| format!("Module not found: {}", module_name))?;

                match module_val.get_function(func_name) {
                    Some(NativeFunc::AdanFunction(ref adan_func)) => {
                        let llvm_fn = codegen_function(ctx, adan_func)
                            .map_err(|e| format!("codegen included function failed: {:?}", e))?;

                        let arg_vals: Result<Vec<BasicValueEnum<'ctx>>, String> =
                            args.iter().map(|a| codegen_expressions(ctx, a)).collect();
                        let arg_vals = arg_vals?;
                        let metadata_args: Vec<BasicMetadataValueEnum> =
                            arg_vals.iter().map(|v| (*v).into()).collect();

                        let call_site = ctx.builder.build_call(llvm_fn, &metadata_args, "calltmp");
                        let valkind = unsafe {
                            std::mem::transmute::<_, BasicValueEnum>(call_site.try_as_basic_value())
                        };
                        Ok(valkind)
                    }
                    Some(NativeFunc::NativeFn(native_fn)) => {
                        let arg_vals: Result<Vec<BasicValueEnum<'ctx>>, String> =
                            args.iter().map(|a| codegen_expressions(ctx, a)).collect();
                        let arg_vals = arg_vals?;
                        Ok(native_fn(ctx, arg_vals))
                    }
                    None => Err(format!("Function '{}' not defined in module '{}'", func_name, module_name)),
                }
            };
            func_val
        }

        Expr::Variable(var_name) => {
            let var_pointer = ctx.variables
                .get(var_name)
                .ok_or_else(|| format!("Variable not declared: {}", var_name))?;
            let loaded = ctx.builder
                .build_load(ctx.f64_type, *var_pointer, "loadtmp")
                .map_err(|e| format!("load failed for variable '{}': {:?}", var_name, e))?;
            Ok(loaded)
        }

        Expr::Binary { left, op, right } => {
            let l = codegen_expressions(ctx, left)?.into_float_value();
            let r = codegen_expressions(ctx, right)?.into_float_value();

            match op {
                Operation::Add => Ok(ctx.builder.build_float_add(l, r, "addtmp")
                    .map_err(|e| format!("float add failed: {:?}", e))?.into()),
                Operation::Subtract => Ok(ctx.builder.build_float_sub(l, r, "subtmp")
                    .map_err(|e| format!("float sub failed: {:?}", e))?.into()),
                Operation::Multiply => Ok(ctx.builder.build_float_mul(l, r, "multmp")
                    .map_err(|e| format!("float mul failed: {:?}", e))?.into()),
                Operation::Divide => Ok(ctx.builder.build_float_div(l, r, "divtmp")
                    .map_err(|e| format!("float div failed: {:?}", e))?.into()),
                Operation::Modulo => Ok(build_float_mod(ctx, l, r)?.into()),
                _ => Err(format!("Binary op not implemented: {:?}", op)),
            }
        }

        _ => Err(format!("Expression variant not implemented: {:?}", expr)),
    }
}
