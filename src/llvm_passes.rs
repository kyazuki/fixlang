// This source file is generated by by passes_optimizer.py.

use super::*;
use inkwell::passes::PassManagerSubType;

pub fn add_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
    passmgr.add_early_cse_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_global_optimizer_pass();
    passmgr.add_aggressive_dce_pass();
    passmgr.add_loop_unroll_and_jam_pass();
    passmgr.add_ipsccp_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_early_cse_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_loop_deletion_pass();
    passmgr.add_constant_merge_pass();
    passmgr.add_strip_symbol_pass();
    passmgr.add_dead_store_elimination_pass();
    passmgr.add_cfg_simplification_pass();
    passmgr.add_constant_merge_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_instruction_combining_pass();
    passmgr.add_scalar_repl_aggregates_pass();
    passmgr.add_tail_call_elimination_pass();
    passmgr.add_promote_memory_to_register_pass();
    passmgr.add_dead_store_elimination_pass();
    passmgr.add_jump_threading_pass();
    passmgr.add_ipsccp_pass();
    passmgr.add_loop_rotate_pass();
    passmgr.add_loop_vectorize_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_always_inliner_pass();
    passmgr.add_strip_dead_prototypes_pass();
    passmgr.add_always_inliner_pass();
    passmgr.add_bit_tracking_dce_pass();
    passmgr.add_always_inliner_pass();
    passmgr.add_strip_dead_prototypes_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_scoped_no_alias_aa_pass();
    passmgr.add_ind_var_simplify_pass();
    passmgr.add_loop_unroll_pass();
    passmgr.add_slp_vectorize_pass();
    passmgr.add_sccp_pass();
    passmgr.add_dead_store_elimination_pass();
    passmgr.add_aggressive_dce_pass();
    passmgr.add_lower_expect_intrinsic_pass();
    passmgr.add_ipsccp_pass();
    passmgr.add_simplify_lib_calls_pass();
    passmgr.add_instruction_combining_pass();
    passmgr.add_ind_var_simplify_pass();
    passmgr.add_constant_merge_pass();
    passmgr.add_correlated_value_propagation_pass();
    passmgr.add_loop_reroll_pass();
    passmgr.add_aggressive_inst_combiner_pass();
}
