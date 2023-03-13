import subprocess
import re
import itertools
import sys
import random

# Optimizes llvm optimization passes by updating llvm_passes.rs.
SOURCE_FILE = './src/llvm_passes.rs'

INITIAL_PASSES = '''
    add_scalar_repl_aggregates_pass
    add_tail_call_elimination_pass
    add_function_inlining_pass
    add_global_optimizer_pass
    add_ipsccp_pass
    add_strip_dead_prototypes_pass
    add_ind_var_simplify_pass
    add_global_dce_pass
    add_promote_memory_to_register_pass
    add_dead_store_elimination_pass
'''
INITIAL_PASSES = INITIAL_PASSES.split('\n')
INITIAL_PASSES = [line.strip() for line in INITIAL_PASSES]
INITIAL_PASSES = [line for line in INITIAL_PASSES if len(line) > 0]

HEADER = '''
// This source file is generated by by passes_optimizer.py.

use super::*;
use inkwell::passes::PassManagerSubType;

pub fn add_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
'''

FOOTER = '''
}
'''

ADD_PASS_FORMAT = 'passmgr.{}();'

FIX_SOURCE_FILE = './examples/prime_loop' # without extension

RUN_BENCH_ITERATION = 10

ADDED_PASSES_NUM = 10

REQUIRED_IMPROVEMENT = 0.97

# All passes
# Exclude:
#  add_scalar_repl_aggregates_pass_with_threshold, 
#  add_internalize_pass, add_gvn_pass (segfaults), 
#  add_instruction_combining_pass (segfaults), 
#  add_memcpy_optimize_pass (segfaults)
#  add_new_gvn_pass (breaks progream)
#  add_licm_pass (breaks progream)
PASSES = '''
add_aggressive_dce_pass
add_aggressive_inst_combiner_pass
add_alignment_from_assumptions_pass
add_always_inliner_pass
add_basic_alias_analysis_pass
add_bit_tracking_dce_pass
add_cfg_simplification_pass
add_constant_merge_pass
add_correlated_value_propagation_pass
add_dead_arg_elimination_pass
add_dead_store_elimination_pass
add_demote_memory_to_register_pass
add_early_cse_mem_ssa_pass
add_early_cse_pass
add_function_attrs_pass
add_function_inlining_pass
add_global_dce_pass
add_global_optimizer_pass
add_ind_var_simplify_pass
add_instruction_simplify_pass
add_ipsccp_pass
add_jump_threading_pass
add_loop_deletion_pass
add_loop_idiom_pass
add_loop_reroll_pass
add_loop_rotate_pass
add_loop_unroll_and_jam_pass
add_loop_unroll_pass
add_loop_vectorize_pass
add_lower_expect_intrinsic_pass
add_lower_switch_pass
add_merge_functions_pass
add_merged_load_store_motion_pass
add_partially_inline_lib_calls_pass
add_promote_memory_to_register_pass
add_prune_eh_pass
add_reassociate_pass
add_scalar_repl_aggregates_pass
add_scalar_repl_aggregates_pass_ssa
add_scalarizer_pass
add_sccp_pass
add_scoped_no_alias_aa_pass
add_simplify_lib_calls_pass
add_slp_vectorize_pass
add_strip_dead_prototypes_pass
add_strip_symbol_pass
add_tail_call_elimination_pass
add_type_based_alias_analysis_pass
'''

def get_all_passes():
    passes = []
    for p in PASSES.split('\n'):
        if len(p.strip()) > 0:
            passes.append(p)
    return passes
    

def write_source_file(passes):
    with open(SOURCE_FILE, 'w') as f:
        f.write(HEADER)
        f.write('\n')
        for p in passes:
            f.write(ADD_PASS_FORMAT.format(p))
            f.write('\n')
        f.write(FOOTER)
        f.write('\n')

def run_benchmark(run_bench_iteration=RUN_BENCH_ITERATION):
    cp = subprocess.run(['cargo', 'run', '--', 'build', FIX_SOURCE_FILE + '.fix'], capture_output = True, text = True)
    if cp.returncode != 0:
        print('build failed.')
        print('stdout:')
        print(cp.stdout)
        print('stderr:')
        print(cp.stderr)
        sys.exit(1)

    cp = subprocess.run(['gcc', FIX_SOURCE_FILE + '.o'], capture_output = True, text = True)
    if cp.returncode != 0:
        print('gcc failed.')
        print('stdout:')
        print(cp.stdout)
        print('stderr:')
        print(cp.stderr)
        sys.exit(1)

    total_time = 0.0
    for _ in range(run_bench_iteration):
        cp = subprocess.run(['time', '-f', '%e', './a.out'], capture_output = True, text = True)
        if cp.returncode != 0:
            print('run failed.')
            print('stdout:')
            print(cp.stdout)
            print('stderr:')
            print(cp.stderr)
            total_time = 100000.0 * run_bench_iteration  # return a long time
            break
        else:
            total_time += float(cp.stderr)

    # parse stdout and get read time.
    return total_time / run_bench_iteration

def print_passes(passes):
    for p in passes:
        print('  ' + p)

def optimize():
    all_passes = get_all_passes()
    optimum_passes = INITIAL_PASSES.copy()

    print('Initial passes:')
    print_passes(optimum_passes)
    
    write_source_file(optimum_passes)
    optimum_time = run_benchmark()
    print('Time with initial passes: {}'.format(optimum_time))

    while True:
        added_passes_count = random.randint(1, ADDED_PASSES_NUM)
        added_passes = []
        for _ in range(added_passes_count):
            idx = random.randint(0, len(all_passes)-1)
            added_passes.append(all_passes[idx])
        print('Try adding passes:')
        print_passes(added_passes)
        passes = optimum_passes.copy()
        for p in added_passes:
            passes.append(p)
        write_source_file(passes)
        time = run_benchmark()
        if time <= optimum_time * REQUIRED_IMPROVEMENT:
            optimum_passes = passes
            optimum_time = time
            print('New optimum passes found!')
        else:
            print('No improvement found. Time: {}'.format(time))

        # minimize passes
        minimized = []
        for p in optimum_passes:
            if random.randint(0,1) == 0:
                minimized.append(p)
        write_source_file(minimized)
        minimized_time = run_benchmark()
        if minimized_time <= optimum_time:
            optimum_passes = minimized
            print("Minimize success!")

        print('Current optimum passes:')
        print_passes(optimum_passes)
        write_source_file(optimum_passes)
        optimum_time = run_benchmark()
        print('Current optimum time: {}'.format(optimum_time))

if __name__ == '__main__':
    optimize()