// Implement built-in functions, types, etc.
use super::*;

pub const PRELUDE_NAME: &str = "Prelude";

const INT_NAME: &str = "Int";
const BOOL_NAME: &str = "Bool";
const ARRAY_NAME: &str = "Array";

pub fn bulitin_type_to_kind_map() -> HashMap<String, Arc<Kind>> {
    let mut ret = HashMap::new();
    ret.insert(INT_NAME.to_string(), kind_star());
    ret.insert(BOOL_NAME.to_string(), kind_star());
    ret.insert(ARRAY_NAME.to_string(), kind_arrow(kind_star(), kind_star()));
    ret
}

// Get Int type.
pub fn int_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(INT_NAME))
}

// Get Bool type.
pub fn bool_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(BOOL_NAME))
}

// Get Array type.
pub fn array_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(ARRAY_NAME))
}

pub fn int(val: i64, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_int_obj =
            ObjectType::int_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i64_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        ptr_to_int_obj
    });
    expr_lit(generator, vec![], val.to_string(), int_lit_ty(), source)
}

pub fn bool(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_obj = ObjectType::bool_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i8_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_obj, bool_type(gc.context), 1, value);
        ptr_to_obj
    });
    expr_lit(generator, vec![], val.to_string(), bool_lit_ty(), source)
}

fn add_lit(lhs: &str, rhs: &str) -> Arc<ExprNode> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let name = format!("add {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(&rhs_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc.builder().build_int_add(lhs_val, rhs_val, "add");
        let ptr_to_int_obj = ObjectType::int_obj_type().create_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        gc.release(gc.scope_get(&lhs_str).ptr);
        gc.release(gc.scope_get(&rhs_str).ptr);
        ptr_to_int_obj
    });
    expr_lit(generator, free_vars, name, int_lit_ty(), None)
}

pub fn add() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("lhs", None, None),
        expr_abs(var_local("rhs", None, None), add_lit("lhs", "rhs"), None),
        None,
    );
    let scm = Scheme::new_arc_from_str(
        &[],
        type_fun(int_lit_ty(), type_fun(int_lit_ty(), int_lit_ty())),
    );
    (expr, scm)
}

fn eq_lit(lhs: &str, rhs: &str) -> Arc<ExprNode> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let name = format!("eq {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(&rhs_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, "eq");
        let value = gc.builder().build_int_cast(
            value,
            ObjectFieldType::Bool
                .to_basic_type(gc.context)
                .into_int_type(),
            "eq_bool",
        );
        let ptr_to_obj = ObjectType::bool_obj_type().create_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(ptr_to_obj, bool_type(gc.context), 1, value);
        gc.release(gc.scope_get(&lhs_str).ptr);
        gc.release(gc.scope_get(&rhs_str).ptr);
        ptr_to_obj
    });
    expr_lit(generator, free_vars, name, bool_lit_ty(), None)
}

// eq = \lhs: a -> \rhs: a -> eq_lit(lhs, rhs): Bool
pub fn eq() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("lhs", None, None),
        expr_abs(var_local("rhs", None, None), eq_lit("lhs", "rhs"), None),
        None,
    );
    let scm = Scheme::new_arc_from_str(
        &[("a", kind_star())],
        type_fun(
            type_tyvar_star("a"),
            type_fun(type_tyvar_star("a"), bool_lit_ty()),
        ),
    );
    (expr, scm)
}

fn fix_lit(b: &str, f: &str, x: &str) -> Arc<ExprNode> {
    let f_str = String::from(f);
    let x_str = String::from(x);
    let name = format!("fix {} {}", f_str, x_str);
    let free_vars = vec![String::from(SELF_NAME), f_str.clone(), x_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let fixf = gc.scope_get(SELF_NAME).ptr;
        let x = gc.scope_get(&x_str).ptr;
        let f = gc.scope_get(&f_str).ptr;
        let f_fixf = gc.apply_lambda(f, fixf);
        let f_fixf_x = gc.apply_lambda(f_fixf, x);
        f_fixf_x
    });
    expr_lit(generator, free_vars, name, type_tyvar_star(b), None)
}

// fix = \f: ((a -> b) -> (a -> b)) -> \x: a -> fix_lit(b, f, x): b
pub fn fix() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("f", None, None),
        expr_abs(var_local("x", None, None), fix_lit("b", "f", "x"), None),
        None,
    );
    let fixed_ty = type_fun(type_tyvar_star("a"), type_tyvar_star("b"));
    let scm = Scheme::new_arc_from_str(
        &[("a", kind_star()), ("b", kind_star())],
        type_fun(type_fun(fixed_ty.clone(), fixed_ty.clone()), fixed_ty),
    );
    (expr, scm)
}

// Implementation of newArray built-in function.
fn new_array_lit(a: &str, size: &str, value: &str) -> Arc<ExprNode> {
    let size_str = String::from(size);
    let value_str = String::from(value);
    let name = format!("newArray {} {}", size, value);
    let name_cloned = name.clone();
    let free_vars = vec![size_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, ArrayField] where ArrayField = [Size, PtrToBuffer].
        let size = gc
            .scope_get_field(&size_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(&size_str).ptr);
        let value = gc.scope_get(&value_str).ptr;
        let array = ObjectType::array_type().create_obj(gc, Some(name_cloned.as_str()));
        let array_ptr_ty = ptr_type(ObjectType::array_type().to_struct_type(gc.context));
        let array = gc.cast_pointer(array, array_ptr_ty);
        let array_field = gc
            .builder()
            .build_struct_gep(array, 1, "array_field")
            .unwrap();
        ObjectFieldType::initialize_array(gc, array_field, size, value);
        array
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tyapp(array_lit_ty(), type_tyvar_star(a)),
        None,
    )
}

// "newArray" built-in function.
// newArray = for<a> \size: Int -> \value: a -> new_array_lit(a, size, value): Array<a>
pub fn new_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("size", None, None),
        expr_abs(
            var_local("value", None, None),
            new_array_lit("a", "size", "value"),
            None,
        ),
        None,
    );
    let scm = Scheme::new_arc_from_str(
        &[("a", kind_star())],
        type_fun(
            int_lit_ty(),
            type_fun(
                type_tyvar_star("a"),
                type_tyapp(array_lit_ty(), type_tyvar_star("a")),
            ),
        ),
    );
    (expr, scm)
}

// Implementation of readArray built-in function.
fn read_array_lit(a: &str, array: &str, idx: &str) -> Arc<ExprNode> {
    let array_str = String::from(array);
    let idx_str = String::from(idx);
    let name = format!("readArray {} {}", array, idx);
    let free_vars = vec![array_str.clone(), idx_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let array_ptr_ty = ptr_type(ObjectType::array_type().to_struct_type(gc.context));
        let array = gc.scope_get(array_str.as_str()).ptr;
        let array = gc.cast_pointer(array, array_ptr_ty);
        let array_field = gc
            .builder()
            .build_struct_gep(array, 1, "array_field")
            .unwrap();
        let idx = gc
            .scope_get_field(&idx_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(&idx_str).ptr);
        let elem = ObjectFieldType::read_array(gc, array_field, idx);
        gc.release(array);
        elem
    });
    expr_lit(generator, free_vars, name, type_tyvar_star(a), None)
}

// "readArray" built-in function.
// readArray = for<a> \arr: Array<a> -> \idx: Int -> (...read_array_lit(a, arr, idx)...): a
pub fn read_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("array", None, None),
        expr_abs(
            var_local("idx", None, None),
            read_array_lit("a", "array", "idx"),
            None,
        ),
        None,
    );
    let scm = Scheme::new_arc_from_str(
        &[("a", kind_star())],
        type_fun(
            type_tyapp(array_lit_ty(), type_tyvar_star("a")),
            type_fun(int_lit_ty(), type_tyvar_star("a")),
        ),
    );
    (expr, scm)
}

// Implementation of writeArray / writeArray! built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn write_array_lit(
    a: &str,
    array: &str,
    idx: &str,
    value: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let array_str = String::from(array);
    let idx_str = String::from(idx);
    let value_str = String::from(value);
    let func_name = String::from({
        if is_unique_version {
            "writeArray!"
        } else {
            "writeArray"
        }
    });
    let name = format!("{} {} {} {}", func_name, array, idx, value);
    let name_cloned = name.clone();
    let free_vars = vec![array_str.clone(), idx_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].

        // Get argments
        let array = gc.scope_get(array_str.as_str()).ptr;
        let idx = gc
            .scope_get_field(idx_str.as_str(), 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(idx_str.as_str()).ptr);
        let value = gc.scope_get(value_str.as_str()).ptr;

        // Get array field.
        let array_str_ty = ObjectType::array_type().to_struct_type(gc.context);
        let array = gc.cast_pointer(array, ptr_type(array_str_ty));
        let array_field = gc.builder().build_struct_gep(array, 1, "").unwrap();

        // Get refcnt.
        let refcnt = gc
            .load_obj_field(array, control_block_type(gc.context), 0)
            .into_int_value();

        // Add unique / shared / cont bbs.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let shared_bb = gc.context.append_basic_block(current_func, "shared_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");

        // Jump to shared_bb if refcnt > 1.
        let one = refcnt_type(gc.context).const_int(1, false);
        let is_unique = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique");
        gc.builder()
            .build_conditional_branch(is_unique, cont_bb, shared_bb);

        // In shared_bb, create new array and clone array field.
        gc.builder().position_at_end(shared_bb);
        if is_unique_version {
            // In case of unique version, panic in this case.
            gc.panic(format!("The argument of {} is shared!\n", func_name.as_str()).as_str());
        }
        let cloned_array = ObjectType::array_type().create_obj(gc, Some(name_cloned.as_str()));
        let cloned_array = gc.cast_pointer(cloned_array, ptr_type(array_str_ty));
        let cloned_array_field = gc.builder().build_struct_gep(cloned_array, 1, "").unwrap();
        ObjectFieldType::clone_array(gc, array_field, cloned_array_field);
        gc.release(array); // Given array should be released here.
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb);

        // Implement cont_bb
        gc.builder().position_at_end(cont_bb);

        // Build phi value of array and array_field.
        let array_phi = gc.builder().build_phi(array.get_type(), "array_phi");
        assert_eq!(array.get_type(), cloned_array.get_type());
        array_phi.add_incoming(&[(&array, current_bb), (&cloned_array, succ_of_shared_bb)]);
        let array = array_phi.as_basic_value().into_pointer_value();
        let array_field_phi = gc
            .builder()
            .build_phi(array_field.get_type(), "array_field_phi");
        assert_eq!(array_field.get_type(), cloned_array_field.get_type());
        array_field_phi.add_incoming(&[
            (&array_field, current_bb),
            (&cloned_array_field, succ_of_shared_bb),
        ]);
        let array_field = array_field_phi.as_basic_value().into_pointer_value();

        // Perform write and return.
        ObjectFieldType::write_array(gc, array_field, idx, value);
        array
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tyapp(array_lit_ty(), type_tyvar_star(a)),
        None,
    )
}

// writeArray built-in function.
// writeArray = for<a> \arr: Array<a> -> \idx: Int -> \value: a -> (...write_array_lit(a, arr, idx)...): Array<a>
pub fn write_array_common(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("array", None, None),
        expr_abs(
            var_local("idx", None, None),
            expr_abs(
                var_local("value", None, None),
                write_array_lit("a", "array", "idx", "value", is_unique_version),
                None,
            ),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(array_lit_ty(), type_tyvar_star("a"));
    let scm = Scheme::new_arc_from_str(
        &[("a", kind_star())],
        type_fun(
            array_ty.clone(),
            type_fun(int_lit_ty(), type_fun(type_tyvar_star("a"), array_ty)),
        ),
    );
    (expr, scm)
}

// writeArray built-in function.
pub fn write_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(false)
}

// writeArray! built-in function.
pub fn write_array_unique() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(true)
}

// `new` built-in function for a given struct.
pub fn struct_new_lit(struct_name: &str, field_names: Vec<String>) -> Arc<ExprNode> {
    let free_vars = field_names.clone();
    let name = format!("{}.new {}", struct_name, field_names.join(" "));
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Get field values.
        let field_ptrs: Vec<PointerValue> = field_names
            .iter()
            .map(|name| gc.scope_get(name).ptr)
            .collect();

        // Create struct object.
        let obj_ty = ObjectType::struct_type(field_names.len());
        let str_ptr = obj_ty.create_obj(gc, Some(&name_cloned));

        // Set fields.
        let struct_ty = obj_ty.to_struct_type(gc.context);
        for (i, field_ptr) in field_ptrs.iter().enumerate() {
            gc.store_obj_field(
                str_ptr,
                struct_ty,
                i as u32 + 1,
                field_ptr.as_basic_value_enum(),
            );
        }

        str_ptr
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tycon(&tycon(struct_name)),
        None,
    )
}

// `new` built-in function for a given struct.
pub fn struct_new(struct_name: &str, definition: &Struct) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Check there is no duplication of field names.
    let mut fields_set: HashMap<String, i32> = HashMap::new();
    for field in &definition.fields {
        if !fields_set.contains_key(&field.name) {
            fields_set.insert(field.name.clone(), 0);
        }
        *fields_set.get_mut(&field.name).unwrap() += 1;
        if fields_set[&field.name] >= 2 {
            error_exit(&format!(
                "error: in definition of struct `{}`, field `{}` is duplicated.",
                struct_name, &field.name
            ));
        }
    }
    let mut expr = struct_new_lit(
        struct_name,
        definition.fields.iter().map(|f| f.name.clone()).collect(),
    );
    let mut ty = type_tycon(&tycon(struct_name));
    for field in definition.fields.iter().rev() {
        expr = expr_abs(var_local(&field.name, None, None), expr, None);
        ty = type_fun(field.ty.clone(), ty);
    }
    let scm = Scheme::new_arc(HashMap::new(), ty);
    (expr, scm)
}

// `get` built-in function for a given struct.
pub fn struct_get_lit(
    var_name: &str,
    field_count: usize, // number of fields in this struct
    field_idx: usize,
    field_ty: Arc<TypeNode>,
    struct_name: &str,
    field_name: &str,
) -> Arc<ExprNode> {
    let var_name_clone = var_name.to_string();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Get struct object.
        let str_ptr = gc.scope_get(&var_name_clone).ptr;

        // Extract field.
        let str_ty = ObjectType::struct_type(field_count).to_struct_type(gc.context);
        let field_ptr = gc.load_obj_field(str_ptr, str_ty, field_idx as u32 + 1);
        let field_ptr = field_ptr.into_pointer_value();

        // Retain field and release struct.
        gc.retain(field_ptr);
        gc.release(str_ptr);

        field_ptr
    });
    let free_vars = vec![var_name.to_string()];
    let name = format!("{}.get{}", struct_name, field_name);
    expr_lit(generator, free_vars, name, field_ty, None)
}

// `get` built-in function for a given struct.
pub fn struct_get(
    struct_name: &str,
    definition: &Struct,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition
        .fields
        .iter()
        .enumerate()
        .find(|(_i, f)| f.name == field_name);
    if field.is_none() {
        error_exit(&format!(
            "error: no field `{}` found in the struct `{}`.",
            &field_name, struct_name,
        ));
    }
    let (field_idx, field) = field.unwrap();

    let field_count = definition.fields.len();
    let str_ty = type_tycon(&tycon(struct_name));
    let expr = expr_abs(
        var_local("f", None, None),
        struct_get_lit(
            "f",
            field_count,
            field_idx,
            field.ty.clone(),
            struct_name,
            field_name,
        ),
        None,
    );
    let ty = type_fun(str_ty, field.ty.clone());
    let scm = Scheme::new_arc(HashMap::new(), ty);
    (expr, scm)
}

// Add bult-in functions to a given ast.
pub fn add_builtin_symbols(program: &mut FixModule) {
    fn add_let(
        program: Arc<ExprNode>,
        namespace: NameSpace,
        name: &str,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
    ) -> Arc<ExprNode> {
        expr_let(
            var_var(name, Some(namespace), Some(scm), None),
            expr,
            program,
            None,
        )
    }

    let expr = program.expr.clone();
    let expr = add_let(expr, NameSpace::new_str(&[PRELUDE_NAME]), "add", add());
    let expr = add_let(expr, NameSpace::new_str(&[PRELUDE_NAME]), "eq", eq());
    let expr = add_let(expr, NameSpace::new_str(&[PRELUDE_NAME]), "fix", fix());
    let expr = add_let(
        expr,
        NameSpace::new_str(&[PRELUDE_NAME]),
        "newArray",
        new_array(),
    );
    let expr = add_let(
        expr,
        NameSpace::new_str(&[PRELUDE_NAME]),
        "readArray",
        read_array(),
    );
    let expr = add_let(
        expr,
        NameSpace::new_str(&[PRELUDE_NAME]),
        "writeArray",
        write_array(),
    );
    let expr = add_let(
        expr,
        NameSpace::new_str(&[PRELUDE_NAME]),
        "writeArray!",
        write_array_unique(),
    );
    program.expr = expr;
}
