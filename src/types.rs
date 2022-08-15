use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ObjectFieldType {
    ControlBlock,
    LambdaFunction,
    SubObject,
    Int,
    Bool,
    Array,
}

impl ObjectFieldType {
    pub fn to_basic_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(context).into(),
            ObjectFieldType::LambdaFunction => ptr_to_lambda_function_type(context).into(),
            ObjectFieldType::SubObject => ptr_to_object_type(context).into(),
            ObjectFieldType::Int => context.i64_type().into(),
            ObjectFieldType::Bool => context.i8_type().into(),
            ObjectFieldType::Array => context
                .struct_type(
                    &[
                        context.i64_type().into(),          // size
                        ptr_to_object_type(context).into(), // ptr to buffer
                    ],
                    false,
                )
                .into(),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct ObjectType {
    pub field_types: Vec<ObjectFieldType>,
}

impl ObjectType {
    pub fn to_struct_type<'ctx>(&self, context: &'ctx Context) -> StructType<'ctx> {
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![];
        for field_type in &self.field_types {
            fields.push(field_type.to_basic_type(context));
        }
        context.struct_type(&fields, false)
    }

    fn shared_obj_type(mut field_types: Vec<ObjectFieldType>) -> Self {
        let mut fields = vec![ObjectFieldType::ControlBlock];
        fields.append(&mut field_types);
        Self {
            field_types: fields,
        }
    }

    pub fn int_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Int])
    }

    pub fn bool_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Bool])
    }

    pub fn lam_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::LambdaFunction]) // Other fields for captured objects may exist but omitted here.
    }

    pub fn array_type() -> Self {
        let fields = vec![ObjectFieldType::Array];
        Self::shared_obj_type(fields)
    }

    fn generate_func_dtor<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
        if gc
            .runtimes
            .contains_key(&RuntimeFunctions::Dtor(self.clone()))
        {
            return *gc
                .runtimes
                .get(&RuntimeFunctions::Dtor(self.clone()))
                .unwrap();
        }
        let struct_type = self.to_struct_type(gc.context);
        let func_type = dtor_type(gc.context);
        let func = gc.module.add_function("dtor", func_type, None);
        let bb = gc.context.append_basic_block(func, "entry");

        let _builder_guard = gc.push_builder();

        let context = gc.context;
        let module = gc.module;

        gc.builder().position_at_end(bb);
        let ptr_to_obj = func.get_first_param().unwrap().into_pointer_value();
        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::SubObject => {
                    let ptr_to_subobj = gc
                        .load_obj_field(ptr_to_obj, struct_type, i as u32)
                        .into_pointer_value();
                    gc.release(ptr_to_subobj);
                }
                ObjectFieldType::ControlBlock => {}
                ObjectFieldType::Int => {}
                ObjectFieldType::LambdaFunction => {}
                ObjectFieldType::Bool => {}
                ObjectFieldType::Array => {
                    let ptr_to_struct = gc.cast_pointer(ptr_to_obj, ptr_type(struct_type));
                    let ptr_to_array = gc
                        .builder()
                        .build_struct_gep(ptr_to_struct, i as u32, "ptr_to_array")
                        .unwrap();
                    Self::destruct_array(gc, ptr_to_array);
                }
            }
        }
        gc.builder().build_return(None);

        // gc.pop_builder();
        gc.runtimes
            .insert(RuntimeFunctions::Dtor(self.clone()), func);
        func
    }

    // Take pointer to array = [size, ptr_to_buffer], call release of ptr_to_bufer[i] for all i and free ptr_to_buffer.
    fn destruct_array<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, ptr_to_array: PointerValue<'c>) {
        // Get fields (size, ptr_to_buffer).
        let array_struct = ObjectFieldType::Array
            .to_basic_type(gc.context)
            .into_struct_type();
        let size = gc
            .load_obj_field(ptr_to_array, array_struct, 0)
            .into_int_value();
        let ptr_to_buffer = gc
            .load_obj_field(ptr_to_array, array_struct, 1)
            .into_pointer_value();

        // Append blocks: loop_check, loop_body and after_loop.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let dtor_func = current_bb.get_parent().unwrap();
        let loop_check_bb = gc
            .context
            .append_basic_block(dtor_func, "loop_release_array_elements");
        let loop_body_bb = gc.context.append_basic_block(dtor_func, "loop_body");
        let after_loop_bb = gc.context.append_basic_block(dtor_func, "after_loop");

        // Allocate and initialize loop counter.
        let counter_type = gc.context.i64_type();
        let counter_ptr = gc
            .builder()
            .build_alloca(counter_type, "release_loop_counter");
        gc.builder()
            .build_store(counter_ptr, counter_type.const_zero());

        // Jump to loop_check bb.
        gc.builder().build_unconditional_branch(loop_check_bb);

        // Implement loop_check bb.
        gc.builder().position_at_end(loop_check_bb);
        let counter_val = gc
            .builder()
            .build_load(counter_ptr, "counter_val")
            .into_int_value();
        let is_end = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, counter_val, size, "is_end");
        gc.builder()
            .build_conditional_branch(is_end, after_loop_bb, loop_body_bb);

        // Implement loop_body bb.
        gc.builder().position_at_end(loop_body_bb);
        {
            // Release object of idx = counter_val
            let obj_ptr = unsafe {
                gc.builder()
                    .build_gep(ptr_to_buffer, &[counter_val.into()], "elem_of_array")
            };
            gc.release(obj_ptr);

            // Increment counter.
            let incremented_counter_val = gc.builder().build_int_add(
                counter_val,
                counter_type.const_int(1, false),
                "incremented_counter_val",
            );
            gc.builder()
                .build_store(counter_ptr, incremented_counter_val);

            // Jump back to loop_check bb.
            gc.builder().build_unconditional_branch(loop_check_bb);
        }

        // Free buffer.
        gc.builder().position_at_end(after_loop_bb);
        gc.builder().build_free(ptr_to_buffer);
    }

    // Create an object
    pub fn create_obj<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        name: Option<&str>,
    ) -> PointerValue<'c> {
        let context = gc.context;
        let struct_type = self.to_struct_type(context);
        // NOTE: Only once allocation is needed since we don't implement weak_ptr
        let ptr_to_obj = gc
            .builder()
            .build_malloc(struct_type, "ptr_to_obj")
            .unwrap();

        let mut object_id = obj_id_type(gc.context).const_int(0, false);

        if SANITIZE_MEMORY {
            let string_ptr = name.unwrap_or("N/A");
            let string_ptr = gc
                .builder()
                .build_global_string_ptr(string_ptr, "name_of_obj");
            let string_ptr = string_ptr.as_pointer_value();
            let string_ptr = gc.builder().build_pointer_cast(
                string_ptr,
                gc.context.i8_type().ptr_type(AddressSpace::Generic),
                "name_of_obj_i8ptr",
            );
            let ptr = gc.cast_pointer(ptr_to_obj, ptr_to_object_type(gc.context));
            let obj_id = gc.call_runtime(
                RuntimeFunctions::ReportMalloc,
                &[ptr.into(), string_ptr.into()],
            );
            object_id = obj_id.try_as_basic_value().unwrap_left().into_int_value();
        }

        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::ControlBlock => {
                    let ptr_to_control_block = gc
                        .builder()
                        .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_control_block")
                        .unwrap();
                    let ptr_to_refcnt = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
                        .unwrap();
                    // The initial value of refcnt should be one (as std::make_shared of C++ does).
                    gc.builder()
                        .build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false));
                    let ptr_to_dtor_field = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 1, "ptr_to_dtor_field")
                        .unwrap();
                    let dtor = self.generate_func_dtor(gc);
                    gc.builder()
                        .build_store(ptr_to_dtor_field, dtor.as_global_value().as_pointer_value());

                    if SANITIZE_MEMORY {
                        let ptr_to_obj_id = gc
                            .builder()
                            .build_struct_gep(ptr_to_control_block, 2, "ptr_to_obj_id")
                            .unwrap();
                        gc.builder().build_store(ptr_to_obj_id, object_id);
                    }
                }
                ObjectFieldType::Int => {}
                ObjectFieldType::SubObject => {}
                ObjectFieldType::LambdaFunction => {}
                ObjectFieldType::Bool => {}
            }
        }
        ptr_to_obj
    }
}

pub fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

fn ptr_to_refcnt_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    refcnt_type(context).ptr_type(AddressSpace::Generic)
}

pub fn obj_id_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

pub fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::Generic)
}

fn dtor_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context
        .void_type()
        .fn_type(&[ptr_to_object_type(context).into()], false)
}

fn ptr_to_dtor_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    dtor_type(context).ptr_type(AddressSpace::Generic)
}

pub fn control_block_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    let mut fields = vec![
        refcnt_type(context).into(),
        ptr_to_dtor_type(context).into(),
    ];
    if SANITIZE_MEMORY {
        fields.push(obj_id_type(context).into())
    }
    context.struct_type(&fields, false)
}

pub fn ptr_to_control_block_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    control_block_type(context).ptr_type(AddressSpace::Generic)
}

pub fn lambda_function_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    // A function that takes argument and context (=lambda object itself).
    ptr_to_object_type(context).fn_type(
        &[
            ptr_to_object_type(context).into(),
            ptr_to_object_type(context).into(),
        ],
        false,
    )
}

fn ptr_to_lambda_function_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    lambda_function_type(context).ptr_type(AddressSpace::Generic)
}

pub fn lambda_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::lam_obj_type().to_struct_type(context)
}

pub fn int_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::int_obj_type().to_struct_type(context)
}

pub fn bool_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::bool_obj_type().to_struct_type(context)
}
