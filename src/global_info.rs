use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub static IS_OBJECTIVE: AtomicBool = AtomicBool::new(false);
pub static IS_CMP_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_DATAFLOW_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_INSTRUCTION_INTERESTING: AtomicI32 = AtomicI32::new(0);

pub(crate) fn print_global_vars() {
    let is_objective = IS_OBJECTIVE.load(Ordering::SeqCst);
    let is_cmp_interesting = IS_CMP_INTERESTING.load(Ordering::SeqCst);
    let is_dataflow_interesting = IS_DATAFLOW_INTERESTING.load(Ordering::SeqCst);
    let is_instruction_interesting = IS_INSTRUCTION_INTERESTING.load(Ordering::SeqCst);

    println!("is_objective: {}", is_objective);
    println!("is_cmp_interesting: {}", is_cmp_interesting);
    println!("is_dataflow_interesting: {}", is_dataflow_interesting);
    println!("is_instruction_interesting: {}", is_instruction_interesting);
}

pub fn get_global_vars() -> [bool; 4] {
    let is_objective = IS_OBJECTIVE.load(Ordering::SeqCst);
    let is_cmp_interesting = IS_CMP_INTERESTING.load(Ordering::SeqCst);
    let is_dataflow_interesting = IS_DATAFLOW_INTERESTING.load(Ordering::SeqCst);
    let is_instruction_interesting = IS_INSTRUCTION_INTERESTING.load(Ordering::SeqCst) != 0;

    [is_objective, is_cmp_interesting, is_dataflow_interesting, is_instruction_interesting]
}