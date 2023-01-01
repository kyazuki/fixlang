// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use std::{cell::RefCell, rc::Rc};

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    intrinsics::Intrinsic,
    targets::{TargetData, TargetMachine},
    types::AnyType,
    values::{BasicMetadataValueEnum, CallSiteValue, StructValue},
};

use super::*;

#[derive(Clone)]
pub struct Variable<'c> {
    pub ptr: VarValue<'c>,
    used_later: u32,
}

#[derive(Clone)]
pub enum VarValue<'c> {
    Local(Object<'c>),
    Global(FunctionValue<'c>, Arc<TypeNode>),
}

#[derive(Clone)]
pub struct Object<'c> {
    pub ptr: PointerValue<'c>,
    pub ty: Arc<TypeNode>,
}

impl<'c> Object<'c> {
    pub fn new(ptr: PointerValue<'c>, ty: Arc<TypeNode>) -> Self {
        assert!(ty.free_vars().is_empty());
        Object { ptr, ty }
    }

    // If boxed type, then create Object from pointer.
    // If unboxed type, then store the value to stack and create Object.
    pub fn from_basic_value_enum<'m>(
        val: BasicValueEnum<'c>,
        ty: Arc<TypeNode>,
        gc: &GenerationContext<'c, 'm>,
    ) -> Object<'c> {
        let ptr = if ty.is_box(gc.type_env()) {
            val.into_pointer_value()
        } else {
            let str = ty.get_struct_type(gc, &vec![]);
            let ptr = gc.builder().build_alloca(str, "alloca_for_unboxed_obj");
            gc.builder().build_store(ptr, val);
            ptr
        };
        Object::new(ptr, ty)
    }

    pub fn is_unbox(&self, type_env: &TypeEnv) -> bool {
        self.ty.is_unbox(type_env)
    }

    pub fn is_box(&self, type_env: &TypeEnv) -> bool {
        self.ty.is_box(type_env)
    }

    pub fn struct_ty<'m>(&self, gc: &GenerationContext<'c, 'm>) -> StructType<'c> {
        get_object_type(&self.ty, &vec![], gc.type_env()).to_struct_type(gc)
    }

    pub fn load_nocap<'m>(&self, gc: &GenerationContext<'c, 'm>) -> StructValue<'c> {
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder()
            .build_load(ptr, "load_unbox")
            .into_struct_value()
    }

    pub fn ptr_to_field_nocap<'m>(
        &self,
        gc: &GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> PointerValue<'c> {
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder()
            .build_struct_gep(ptr, field_idx, "ptr_to_field_nocap")
            .unwrap()
    }

    pub fn load_field_nocap<'m>(
        &self,
        gc: &GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> BasicValueEnum<'c> {
        let struct_ty = self.struct_ty(gc);
        gc.load_obj_field(self.ptr, struct_ty, field_idx)
    }

    pub fn store_field_nocap<'m, V>(&self, gc: &GenerationContext<'c, 'm>, field_idx: u32, val: V)
    where
        V: BasicValue<'c>,
    {
        let struct_ty = self.struct_ty(gc);
        gc.store_obj_field(self.ptr, struct_ty, field_idx, val)
    }

    // Get function pointer to destructor.
    pub fn get_dtor_ptr_boxed<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
        assert!(self.is_box(gc.type_env()));
        if self.ty.is_function() {
            self.load_field_nocap(gc, DTOR_IDX).into_pointer_value()
        } else {
            get_dtor_ptr(&self.ty, &vec![], gc)
        }
    }

    // Get dtor function.
    pub fn get_dtor_unboxed<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> Option<FunctionValue<'c>> {
        assert!(self.is_unbox(gc.type_env()));
        create_dtor(&self.ty, &vec![], gc)
    }
}

// Additional implementation of Object with another set of life parameters.
impl<'s, 'c: 's, 'm> Object<'c> {}

impl<'c> VarValue<'c> {
    // Get pointer.
    pub fn get<'m>(&self, gc: &GenerationContext<'c, 'm>) -> Object<'c> {
        match self {
            VarValue::Local(ptr) => ptr.clone(),
            VarValue::Global(fun, ty) => {
                let ptr = gc
                    .builder()
                    .build_call(fun.clone(), &[], "get_ptr")
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_pointer_value();
                Object {
                    ptr,
                    ty: ty.clone(),
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Scope<'c> {
    data: HashMap<NameSpacedName, Vec<Variable<'c>>>,
}

impl<'c> Scope<'c> {
    fn push_local(self: &mut Self, var: &NameSpacedName, obj: &Object<'c>) {
        // TODO: add assertion that var is local (or change var to Name).
        if !self.data.contains_key(var) {
            self.data.insert(var.clone(), Default::default());
        }
        self.data.get_mut(var).unwrap().push(Variable {
            ptr: VarValue::Local(obj.clone()),
            used_later: 0,
        });
    }

    fn pop_local(&mut self, var: &NameSpacedName) {
        // TODO: add assertion that var is local (or change var to Name).
        self.data.get_mut(var).unwrap().pop();
        if self.data.get(var).unwrap().is_empty() {
            self.data.remove(var);
        }
    }

    pub fn get(self: &Self, var: &NameSpacedName) -> Variable<'c> {
        self.data.get(var).unwrap().last().unwrap().clone()
    }

    fn modify_used_later(self: &mut Self, vars: &HashSet<NameSpacedName>, by: i32) {
        for var in vars {
            if !var.is_local() {
                continue;
            }
            let used_later = &mut self
                .data
                .get_mut(var)
                .unwrap()
                .last_mut()
                .unwrap()
                .used_later;
            *used_later = add_i32_to_u32(*used_later, by);
        }
    }
    fn increment_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.modify_used_later(names, 1);
    }
    fn decrement_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.modify_used_later(names, -1);
    }
}

fn add_i32_to_u32(u: u32, i: i32) -> u32 {
    if i.is_negative() {
        u - i.wrapping_abs() as u32
    } else {
        u + i as u32
    }
}

pub struct GenerationContext<'c, 'm> {
    pub context: &'c Context,
    pub module: &'m Module<'c>,
    builders: Rc<RefCell<Vec<Rc<Builder<'c>>>>>,
    scope: Rc<RefCell<Vec<Scope<'c>>>>,
    global: HashMap<NameSpacedName, Variable<'c>>,
    pub runtimes: HashMap<RuntimeFunctions, FunctionValue<'c>>,
    pub typechecker: Option<TypeCheckContext>,
    pub target: Either<TargetMachine, ExecutionEngine<'c>>,
    target_data_cache: Option<TargetData>,
}

pub struct PopBuilderGuard<'c> {
    builders: Rc<RefCell<Vec<Rc<Builder<'c>>>>>,
}

impl<'c> Drop for PopBuilderGuard<'c> {
    fn drop(&mut self) {
        self.builders.borrow_mut().pop().unwrap();
    }
}

pub struct PopScopeGuard<'c> {
    scope: Rc<RefCell<Vec<Scope<'c>>>>,
}

impl<'c> Drop for PopScopeGuard<'c> {
    fn drop(&mut self) {
        self.scope.borrow_mut().pop();
    }
}

impl<'c, 'm> GenerationContext<'c, 'm> {
    // Store stack pointer.
    pub fn save_stack(&mut self) -> PointerValue<'c> {
        let intrinsic = Intrinsic::find("llvm.stacksave").unwrap();
        let func = intrinsic.get_declaration(&self.module, &[]).unwrap();
        self.builder()
            .build_call(func, &[], "save_stack")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }

    // Restore stack pointer.
    pub fn restore_stack(&mut self, pos: PointerValue<'c>) {
        let intrinsic = Intrinsic::find("llvm.stackrestore").unwrap();
        let func = intrinsic
            .get_declaration(
                &self.module,
                &[self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::Generic)
                    .into()],
            )
            .unwrap();
        self.builder()
            .build_call(func, &[pos.into()], "restore_stack");
    }

    pub fn type_env(&self) -> &TypeEnv {
        &self.typechecker.as_ref().unwrap().type_env
    }

    pub fn target_data(&mut self) -> &TargetData {
        if self.target_data_cache.is_some() {
            return self.target_data_cache.as_ref().unwrap();
        }
        match &self.target {
            Either::Left(tm) => {
                self.target_data_cache = Some(tm.get_target_data());
                self.target_data_cache.as_ref().unwrap()
            }
            Either::Right(ee) => ee.get_target_data(),
        }
    }

    pub fn sizeof(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data().get_store_size(ty)
    }

    // Build malloc.
    pub fn malloc(&mut self, ty: StructType<'c>) -> PointerValue<'c> {
        if USE_LEAKY_ALLOCATOR {
            let ptr_to_heap = self
                .module
                .get_global(LEAKY_HEAP_NAME)
                .unwrap()
                .as_basic_value_enum()
                .into_pointer_value();
            let heap = self
                .builder()
                .build_load(ptr_to_heap, "heap")
                .into_pointer_value();
            let heap_ty = heap.get_type();
            {
                let ptr_int = self.context.ptr_sized_int_type(self.target_data(), None);
                let heap = self.builder().build_ptr_to_int(heap, ptr_int, "heap_int");
                let offset = ty.size_of().unwrap();
                let new_heap = self.builder().build_int_add(heap, offset, "hew_heap_int");
                let new_heap = self
                    .builder()
                    .build_int_to_ptr(new_heap, heap_ty, "new_heap");
                self.builder().build_store(ptr_to_heap, new_heap);
            }
            self.cast_pointer(heap, ptr_type(ty))
        } else {
            self.builder().build_malloc(ty, "malloc").unwrap()
        }
    }

    // Build free.
    pub fn free(&self, ptr: PointerValue<'c>) {
        if USE_LEAKY_ALLOCATOR {
        } else {
            self.builder().build_free(ptr);
        }
    }

    // Create new gc.
    pub fn new(
        ctx: &'c Context,
        module: &'m Module<'c>,
        target: Either<TargetMachine, ExecutionEngine<'c>>,
    ) -> Self {
        let ret = Self {
            context: ctx,
            module,
            builders: Rc::new(RefCell::new(vec![Rc::new(ctx.create_builder())])),
            scope: Rc::new(RefCell::new(vec![Default::default()])),
            global: Default::default(),
            runtimes: Default::default(),
            typechecker: None,
            target,
            target_data_cache: None,
        };
        ret
    }

    // Get builder.
    pub fn builder(&self) -> Rc<Builder<'c>> {
        self.builders.borrow().last().unwrap().clone()
    }

    // Push a new builder.
    pub fn push_builder(&mut self) -> PopBuilderGuard<'c> {
        self.builders
            .borrow_mut()
            .push(Rc::new(self.context.create_builder()));
        PopBuilderGuard {
            builders: self.builders.clone(),
        }
    }

    // Add a global object.
    pub fn add_global_object(
        &mut self,
        name: NameSpacedName,
        accessor: FunctionValue<'c>,
        ty: Arc<TypeNode>,
    ) {
        if self.global.contains_key(&name) {
            error_exit(&format!("duplicate symbol: {}", name.to_string()));
        } else {
            self.global.insert(
                name.clone(),
                Variable {
                    ptr: VarValue::Global(accessor, ty),
                    used_later: u32::MAX / 2,
                },
            );
        }
    }

    // Push a new scope.
    pub fn push_scope(&mut self) -> PopScopeGuard<'c> {
        self.scope.borrow_mut().push(Default::default());
        PopScopeGuard {
            scope: self.scope.clone(),
        }
    }

    // Get the value of a variable.
    pub fn get_var(&self, var: &NameSpacedName) -> Variable<'c> {
        if var.is_local() {
            self.scope.borrow().last().unwrap().get(var)
        } else {
            self.global.get(var).unwrap().clone()
        }
    }

    // Lock variables in scope to avoid being moved out.
    fn scope_lock_as_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .increment_used_later(names);
    }

    // Unlock variables in scope.
    fn scope_unlock_as_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .decrement_used_later(names);
    }

    // Get field of object in the scope.
    pub fn get_var_field(self: &Self, var: &NameSpacedName, field_idx: u32) -> BasicValueEnum<'c> {
        let obj = self.get_var(var).ptr.get(self);
        obj.load_field_nocap(self, field_idx)
    }

    // Push scope.
    fn scope_push(self: &mut Self, var: &NameSpacedName, obj: &Object<'c>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .push_local(var, obj)
    }

    // Pop scope.
    fn scope_pop(self: &mut Self, var: &NameSpacedName) {
        self.scope.borrow_mut().last_mut().unwrap().pop_local(var);
    }

    pub fn get_var_retained_if_used_later(&mut self, var_name: &NameSpacedName) -> Object<'c> {
        let var = self.get_var(var_name);
        let mut obj = var.ptr.get(self);
        if var.used_later > 0 {
            self.retain(obj.clone());
            if obj.is_unbox(self.type_env()) {
                obj = Object::from_basic_value_enum(
                    obj.load_nocap(self).as_basic_value_enum(),
                    obj.ty,
                    self,
                );
            }
        }
        obj
    }

    pub fn cast_pointer(&self, from: PointerValue<'c>, to: PointerType<'c>) -> PointerValue<'c> {
        if from.get_type() == to {
            from
        } else {
            self.builder().build_pointer_cast(from, to, "pointer_cast")
        }
    }

    // Get pointer to control block of a given object.
    pub fn get_control_block_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        self.cast_pointer(obj, ptr_to_control_block_type(self.context))
    }

    // Get pointer to reference counter of a given object.
    pub fn get_refcnt_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        let ptr_control_block = self.get_control_block_ptr(obj);
        self.builder()
            .build_struct_gep(ptr_control_block, 0, "ptr_to_refcnt")
            .unwrap()
    }

    // Take an pointer of struct and return the loaded value of a pointer field.
    pub fn load_obj_field(
        &self,
        obj: PointerValue<'c>,
        ty: StructType<'c>,
        index: u32,
    ) -> BasicValueEnum<'c> {
        let ptr = self.cast_pointer(obj, ptr_type(ty));
        let ptr_to_field = self
            .builder()
            .build_struct_gep(ptr, index, "ptr_to_field")
            .unwrap();
        self.builder().build_load(ptr_to_field, "field_value")
    }

    // Take an pointer of struct and store a value value into a pointer field.
    pub fn store_obj_field<V>(
        &self,
        obj: PointerValue<'c>,
        ty: StructType<'c>,
        index: u32,
        value: V,
    ) where
        V: BasicValue<'c>,
    {
        let ptr = self.cast_pointer(obj, ptr_type(ty));
        let ptr_to_field = self
            .builder()
            .build_struct_gep(ptr, index, "ptr_to_field")
            .unwrap();
        self.builder().build_store(ptr_to_field, value);
    }

    // Take a closure object and return function pointer.
    fn get_lambda_func_ptr(&self, obj: Object<'c>) -> PointerValue<'c> {
        obj.load_field_nocap(self, LAMBDA_FUNCTION_IDX)
            .into_pointer_value()
    }

    // Apply a object to a lambda.
    pub fn apply_lambda(&self, lambda: Object<'c>, arg: Object<'c>) -> Object<'c> {
        // If argument is unboxed, load it.
        let arg = if arg.ty.is_unbox(self.type_env()) {
            arg.load_nocap(self).as_basic_value_enum()
        } else {
            arg.ptr.as_basic_value_enum()
        };

        // Call function.
        let ptr_to_func = self.get_lambda_func_ptr(lambda.clone());
        let lambda_func = CallableValue::try_from(ptr_to_func).unwrap();
        let ret =
            self.builder()
                .build_call(lambda_func, &[arg.into(), lambda.ptr.into()], "call_lambda");
        ret.set_tail_call(true);
        let ret = ret.try_as_basic_value().unwrap_left();

        let dst_ty = lambda.ty.get_funty_dst();
        Object::from_basic_value_enum(ret, dst_ty, self)
    }

    // Retain object.
    pub fn retain(&mut self, obj: Object<'c>) {
        if obj.is_box(self.type_env()) {
            let ptr = self.cast_pointer(obj.ptr, ptr_to_object_type(self.context));
            self.call_runtime(RuntimeFunctions::RetainBoxedObject, &[ptr.into()]);
        } else {
            let obj_type = get_object_type(&obj.ty, &vec![], self.type_env());
            let struct_type = obj_type.to_struct_type(self);
            let ptr = self.cast_pointer(obj.ptr, ptr_type(struct_type));
            let mut union_tag: Option<IntValue<'c>> = None;
            for (i, ft) in obj_type.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::ControlBlock => unreachable!(),
                    ObjectFieldType::DtorFunction => unreachable!(),
                    ObjectFieldType::LambdaFunction(_) => unreachable!(),
                    ObjectFieldType::I64 => {}
                    ObjectFieldType::Bool => {}
                    ObjectFieldType::SubObject(ty) => {
                        let ptr = if ty.is_box(self.type_env()) {
                            self.load_obj_field(ptr, struct_type, i as u32)
                                .into_pointer_value()
                        } else {
                            self.builder()
                                .build_struct_gep(ptr, i as u32, &format!("ptr_to_{}nd_field", i))
                                .unwrap()
                        };
                        self.retain(Object::new(ptr, ty.clone()));
                    }
                    ObjectFieldType::UnionBuf => {
                        let buf = self
                            .load_obj_field(ptr, struct_type, i as u32)
                            .into_pointer_value();
                        ObjectFieldType::retain_union_buf(
                            self,
                            buf,
                            union_tag.unwrap(),
                            &obj.ty.fields_types(self.type_env()),
                        );
                    }
                    ObjectFieldType::UnionTag => {
                        union_tag = Some(
                            self.load_obj_field(ptr, struct_type, i as u32)
                                .into_int_value(),
                        );
                    }
                    ObjectFieldType::ArraySizeBuf(_) => unreachable!(),
                }
            }
        }
    }

    // Release object.
    pub fn release(&mut self, obj: Object<'c>) {
        if obj.is_box(self.type_env()) {
            let ptr = self.cast_pointer(obj.ptr, ptr_to_object_type(self.context));
            let dtor = obj.get_dtor_ptr_boxed(self);
            self.call_runtime(
                RuntimeFunctions::ReleaseBoxedObject,
                &[ptr.into(), dtor.into()],
            );
        } else {
            match obj.get_dtor_unboxed(self) {
                Some(dtor) => {
                    self.builder()
                        .build_call(dtor, &[obj.ptr.into()], "dtor_of_unboxed");
                }
                None => {}
            }
        }
    }

    // Printf Rust's &str.
    pub fn printf(&self, string: &str) {
        let string_ptr = self.builder().build_global_string_ptr(string, "rust_str");
        let string_ptr = string_ptr.as_pointer_value();
        self.call_runtime(RuntimeFunctions::Printf, &[string_ptr.into()]);
    }

    // Panic with Rust's &str (i.e, print string and abort.)
    pub fn panic(&self, string: &str) {
        self.printf(string);
        self.call_runtime(RuntimeFunctions::Abort, &[]);
    }

    // Get object id of a object
    pub fn get_obj_id(&self, ptr_to_obj: PointerValue<'c>) -> IntValue<'c> {
        assert!(SANITIZE_MEMORY);
        self.load_obj_field(ptr_to_obj, control_block_type(self.context), 1)
            .into_int_value()
    }

    // Call a runtime function.
    pub fn call_runtime(
        &self,
        func: RuntimeFunctions,
        args: &[BasicMetadataValueEnum<'c>],
    ) -> CallSiteValue<'c> {
        self.builder()
            .build_call(*self.runtimes.get(&func).unwrap(), args, "call_runtime")
    }

    // Evaluate expression.
    pub fn eval_expr(&mut self, expr: Arc<ExprNode>) -> Object<'c> {
        let expr = expr.set_inferred_type(
            self.typechecker
                .as_ref()
                .unwrap()
                .substitute_type(&expr.inferred_ty.clone().unwrap()),
        );
        let mut ret = match &*expr.expr {
            Expr::Var(var) => self.eval_var(var.clone()),
            Expr::Lit(lit) => self.eval_lit(lit.clone(), expr.inferred_ty.clone().unwrap().clone()),
            Expr::App(lambda, arg) => self.eval_app(lambda.clone(), arg.clone()),
            Expr::Lam(arg, val) => {
                self.eval_lam(arg.clone(), val.clone(), expr.inferred_ty.clone().unwrap())
            }
            Expr::Let(var, bound, expr) => self.eval_let(var.clone(), bound.clone(), expr.clone()),
            Expr::If(cond_expr, then_expr, else_expr) => {
                self.eval_if(cond_expr.clone(), then_expr.clone(), else_expr.clone())
            }
            Expr::TyAnno(e, _) => self.eval_expr(e.clone()),
        };
        ret.ty = expr.inferred_ty.clone().unwrap();
        ret
    }

    // Evaluate variable.
    fn eval_var(&mut self, var: Arc<Var>) -> Object<'c> {
        self.get_var_retained_if_used_later(&var.name)
    }

    // Evaluate application
    fn eval_app(&mut self, lambda: Arc<ExprNode>, arg: Arc<ExprNode>) -> Object<'c> {
        self.scope_lock_as_used_later(arg.free_vars());
        let lambda_code = self.eval_expr(lambda);
        self.scope_unlock_as_used_later(arg.free_vars());
        let arg_code = self.eval_expr(arg);
        self.apply_lambda(lambda_code, arg_code)
    }

    // Evaluate literal
    fn eval_lit(&mut self, lit: Arc<Literal>, ty: Arc<TypeNode>) -> Object<'c> {
        (lit.generator)(self, &ty)
    }

    // Evaluate lambda abstraction.
    fn eval_lam(&mut self, arg: Arc<Var>, val: Arc<ExprNode>, lam_ty: Arc<TypeNode>) -> Object<'c> {
        let context = self.context;
        let module = self.module;

        // Calculate captured variables.
        let mut captured_names = val.free_vars().clone();
        captured_names.remove(&arg.name);
        captured_names.remove(&NameSpacedName::local(SELF_NAME));
        // We need not and should not capture global variable
        // If we capture global variable, then global recursive function such as
        // "main = \x -> if x == 0 then 0 else x + main (x-1)" results in infinite recursion at it's initialization.
        let captured_vars = captured_names
            .into_iter()
            .filter(|name| name.is_local())
            .map(|name| (name.clone(), self.get_var(&name).ptr.get(self).ty))
            .collect::<Vec<_>>();
        let captured_types = captured_vars
            .iter()
            .map(|(_name, ty)| ty.clone())
            .collect::<Vec<_>>();

        // Determine the type of closure
        let obj_type = lam_ty.get_object_type(&captured_types, self.type_env());
        let closure_ty = obj_type.to_struct_type(self);

        // Declare lambda function
        let lam_fn_ty = lambda_function_type(&lam_ty, self);
        let lam_fn = module.add_function(
            &format!("lambda_{}", lam_ty.to_string_normalize()),
            lam_fn_ty,
            None,
        );

        // Implement lambda function
        {
            // Create new builder and set up
            let _builder_guard = self.push_builder();
            let bb = context.append_basic_block(lam_fn, "entry");
            self.builder().position_at_end(bb);

            // Create new scope
            let _scope_guard = self.push_scope();

            // Set up new scope
            // Push argment on scope.
            let arg_val = lam_fn.get_first_param().unwrap();
            let arg_ty = lam_ty.get_funty_src();
            let arg_obj = Object::from_basic_value_enum(arg_val, arg_ty, self);
            self.scope_push(&arg.name, &arg_obj);

            // Push SELF on scope.
            let closure_ptr = lam_fn.get_nth_param(1).unwrap().into_pointer_value();
            let closure_obj = Object::new(closure_ptr, lam_ty.clone());
            self.scope_push(&NameSpacedName::local(SELF_NAME), &closure_obj);

            // Push captured variables on scope.
            for (i, (cap_name, cap_ty)) in captured_vars.iter().enumerate() {
                let cap_val =
                    self.load_obj_field(closure_ptr, closure_ty, i as u32 + CAPTURED_OBJECT_IDX);
                let cap_obj = Object::from_basic_value_enum(cap_val, cap_ty.clone(), self);
                self.retain(cap_obj.clone());
                self.scope_push(cap_name, &cap_obj);
            }

            // Release SELF and arg if unused
            if !val.free_vars().contains(&NameSpacedName::local(SELF_NAME)) {
                self.release(closure_obj);
            }
            if !val.free_vars().contains(&arg.name) {
                self.release(arg_obj);
            }

            // Calculate body.
            let val = self.eval_expr(val.clone());

            // Return result
            let retval = if val.is_box(self.type_env()) {
                val.load_nocap(self).as_basic_value_enum()
            } else {
                self.cast_pointer(val.ptr, ptr_to_object_type(self.context))
                    .as_basic_value_enum()
            };
            self.builder().build_return(Some(&retval));
        }
        // Allocate and set up closure
        let name = expr_abs(arg, val, None).expr.to_string();
        let obj = allocate_obj(lam_ty, &captured_types, self, Some(name.as_str()));
        self.store_obj_field(
            obj.ptr,
            closure_ty,
            LAMBDA_FUNCTION_IDX,
            lam_fn.as_global_value().as_pointer_value(),
        );
        for (i, (cap_name, _cap_ty)) in captured_vars.iter().enumerate() {
            let cap_obj = self.get_var_retained_if_used_later(cap_name);
            let cap_val = if cap_obj.is_box(self.type_env()) {
                cap_obj.ptr.as_basic_value_enum()
            } else {
                self.builder()
                    .build_load(cap_obj.ptr, "load_form_store_to_lambda")
            };
            self.store_obj_field(obj.ptr, closure_ty, i as u32 + CAPTURED_OBJECT_IDX, cap_val);
        }

        // Return closure object
        obj
    }

    // Evaluate let
    fn eval_let(&mut self, var: Arc<Var>, bound: Arc<ExprNode>, val: Arc<ExprNode>) -> Object<'c> {
        let var_name = &var.name;
        let mut used_in_val_except_var = val.free_vars().clone();
        used_in_val_except_var.remove(var_name);
        self.scope_lock_as_used_later(&used_in_val_except_var);
        let bound_code = self.eval_expr(bound.clone());
        self.scope_unlock_as_used_later(&used_in_val_except_var);
        self.scope_push(&var_name, &bound_code);
        if !val.free_vars().contains(var_name) {
            self.release(bound_code);
        }
        let val_code = self.eval_expr(val.clone());
        self.scope_pop(&var_name);
        val_code
    }

    // Evaluate if
    fn eval_if(
        &mut self,
        cond_expr: Arc<ExprNode>,
        then_expr: Arc<ExprNode>,
        else_expr: Arc<ExprNode>,
    ) -> Object<'c> {
        let mut used_then_or_else = then_expr.free_vars().clone();
        used_then_or_else.extend(else_expr.free_vars().clone());
        self.scope_lock_as_used_later(&used_then_or_else);
        let ptr_to_cond_obj = self.eval_expr(cond_expr);
        self.scope_unlock_as_used_later(&used_then_or_else);
        let cond_val = ptr_to_cond_obj.load_field_nocap(self, 0).into_int_value();
        self.release(ptr_to_cond_obj);
        let cond_val =
            self.builder()
                .build_int_cast(cond_val, self.context.bool_type(), "cond_val_i1");
        let bb = self.builder().get_insert_block().unwrap();
        let func = bb.get_parent().unwrap();
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let cont_bb = self.context.append_basic_block(func, "cont");
        self.builder()
            .build_conditional_branch(cond_val, then_bb, else_bb);

        self.builder().position_at_end(then_bb);
        // Release variables used only in the else block.
        for var_name in else_expr.free_vars() {
            if !then_expr.free_vars().contains(var_name) && self.get_var(var_name).used_later == 0 {
                self.release(self.get_var(var_name).ptr.get(self));
            }
        }
        let then_val = self.eval_expr(then_expr.clone());
        let then_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(else_bb);
        // Release variables used only in the then block.
        for var_name in then_expr.free_vars() {
            if !else_expr.free_vars().contains(var_name) && self.get_var(var_name).used_later == 0 {
                self.release(self.get_var(var_name).ptr.get(self));
            }
        }
        let else_val = self.eval_expr(else_expr);
        let else_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(cont_bb);
        let phi = self
            .builder()
            .build_phi(ptr_to_object_type(self.context), "phi");
        phi.add_incoming(&[(&then_val.ptr, then_bb), (&else_val.ptr, else_bb)]);
        Object::new(
            phi.as_basic_value().into_pointer_value(),
            then_expr.inferred_ty.clone().unwrap(),
        )
    }
}

pub fn ptr_type<'c>(ty: StructType<'c>) -> PointerType<'c> {
    ty.ptr_type(AddressSpace::Generic)
}

pub static SELF_NAME: &str = "%SELF%";
