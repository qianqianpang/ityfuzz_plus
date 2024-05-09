use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use bytes::Bytes;
use lazy_static::lazy_static;
use rand::Rng;
use revm_primitives::Env;
use tch::{Kind, nn, nn::Module, nn::Optimizer, nn::OptimizerConfig, Tensor};

use crate::evm::input::{EVMInput, EVMInputTy};
use crate::evm::mutator::AccessPattern;
use crate::evm::types::EVMAddress;
use crate::global_info::get_value;
use crate::state_input::StagedVMState;

lazy_static! {
    pub static ref GLOBAL_INPUT: Mutex<EVMInput> = Mutex::new(EVMInput {
        input_type: EVMInputTy::ABI,
        caller: EVMAddress::default(),
        contract: EVMAddress::default(),
        data: None,
        sstate: StagedVMState::default(),
        sstate_idx: 0,
        txn_value: None,
        step: false,
        env: Env::default(),
        access_pattern: Arc::new(Mutex::new(AccessPattern::default())),
        liquidation_percent: 0,
        direct_data: Bytes::new(),
        randomness: Vec::new(),
        repeat: 0,
        swap_data: HashMap::new(),
    });
}

pub fn get_global_input() -> EVMInput {
    GLOBAL_INPUT.lock().unwrap().clone()
}

pub fn set_global_input(new_input: EVMInput) {
    *GLOBAL_INPUT.lock().unwrap() = new_input;
}

//GLOBAL_MUTATION==========================================================================================================================
lazy_static! {
    pub static ref GLOBAL_MUTATION: Mutex<i64> = Mutex::new(0);
}

pub fn set_global_mutation(value: i64) {
    let mut global_mutation = GLOBAL_MUTATION.lock().unwrap();
    *global_mutation = value;
}
lazy_static! {
    pub static ref MUTATOR_SELECTION: Mutex<HashMap<&'static str, u8>> = {
        let mut m = HashMap::new();
        m.insert("0_mutate_mode", 0);
        m.insert("1_mutate_method", 0);
        m.insert("2_mutate_input", 0);
        m.insert("3_env_args", 0);
        m.insert("4_mutate_field", 0);
        m.insert("5_mutate_metho", 0);
        m.insert("6_byte_expansion", 0);
        m.insert("7_detail_mutation", 0);
        Mutex::new(m)
    };
}
pub fn set_mutator_selection() -> HashMap<&'static str, u8> {
    let global_mutation = *GLOBAL_MUTATION.lock().unwrap();
    let global_mutation_string = global_mutation.to_string();
    let mutations: Vec<_> = global_mutation_string.chars().map(|c| c.to_digit(10).unwrap() as u8).collect();
    println!("global_mutation: {}", global_mutation);
    let keys = vec![
        "0_mutate_mode",
        "1_mutate_method",
        "2_env_args",
        "3_mutate_field",
        "4_mutate_method",
        "5_byte_expansion",
        "6_detail_mutation",
    ];
    let mut mutator_selection: HashMap<_, _> = keys.into_iter().zip(mutations.clone().into_iter()).collect();
    let detail_mutation_value = (mutations[6] as i64) * 10 + (mutations[7] as i64);
    mutator_selection.insert("6_detail_mutation", detail_mutation_value as u8);
    // println!("{:?}", mutator_selection);

    mutator_selection.clone()
}


pub fn get_mutator_selection() -> HashMap<&'static str, u8> {
    MUTATOR_SELECTION.lock().unwrap().clone()
}
//action设计======================================================================================================================
lazy_static! {
    //最大值  520190016，可能要改为f32  f64??????
    static ref ACTIONS: Mutex<Vec<i64>> = Mutex::new(Vec::new());
}

use csv::Reader;
use std::error::Error;
use crate::evm::LOSS_VALUES;

fn read_nums_from_csv(file_path: &str) -> Result<Vec<i64>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut nums = Vec::new();

    for result in reader.records() {
        let record = result?;
        for field in record.iter() {
            if let Ok(num) = field.parse::<i64>() {
                nums.push(num);
            }
        }
    }

    Ok(nums)
}
fn encode_actions() -> Vec<i64> {
    let nums = read_nums_from_csv("action.csv").unwrap();
    println!("长长------{}", nums.len());
    // let nums = vec![10110101,
    // 10110102,
    // 10110103,
    // 10110104,
    // 10110105,
    // 10120101,
    // 10120102,
    // 10120103];
    let mut actions = ACTIONS.lock().unwrap();
    for num in nums {
        actions.push(num);
    }
    // for action in actions.iter() {
    //     println!("{}", action);
    // }
    actions.clone()
}
//state的设计和方法================================================================================================================
// 通常会将所有的输入数据转换为浮点数（通常是f32），因为神经网络的运算（如加法、乘法、激活函数等）都是在浮点数上进行的。
// 状态通常是一个向量，表示环境的当前状态。在神经网络中，这通常是一个浮点数类型的张量（Tensor）。
pub struct State {
    // data: BoxedABI,
    // sstate_state: EVMState,
    //state,swap_data,reentrancy_metadata,integer_overflow,arbitrary_calls,typed_bug,self_destruct,bug_hit,flashloan_data,post_execution,balance
    sstate_initialize: bool,
    // txn_value: Option<EVMU256>,
    step: bool,
    // env: Env,
    //可以分开env_cfg,env_block,env_tx
    // access_pattern: Arc<Mutex<AccessPattern>>,
    liquidation_percent: u8,
    // direct_data: Vec<u8>,
    // randomness: Vec<u8>,
    repeat: u64,
    // swap_data:
}

impl State {
    pub fn to_tensor(&self) -> Tensor {
        let sstate_initialize_f32 = if self.sstate_initialize { 1.0 } else { 0.0 };
        let step_f32 = if self.step { 1.0 } else { 0.0 };
        let liquidation_percent_f32 = self.liquidation_percent as f32;
        let repeat_f32 = self.repeat as f32;

        let input_data = vec![sstate_initialize_f32, step_f32, liquidation_percent_f32, repeat_f32];
        let input_tensor = Tensor::from_slice(&input_data);
        input_tensor
    }
}


// FuzzEnv========================================================================================================
pub struct FuzzEnv {
    state: EVMInput,
}

impl FuzzEnv {
    pub fn new() -> FuzzEnv {
        FuzzEnv {
            state: get_global_input(),
        }
    }

    pub fn reset(&mut self) -> Tensor {
        let global_input = get_global_input();
        let state = State {
            sstate_initialize: global_input.sstate.initialized,
            step: global_input.step,
            liquidation_percent: global_input.liquidation_percent,
            repeat: global_input.repeat as u64,
        };
        self.state = global_input;
        state.to_tensor()
    }
    pub fn step_1(&mut self, action: i64) {
        //返回一个元组，包含一个Tensor（新的状态），一个i64（奖励）和一个bool（表示任务是否完成）
        // 执行动作——根据action 进行变异（改变state）
        // 执行动作——根据action 进行变异（改变state）

        set_global_mutation(action);
        set_mutator_selection();
    }

    pub fn step_2(&mut self) -> (Tensor, i64){
        let reward = get_value() as i64;

        let global_input = get_global_input();
        let state = State {
            sstate_initialize: global_input.sstate.initialized,
            step: global_input.step,
            liquidation_percent: global_input.liquidation_percent,
            repeat: global_input.repeat as u64,
        };
        (state.to_tensor(),reward)
    }
}
//ReplayBuffer存储经验元组（state, action, reward, next_state）========================================================
pub struct ReplayBuffer {
    buffer: VecDeque<(Tensor, i64, i64, Tensor)>,
    capacity: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> ReplayBuffer {
        ReplayBuffer {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, state: Tensor, action: i64, reward: i64, next_state: Tensor) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back((state, action, reward, next_state));
    }

    pub fn sample(&self, batch_size: usize) -> Option<Vec<(Tensor, i64, i64, Tensor)>> {
        if self.buffer.len() < batch_size {
            None
        } else {
            Some(self.buffer.iter().map(|(s, a, r, ns)| (s.copy(), *a, *r, ns.copy())).take(batch_size).collect())
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
//DQN Net===============================================================================================================
// #[derive(Debug)]
// pub struct DqnNet {
//     fc1: nn::Linear,
//     fc2: nn::Linear,
//     fc3: nn::Linear,
// }
//
// impl DqnNet {
//     // output_dim表示可能的动作的数量 现在=13，输入的维度现在是4
//     pub fn new(vs: &nn::Path, input_dim: i64, output_dim: i64) -> DqnNet {
//         let fc1 = nn::linear(vs / "fc1", input_dim, 128, Default::default());
//         let fc2 = nn::linear(vs / "fc2", 128, 64, Default::default());
//         let fc3 = nn::linear(vs / "fc3", 64, output_dim, Default::default());
//         DqnNet { fc1, fc2, fc3 }
//     }
// }
//
// impl Module for DqnNet {
//     fn forward(&self, xs: &Tensor) -> Tensor {
//         xs.apply(&self.fc1)
//             .relu()
//             .apply(&self.fc2)
//             .relu()
//             .apply(&self.fc3)
//     }
// }
#[derive(Debug)]
pub struct DqnNet<'a> {
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc5: nn::Linear,
    vs: &'a nn::VarStore,  // 添加生命周期参数'a
}

impl<'a> DqnNet<'a> {
    pub fn new(vs: &'a nn::VarStore, input_dim: i64, output_dim: i64) -> DqnNet<'a> {
        let fc1 = nn::linear(vs.root() / "fc1", input_dim, 256, Default::default());
        let fc2 = nn::linear(vs.root() / "fc2", 256, 128, Default::default());
        let fc3 = nn::linear(vs.root() / "fc3", 128, 64, Default::default());
        let fc4 = nn::linear(vs.root() / "fc4", 64, 32, Default::default());
        let fc5 = nn::linear(vs.root() / "fc5", 32, output_dim, Default::default());
        DqnNet { fc1, fc2, fc3, fc4, fc5, vs } // vs字段类型改为引用类型
    }

    // 新增方法
    pub fn save(&self, path: &str) -> tch::Result<()> {
        self.vs.save(path)
    }
}

impl<'a> Module for DqnNet<'a> {
    fn forward(&self, xs: &Tensor) -> Tensor {
        xs.apply(&self.fc1)
            .relu()
            .apply(&self.fc2)
            .relu()
            .apply(&self.fc3)
            .relu()
            .apply(&self.fc4)
            .relu()
            .apply(&self.fc5)
    }
}

//DQNAgent==========================================================================================================
pub struct DQNAgent {
    state_dim: i64,
    action_dim: i64,
    pub(crate) model: DqnNet<'static>,  // 修改生命周期参数为'static
    pub(crate) replay_buffer: ReplayBuffer,
    optimizer: Optimizer,
    actions: Vec<i64>,
}

impl DQNAgent {  // 修改生命周期参数为'static
    pub fn new(vs: &'static nn::VarStore, state_dim: i64, action_dim: i64, replay_buffer_capacity: usize) -> DQNAgent {
        let model = DqnNet::new(vs, state_dim, action_dim);
        let replay_buffer = ReplayBuffer::new(replay_buffer_capacity);
        let optimizer = nn::Adam::default().build(vs, 1e-7).unwrap();
        let actions = encode_actions();
        DQNAgent { state_dim, action_dim, model, replay_buffer, optimizer, actions }
    }

    // pub fn train(&mut self, env: &mut FuzzEnv, episodes: usize, batch_size: usize) {
    //     for _ in 0..episodes {
    //         let mut state = env.reset();
    //         let mut done = false;
    //         while !done {
    //             let action = self.get_action(&state);
    //             //
    //             let (next_state, reward, is_done) = env.step(action);
    //             self.replay_buffer.push(state, action, reward, next_state.clone(&next_state));
    //             state = next_state;
    //             self.update_model(batch_size);
    //             println!("update model===========");
    //             done = is_done;
    //         }
    //     }
    // }
    // pub fn evaluate(&self, env: &mut FuzzEnv, episodes: usize) -> f64 {
    //     let mut total_rewards = 0.0;
    //     for _ in 0..episodes {
    //         let mut state = env.reset();
    //         let mut done = false;
    //         while !done {
    //             let action = self.get_action(&state);
    //             let (next_state, reward, is_done) = env.step(action);
    //             state = next_state;
    //             total_rewards += reward as f64;
    //             done = is_done;
    //         }
    //     }
    //     //要不要修改该类型i64????
    //     total_rewards / episodes as f64
    // }
    pub fn update_model(&mut self, batch_size: usize) {
        if self.replay_buffer.len() < batch_size {
            return;
        }
        let samples = self.replay_buffer.sample(batch_size).unwrap();
        let mut states = Vec::new();
        let mut actions = Vec::new();
        let mut rewards = Vec::new();
        let mut next_states = Vec::new();
        for (state, action, reward, next_state) in samples.into_iter() {
            states.push(state);
            actions.push(action);
            rewards.push(reward);
            next_states.push(next_state);
        }


        let state = Tensor::stack(&states, 0);
        // let action = Tensor::from_slice(&actions);
        let action = Tensor::from_slice(&actions).unsqueeze(-1);
        let reward = Tensor::from_slice(&rewards);
        let next_state = Tensor::stack(&next_states, 0);

         // let curr_q_values = self.model.forward(&states).g((actions.unsqueeze(-1), )).squeeze1(-1);.
        // g((actions.unsqueeze(-1), )): 这是在获取执行特定动作的预期回报值。g方法返回输入张量（这里是神经网络的输出）在指定维度（这里是-1，表示动作的维度）上的元素。这里的actions.unsqueeze(-1)是指定索引。
        // .squeeze1(-1): 这是在移除张量中长度为1的维度。squeeze1方法将输入张量（这里是g方法的输出）在指定维度（这里是-1，表示最后一个维度）上的长度为1的维度移除。这通常用于移除不必要的维度，使得张量的形状更加简洁。
        // println!("State shape: {:?}", state.size());
        // println!("Action shape: {:?}", action.unsqueeze(-1).size());
        // println!("Action values: {:?}", action);
        // let curr_q_value = self.model.forward(&state).gather(-1, &action.unsqueeze(-1), false).squeeze_dim(-1);//不知道对不对？？？？
        let curr_q_value = self.model.forward(&state).gather(-1, &action, false).squeeze_dim(-1);

        let next_q_value = self.model.forward(&next_state).max_dim(-1, false).0.detach();
        let expected_q_value = reward.to_kind(Kind::Float) + 0.99 * next_q_value;

        let loss = curr_q_value.mse_loss(&expected_q_value, tch::Reduction::Mean);

        println!("loss-------------: {:?}", loss);
        let loss_value = loss.double_value(&[]) as f32;
        let mut loss_values = LOSS_VALUES.lock().unwrap();
        loss_values.push(loss_value);

        self.optimizer.zero_grad();
        loss.backward();
        self.optimizer.step();
    }

    pub fn get_action(&mut self, state: &Tensor, epsilon: f64) -> (i64,i64) {
        //epsilon-greedy 策略：以一定的概率随机选择一个动作
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < epsilon {
            let action_index = rng.gen_range(0..self.actions.len());
            let action = self.actions[action_index];
            (action, action_index as i64)
        } else {
            // 使用模型对输入的状态进行前向传播，得到Q值
            let q_value = self.model.forward(&state.unsqueeze(0));
            // 使用argmax函数找到Q值中最大值的索引，这个索引就是最佳的动作
            // -1表示在最后一个维度上找最大值的索引；false表示不保持维度，即降维
            // let action = q_value.argmax(-1, false).int64_value(&[]);
            let action_index = q_value.argmax(-1, false).int64_value(&[]);
            let action_index = action_index as usize % self.actions.len();
            let action = self.actions[action_index];
            (action, action_index as i64)
        }
    }
}