use std::collections::HashMap;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, FloatValue, IntValue, FunctionValue, PointerValue},
    types::{FloatType, IntType},
    AddressSpace,
};

pub struct CodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    // Cached types, helps us avoid calling something like `context.f64_type()` over and over.
    // Add onto this over time, if necessary.
    pub f64_type: FloatType<'ctx>,
    pub bool_type: IntType<'ctx>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGenContext<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();

        Self {
            context,
            builder,
            module,

            // Refer to line 15 for explanations on what these're used for.
            f64_type: context.f64_type(),
            bool_type: context.bool_type(),
            variables: HashMap::new(),
        }
    }

    pub fn build_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let entry = function.get_first_basic_block().unwrap();
        let builder = self.context.create_builder();

        builder.position_at_end(entry);
        builder.build_alloca(self.f64_type, name).expect("build allocation failed").into()
    }

    pub fn build_return(&self, value: Option<BasicValueEnum<'ctx>>) {
        match value {
            Some(v) => self.builder.build_return(Some(&v)),
            None => self.builder.build_return(None),
        };
    }
}
