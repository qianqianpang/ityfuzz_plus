use std::fmt::Debug;

use libafl::{
    Error,
    inputs::Input,
    mutators::MutationResult,
    prelude::{HasMaxSize, HasRand, Mutator, State},
    schedulers::Scheduler,
    state::HasMetadata,
};
use libafl_bolts::{Named, prelude::Rand};
use revm_interpreter::Interpreter;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    evm::{
        abi::ABIAddressToInstanceMap,
        input::EVMInputTy::Borrow,
        types::{convert_u256_to_h160, EVMAddress, EVMU256},
        vm::{Constraint, EVMStateT},
    },
    generic_vm::vm_state::VMStateT,
    input::{ConciseSerde, VMInputT},
    state::{HasCaller, HasItyState, HasPresets, InfantStateState},
};
use crate::dqn_alogritm::get_mutator_selection;
/// Mutator for EVM inputs
use crate::evm::input::EVMInputT;
// use crate::dqn_alogritm::set_global_input;
use crate::global_info::increment_mutation_op;

use super::onchain::flashloan::CAN_LIQUIDATE;

/// [`AccessPattern`] records the access pattern of the input during execution.
/// This helps to determine what is needed to be fuzzed. For instance, we don't
/// need to mutate caller if the execution never uses it.
///
/// Each mutant should report to its parent's access pattern
/// if a new corpus item is added, it should inherit the access pattern of its
/// source
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AccessPattern {
    pub caller: bool,
    // or origin
    pub balance: Vec<EVMAddress>,
    // balance queried for accounts
    pub call_value: bool,
    pub gas_price: bool,
    pub number: bool,
    pub coinbase: bool,
    pub timestamp: bool,
    pub prevrandao: bool,
    pub gas_limit: bool,
    pub chain_id: bool,
    pub basefee: bool,
    pub difficulty: bool,
    // pub limit_contract_code_size: bool,
}

impl AccessPattern {
    /// Create a new access pattern with all fields set to false
    /// 将这些访问模式设置为true将增加实验的复杂性，因为它增加了实验的可能状态空间。
    /// 这可能会使得实验更难以理解和控制，但也可能使得实验能够覆盖到更多的可能情况。
    pub fn new() -> Self {
        Self {
            balance: vec![],
            caller: true,
            call_value: true,
            gas_price: true,
            number: true,
            coinbase: true,
            timestamp: true,
            prevrandao: true,
            gas_limit: true,
            chain_id: false,
            basefee: true,
            difficulty: true,
            // limit_contract_code_size: false,
        }
    }

    /// Record access pattern of current opcode executed by the interpreter
    pub fn decode_instruction(&mut self, interp: &Interpreter) {
        match unsafe { *interp.instruction_pointer } {
            0x31 => self.balance.push(convert_u256_to_h160(interp.stack.peek(0).unwrap())),
            0x33 => self.caller = true,
            0x3a => self.gas_price = true,
            0x43 => self.number = true,
            0x41 => self.coinbase = true,
            0x42 => self.timestamp = true,
            0x44 => self.prevrandao = true,
            0x45 => self.gas_limit = true,
            0x46 => self.chain_id = true,
            0x48 => self.basefee = true,
            // 0x50 => self.difficulty = true,
            // 0x52 => self.limit_contract_code_size = true,
            _ => {}
        }
    }
    pub fn to_u32(&self) -> u32 {
        let mut result = 0;

        if self.caller { result |= 1 << 0; }
        if self.call_value { result |= 1 << 1; }
        if self.gas_price { result |= 1 << 2; }
        if self.number { result |= 1 << 3; }
        if self.coinbase { result |= 1 << 4; }
        if self.timestamp { result |= 1 << 5; }
        if self.prevrandao { result |= 1 << 6; }
        if self.gas_limit { result |= 1 << 7; }
        if self.chain_id { result |= 1 << 8; }
        if self.basefee { result |= 1 << 9; }
        if self.difficulty { result |= 1 << 10; }
        // balance长度值放在最后
        result |= (self.balance.len() as u32) << 11;

        result
    }
}

/// [`FuzzMutator`] is a mutator that mutates the input based on the ABI and
/// access pattern
pub struct FuzzMutator<VS, Loc, Addr, SC, CI>
    where
        VS: Default + VMStateT,
        SC: Scheduler<State=InfantStateState<Loc, Addr, VS, CI>>,
        Addr: Serialize + DeserializeOwned + Debug + Clone,
        Loc: Serialize + DeserializeOwned + Debug + Clone,
        CI: Serialize + DeserializeOwned + Debug + Clone + ConciseSerde,
{
    /// Scheduler for selecting the next VM state to use if we decide to mutate
    /// the VM state of the input
    pub infant_scheduler: SC,
    pub phantom: std::marker::PhantomData<(VS, Loc, Addr, CI)>,
}

impl<VS, Loc, Addr, SC, CI> FuzzMutator<VS, Loc, Addr, SC, CI>
    where
        VS: Default + VMStateT,
        SC: Scheduler<State=InfantStateState<Loc, Addr, VS, CI>>,
        Addr: Serialize + DeserializeOwned + Debug + Clone,
        Loc: Serialize + DeserializeOwned + Debug + Clone,
        CI: Serialize + DeserializeOwned + Debug + Clone + ConciseSerde,
{
    /// Create a new [`FuzzMutator`] with the given scheduler
    pub fn new(infant_scheduler: SC) -> Self {
        Self {
            infant_scheduler,
            phantom: Default::default(),
        }
    }

    fn ensures_constraint<I, S>(input: &mut I, state: &mut S, new_vm_state: &VS, constraints: Vec<Constraint>) -> bool
        where
            I: VMInputT<VS, Loc, Addr, CI> + Input + EVMInputT,
            S: State + HasRand + HasMaxSize + HasItyState<Loc, Addr, VS, CI> + HasCaller<Addr> + HasMetadata,
    {
        // precheck
        for constraint in &constraints {
            match constraint {
                Constraint::MustStepNow => {
                    if input.get_input_type() == Borrow {
                        return false;
                    }
                }
                Constraint::Contract(_) => {
                    if input.get_input_type() == Borrow {
                        return false;
                    }
                }
                _ => {}
            }
        }

        for constraint in constraints {
            match constraint {
                Constraint::Caller(caller) => {
                    input.set_caller_evm(caller);
                }
                Constraint::Value(value) => {
                    input.set_txn_value(value);
                }
                Constraint::Contract(target) => {
                    let rand_int = state.rand_mut().next();
                    let always_none = state.rand_mut().next() % 30 == 0;
                    let abis = state
                        .metadata_map()
                        .get::<ABIAddressToInstanceMap>()
                        .expect("ABIAddressToInstanceMap not found");
                    let abi = match abis.map.get(&target) {
                        Some(abi) => {
                            if !abi.is_empty() && !always_none {
                                Some((*abi)[rand_int as usize % abi.len()].clone())
                            } else {
                                None
                            }
                        }
                        None => None,
                    };
                    input.set_contract_and_abi(target, abi);
                }
                Constraint::NoLiquidation => {
                    input.set_liquidation_percent(0);
                }
                Constraint::MustStepNow => {
                    input.set_step(true);
                    // todo(@shou): move args into
                    // debug!("vm state: {:?}", input.get_state());
                    input.set_as_post_exec(new_vm_state.get_post_execution_needed_len());
                    input.mutate(state);
                }
            }
        }
        true
    }
}

impl<VS, Loc, Addr, SC, CI> Named for FuzzMutator<VS, Loc, Addr, SC, CI>
    where
        VS: Default + VMStateT,
        SC: Scheduler<State=InfantStateState<Loc, Addr, VS, CI>>,
        Addr: Serialize + DeserializeOwned + Debug + Clone,
        Loc: Serialize + DeserializeOwned + Debug + Clone,
        CI: Serialize + DeserializeOwned + Debug + Clone + ConciseSerde,
{
    fn name(&self) -> &str {
        "FuzzMutator"
    }
}


impl<VS, Loc, Addr, I, S, SC, CI> Mutator<I, S> for FuzzMutator<VS, Loc, Addr, SC, CI>
    where
        I: VMInputT<VS, Loc, Addr, CI> + Input + EVMInputT,
        S: State + HasRand + HasMaxSize + HasItyState<Loc, Addr, VS, CI> + HasCaller<Addr> + HasMetadata + HasPresets,
        SC: Scheduler<State=InfantStateState<Loc, Addr, VS, CI>>,
        VS: Default + VMStateT + EVMStateT,
        Addr: PartialEq + Debug + Serialize + DeserializeOwned + Clone,
        Loc: Serialize + DeserializeOwned + Debug + Clone,
        CI: Serialize + DeserializeOwned + Debug + Clone + ConciseSerde,
{
    /// Mutate the input
    #[allow(unused_assignments)]
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        if !input.get_staged_state().initialized {
            let concrete = state.get_infant_state(&mut self.infant_scheduler).unwrap();
            input.set_staged_state(concrete.1, concrete.0);
        }
        let should_havoc = state.rand_mut().below(100) < 60; // (amount_of_args * 10) as u64;

        // determine how many times we should mutate the input
        let havoc_times = if should_havoc {
            state.rand_mut().below(10) + 1
        } else {
            1
        };


        //把外包放到外面，加了很多 MutationResult::Mutated MutationResult::Skipped;删掉了mutated  不知道对不对????
        let mut mutator = || -> MutationResult {
            let mutator_selection = get_mutator_selection();
            match mutator_selection.get("0_mutate_mode") {
                // use exploit template
                Some(&0) => {
                    if state.has_preset() {
                        match mutator_selection.get("1_mutate_method") {
                            Some(&0) => {
                                if input.get_input_type() != Borrow {
                                    match state.get_next_call() {
                                        Some((addr, abi)) => {
                                            input.set_contract_and_abi(addr, Some(abi));
                                            input.mutate(state)
                                        }
                                        None => { MutationResult::Skipped }
                                    }
                                } else {
                                    MutationResult::Skipped
                                }
                            }
                            Some(&1) => { MutationResult::Skipped }
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        MutationResult::Skipped
                    }
                }
                Some(&1) => {
                    //mutate state
                    if !input.is_step() {
                        match mutator_selection.get("1_mutate_method") {
                            Some(&0) => {
                                let old_idx = input.get_state_idx();
                                let (idx, new_state) = state.get_infant_state(&mut self.infant_scheduler).unwrap();
                                if idx != old_idx {
                                    if !state.has_caller(&input.get_caller()) {
                                        input.set_caller(state.get_rand_caller());
                                    }

                                    if Self::ensures_constraint(input, state, &new_state.state, new_state.state.get_constraints()) {
                                        input.set_staged_state(new_state, idx);
                                        MutationResult::Mutated
                                    } else {
                                        MutationResult::Skipped
                                    }
                                } else {
                                    MutationResult::Skipped
                                }
                            }
                            Some(&1) => { MutationResult::Skipped }
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        MutationResult::Skipped
                    }
                }
                Some(&2) => {
                    //mutate data
                    if input.get_staged_state().state.has_post_execution() && !input.is_step() {
                        match mutator_selection.get("1_mutate_method") {
                            Some(&0) => {
                                macro_rules! turn_to_step {
                                    () => {
                                        input.set_step(true);
                                        // todo(@shou): move args into
                                        input.set_as_post_exec(input.get_state().get_post_execution_needed_len());
                                        for _ in 0..havoc_times - 1 {
                                            input.mutate(state);
                                        }
                                    };
                                }
                                if input.get_input_type() != Borrow {
                                    turn_to_step!();
                                    MutationResult::Mutated
                                } else {
                                    MutationResult::Skipped
                                }
                            }
                            Some(&1) => {
                                MutationResult::Skipped
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        MutationResult::Skipped
                    }
                }
                Some(&3) => {
                    //MUTATE_BYTE
                    if input.is_step() {
                        let res = match mutator_selection.get("1_mutate_method") {
                            Some(&0) => {
                                if unsafe { CAN_LIQUIDATE } {
                                    let prev_percent = input.get_liquidation_percent();
                                    input.set_liquidation_percent(if state.rand_mut().below(100) < 80 { 10 } else { 0 } as u8);
                                    if prev_percent != input.get_liquidation_percent() {
                                        MutationResult::Mutated
                                    } else {
                                        MutationResult::Skipped
                                    }
                                } else {
                                    MutationResult::Skipped
                                }
                            }
                            Some(&1) => {
                                increment_mutation_op("MUTATE_BYTE", "MUTATE_NORMAL");
                                input.mutate(state)
                            }
                            _ => {
                                unreachable!()
                            }
                        };
                        input.set_txn_value(EVMU256::ZERO);
                        return res;
                    } else {
                        MutationResult::Skipped
                    }
                }
                Some(&4) => {
                    //MUTATE_BORROW
                    if input.get_input_type() == Borrow {
                        let rand_u8 = state.rand_mut().below(255) as u8;
                        return match mutator_selection.get("1_mutate_method") {
                            Some(&0) => {
                                increment_mutation_op("MUTATE_BORROW", "MUTATE_RANDOMNESS");
                                input.set_randomness(vec![rand_u8; 1]);
                                MutationResult::Mutated
                            }
                            Some(&1) => {
                                increment_mutation_op("MUTATE_BORROW", "MUTATE_NORMAL");
                                input.mutate(state)
                            }
                            _ => {
                                unreachable!()
                            }
                        };
                    } else {
                        MutationResult::Skipped
                    }
                }
                Some(&5) => {
                    //MUTATE_ALL
                    match mutator_selection.get("1_mutate_method") {
                        Some(&0) => {
                            increment_mutation_op("MUTATE_ALL", "MUTATE_LIQUIDATION");
                            let prev_percent = input.get_liquidation_percent();
                            input.set_liquidation_percent(if state.rand_mut().below(100) < 80 { 10 } else { 0 } as u8);
                            if prev_percent != input.get_liquidation_percent() {
                                MutationResult::Mutated
                            } else {
                                MutationResult::Skipped
                            }
                        }
                        Some(&1) => {
                            increment_mutation_op("MUTATE_ALL", "MUTATE_RANDOMNESSL");
                            let rand_u8 = state.rand_mut().below(255) as u8;
                            input.set_randomness(vec![rand_u8; 1]);
                            MutationResult::Mutated
                        }
                        Some(&2) => {
                            increment_mutation_op("MUTATE_ALL", "MUTATE_NORMAL");
                            input.mutate(state)
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        };

        let mut res = MutationResult::Skipped;
        let mut tries = 0;

        // try to mutate the input for [`havoc_times`] times with 20 retries if
        // the input is not mutated
        while res != MutationResult::Mutated && tries < 20 {
            for _ in 0..havoc_times {
                if mutator() == MutationResult::Mutated {
                    res = MutationResult::Mutated;
                }
            }
            tries += 1;
        }
        Ok(res)
    }
}
