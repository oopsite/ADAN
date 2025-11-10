use crate::parser::ast::{FunctionDecl, Statement, Expr};
use crate::code_gen::builder::{CodeGenContext, ModuleValue};
use crate::code_gen::expressions::codegen_expressions;
use inkwell::values::*;
use inkwell::types::{BasicTypeEnum, BasicMetadataTypeEnum};
use crate::lexer::token::Types;
use inkwell::AddressSpace;
use std::path::Path;
use std::collections::HashMap;

pub fn codegen_function<'ctx>(ctx: &mut CodeGenContext<'ctx>, declaration: &FunctionDecl) -> Result<FunctionValue<'ctx>, String> {
    let param_types: Vec<BasicMetadataTypeEnum> = declaration.params.iter().map(|_| ctx.f64_type.into()).collect();
    let fn_type = ctx.f64_type.fn_type(&param_types, false);
    let func = ctx.module.add_function(&declaration.name, fn_type, None);
    let entry = ctx.context.append_basic_block(func, "entry");

    ctx.builder.position_at_end(entry);
    for (i, param_name) in declaration.params.iter().enumerate() {
        let param = func.get_nth_param(i as u32).unwrap();
        let alloca = ctx.builder.build_alloca(ctx.f64_type, param_name).unwrap();

        ctx.builder.build_store(alloca, param);
        ctx.variables.insert(param_name.clone(), alloca);
    }

    for stmt in &declaration.body {
        codegen_statements(ctx, stmt)?;
    }

    if func.get_last_basic_block().unwrap().get_terminator().is_none() {
        ctx.builder.build_return(Some(&ctx.f64_type.const_float(0.0)));
    }

    if !func.verify(true) {
        return Err("Function verification failed".to_string());
    }

    Ok(func)
}

pub fn codegen_statements<'ctx>(ctx: &mut CodeGenContext<'ctx>, stmt: &Statement) -> Result<(), String> {
    match stmt {
        Statement::Expression(expr) => {
            codegen_expressions(ctx, expr);
            Ok(())
        },

        Statement::VarDecl { name, var_type, initializer } => {
            let llvm_type: BasicTypeEnum<'ctx> = match var_type {
                Some(ty) => match ty {
                    Types::i8 | Types::i32 | Types::i64 | Types::u8 | Types::u32 | Types::u64 | Types::Boolean | Types::Char =>
                        ctx.context.i64_type().into(),
                    Types::f32 | Types::f64 =>
                        ctx.context.f64_type().into(),
                    Types::String | Types::Array | Types::Object =>
                        ctx.context.ptr_type(AddressSpace::from(0)).into(),
                },
                None => ctx.context.i64_type().into(),
            };

            let default_val = match var_type {
                Some(ty) => match ty {
                    Types::i8 | Types::i32 | Types::i64 | Types::u8 | Types::u32 | Types::u64 => {
                        llvm_type.into_int_type().const_int(0, false).into()
                    },
                    Types::f32 | Types::f64 => {
                        llvm_type.into_float_type().const_float(0.0).into()
                    },
                    Types::Boolean => llvm_type.into_int_type().const_int(0, false).into(),
                    Types::Char => llvm_type.into_int_type().const_int(0, false).into(),
                    Types::String | Types::Array | Types::Object => llvm_type.into_pointer_type().const_null().into(),
                },
                None => llvm_type.into_int_type().const_int(0, false).into(),
            };

            let value = initializer.as_ref().map(|e| codegen_expressions(ctx, e)).unwrap_or(default_val);
            let pointer = ctx.builder.build_alloca(llvm_type, name).unwrap();

            ctx.builder.build_store(pointer, value);
            ctx.variables.insert(name.clone(), pointer);
            Ok(())
        }


        Statement::Block(statements) => {
            for s in statements {
                codegen_statements(ctx, s)?;
            }
            Ok(())
        },

        Statement::If { condition, then_branch, else_branch } => {
            let cond = codegen_expressions(ctx, condition).into_float_value();
            let zero = ctx.context.f64_type().const_float(0.0);
            let comparison = ctx.builder.build_float_compare(inkwell::FloatPredicate::ONE, cond, zero, "ifcond").unwrap();
            let func = ctx.builder.get_insert_block().ok_or("No insert block")?.get_parent().ok_or("No parent function")?;
            let then_block = ctx.context.append_basic_block(func, "then");
            let else_block = else_branch.as_ref().map(|_| ctx.context.append_basic_block(func, "else"));
            let merge = ctx.context.append_basic_block(func, "ifcont");
            let target_block = else_block.unwrap_or(merge);

            ctx.builder.build_conditional_branch(comparison, then_block, target_block).unwrap();
            ctx.builder.position_at_end(then_block);
            codegen_statements(ctx, then_branch)?;
            ctx.builder.build_unconditional_branch(merge).unwrap();
            
            if let (Some(e), Some(else_block)) = (else_branch.as_ref(), else_block.as_ref()) {
                ctx.builder.position_at_end(*else_block);
                codegen_statements(ctx, &e);
                ctx.builder.build_unconditional_branch(merge).unwrap();
            }

            ctx.builder.position_at_end(merge);
            Ok(())
        },

        Statement::While { condition, body } => {
            let func = ctx.builder.get_insert_block().ok_or("No insert block")?.get_parent().ok_or("No parent function")?;
            let cond_block = ctx.context.append_basic_block(func, "whilecond");
            let body_block = ctx.context.append_basic_block(func, "whilebody");
            let merge = ctx.context.append_basic_block(func, "whilecont");

            ctx.builder.build_unconditional_branch(cond_block).unwrap();
            ctx.builder.position_at_end(cond_block);

            let condition_value = codegen_expressions(ctx, condition).into_float_value();
            let zero = ctx.context.f64_type().const_float(0.0);
            let comparison = ctx.builder.build_float_compare(inkwell::FloatPredicate::ONE, condition_value, zero, "whilecond").unwrap();

            ctx.builder.build_conditional_branch(comparison, body_block, merge).unwrap();
            ctx.builder.position_at_end(body_block);
            codegen_statements(ctx, body)?;
            ctx.builder.build_unconditional_branch(cond_block).unwrap();
            ctx.builder.position_at_end(merge);
            Ok(())
        },

        Statement::Function(declaration) => {
            codegen_function(ctx, declaration)?;
            Ok(())
        },

        Statement::Return { value } => {
            let return_value = value.as_ref().map(|v| codegen_expressions(ctx, v))
                .unwrap_or_else(|| ctx.f64_type.const_float(0.0).into());

            ctx.builder.build_return(Some(&return_value));
            Ok(())
        },

        // For now only works for built-in libraries. Third party libraries will not be supported
        // until the package manager is fully settled and finished.
        Statement::Include(path) => {
            println!("Including module: {}", path);

            let parts: Vec<&str> = path.split('.').collect();
            let alias = parts.last().unwrap().to_string();
            let relative_path = path.strip_prefix("adan.").unwrap_or(path);
            let file_path = Path::new("src").join(relative_path.replace('.', "/")).with_extension("rs");
            println!("Resolved include path: {}", file_path.display());

            let contents = std::fs::read_to_string(&file_path)
                .map_err(|_| format!("Include file not found: {}", file_path.display()))?;

            let mut lexer = crate::lexer::lexer::Lexer::new(&contents);
            let tokens = lexer.tokenize()?;
            let mut parser = crate::parser::parser::Parser::new(tokens);
            let included_stmts = parser.parse()?;

            let mut module_val = ModuleValue {
                functions: HashMap::new(),
                variables: HashMap::new(),
            };

            for stmt in included_stmts {
                match &stmt {
                    Statement::Function(func) => {
                        println!("Included function: {}", func.name);
                        module_val.functions.insert(func.name.clone(), func.clone());
                    }
                    Statement::VarDecl { name, .. } => {
                        println!("Included variable: {}", name);
                        let ptr = ctx.builder.build_alloca(ctx.f64_type, &name)
                            .map_err(|e| format!("Failed to build alloca: {:?}", e))?;
                        module_val.variables.insert(name.clone(), ptr);
                    }
                    _ => {}
                }
            }

            ctx.modules.insert(alias.clone(), module_val);
            println!("Module '{}' included successfully", alias);

            Ok(())
        }

        _ => unimplemented!(),
    }
}
