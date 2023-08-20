use build_time::build_time_utc;
use std::{env, fs, fs::create_dir_all, path::PathBuf, process::Command, time::SystemTime};

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    passes::{PassManager, PassManagerSubType},
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use super::*;

fn execute_main_module<'c>(ee: &ExecutionEngine<'c>, config: &Configuration) -> i32 {
    // If sanitize_memory, load `libfixsanitizer.so`.
    if config.sanitize_memory {
        let path = "./sanitizer/libfixsanitizer.so";
        let err = load_library_permanently(path);
        if err {
            error_exit(&format!("Failed to load \"{}\".", path));
        }
    }
    // Load runtime library.
    let build_time_hash = format!("{:x}", md5::compute(build_time_utc!()));
    let runtime_so_path =
        PathBuf::from(INTERMEDIATE_PATH).join(format!("libfixruntime_{}.so", build_time_hash));
    if !runtime_so_path.exists() {
        let runtime_c_path = PathBuf::from(INTERMEDIATE_PATH).join("fixruntime.c");
        fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate directory.");
        fs::write(&runtime_c_path, include_str!("runtime.c"))
            .expect(&format!("Failed to generate runtime.c"));
        // Create library binary file.
        Command::new("cc")
            .arg("-shared")
            .arg("-fpic")
            .arg("-o")
            .arg(runtime_so_path.to_str().unwrap())
            .arg(runtime_c_path.to_str().unwrap())
            .output()
            .expect("Failed to run cc.");
    }
    load_library_permanently(runtime_so_path.to_str().unwrap());

    // Load dynamically linked libraries specified by user.
    for (lib_name, _) in &config.linked_libraries {
        let lib_name = format!("lib{}.so", lib_name);
        let err = load_library_permanently(&lib_name);
        if err {
            error_exit(&format!("Failed to load \"{}\".", lib_name));
        }
    }
    unsafe {
        let func = ee
            .get_function::<unsafe extern "C" fn() -> i32>("main")
            .unwrap();
        func.call()
    }
}

fn build_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    target: Either<TargetMachine, ExecutionEngine<'c>>,
    mut fix_mod: Program,
    config: Configuration,
) -> Either<TargetMachine, ExecutionEngine<'c>> {
    // Calculate last affected dates.
    fix_mod.set_last_affected_dates();

    // Calculate list of type constructors.
    fix_mod.calculate_type_env();

    // Infer namespaces of traits and types that appear in declarations (not in expressions).
    fix_mod.resolve_namespace_in_declaration();

    // Resolve type aliases that appear in declarations (not in expressions).
    fix_mod.resolve_type_aliases_in_declaration();

    // Validate user-defined types.
    fix_mod.validate_type_defns();

    // Add struct / union methods
    fix_mod.add_methods();

    // Validate trait env.
    fix_mod.validate_trait_env();

    // Create symbols.
    fix_mod.create_trait_method_symbols();

    // Set and check kinds that appear in the module.
    fix_mod.set_kinds();

    // Create typeckecker.
    let mut typechecker = TypeCheckContext::new(
        fix_mod.trait_env.clone(),
        fix_mod.type_env(),
        fix_mod.visible_mods.clone(),
    );

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &fix_mod.global_values {
        typechecker
            .scope
            .add_global(name.name.clone(), &name.namespace, &defn.scm);
    }

    // Instantiate main function and all called functions.
    let main_expr = fix_mod.instantiate_main_function(&typechecker);

    // Perform uncurrying optimization.
    if config.uncurry_optimization {
        uncurry_optimization(&mut fix_mod);
    }

    // Create GenerationContext.
    let mut gc = GenerationContext::new(&context, &module, target, config, fix_mod.type_env());

    // Build runtime functions.
    build_runtime(&mut gc);

    // Generate codes.
    fix_mod.generate_code(&mut gc);

    // Add main function.
    let main_fn_type = context.i32_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Run main object.
    let main_obj = gc.eval_expr(main_expr, None); // `IO ()`
    let main_lambda_val = main_obj.load_field_nocap(&mut gc, 0);
    let main_lambda_ty = type_fun(make_tuple_ty(vec![]), make_tuple_ty(vec![]));
    let main_lambda = Object::create_from_value(main_lambda_val, main_lambda_ty, &mut gc);
    let unit = allocate_obj(
        make_tuple_ty(vec![]),
        &vec![],
        None,
        &mut gc,
        Some("unit_for_main_io"),
    );
    let ret = gc.apply_lambda(main_lambda, vec![unit], None);
    gc.release(ret);

    // Perform leak check
    if gc.config.sanitize_memory {
        // Deallocate all global objects.
        let mut global_names = vec![];
        for (name, _) in &gc.global {
            global_names.push(name.clone());
        }
        for name in global_names {
            let obj = gc.get_var(&name).ptr.get(&gc);
            gc.release(obj);
        }

        gc.call_runtime(RuntimeFunctions::CheckLeak, &[]);
    }

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)));

    // Print LLVM bitcode to file
    // module.print_to_file("main.ll").unwrap();

    // Run optimization
    let passmgr = PassManager::create(());

    passmgr.add_verifier_pass();
    add_passes(&passmgr);

    passmgr.run_on(module);
    unsafe {
        module.run_in_pass_manager(&passmgr);
    }

    // Verify LLVM module.
    // Maybe not needed at now?
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }

    gc.target
}

#[allow(dead_code)]
pub fn run_source(source: &str, config: Configuration) -> i32 {
    let mut target_mod = make_std_mod();

    let source_mod = parse_source(source, "{filename unspecified}");
    target_mod.link(source_mod);
    target_mod.resolve_imports();

    run_module(target_mod, config)
}

pub fn run_module(fix_mod: Program, config: Configuration) -> i32 {
    let ctx = Context::create();
    let module = ctx.create_module("Main");
    let ee = module
        .create_jit_execution_engine(config.llvm_opt_level)
        .unwrap();
    let ee = build_module(&ctx, &module, Either::Right(ee), fix_mod, config.clone()).unwrap_right();
    execute_main_module(&ee, &config)
}

// Return file content and last modified.
pub fn read_file(path: &Path) -> (String, UpdateDate) {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        Ok(_) => (),
    }
    let last_modified: Option<UpdateDate> = match file.metadata() {
        Err(why) => {
            println!("Failed to get last modified date of {}: {}", display, why);
            None
        }
        Ok(md) => match md.modified() {
            Err(why) => {
                println!("Failed to get last modified date of {}: {}", display, why);
                None
            }
            Ok(time) => Some(UpdateDate(time.into())),
        },
    };
    if last_modified.is_none() {
        println!("Build cache for {} will be ignored", display);
    }
    let last_modified = last_modified.unwrap_or(UpdateDate(SystemTime::now().into()));
    (s, last_modified)
}

// Create a directory if it doesn't exist, and return its path.
pub fn touch_directory<P>(rel_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let cur_dir = match env::current_dir() {
        Err(why) => panic!("Failed to get current directory: {}", why),
        Ok(dir) => dir,
    };
    let res = cur_dir.join(rel_path);
    match create_dir_all(&res) {
        Err(why) => panic!("Failed to create directory {}: {}", res.display(), why),
        Ok(_) => {}
    };
    res
}

pub fn load_file(config: &Configuration) -> Program {
    // Link all modules specified in source_files.
    let mut target_mod = make_std_mod();
    for file_path in &config.source_files {
        let (content, last_modified) = read_file(file_path);
        let mut fix_mod = parse_source(&content, file_path.to_str().unwrap());
        fix_mod.set_last_update(fix_mod.get_name_if_single_module(), last_modified);
        target_mod.link(fix_mod);
    }
    target_mod.resolve_imports();
    target_mod
}

pub fn run_file(config: Configuration) -> i32 {
    run_module(load_file(&config), config)
}

fn get_target_machine(opt_level: OptimizationLevel) -> TargetMachine {
    let _native = Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| error_exit(&format!("failed to initialize native: {}", e)))
        .unwrap();
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| {
            error_exit(&format!("failed to create target: {}", e));
        })
        .unwrap();
    let cpu_name = TargetMachine::get_host_cpu_name();
    let target_machine = target.create_target_machine(
        &triple,
        cpu_name.to_str().unwrap(),
        TargetMachine::get_host_cpu_features().to_str().unwrap(),
        opt_level,
        RelocMode::Default,
        CodeModel::Default,
    );
    match target_machine {
        Some(tm) => tm,
        None => error_exit("Failed to creeate target machine."),
    }
}

pub fn build_file(config: Configuration) {
    let obj_path = PathBuf::from(INTERMEDIATE_PATH).join("a.o");
    let exec_path = PathBuf::from("a.out");

    // Create intermediate directory.
    fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate .");

    let tm = get_target_machine(config.llvm_opt_level);

    let fix_mod = load_file(&config);

    let ctx = Context::create();
    let module = ctx.create_module("Main");
    module.set_triple(&tm.get_triple());
    module.set_data_layout(&tm.get_target_data().get_data_layout());

    let tm = build_module(&ctx, &module, Either::Left(tm), fix_mod, config.clone()).unwrap_left();
    tm.write_to_file(&module, inkwell::targets::FileType::Object, &obj_path)
        .map_err(|e| error_exit(&format!("Failed to write to file: {}", e)))
        .unwrap();

    let mut libs_opts = vec![];
    for (lib_name, link_type) in &config.linked_libraries {
        match link_type {
            LinkType::Static => libs_opts.push("-Wl,-Bstatic".to_string()),
            LinkType::Dynamic => libs_opts.push("-Wl,-Bdynamic".to_string()),
        }
        libs_opts.push(format!("-l{}", lib_name));
    }

    // Build runtime.c to object file.
    let build_time_hash = format!("{:x}", md5::compute(build_time_utc!()));
    let runtime_obj_path =
        PathBuf::from(INTERMEDIATE_PATH).join(format!("fixruntime_{}.o", build_time_hash));
    if !runtime_obj_path.exists() {
        let runtime_c_path = PathBuf::from(INTERMEDIATE_PATH).join("fixruntime.c");
        fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate directory.");
        fs::write(&runtime_c_path, include_str!("runtime.c"))
            .expect(&format!("Failed to generate runtime.c"));
        // Create library object file.
        let output = Command::new("cc")
            .arg("-o")
            .arg(runtime_obj_path.to_str().unwrap())
            .arg("-c")
            .arg(runtime_c_path.to_str().unwrap())
            .output()
            .expect("Failed to call cc.");
        if output.stderr.len() > 0 {
            eprintln!("{:?}", String::from_utf8(output.stderr).unwrap());
        }
    }

    let output = Command::new("cc")
        .args(libs_opts)
        .arg("-o")
        .arg(exec_path.to_str().unwrap())
        .arg(obj_path.to_str().unwrap())
        .arg(runtime_obj_path.to_str().unwrap())
        .output()
        .expect("Failed to call cc.");
    if output.stderr.len() > 0 {
        eprintln!("{:?}", String::from_utf8(output.stderr).unwrap());
    }
}
