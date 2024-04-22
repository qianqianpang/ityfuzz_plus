use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use bytes::Bytes;
use itertools::Itertools;
use lazy_static::lazy_static;
use revm_primitives::Env;
use revm_primitives::ruint::Uint;
use tch::{Kind, nn, nn::Module, nn::Optimizer, nn::OptimizerConfig, Tensor};
use std::vec::Vec;
use crate::evm::abi::BoxedABI;
use crate::evm::input::{EVMInput, EVMInputTy};
use crate::evm::mutator::AccessPattern;
use crate::evm::types::{EVMAddress, EVMU256};
use crate::evm::vm::EVMState;
use crate::generic_vm::vm_state::VMStateT;
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

//action设计===========也可以是数组？？tensor=============================================================================
lazy_static! {
    //最大值  520190016,i32表示从-2147483648到2147483647的整数，可能要改为f32  f64??????
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
// 通常会将所有的输入数据转换为浮点数（通常是32位浮点数，即f32），因为神经网络的运算（如加法、乘法、激活函数等）都是在浮点数上进行的。
pub struct State {
    // data: BoxedABI,
    // sstate_state: EVMState,
    //state,swap_data,reentrancy_metadata,integer_overflow,arbitrary_calls,typed_bug,self_destruct,bug_hit,flashloan_data,post_execution,balance
    //state（状态）：状态通常是一个向量，表示环境的当前状态。在神经网络中，这通常是一个浮点数类型的张量（Tensor）。例如，在Rust中，你可能会使用tch::Tensor来表示状态。
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


// pub fn env_to_u32(env: &Env) -> u32 {
//     let mut result = 0;
//
//     // For CfgEnv
//     result |= (env.cfg.chain_id.low_u32() as u32) << 0;
//     result |= (env.cfg.spec_id as u32) << 1;
//     if let Some(limit_contract_code_size) = env.cfg.limit_contract_code_size {
//         result |= (limit_contract_code_size as u32) << 2;
//     }
//     result |= (env.cfg.memory_limit as u32) << 3;
//
//     // For BlockEnv
//     result |= (env.block.number.low_u32() as u32) << 4;
//     result |= (env.block.coinbase.low_u32() as u32) << 5;
//     result |= (env.block.timestamp.low_u32() as u32) << 6;
//     result |= (env.block.difficulty.low_u32() as u32) << 7;
//     if let Some(prevrandao) = env.block.prevrandao {
//         result |= (prevrandao.low_u32() as u32) << 8;
//     }
//     result |= (env.block.basefee.low_u32() as u32) << 9;
//     result |= (env.block.gas_limit.low_u32() as u32) << 10;
//
//     // For TxEnv
//     result |= (env.tx.caller.low_u32() as u32) << 11;
//     result |= (env.tx.gas_limit as u32) << 12;
//     result |= (env.tx.gas_price.low_u32() as u32) << 13;
//     if let Some(gas_priority_fee) = env.tx.gas_priority_fee {
//         result |= (gas_priority_fee.low_u32() as u32) << 14;
//     }
//     result |= (env.tx.value.low_u32() as u32) << 15;
//     if let Some(chain_id) = env.tx.chain_id {
//         result |= (chain_id as u32) << 16;
//     }
//     if let Some(nonce) = env.tx.nonce {
//         result |= (nonce as u32) << 17;
//     }
//
//     result
// }
// fn hash_to_u32(value: &Uint<256, 4>) -> u32 {
//     let mut hasher = DefaultHasher::new();
//     value.hash(&mut hasher);
//     hasher.finish() as u32
// }
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


// pub struct FuzzEnv {
//     state: EVMInput,
// }
//
// impl FuzzEnv {
//     pub fn new() -> FuzzEnv {
//         FuzzEnv {
//             state: get_global_input(),
//         }
//     }
//
//     pub fn reset(&mut self) -> Tensor {
//         self.state = get_global_input();
//         // Convert the state to a Tensor and return it
//         // You need to implement the conversion from EVMInput to Tensor
//     }
//
//     pub fn step(&mut self, action: i64) -> (Tensor, f64, bool) {
//         // Modify the state based on the action
//         // You need to implement this part
//
//         let reward = get_value() as f64;
//         let done = false; // You need to determine when the task is done
//
//         // Convert the state to a Tensor and return it with the reward and done
//         // You need to implement the conversion from EVMInput to Tensor
//     }
// }
//
// pub fn train(agent: &mut DQNAgent, env: &mut dyn Env, episodes: usize, batch_size: usize) {
//     for _ in 0..episodes {
//         let mut state = env.reset();
//         let mut done = false;
//         while !done {
//             for stage_idx in 0..agent.action_dims.len() {
//                 let action = agent.get_action(&state, stage_idx);
//                 let (next_state, reward, is_done) = env.step(action);
//                 agent.replay_buffers[stage_idx].push(state, action, reward as f64, next_state.clone());
//                 state = next_state;
//                 agent.update_model(batch_size, stage_idx);
//                 done = is_done;
//             }
//         }
//     }
// }
//
// pub fn evaluate(agent: &DQNAgent, env: &mut dyn Env, episodes: usize) -> f64 {
//     let mut total_rewards = 0.0;
//     for _ in 0..episodes {
//         let mut state = env.reset();
//         let mut done = false;
//         while !done {
//             for stage_idx in 0..agent.action_dims.len() {
//                 let action = agent.get_action(&state, stage_idx);
//                 let (next_state, reward, is_done) = env.step(action);
//                 state = next_state;
//                 total_rewards += reward;
//                 done = is_done;
//             }
//         }
//     }
//     total_rewards / episodes as f64
// }
//
//
// fn main() {
//     let state_dim = 12;
//     let action_dims = vec![16, 11, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2];
//     let replay_buffer_capacity = 1000;
//     let mut agent = DQNAgent::new(&nn::VarStore::new(tch::Device::Cpu), state_dim, action_dims, replay_buffer_capacity);
//     let mut env = FuzzEnv::new();
//
//     let train_episodes = 1000;
//     let batch_size = 64;
//     train(&mut agent, &mut env, train_episodes, batch_size);
//
//     let eval_episodes = 100;
//     let avg_reward = evaluate(&agent, &mut env, eval_episodes);
//     println!("Average reward over {} episodes: {}", eval_episodes, avg_reward);
// }