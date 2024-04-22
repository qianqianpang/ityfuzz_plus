use std::sync::atomic::{AtomicBool, AtomicI32, Ordering, AtomicUsize, AtomicI64};
use std::collections::HashMap;
use lazy_static::lazy_static;
use rand::Rng;
use std::sync::Mutex;
pub static IS_OBJECTIVE: AtomicBool = AtomicBool::new(false);
pub static IS_CMP_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_DATAFLOW_INTERESTING: AtomicBool = AtomicBool::new(false);
pub static IS_INSTRUCTION_INTERESTING: AtomicI32 = AtomicI32::new(0);
pub static VALUE: AtomicI32 = AtomicI32::new(0);




//超参数
pub static mut RANDOM_P: f64 = 0.5;
const A: f64 = 3.0;
const B: f64 = 2.0;
const C: f64 = 1.0;
const D: f64 = 1.0;
//feedback_info
pub(crate) fn print_feedback_info() {
    let is_objective = IS_OBJECTIVE.load(Ordering::SeqCst);
    let is_cmp_interesting = IS_CMP_INTERESTING.load(Ordering::SeqCst);
    let is_dataflow_interesting = IS_DATAFLOW_INTERESTING.load(Ordering::SeqCst);
    let is_instruction_interesting = IS_INSTRUCTION_INTERESTING.load(Ordering::SeqCst);

    println!("is_objective: {}", is_objective);
    println!("is_cmp_interesting: {}", is_cmp_interesting);
    println!("is_dataflow_interesting: {}", is_dataflow_interesting);
    println!("is_instruction_interesting: {}", is_instruction_interesting);
}

pub fn get_feedback_info() -> [bool; 4] {
    let is_objective = IS_OBJECTIVE.load(Ordering::SeqCst);
    let is_cmp_interesting = IS_CMP_INTERESTING.load(Ordering::SeqCst);
    let is_dataflow_interesting = IS_DATAFLOW_INTERESTING.load(Ordering::SeqCst);
    let is_instruction_interesting = IS_INSTRUCTION_INTERESTING.load(Ordering::SeqCst) != 0;

    [is_objective, is_cmp_interesting, is_dataflow_interesting, is_instruction_interesting]
}


//ptable
lazy_static! {
    //这里的初始值，也可以随机初始化  为和=1的值
    pub static ref P_TABLE: Mutex<HashMap<&'static str, HashMap<&'static str, f64>>> = {
        let mut table = HashMap::new();

         // 添选择变异env  or abi-args
        let mut input_mutate_map = HashMap::new();
        input_mutate_map.insert("INPUT_MUTATE_ENV", 0.05);
        input_mutate_map.insert("INPUT_MUTATE_ARGS", 0.95);
        table.insert("INPUT_MUTATE", input_mutate_map);

        //变异ABI-args
        let mut t256_address_map = HashMap::new();
        t256_address_map.insert("T256_ADDRESS_RANDOM", 0.633368429490994);
        t256_address_map.insert("T256_ADDRESS_SELF", 0.36663157050900597);
        table.insert("T256_ADDRESS", t256_address_map);

        let mut tarray_dynamic_map = HashMap::new();
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_RANDOM", 0.28600704563912777);
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_INCREASE", 0.34270570908252);
        tarray_dynamic_map.insert("TARRAY_DYNAMIC_DECREASE", 0.3712872452783522);
        table.insert("TARRAY_DYNAMIC", tarray_dynamic_map);

        let mut tunknown = HashMap::new();
        tunknown.insert("TUNKNOWN_SLOT", 0.5275115121939853);
        tunknown.insert("TUNKNOWN_ABI", 0.4724884878060147);
        table.insert("TUNKNOWN", tunknown);

        //变异env  10
        let mut env_map = HashMap::new();
        env_map.insert("ENV_CALLER", 0.10628936761790735);
        env_map.insert("ENV_BALANCE", 0.012087685458521048);
        env_map.insert("ENV_GASPRICE", 0.024347533568454243);
        env_map.insert("ENV_BASEFEE", 0.16699855358421933);
        env_map.insert("ENV_TIMESTAMP", 0.06928435516295682);
        env_map.insert("ENV_COINBASE", 0.10595742814793209);
        env_map.insert("ENV_GASLIMIT", 0.006449580092650405);
        env_map.insert("ENV_NUMBER", 0.16210237119036905);
        env_map.insert("ENV_CALLVALUE", 0.06042721599214044);
        env_map.insert("ENV_PREVRANDAO", 0.2196002076744771);
        env_map.insert("ENV_DIFFICULTY", 0.06645570151037233);
        table.insert("ENV", env_map);


        //具体的byte_mutations操作  17
        let mut byte_mutations_map = HashMap::new();
        byte_mutations_map.insert("ByteRandMutator", 0.04859009581991758);
        byte_mutations_map.insert("ByteInterestingMutator", 0.10802973675191048);
        byte_mutations_map.insert("DwordAddMutator", 0.04342976642042233);
        byte_mutations_map.insert("ByteNegMutator", 0.08530105944006433);
        byte_mutations_map.insert("ByteIncMutator", 0.07497920490450842);
        byte_mutations_map.insert("ByteAddMutator", 0.11014066253434186);
        byte_mutations_map.insert("ByteFlipMutator", 0.020403173323224978);
        byte_mutations_map.insert("ByteDecMutator", 0.09477373713374436);
        byte_mutations_map.insert("BytesSwapMutator", 0.021522091252595292);
        byte_mutations_map.insert("BitFlipMutator", 0.06271614938134926);
        byte_mutations_map.insert("ConstantHintedMutator", 0.040649101030637465);
        byte_mutations_map.insert("BytesRandSetMutator", 0.040372126465043574);
        byte_mutations_map.insert("WordInterestingMutator", 0.030164440054414102);
        byte_mutations_map.insert("DwordInterestingMutator", 0.11150981503928818);
        byte_mutations_map.insert("BytesSetMutator", 0.04896178921838225);
        byte_mutations_map.insert("WordAddMutator", 0.015502141527304095);
        byte_mutations_map.insert("QwordAddMutator", 0.04295490970285134);
        table.insert("BYTE_MUTATIONS", byte_mutations_map);
        //20
        let mut byte_mutations_expansion_map = HashMap::new();
        byte_mutations_expansion_map.insert("BitFlipMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteFlipMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteIncMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteDecMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteNegMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteRandMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteAddMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("WordAddMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("DwordAddMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("QwordAddMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("ByteInterestingMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("WordInterestingMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("DwordInterestingMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesSetMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesRandSetMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesSwapMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesCopyMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesExpandMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesInsertMutator", 1.0 / 20.0);
        byte_mutations_expansion_map.insert("BytesRandInsertMutator", 1.0 / 20.0);
        table.insert("BYTE_MUTATIONS_EXPANSION", byte_mutations_expansion_map);

        //选择mutate with template,state,data,bibao
        let mut mutate_template_map = HashMap::new();
        mutate_template_map.insert("USE_TEMPLATE", 0.2972036085374526);
        mutate_template_map.insert("NOT_USE", 0.7027963914625475);
        table.insert("MUTATE_TEMPLATE", mutate_template_map);

        let mut mutate_state_map = HashMap::new();
        mutate_state_map.insert("USE_STATE", 0.49718431275861);
        mutate_state_map.insert("NOT_USE", 0.5028156872413899);
        table.insert("MUTATE_STATE", mutate_state_map);

        let mut mutate_data_map = HashMap::new();
        mutate_data_map.insert("USE_DATA", 0.1656941526078328);
        mutate_data_map.insert("NOT_USE", 0.8343058473921672);
        table.insert("MUTATE_DATA", mutate_data_map);

        let mut mutate_byte_map = HashMap::new();
        mutate_byte_map.insert("MUTATE_LIQUIDATE", 0.23296299140488302);
        mutate_byte_map.insert("MUTATE_NORMAL", 0.7670370085951169);
        table.insert("MUTATE_BYTE", mutate_byte_map);

         let mut mutate_borrow_map = HashMap::new();
        mutate_borrow_map.insert("MUTATE_RANDOMNESS", 0.46078431249802076);
        mutate_borrow_map.insert("MUTATE_NORMAL", 0.5392156875019793);
        table.insert("MUTATE_BORROW", mutate_borrow_map);

        let mut mutate_all_map = HashMap::new();
        mutate_all_map.insert("MUTATE_LIQUIDATION", 0.1963891872557437);
        mutate_all_map.insert("MUTATE_RANDOMNESSL", 0.4344037307380694);
        mutate_all_map.insert("MUTATE_NORMAL", 0.36920708200618685);
        table.insert("MUTATE_ALL", mutate_all_map);

        Mutex::new(table)
    };
}


pub fn select_mutation_action(p_table: &Mutex<HashMap<&'static str, HashMap<&'static str, f64>>>, action_type: &'static str, p: f64) -> &'static str {
    let p_table = p_table.lock().unwrap();
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

pub fn print_p_table() {
    println!("================P_TABLE======");
    let p_table = P_TABLE.lock().unwrap();
    for (key, value_map) in p_table.iter() {
        println!("{}:", key);
        for (sub_key, sub_value) in value_map.iter() {
            println!("     {}: {}", sub_key, sub_value);
        }
    }
}

pub fn reset_p_table() {//置为均值
    let mut p_table = P_TABLE.lock().unwrap();
    for (_key, value_map) in p_table.iter_mut() {
        let equal_value = 1.0 / value_map.len() as f64;
        for (_sub_key, sub_value) in value_map.iter_mut() {
            *sub_value = equal_value;
        }
    }
}

// 生成一个长度为n的随机概率向量，这个向量的元素之和为1
fn generate_random_probabilities(n: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut probabilities: Vec<f64> = (0..n).map(|_| rng.gen::<f64>()).collect();
    let sum: f64 = probabilities.iter().sum();
    probabilities.iter_mut().for_each(|x| *x /= sum);
    probabilities
}

// 置为随机值
pub fn reset_p_table2() {
    let mut p_table = P_TABLE.lock().unwrap();
    for (_key, value_map) in p_table.iter_mut() {
        let probabilities = generate_random_probabilities(value_map.len());
        for ((_sub_key, sub_value), &probability) in value_map.iter_mut().zip(probabilities.iter()) {
            *sub_value = probability;
        }
    }
}
//MUTATION_OP
lazy_static! {
    pub static ref MUTATION_OP: Mutex<HashMap<&'static str, HashMap<&'static str, i32>>> = {
        let mut table = HashMap::new();

        let p_table = P_TABLE.lock().unwrap();
        for (key, value_map) in p_table.iter() {
            let mut inner_map = HashMap::new();
            for sub_key in value_map.keys() {
                inner_map.insert(*sub_key, 0);
            }
            table.insert(*key, inner_map);
        }

        Mutex::new(table)
    };
}
pub fn print_mutation_op() {
    println!("================MUTATION_OP======");
    let mutation_op = MUTATION_OP.lock().unwrap();
    for (key, value_map) in mutation_op.iter() {
        println!("{}:", key);
        for (sub_key, sub_value) in value_map.iter() {
            println!("     {}: {}", sub_key, sub_value);
        }
    }
}

pub fn increment_mutation_op(key: &'static str, sub_key: &'static str) {
    let mut mutation_op = MUTATION_OP.lock().unwrap();
    if let Some(value_map) = mutation_op.get_mut(key) {
        if let Some(value) = value_map.get_mut(sub_key) {
            *value += 1;
        }
    }
}
//value相关函数
pub fn store_value(value: i32) {
    VALUE.store(value, Ordering::SeqCst);
}

pub fn load_value() -> i32 {
    VALUE.load(Ordering::SeqCst)
}
pub fn calculate_value() {
    let is_objective = IS_OBJECTIVE.load(Ordering::SeqCst) as i32;
    let is_cmp_interesting = IS_CMP_INTERESTING.load(Ordering::SeqCst) as i32;
    let is_dataflow_interesting = IS_DATAFLOW_INTERESTING.load(Ordering::SeqCst) as i32;
    let is_instruction_interesting = IS_INSTRUCTION_INTERESTING.load(Ordering::SeqCst) as i32;

    let value = A as i32 * is_objective
        + B as i32 * is_cmp_interesting
        + C as i32 * is_dataflow_interesting
        + D as i32 * is_instruction_interesting;

    VALUE.store(value, Ordering::SeqCst);
}

pub fn get_value() -> i32 {
    VALUE.load(Ordering::SeqCst)
}

pub fn print_value() {
    let value = VALUE.load(Ordering::SeqCst);
    println!("Value============================================================================: {}", value);
}
//更新table
pub fn adjust_p_table() {
    let value = get_value();
    if value > 1 {
        let mutation_op = MUTATION_OP.lock().unwrap();
        let mut p_table = P_TABLE.lock().unwrap();
        for (key, value_map) in mutation_op.iter() {
            for (sub_key, sub_value) in value_map.iter() {
                if *sub_value != 0 {
                    if let Some(p_value_map) = p_table.get_mut(*key) {
                        if let Some(p_value) = p_value_map.get_mut(*sub_key) {
                            *p_value += 0.1; // 增加10%的概率
                            if *p_value > 1.0 {
                                *p_value = 1.0; // 确保概率不超过1
                            }
                        }
                    }
                }
            }
        }
    }
    VALUE.store(0, Ordering::SeqCst);

    // Reset all values in MUTATION_OP to 0
    let mut mutation_op = MUTATION_OP.lock().unwrap();
    for (_key, value_map) in mutation_op.iter_mut() {
        for (_sub_key, sub_value) in value_map.iter_mut() {
            *sub_value = 0;
        }
    }
}
pub(crate) static MUTATE_SUCCESS_COUNT: AtomicUsize = AtomicUsize::new(0);