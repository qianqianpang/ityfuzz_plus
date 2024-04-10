use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::collections::HashMap;
use lazy_static::lazy_static;


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



lazy_static! {
    pub static ref P_TABLE: HashMap<&'static str, HashMap<&'static str, f64>> = {
        let mut table = HashMap::new();
        let mut inner_map = HashMap::new();
        inner_map.insert("T256_ADDRESS_RANDOM", 0.5);
        inner_map.insert("T256_ADDRESS_SELF", 0.5);
        table.insert("T256_ADDRESS", inner_map);
        table
    };
}

pub fn select_max_probability_action(p_table: &HashMap<&'static str, HashMap<&'static str, f64>>, action_type: &'static str) -> &'static str {
    let action_map = p_table.get(action_type).unwrap();
    let max_probability_action = action_map.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
    max_probability_action.0
}