use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::vec::Vec;

use bytes::Bytes;
use lazy_static::lazy_static;
use revm_primitives::Env;
use tch::{Kind, nn, nn::Module, nn::Optimizer, nn::OptimizerConfig, Tensor};

use crate::evm::input::{EVMInput, EVMInputTy};
use crate::evm::mutator::AccessPattern;
use crate::evm::types::EVMAddress;
use crate::global_info::{get_value, IS_OBJECTIVE};
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


lazy_static! {
    pub static ref GLOBAL_MUTATION: Mutex<i64> = Mutex::new(0);
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
    let mutations: Vec<_>=global_mutation_string.chars().map(|c| c as u8).collect();

    let mut mutator_selection = MUTATOR_SELECTION.lock().unwrap();
    mutator_selection.insert("0_mutate_mode", mutations[0]);
    mutator_selection.insert("1_mutate_method", mutations[1]);
    mutator_selection.insert("2_env_args", mutations[2]);
    mutator_selection.insert("3_mutate_field", mutations[3]);
    mutator_selection.insert("4_mutate_method", mutations[4]);
    mutator_selection.insert("5_byte_expansion", mutations[5]);
    let detail_mutation_value = (mutations[6] as i64) * 10 + (mutations[7] as i64);
    mutator_selection.insert("6_detail_mutation", detail_mutation_value as u8);

    mutator_selection.clone()
}


pub fn get_mutator_selection() -> HashMap<&'static str, u8> {
    MUTATOR_SELECTION.lock().unwrap().clone()
}
//action设计========================================================================================
lazy_static! {
    //最大值  520190016，可能要改为f32  f64??????
    static ref ACTIONS: Mutex<Vec<i64>> = Mutex::new(Vec::new());
}
fn encode_actions() -> Vec<i64> {
    let nums = vec![0, 1, 10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 52];
    let mut actions = ACTIONS.lock().unwrap();
    for num in nums {
        actions.push(num);
    }
    for action in actions.iter() {
        println!("{}", action);
    }
    actions.clone()
}
//state的设计和方法==================================================================================
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
    pub fn step(&mut self, action: i64) -> (Tensor, i64, bool) {
        //返回一个元组，包含一个Tensor（新的状态），一个i64（奖励）和一个bool（表示任务是否完成）
        // Modify the state based on the action

        let reward = get_value() as i64;
        let done = IS_OBJECTIVE.load(Ordering::SeqCst);

        let global_input = get_global_input();
        let state = State {
            sstate_initialize: global_input.sstate.initialized,
            step: global_input.step,
            liquidation_percent: global_input.liquidation_percent,
            repeat: global_input.repeat as u64,
        };
        (state.to_tensor(),reward,done)
    }
}
//ReplayBuffer存储经验元组（state, action, reward, next_state）==========================================
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
//DQN===========================================================================================
#[derive(Debug)]
pub struct DqnNet {
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
}

impl DqnNet {
    // output_dim表示可能的动作的数量 现在=13，输入的维度现在是4
    pub fn new(vs: &nn::Path, input_dim: i64, output_dim: i64) -> DqnNet {
        let fc1 = nn::linear(vs / "fc1", input_dim, 128, Default::default());
        let fc2 = nn::linear(vs / "fc2", 128, 64, Default::default());
        let fc3 = nn::linear(vs / "fc3", 64, output_dim, Default::default());
        DqnNet { fc1, fc2, fc3 }
    }
}

impl Module for DqnNet {
    fn forward(&self, xs: &Tensor) -> Tensor {
        xs.apply(&self.fc1)
            .relu()
            .apply(&self.fc2)
            .relu()
            .apply(&self.fc3)
    }
}




//DQNAgent==================================================================================
pub struct DQNAgent {
    state_dim: i64,
    action_dim: i64,
    model: DqnNet,
    replay_buffer: ReplayBuffer,
    optimizer: Optimizer,
}

impl DQNAgent {
    pub fn new(vs: &nn::Path, state_dim: i64, action_dim: i64, replay_buffer_capacity: usize) -> DQNAgent {
        let model = DqnNet::new(vs, state_dim, action_dim);
        let replay_buffer = ReplayBuffer::new(replay_buffer_capacity);
        let vs = nn::VarStore::new(tch::Device::Cpu);
        let optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();
        DQNAgent { state_dim, action_dim, model, replay_buffer, optimizer }
    }

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
        let action = Tensor::from_slice(&actions);
        let reward = Tensor::from_slice(&rewards);
        let next_state = Tensor::stack(&next_states, 0);

         // let curr_q_values = self.model.forward(&states).g((actions.unsqueeze(-1), )).squeeze1(-1);.
        // g((actions.unsqueeze(-1), )): 这是在获取执行特定动作的预期回报值。g方法返回输入张量（这里是神经网络的输出）在指定维度（这里是-1，表示动作的维度）上的元素。这里的actions.unsqueeze(-1)是指定索引。
        // .squeeze1(-1): 这是在移除张量中长度为1的维度。squeeze1方法将输入张量（这里是g方法的输出）在指定维度（这里是-1，表示最后一个维度）上的长度为1的维度移除。这通常用于移除不必要的维度，使得张量的形状更加简洁。
        let curr_q_value = self.model.forward(&state).gather(-1, &action.unsqueeze(-1), false).squeeze_dim(-1);//不知道对不对？？？？
        let next_q_value = self.model.forward(&next_state).max_dim(-1, false).0.detach();
        let expected_q_value = reward.to_kind(Kind::Float) + 0.99 * next_q_value;

        let loss = curr_q_value.mse_loss(&expected_q_value, tch::Reduction::Mean);

        self.optimizer.zero_grad();
        loss.backward();
        self.optimizer.step();
    }

    pub fn get_action(&self, state: &Tensor) -> i64 {
        let q_value = self.model.forward(&state.unsqueeze(0));
        // let action = q_value.argmax1(-1, false).item::<i64>();
        let action = q_value.argmax(-1, false).int64_value(&[]);//argmax1(-1, false).item::<i64>();
        action
    }
}

impl DQNAgent {
    pub fn train(&mut self, env: &mut FuzzEnv, episodes: usize, batch_size: usize) {
    for _ in 0..episodes {
        let mut state = env.reset();
        let mut done = false;
        while !done {
            let action = self.get_action(&state);
            let (next_state, reward, is_done) = env.step(action);
            self.replay_buffer.push(state, action, reward, next_state.clone(&Default::default()));
            state = next_state;
            self.update_model(batch_size);
            done = is_done;
        }
    }
}

pub fn evaluate(&self, env: &mut FuzzEnv, episodes: usize) -> f64 {
    let mut total_rewards = 0.0;
    for _ in 0..episodes {
        let mut state = env.reset();
        let mut done = false;
        while !done {
            let action = self.get_action(&state);
            let (next_state, reward, is_done) = env.step(action);
            state = next_state;
            total_rewards += reward as f64;
            done = is_done;
        }
    }
    //要不要修改该类型i64????
    total_rewards / episodes as f64
}
}
