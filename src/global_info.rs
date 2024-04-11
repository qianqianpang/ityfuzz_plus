use std::sync::atomic::{AtomicBool, AtomicI32, Ordering, AtomicUsize};
use std::collections::HashMap;
use lazy_static::lazy_static;
use rand::Rng;
pub static IS_OBJECTIVE: AtomicBool = AtomicBool::new(false);
pub static IS_CMP_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_DATAFLOW_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_INSTRUCTION_INTERESTING: AtomicI32 = AtomicI32::new(0);
//超参数
pub static mut RANDOM_P: f64 = 0.7;

// unsafe {
// P = 0.5;
// }
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
    //这里的初始值，也可以随机初始化  为和=1的值
    pub static ref P_TABLE: HashMap<&'static str, HashMap<&'static str, f64>> = {
        let mut table = HashMap::new();

        let mut t256_address_map = HashMap::new();
        t256_address_map.insert("T256_ADDRESS_RANDOM", 0.5);
        t256_address_map.insert("T256_ADDRESS_SELF", 0.5);
        table.insert("T256_ADDRESS", t256_address_map);

        let mut tarray_dynamic_map = HashMap::new();
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_RANDOM", 0.4);
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_INCREASE", 0.3);
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_DECREASE", 0.3);
        table.insert("TARRAY_DYNAMIC", tarray_dynamic_map);

        let mut tunknown = HashMap::new();
        tunknown.insert("TUNKNOWN_SLOT", 0.5);
        tunknown.insert("TUNKNOWN_ABI", 0.3);
        table.insert("TUNKNOWN", tunknown);

        table
    };
}


pub fn select_mutation_action(p_table: &HashMap<&'static str, HashMap<&'static str, f64>>, action_type: &'static str, p: f64) -> &'static str {
    let action_map = p_table.get(action_type).unwrap();
    let mut rng = rand::thread_rng();
    let random_number: f64 = rng.gen(); // generates a float between 0 and 1

    if random_number < p {
        // select the action with the maximum probability
        let max_probability_action = action_map.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
        max_probability_action.0
    } else {
        // select a random action
        let actions: Vec<&&str> = action_map.keys().collect();
        let random_action = actions[rng.gen_range(0..actions.len())];
        *random_action
    }
}




pub(crate) static MUTATE_SUCCESS_COUNT: AtomicUsize = AtomicUsize::new(0);