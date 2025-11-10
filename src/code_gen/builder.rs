use std::collections::HashMap;
use std::fs;
use std::path::Path;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, PointerValue},
    types::{FloatType, IntType},
};
use crate::parser::ast::FunctionDecl;

pub struct CodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    pub f64_type: FloatType<'ctx>,
    pub bool_type: IntType<'ctx>,

    pub variables: HashMap<String, PointerValue<'ctx>>,
    pub modules: HashMap<String, ModuleValue<'ctx>>,
}

pub enum NativeFunc<'ctx> {
    AdanFunction(FunctionDecl),
    NativeFn(fn(&mut CodeGenContext<'ctx>, Vec<BasicValueEnum<'ctx>>) -> BasicValueEnum<'ctx>),
}

pub struct ModuleValue<'ctx> {
    pub functions: HashMap<String, NativeFunc<'ctx>>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> ModuleValue<'ctx> {
    pub fn get_function(&self, name: &str) -> Option<&NativeFunc<'ctx>> {
        self.functions.get(name)
    }
}

impl<'ctx> CodeGenContext<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();

        Self {
            context,
            builder,
            module,
            f64_type: context.f64_type(),
            bool_type: context.bool_type(),
            variables: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn register_native_fn(&mut self, module_name: &str, fn_name: &str, func: fn(&mut CodeGenContext<'ctx>, Vec<BasicValueEnum<'ctx>>) -> BasicValueEnum<'ctx>) {
        let module = self.modules.entry(module_name.to_string())
            .or_insert(ModuleValue {
                functions: HashMap::new(),
                variables: HashMap::new(),
            });

        module.functions.insert(fn_name.to_string(), NativeFunc::NativeFn(func));
    }

    pub fn load_native_modules(&mut self, native_dir: &str) {
        let paths = fs::read_dir(native_dir).expect("Failed to read native modules folder");

        for entry in paths {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() && path.extension().map(|s| s == "rs").unwrap_or(false) {
                let module_name = path.file_stem().unwrap().to_string_lossy();

                match module_name.as_ref() {
                    "io" => crate::native::io::register_native(self),
                    _ => {}
                }
            }
        }
    }

    pub fn build_alloca(&self, name: &str) -> Result<PointerValue<'ctx>, inkwell::builder::BuilderError> {
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let entry = function.get_first_basic_block().unwrap();
        let builder = self.context.create_builder();

        builder.position_at_end(entry);
        builder.build_alloca(self.f64_type, name)
    }

    pub fn build_return(&self, value: Option<BasicValueEnum<'ctx>>) {
        match value {
            Some(v) => { let _ = self.builder.build_return(Some(&v)); },
            None => { let _ = self.builder.build_return(None); },
        };
    }
}