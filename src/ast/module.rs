use std::time::SystemTime;

use chrono::{DateTime, Utc};

use inkwell::module::Linkage;
use serde::{Deserialize, Serialize};
use std::io::Write;

use super::*;

const MAIN_FUNCTION_NAME: &str = "main";
const MAIN_MODULE_NAME: &str = "Main";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "%";
pub const GETTER_SYMBOL: &str = "@";
pub const SETTER_SYMBOL: &str = "=";

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Rc<HashMap<TyCon, TyConInfo>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Rc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: HashMap<TyCon, TyConInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Rc::new(tycons),
        }
    }

    pub fn kinds(&self) -> HashMap<TyCon, Rc<Kind>> {
        let mut res = HashMap::default();
        for (tc, ti) in self.tycons.as_ref().iter() {
            res.insert(tc.clone(), ti.kind.clone());
        }
        res
    }
}

#[derive(Clone)]
pub struct InstantiatedSymbol {
    pub template_name: FullName,
    pub ty: Rc<TypeNode>,
    pub expr: Option<Rc<ExprNode>>,
    pub type_resolver: TypeResolver, // type resolver for types in expr.
}

pub struct GlobalValue {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub scm: Rc<Scheme>,
    pub expr: SymbolExpr,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl GlobalValue {
    pub fn resolve_namespace_in_declaration(&mut self, ctx: &NameResolutionContext) {
        self.scm = self.scm.resolve_namespace(ctx);
    }

    pub fn set_kinds(
        &mut self,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Rc<Kind>>,
    ) {
        self.scm = self.scm.set_kinds(trait_kind_map);
        self.scm.check_kinds(kind_map, trait_kind_map);
        match &mut self.expr {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(ms) => {
                for m in ms {
                    m.ty = m.ty.set_kinds(trait_kind_map);
                    m.ty.check_kinds(kind_map, trait_kind_map);
                }
            }
        }
    }
}

// Expression of global symbol.
#[derive(Clone)]
pub enum SymbolExpr {
    Simple(TypedExpr),       // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<MethodImpl>), // Trait method implementations.
}

// Pair of expression and type resolver for it.
#[derive(Clone, Serialize, Deserialize)]
pub struct TypedExpr {
    pub expr: Rc<ExprNode>,
    pub type_resolver: TypeResolver,
}

impl TypedExpr {
    pub fn from_expr(expr: Rc<ExprNode>) -> Self {
        TypedExpr {
            expr,
            type_resolver: TypeResolver::default(),
        }
    }

    pub fn calculate_free_vars(&mut self) {
        self.expr = calculate_free_vars(self.expr.clone());
    }

    // When unification fails, it has no side effect to self.
    pub fn unify_to(&mut self, target_ty: &Rc<TypeNode>) -> bool {
        return self
            .type_resolver
            .unify(&self.expr.ty.as_ref().unwrap(), target_ty);
    }
}

// Trait method implementation
#[derive(Clone)]
pub struct MethodImpl {
    // Type of this method.
    // For example, in case "impl [a: Show, b: Show] (a, b): Show {...}",
    // the type of method "show" is "[a: Show, b: Show] (a, b) -> String",
    pub ty: Rc<Scheme>,
    // Expression of this implementation
    pub expr: TypedExpr,
    // Module where this implmentation is given.
    // NOTE:
    // For trait method, `define_module` may not differ to the first component of namespace of the function.
    // For example, if `Main` module implements `Eq : SomeType`, then implementation of `eq` for `SomeType` is defined in `Main` module,
    // but its name as a function is still `Std::Eq::eq`.
    pub define_module: Name,
}

pub struct NameResolutionContext {
    pub types: HashSet<FullName>,
    pub traits: HashSet<FullName>,
    pub imported_modules: HashSet<Name>,
}

#[derive(PartialEq)]
pub enum NameResolutionType {
    Type,
    Trait,
}

impl<'a> NameResolutionContext {
    pub fn resolve(
        &self,
        ns: &FullName,
        type_or_trait: NameResolutionType,
    ) -> Result<FullName, String> {
        let candidates = if type_or_trait == NameResolutionType::Type {
            &self.types
        } else {
            &self.traits
        };
        let candidates = candidates
            .iter()
            .filter(|name| self.imported_modules.contains(&name.module()))
            .filter_map(|id| {
                if ns.is_suffix(id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            let msg = match type_or_trait {
                NameResolutionType::Type => {
                    format!("Unknown type name: {}", ns.to_string())
                }
                NameResolutionType::Trait => {
                    format!("Unknown trait name: {}", ns.to_string())
                }
            };
            Err(msg)
        } else if candidates.len() == 1 {
            Ok(candidates[0].clone())
        } else {
            // candidates.len() >= 2
            let msg = if type_or_trait == NameResolutionType::Type {
                format!("Type name `{}` is ambiguous.", ns.to_string())
            } else {
                format!("Trait name `{}` is ambiguous.", ns.to_string())
            };
            Err(msg)
        }
    }
}

#[derive(Clone)]
pub struct UpdateDate(pub DateTime<Utc>);

impl UpdateDate {
    pub fn max(&self, other: &UpdateDate) -> UpdateDate {
        UpdateDate(self.0.max(other.0))
    }
}

impl Serialize for UpdateDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for UpdateDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UpdateDateVisitor)
    }
}

struct UpdateDateVisitor;
impl<'de> serde::de::Visitor<'de> for UpdateDateVisitor {
    type Value = UpdateDate;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UpdateDate(
            DateTime::parse_from_rfc3339(v).unwrap().with_timezone(&Utc),
        ))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("String for UpdateDate")
    }
}

// Module of fix-lang.
// To avoid confliction with "inkwell::Module", we name it as `FixModule`.
pub struct FixModule {
    pub name: Name,
    pub unresolved_imports: Vec<ImportStatement>,
    // A map to represent modules imported by each submodule.
    // Each module imports itself.
    // This is used to name-resolution and overloading resolution,
    pub imported_mod_map: HashMap<Name, HashSet<Name>>,
    // Modules linked to this module.
    // TODO maybe we can get this from keys-set of imported_mod_map?
    pub linked_mods: HashSet<Name>,
    pub type_defns: Vec<TypeDefn>,
    pub global_values: HashMap<FullName, GlobalValue>,
    pub instantiated_global_symbols: HashMap<FullName, InstantiatedSymbol>,
    pub deferred_instantiation: HashMap<FullName, InstantiatedSymbol>,
    pub trait_env: TraitEnv,
    pub type_env: TypeEnv,
    // Last update date for each linked modules.
    pub last_updates: HashMap<Name, UpdateDate>,
    // Last affected date for each linked modules.
    // Last affected date is defined as the maximum value of last update dates of all imported modules.
    pub last_affected_dates: HashMap<Name, UpdateDate>,
}

impl FixModule {
    // Create empty module.
    pub fn new(name: Name) -> FixModule {
        let mut fix_mod = FixModule {
            name: name.clone(),
            unresolved_imports: vec![],
            imported_mod_map: Default::default(),
            linked_mods: Default::default(),
            type_defns: Default::default(),
            global_values: Default::default(),
            instantiated_global_symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
            last_updates: Default::default(),
            last_affected_dates: Default::default(),
        };
        fix_mod.linked_mods.insert(name.clone());
        fix_mod.insert_imported_mod_map(&name, &name);
        fix_mod.insert_imported_mod_map(&name, &STD_NAME.to_string());
        fix_mod.set_last_update(UpdateDate(SystemTime::now().into())); // Later updated to source file's last modified date.
        fix_mod
    }

    // Set this module's last update.
    pub fn set_last_update(&mut self, time: UpdateDate) {
        self.last_updates.insert(self.name.clone(), time);
    }

    // Add import statements.
    pub fn add_import_statements(&mut self, mut imports: Vec<ImportStatement>) {
        for import in &imports {
            let mod_name = self.name.clone();
            self.insert_imported_mod_map(&mod_name, &import.module);
        }
        self.unresolved_imports.append(&mut imports);
    }

    // Add traits.
    pub fn add_traits(&mut self, trait_infos: Vec<TraitInfo>, trait_impls: Vec<TraitInstance>) {
        self.trait_env.add(trait_infos, trait_impls);
    }

    // Register declarations of user-defined types.
    pub fn add_type_defns(&mut self, mut type_defns: Vec<TypeDefn>) {
        self.type_defns.append(&mut type_defns);
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) {
        let mut tycons = bulitin_tycons();
        for type_decl in &self.type_defns {
            let tycon = type_decl.tycon();
            if tycons.contains_key(&tycon) {
                error_exit_with_src(
                    &format!("Type `{}` is already defined.", tycon.to_string()),
                    &None,
                );
            }
            tycons.insert(tycon, type_decl.tycon_info());
        }
        self.type_env = TypeEnv::new(tycons);
    }

    // Get list of type constructors including user-defined types.
    pub fn type_env(&self) -> TypeEnv {
        self.type_env.clone()
    }

    // Get of list of tycons that can be used for namespace resolution.
    pub fn tycon_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in self.type_env().tycons.iter() {
            res.insert(k.name.clone());
        }
        res
    }

    // Get of list of traits that can be used for namespace resolution.
    pub fn trait_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in &self.trait_env.traits {
            res.insert(k.name.clone());
        }
        res
    }

    // Add a global value.
    pub fn add_global_value(&mut self, name: FullName, (expr, scm): (Rc<ExprNode>, Rc<Scheme>)) {
        if self.global_values.contains_key(&name) {
            error_exit(&format!(
                "duplicated definition for global value: `{}`",
                name.to_string()
            ));
        }
        self.global_values.insert(
            name,
            GlobalValue {
                scm,
                expr: SymbolExpr::Simple(TypedExpr::from_expr(expr)),
            },
        );
    }

    // Add global values
    pub fn add_global_values(
        &mut self,
        exprs: Vec<(FullName, Rc<ExprNode>)>,
        types: Vec<(FullName, Rc<Scheme>)>,
    ) {
        struct GlobalValue {
            expr: Option<Rc<ExprNode>>,
            ty: Option<Rc<Scheme>>,
        }

        let mut global_values: HashMap<FullName, GlobalValue> = Default::default();
        for (name, expr) in exprs {
            if !global_values.contains_key(&name) {
                global_values.insert(
                    name,
                    GlobalValue {
                        expr: Some(expr),
                        ty: None,
                    },
                );
            } else {
                let gs = global_values.get_mut(&name).unwrap();
                if gs.expr.is_some() {
                    error_exit(&format!(
                        "duplicated definition signature for global value: `{}`",
                        name.to_string()
                    ));
                } else {
                    gs.expr = Some(expr);
                }
            }
        }
        for (name, ty) in types {
            if !global_values.contains_key(&name) {
                global_values.insert(
                    name,
                    GlobalValue {
                        ty: Some(ty),
                        expr: None,
                    },
                );
            } else {
                let gs = global_values.get_mut(&name).unwrap();
                if gs.ty.is_some() {
                    error_exit(&format!(
                        "duplicated type signature for `{}`",
                        name.to_string()
                    ));
                } else {
                    gs.ty = Some(ty);
                }
            }
        }

        for (name, gv) in global_values {
            if gv.expr.is_none() {
                error_exit(&format!(
                    "global value `{}` lacks type signature",
                    name.to_string()
                ))
            }
            if gv.ty.is_none() {
                error_exit(&format!(
                    "global value `{}` lacks definition",
                    name.to_string()
                ))
            }
            self.add_global_value(name, (gv.expr.unwrap(), gv.ty.unwrap()))
        }
    }

    // Generate codes of global symbols.
    pub fn generate_code(&self, gc: &mut GenerationContext) {
        // First,
        // - For function pointer, declare the function and register it to global variable.
        // - For others, create global variable and declare accessor function and register it to global variable.
        let global_objs = self
            .instantiated_global_symbols
            .iter()
            .map(|(name, sym)| {
                gc.typeresolver = sym.type_resolver.clone();
                let obj_ty = sym.type_resolver.substitute_type(&sym.ty);
                if obj_ty.is_funptr() {
                    let lam = sym.expr.as_ref().unwrap().clone();
                    let lam = lam.set_inferred_type(obj_ty.clone());
                    let lam_fn = gc.declare_lambda_function(lam);
                    gc.add_global_object(name.clone(), lam_fn, obj_ty.clone());
                    (None, None, lam_fn, sym.clone(), obj_ty)
                } else {
                    let flag_name = format!("InitFlag{}", name.to_string());
                    let global_var_name = format!("GlobalVar{}", name.to_string());
                    let acc_fn_name = format!("Get{}", name.to_string());

                    let obj_embed_ty = obj_ty.get_embedded_type(gc, &vec![]);

                    // Add global variable.
                    let global_var = gc.module.add_global(obj_embed_ty, None, &global_var_name);
                    global_var.set_initializer(&obj_embed_ty.const_zero());
                    let global_var = global_var.as_basic_value_enum().into_pointer_value();

                    // Add initialized flag.
                    let flag_ty = gc.context.i8_type();
                    let init_flag = gc.module.add_global(flag_ty, None, &flag_name);
                    init_flag.set_initializer(&flag_ty.const_zero());
                    let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

                    // Add accessor function.
                    let acc_fn_type = ptr_to_object_type(gc.context).fn_type(&[], false);
                    let acc_fn =
                        gc.module
                            .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::Internal));

                    // Register the accessor function to gc.
                    gc.add_global_object(name.clone(), acc_fn, obj_ty.clone());

                    // Return global variable and accessor function.
                    (
                        Some(global_var),
                        Some(init_flag),
                        acc_fn,
                        sym.clone(),
                        obj_ty,
                    )
                }
            })
            .collect::<Vec<_>>();

        // Implement functions.
        for (global_var, init_flag, acc_fn, sym, obj_ty) in global_objs {
            gc.typeresolver = sym.type_resolver;
            if obj_ty.is_funptr() {
                // Implement lambda function.
                let lam_fn = acc_fn;
                let lam = sym.expr.as_ref().unwrap().clone();
                let lam = lam.set_inferred_type(obj_ty);
                gc.implement_lambda_function(lam, lam_fn, None);
            } else {
                // Implement accessor function.
                let global_var = global_var.unwrap();
                let init_flag = init_flag.unwrap();
                let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
                gc.builder().position_at_end(entry_bb);
                let flag = gc
                    .builder()
                    .build_load(init_flag, "load_init_flag")
                    .into_int_value();
                let is_zero = gc.builder().build_int_compare(
                    IntPredicate::EQ,
                    flag,
                    flag.get_type().const_zero(),
                    "flag_is_zero",
                );
                let init_bb = gc.context.append_basic_block(acc_fn, "flag_is_zero");
                let end_bb = gc.context.append_basic_block(acc_fn, "flag_is_nonzero");
                gc.builder()
                    .build_conditional_branch(is_zero, init_bb, end_bb);

                // If flag is zero, then create object and store it to the global variable.
                gc.builder().position_at_end(init_bb);
                // Prepare memory space for rvo.
                let rvo = if obj_ty.is_unbox(gc.type_env()) {
                    Some(Object::new(global_var, obj_ty))
                } else {
                    None
                };
                // Execute expression.
                let obj = gc.eval_expr(sym.expr.unwrap().clone(), rvo.clone());

                if gc.config.preretain_global && obj.is_box(gc.type_env()) {
                    let obj_ptr = obj.ptr(gc);
                    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
                    // Pre-retain global objects (to omit retaining later).
                    let infty = refcnt_type(gc.context).const_int(u64::MAX / 2, false);
                    gc.builder().build_store(ptr_to_refcnt, infty);
                }
                // If we didn't rvo, then store the result to global_ptr.
                if rvo.is_none() {
                    let obj_val = obj.value(gc);
                    gc.builder().build_store(global_var, obj_val);
                }

                // Set the initialized flag 1.
                gc.builder()
                    .build_store(init_flag, gc.context.i8_type().const_int(1, false));

                if gc.config.sanitize_memory && obj.is_box(gc.type_env()) {
                    // Mark this object as global.
                    let ptr = obj.ptr(gc);
                    let obj_id = gc.get_obj_id(ptr);
                    gc.call_runtime(RuntimeFunctions::MarkGlobal, &[obj_id.into()]);
                }
                gc.builder().build_unconditional_branch(end_bb);

                // Return object.
                gc.builder().position_at_end(end_bb);
                let ret = if obj.is_box(gc.type_env()) {
                    gc.builder()
                        .build_load(global_var, "PtrToObj")
                        .into_pointer_value()
                } else {
                    global_var
                };
                let ret = gc.cast_pointer(ret, ptr_to_object_type(gc.context));
                gc.builder().build_return(Some(&ret));
            }
        }
    }

    // Resolve namespace of type and trats in expression, and perform typechecking.
    // The result will be written to `te`.
    fn resolve_namespace_and_check_type(
        &self,
        te: &mut TypedExpr,
        required_scheme: &Rc<Scheme>,
        name: &FullName,
        define_module: &Name,
        tc: &TypeCheckContext,
    ) {
        fn cache_file_name(name: &FullName, define_module: &Name, scheme: &Rc<Scheme>) -> String {
            format!(
                "{}@{}@{}",
                name.to_string(),
                define_module,
                scheme.to_string()
            )
        }
        fn load_cache(
            name: &FullName,
            define_module: &Name,
            define_module_last_affected: &UpdateDate,
            required_scheme: &Rc<Scheme>,
        ) -> Option<TypedExpr> {
            let cache_file_name = cache_file_name(name, define_module, required_scheme);
            let cache_dir = touch_directory(".fixlang/type_check_cache");
            let cache_file = cache_dir.join(cache_file_name);
            let cache_file_display = cache_file.display();
            if !cache_file.exists() {
                return None;
            }
            let mut cache_file = match File::open(&cache_file) {
                Err(_) => {
                    return None;
                }
                Ok(file) => file,
            };
            let mut cache_bytes = vec![];
            match cache_file.read_to_end(&mut cache_bytes) {
                Ok(_) => {}
                Err(why) => {
                    eprintln!("Failed to read cache file {}: {}.", cache_file_display, why);
                    return None;
                }
            }
            let (expr, last_update): (TypedExpr, UpdateDate) =
                match serde_pickle::from_slice(&cache_bytes, Default::default()) {
                    Ok(res) => res,
                    Err(why) => {
                        eprintln!(
                            "Failed to parse content of cache file {}: {}.",
                            cache_file_display, why
                        );
                        return None;
                    }
                };
            if last_update.0 < define_module_last_affected.0 {
                return None;
            }
            Some(expr)
        }

        fn save_cache(
            te: &TypedExpr,
            required_scheme: &Rc<Scheme>,
            name: &FullName,
            define_module: &Name,
            last_updated: &UpdateDate,
        ) {
            let cache_file_name = cache_file_name(name, define_module, required_scheme);
            let cache_dir = touch_directory(".fixlang/type_check_cache");
            let cache_file = cache_dir.join(cache_file_name);
            let cache_file_display = cache_file.display();
            let mut cache_file = match File::create(&cache_file) {
                Err(_) => {
                    eprintln!("Failed to create cache file {}.", cache_file_display);
                    return;
                }
                Ok(file) => file,
            };
            let serialized = serde_pickle::to_vec(&(te, last_updated), Default::default()).unwrap();
            match cache_file.write_all(&serialized) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to write cache file {}.", cache_file_display);
                }
            }
        }

        // Load type-checking cache file.
        let last_affected_date = self.last_affected_dates.get(define_module).unwrap();
        let opt_cache = load_cache(name, define_module, last_affected_date, required_scheme);
        if opt_cache.is_some() {
            // If cache is available,
            *te = opt_cache.unwrap();
            te.type_resolver.kind_map = tc.type_env.kinds();
            return;
        }

        // Perform namespace inference.
        let nrctx = NameResolutionContext {
            types: self.tycon_names(),
            traits: self.trait_names(),
            imported_modules: self.imported_mod_map[define_module].clone(),
        };
        te.expr = te.expr.resolve_namespace(&nrctx);

        // Perform type-checking.
        let mut tc = tc.clone();
        tc.current_module = Some(define_module.clone());
        te.expr = tc.check_type(te.expr.clone(), required_scheme.clone());
        te.type_resolver = tc.resolver;

        // Save the result to cache file.
        save_cache(te, required_scheme, name, define_module, last_affected_date);
    }

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, sym: &mut InstantiatedSymbol, tc: &TypeCheckContext) {
        assert!(sym.expr.is_none());
        let global_sym = self.global_values.get(&sym.template_name).unwrap();
        let typed_expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                // Perform type-checking.
                let define_module = sym.template_name.module();
                let mut e = e.clone();
                self.resolve_namespace_and_check_type(
                    &mut e,
                    &global_sym.scm,
                    &sym.template_name,
                    &define_module,
                    tc,
                );
                // Calculate free vars.
                e.calculate_free_vars();
                // Specialize e's type to the required type `sym.ty`.
                let ok = e.unify_to(&sym.ty);
                assert!(ok);
                e
            }
            SymbolExpr::Method(impls) => {
                let mut opt_e: Option<TypedExpr> = None;
                for method in impls {
                    // Check if the type of this implementation unify with the required type `sym.ty`.
                    let mut tc0 = tc.clone();
                    let (_, method_ty) = tc0.instantiate_scheme(&method.ty, false);
                    if Substitution::unify(&tc.type_env.kinds(), &method_ty, &sym.ty).is_none() {
                        continue;
                    }
                    // Perform type-checking.
                    let define_module = method.define_module.clone();
                    let mut e = method.expr.clone();
                    self.resolve_namespace_and_check_type(
                        &mut e,
                        &method.ty,
                        &sym.template_name,
                        &define_module,
                        tc,
                    );
                    // Calculate free vars.
                    e.calculate_free_vars();
                    // Specialize e's type to required type `sym.ty`
                    let ok = e.unify_to(&sym.ty);
                    if !ok {
                        println!("{}", e.expr.ty.as_ref().unwrap().to_string());
                        println!("{}", sym.ty.to_string());
                    }
                    assert!(ok);
                    opt_e = Some(e);
                    break;
                }
                opt_e.unwrap()
            }
        };
        sym.expr = Some(self.instantiate_expr(&typed_expr.type_resolver, &typed_expr.expr));
        sym.type_resolver = typed_expr.type_resolver;
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self, tc: &TypeCheckContext) {
        while !self.deferred_instantiation.is_empty() {
            let (name, sym) = self.deferred_instantiation.iter().next().unwrap();
            let name = name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(&mut sym, tc);
            self.deferred_instantiation.remove(&name);
            self.instantiated_global_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self, tc: &TypeCheckContext) -> Rc<ExprNode> {
        let main_func_name = FullName::from_strs(&[MAIN_MODULE_NAME], MAIN_FUNCTION_NAME);
        if !self.global_values.contains_key(&main_func_name) {
            error_exit(&format!("{} not found.", main_func_name.to_string()));
        }
        let main_ty = make_io_unit_ty();
        let inst_name = self.require_instantiated_symbol(&main_func_name, &main_ty);
        self.instantiate_symbols(tc);
        expr_var(inst_name, None).set_inferred_type(main_ty)
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, tr: &TypeResolver, expr: &Rc<ExprNode>) -> Rc<ExprNode> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let ty = tr.substitute_type(&expr.ty.as_ref().unwrap());
                    let instance = self.require_instantiated_symbol(&v.name, &ty);
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::Lit(_) => expr.clone(),
            Expr::App(fun, args) => {
                let fun = self.instantiate_expr(tr, fun);
                let args = args
                    .iter()
                    .map(|arg| self.instantiate_expr(tr, arg))
                    .collect::<Vec<_>>();
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_, body) => expr.set_lam_body(self.instantiate_expr(tr, body)),
            Expr::Let(_, bound, val) => {
                let bound = self.instantiate_expr(tr, bound);
                let val = self.instantiate_expr(tr, val);
                expr.set_let_bound(bound).set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.instantiate_expr(tr, cond);
                let then_expr = self.instantiate_expr(tr, then_expr);
                let else_expr = self.instantiate_expr(tr, else_expr);
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, _) => {
                let e = self.instantiate_expr(tr, e);
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_, fields) => {
                let mut expr = expr.clone();
                for (field_name, field_expr) in fields {
                    let field_expr = self.instantiate_expr(tr, field_expr);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = expr.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.instantiate_expr(tr, e);
                    expr = expr.set_array_lit_elem(e, i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = expr.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = self.instantiate_expr(tr, e);
                    expr = expr.set_call_c_arg(e, i);
                }
                expr
            }
        };
        // If the type of an expression contains undetermied type variable after instantiation, raise an error.
        if !tr
            .substitute_type(ret.ty.as_ref().unwrap())
            .free_vars()
            .is_empty()
        {
            error_exit_with_src(
                "The type of an expression cannot be determined. You need to add type annotation to help type inference.",
                &expr.source,
            );
        }
        calculate_free_vars(ret)
    }

    // Require instantiate generic symbol such that it has a specified type.
    pub fn require_instantiated_symbol(&mut self, name: &FullName, ty: &Rc<TypeNode>) -> FullName {
        if !ty.free_vars().is_empty() {
            error_exit(&format!("Cannot instantiate global value `{}` of type `{}` since the type contains undetermined type variable. Maybe you need to add a type annotation.", name.to_string(), ty.to_string_normalize()));
        }
        let inst_name = self.determine_instantiated_symbol_name(name, ty);
        if !self.instantiated_global_symbols.contains_key(&inst_name)
            && !self.deferred_instantiation.contains_key(&inst_name)
        {
            self.deferred_instantiation.insert(
                inst_name.clone(),
                InstantiatedSymbol {
                    template_name: name.clone(),
                    ty: ty.clone(),
                    expr: None,
                    type_resolver: TypeResolver::default(), // This field will be set in the end of instantiation.
                },
            );
        }
        inst_name
    }

    // Determine the name of instantiated generic symbol so that it has a specified type.
    // tc: a typechecker (substituion) under which ty should be interpreted.
    fn determine_instantiated_symbol_name(&self, name: &FullName, ty: &Rc<TypeNode>) -> FullName {
        assert!(ty.free_vars().is_empty());
        let hash = ty.hash();
        let mut name = name.clone();
        name.name += INSTANCIATED_NAME_SEPARATOR;
        name.name += &hash;
        name
    }

    // Create symbols of trait methods from TraitEnv.
    pub fn create_trait_method_symbols(&mut self) {
        for (trait_id, trait_info) in &self.trait_env.traits {
            for (method_name, _) in &trait_info.methods {
                let method_ty = trait_info.method_scheme(method_name);
                let mut method_impls: Vec<MethodImpl> = vec![];
                let instances = self.trait_env.instances.get(trait_id);
                if let Some(insntances) = instances {
                    for trait_impl in insntances {
                        let scm = trait_impl.method_scheme(method_name, trait_info);
                        let expr = trait_impl.method_expr(method_name);
                        method_impls.push(MethodImpl {
                            ty: scm,
                            expr: TypedExpr::from_expr(expr),
                            define_module: trait_impl.define_module.clone(),
                        });
                    }
                }
                let method_name = FullName::new(&trait_id.name.to_namespace(), &method_name);
                self.global_values.insert(
                    method_name,
                    GlobalValue {
                        scm: method_ty,
                        expr: SymbolExpr::Method(method_impls),
                    },
                );
            }
        }
    }

    pub fn set_kinds(&mut self) {
        self.trait_env.set_kinds();
        let kind_map = &self.type_env().kinds();
        let trait_kind_map = self.trait_env.trait_kind_map();
        for (_name, sym) in &mut self.global_values {
            sym.set_kinds(kind_map, &trait_kind_map);
        }
    }

    // Infer namespaces to traits and types that appear in declarations (not in expressions).
    // NOTE: names of in the definition of types/traits/global_values have to be full-named already when this function called.
    pub fn resolve_namespace_in_declaration(&mut self) {
        let mut ctx = NameResolutionContext {
            types: self.tycon_names(),
            traits: self.trait_names(),
            imported_modules: HashSet::default(),
        };
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (tc, ti) in &mut tycons {
                ctx.imported_modules = self.imported_mod_map[&tc.name.module()].clone();
                ti.resolve_namespace(&ctx);
            }
            self.type_env.tycons = Rc::new(tycons);
        }

        self.trait_env
            .resolve_namespace(&mut ctx, &self.imported_mod_map);
        for decl in &mut self.type_defns {
            ctx.imported_modules = self.imported_mod_map[&decl.name.module()].clone();
            decl.resolve_namespace(&ctx);
        }
        for (name, sym) in &mut self.global_values {
            ctx.imported_modules = self.imported_mod_map[&name.module()].clone();
            sym.resolve_namespace_in_declaration(&ctx);
        }
    }

    // Validate user-defined types
    pub fn validate_type_defns(&self) {
        for type_defn in &self.type_defns {
            type_defn.check_tyvars();
            let type_name = &type_defn.name;
            match &type_defn.value {
                TypeDeclValue::Struct(str) => match Field::check_duplication(&str.fields) {
                    Some(field_name) => {
                        error_exit(&format!(
                            "Duplicate field `{}` for struct `{}`",
                            field_name,
                            type_name.to_string()
                        ));
                    }
                    _ => {}
                },
                TypeDeclValue::Union(union) => match Field::check_duplication(&union.fields) {
                    Some(field_name) => {
                        error_exit(&format!(
                            "Duplicate field `{}` for union `{}`",
                            field_name,
                            type_name.to_string()
                        ));
                    }
                    _ => {}
                },
            }
        }
    }

    pub fn validate_trait_env(&mut self) {
        let kind_map = self.type_env.kinds();
        self.trait_env.validate(&kind_map);
    }

    pub fn add_methods(self: &mut FixModule) {
        for decl in &self.type_defns.clone() {
            match &decl.value {
                TypeDeclValue::Struct(str) => {
                    let struct_name = decl.name.clone();
                    for field in &str.fields {
                        self.add_global_value(
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("{}{}", GETTER_SYMBOL, &field.name),
                            ),
                            struct_get(&struct_name, decl, &field.name),
                        );
                        for is_unique in [false, true] {
                            self.add_global_value(
                                FullName::new(
                                    &decl.name.to_namespace(),
                                    &format!(
                                        "mod_{}{}",
                                        &field.name,
                                        if is_unique { "!" } else { "" }
                                    ),
                                ),
                                struct_mod(&struct_name, decl, &field.name, is_unique),
                            );
                            self.add_global_value(
                                FullName::new(
                                    &decl.name.to_namespace(),
                                    &format!(
                                        "{}{}{}",
                                        SETTER_SYMBOL,
                                        &field.name,
                                        if is_unique { "!" } else { "" }
                                    ),
                                ),
                                struct_set(&struct_name, decl, &field.name, is_unique),
                            )
                        }
                    }
                }
                TypeDeclValue::Union(union) => {
                    let union_name = &decl.name;
                    for field in &union.fields {
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &field.name),
                            union_new(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &format!("as_{}", field.name)),
                            union_as(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &format!("is_{}", field.name)),
                            union_is(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("mod_{}", field.name),
                            ),
                            union_mod_function(&union_name, &field.name, decl),
                        );
                    }
                }
            }
        }
    }

    // Link two modules.
    pub fn link(&mut self, mut other: FixModule) {
        // TODO: check if a module defined by a single source file.

        // If already linked, do nothing.
        if self.linked_mods.contains(&other.name) {
            return;
        }
        self.linked_mods.insert(other.name);

        self.unresolved_imports
            .append(&mut other.unresolved_imports);

        // Merge imported_mod_map.
        for (importer, importee) in &other.imported_mod_map {
            if let Some(known_importee) = self.imported_mod_map.get(importer) {
                assert_eq!(known_importee, importee);
            } else {
                self.imported_mod_map
                    .insert(importer.clone(), importee.clone());
            }
        }

        // Merge types.
        self.add_type_defns(other.type_defns);

        // Merge traits and instances.
        self.trait_env.import(other.trait_env);

        // Merge global values.
        for (name, gv) in other.global_values {
            let ty = gv.scm;
            if let SymbolExpr::Simple(expr) = gv.expr {
                self.add_global_value(name, (expr.expr, ty));
            }
        }

        // Merge last updates.
        for (module, dt) in other.last_updates {
            self.last_updates.insert(module, dt);
        }
    }

    pub fn resolve_imports(&mut self) {
        while self.unresolved_imports.len() > 0 {
            let import = self.unresolved_imports.pop().unwrap();

            // If import is already resolved, do nothing.
            if self.imported_mod_map.contains_key(&import.module) {
                continue;
            }

            // Search for bulit-in modules.
            if import.module == "Debug" {
                self.link(parse_source(include_str!("../debug.fix"), "debug.fix"));
                continue;
            }
            if import.module == "HashMap" {
                self.link(parse_source(include_str!("../hashmap.fix"), "hashmap.fix"));
                continue;
            }
            if import.module == "Hash" {
                self.link(parse_source(include_str!("../hash.fix"), "hash.fix"));
                continue;
            }

            error_exit_with_src(
                &format!("Cannot find module `{}`", import.module),
                &import.source,
            );
        }
    }

    pub fn insert_imported_mod_map(&mut self, importer: &Name, imported: &Name) {
        if !self.imported_mod_map.contains_key(importer) {
            self.imported_mod_map
                .insert(importer.clone(), Default::default());
        }
        self.imported_mod_map
            .get_mut(importer)
            .unwrap()
            .insert(imported.clone());
    }

    // Create a graph of modules. If module A imports module B, an edge from B to A is added.
    pub fn importing_module_graph(&self) -> (Graph<Name>, HashMap<Name, usize>) {
        let (mut graph, elem_to_idx) = Graph::from_set(self.linked_mods.clone());
        for (from, tos) in &self.imported_mod_map {
            for to in tos {
                graph.connect(
                    *elem_to_idx.get(from).unwrap(),
                    *elem_to_idx.get(to).unwrap(),
                );
            }
        }
        (graph, elem_to_idx)
    }

    // Calculate and set last_affected_dates from last_updates.
    pub fn set_last_affected_dates(&mut self) {
        self.last_affected_dates = Default::default();
        let (imported_graph, mod_to_node) = self.importing_module_graph();
        for module in &self.linked_mods {
            let mut last_affected = self.last_updates.get(module).unwrap().clone();
            let imported_modules =
                imported_graph.reachable_nodes(*mod_to_node.get(module).unwrap());
            for imported_module in imported_modules {
                let imported_module = imported_graph.get(imported_module);
                last_affected = last_affected.max(self.last_updates.get(imported_module).unwrap());
            }
            self.last_affected_dates
                .insert(module.clone(), last_affected);
        }
    }
}
