//! The power schedules. This stage should be invoked after the calibration
//! stage.
use core::{fmt::Debug, marker::PhantomData};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use crate::evm::MUTATE_SUCCESS_COUNT;
use lazy_static::lazy_static;
use libafl::{corpus::{Corpus, CorpusId}, Error, ExecuteInputResult, executors::{Executor, HasObservers}, fuzzer::Evaluator, mark_feature_time, mutators::Mutator, prelude::Testcase, stages::{mutational::MutatedTransform, MutationalStage, Stage}, start_timer, state::{HasClientPerfMonitor, HasCorpus, HasMetadata, HasRand, UsesState}};
use libafl::mutators::MutationResult;
use libafl::prelude::mutational::MutatedTransformPost;
use libafl_bolts::ErrorBacktrace;
use plotters::prelude::*;

use crate::evm::{ACTION_COUNTS, EPSILON, LOSS_VALUES, REWARD_VALUES, SOLUTION_FLAG};
// use crate::evm::{AGENT, ENV, EPISODES, BATCH_SIZE};
use crate::global_info::{calculate_value};

pub trait TestcaseScoreWithId<S>
    where
        S: HasMetadata + HasCorpus,
{
    /// Computes the favor factor of a [`Testcase`]. Lower is better.
    fn compute(state: &S, entry: &mut Testcase<S::Input>, id: CorpusId) -> Result<f64, Error>;
}

/// The mutational stage using power schedules
#[derive(Clone, Debug)]
pub struct PowerMutationalStageWithId<E, F, EM, I, M, Z> {
    mutator: M,
    #[allow(clippy::type_complexity)]
    phantom: PhantomData<(E, F, EM, I, Z)>,
}

impl<E, F, EM, I, M, Z> UsesState for PowerMutationalStageWithId<E, F, EM, I, M, Z>
    where
        E: UsesState,
{
    type State = E::State;
}

impl<E, F, EM, I, M, Z> MutationalStage<E, EM, I, M, Z> for PowerMutationalStageWithId<E, F, EM, I, M, Z>
    where
        E: Executor<EM, Z> + HasObservers,
        EM: UsesState<State = E::State>,
        F: TestcaseScoreWithId<E::State>,
        M: Mutator<I, E::State>,
        E::State: HasClientPerfMonitor + HasCorpus + HasMetadata + HasRand,
        Z: Evaluator<E, EM, State = E::State>,
        I: MutatedTransform<E::Input, E::State> + Clone,
{
    /// The mutator, added to this stage
    #[inline]
    fn mutator(&self) -> &M {
        &self.mutator
    }

    /// The list of mutators, added to this stage (as mutable ref)
    #[inline]
    fn mutator_mut(&mut self) -> &mut M {
        &mut self.mutator
    }

    /// Gets the number of iterations as a random number
    #[allow(clippy::cast_sign_loss)]
    fn iterations(&self, state: &mut E::State, corpus_idx: CorpusId) -> Result<u64, Error> {
        // Update handicap
        let mut testcase = state.corpus().get(corpus_idx)?.borrow_mut();
        let score = F::compute(state, &mut *testcase, corpus_idx)? as u64;

        Ok(score)
    }

    fn perform_mutational(
        &mut self,
        fuzzer: &mut Z,
        executor: &mut E,
        state: &mut E::State,
        manager: &mut EM,
        corpus_idx: CorpusId,
    ) -> Result<(), Error> {
        let num = self.iterations(state, corpus_idx)?;

        start_timer!(state);
        let mut testcase = state.corpus().get(corpus_idx)?.borrow_mut();
        let Ok(input) = I::try_transform_from(&mut testcase, state, corpus_idx) else {
            return Ok(());
        };
        drop(testcase);
        mark_feature_time!(state, PerfFeature::GetInputFromCorpus);

        for i in 0..num {
            let mut input = input.clone();

            start_timer!(state);
            let mutated = self.mutator_mut().mutate(state, &mut input, i as i32)?;
            mark_feature_time!(state, PerfFeature::Mutate);

            if mutated == MutationResult::Skipped {
                continue;
            }

            // Time is measured directly the `evaluate_input` function
            let (untransformed, post) = input.try_transform_into(state)?;
            let (res, corpus_idx) = fuzzer.evaluate_input(state, executor, manager, untransformed)?;
            match res {
                ExecuteInputResult::Solution => {
                    // println!("result is ExecuteInputResult::Solution");
                    SOLUTION_FLAG.store(1, Ordering::SeqCst);
                }
                _ => {
                    // println!("result is not ExecuteInputResult::Solution");
                    // 继续循环
                }
            }

            start_timer!(state);
            self.mutator_mut().post_exec(state, i as i32, corpus_idx)?;
            post.post_exec(state, i as i32, corpus_idx)?;
            mark_feature_time!(state, PerfFeature::MutatePostExec);
        }
        Ok(())
    }
}
lazy_static! {
    static ref VAR_STORE: Mutex<tch::nn::VarStore> = Mutex::new(tch::nn::VarStore::new(tch::Device::Cpu));
}

impl<E, F, EM, I, M, Z> Stage<E, EM, Z> for PowerMutationalStageWithId<E, F, EM, I, M, Z>
    where
        E: Executor<EM, Z> + HasObservers,
        EM: UsesState<State = E::State>,
        F: TestcaseScoreWithId<E::State>,
        M: Mutator<I, E::State>,
        E::State: HasClientPerfMonitor + HasCorpus + HasMetadata + HasRand,
        Z: Evaluator<E, EM, State = E::State>,
        I: MutatedTransform<E::Input, E::State> + Clone,
{
    #[inline]
    #[allow(clippy::let_and_return)]
    fn perform(
        &mut self,
        fuzzer: &mut Z,
        executor: &mut E,
        state: &mut E::State,
        manager: &mut EM,
        corpus_idx: CorpusId,
    ) -> Result<(), Error> {
        // 在即将变异的地方增加计数

        MUTATE_SUCCESS_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("===============================================================执行mutate stage perform======================================================================");

        //dqn_1
        let mut env = crate::evm::ENV.lock().unwrap();
        let episodes = *crate::evm::EPISODES.lock().unwrap();
        let batch_size = *crate::evm::BATCH_SIZE.lock().unwrap();
        let mut agent = crate::evm::AGENT.lock().unwrap();
        // let mut var_store = VAR_STORE.lock().unwrap();
        // var_store.load("./test_model").unwrap();
        // let mut agent = DQNAgent::new_from_model(&mut var_store, "./test_model", *crate::evm::STATE_DIM.lock().unwrap() as i64, *crate::evm::ACTION_DIM.lock().unwrap() as i64, *crate::evm::REPLAY_BUFFER_CAPACITY.lock().unwrap() as usize).unwrap();

        let mut state_tensor = env.reset();
        // let epsilon = 0.8;
        let mut epsilon = EPSILON.lock().unwrap();
        let (action,action_index) = agent.get_action(&state_tensor, *epsilon);
        env.step_1(action);

        //执行变异
        let ret = self.perform_mutational(fuzzer, executor, state, manager, corpus_idx);

        //dqn_评估
        let (next_state, reward) = env.step_2();
        agent.replay_buffer.push(state_tensor, action_index, reward, next_state.clone(&next_state));
        state_tensor=next_state;
        agent.update_model(batch_size as usize);
        //
        // *epsilon = (*epsilon * *EPSILON_DECAY.lock().unwrap()).max(*FINAL_EPSILON.lock().unwrap());
        println!("update model===========");
        if MUTATE_SUCCESS_COUNT.load(std::sync::atomic::Ordering::SeqCst) % 5000 == 0 {
            // Save the model 画loss图
            agent.model.save("./dqn_net.ot").unwrap();
            let loss_values = LOSS_VALUES.lock().unwrap();
            match plot_loss_values(&loss_values) {
                Ok(_) => (),
                Err(e) => return Err(Error::Unknown(format!("{}", e), ErrorBacktrace::new())),
            }

            plot_reward_values();
            let filename = "res/action_counts.png";
            match plot_action_counts(&ACTION_COUNTS, filename) {
                Ok(_) => println!("Pie chart saved to {}", filename),
                Err(e) => eprintln!("Error generating pie chart: {}", e),
            }
        }
        // let avg_reward = agent.evaluate(&mut *env, episodes.try_into().unwrap());
        // println!("Average reward: {}", avg_reward);

        calculate_value();

        // adjust_p_table();
        ret
    }
}
pub fn plot_reward_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("res/rewards.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Reward Values Over Time", ("Arial", 20).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0f32..100f32, 0f32..100f32)?;

    chart.configure_mesh().draw()?;

    let reward_values = REWARD_VALUES.lock().unwrap();
    let data: Vec<(f32, f32)> = (*reward_values)
        .iter()
        .enumerate()
        .map(|(i, val)| (i as f32, *val as f32))
        .collect();

    chart.draw_series(LineSeries::new(data, &RED))?;

    Ok(())
}
fn plot_action_counts(action_counts: &Mutex<HashMap<i32, i64>>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let action_counts = action_counts.lock().unwrap();

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Action Counts", ("sans-serif", 50).into_font())
        .build_cartesian_2d(0i32..(action_counts.len() as i32), 0i64..*action_counts.values().max().unwrap())?;

    chart.configure_mesh().draw()?;

    let total: i64 = action_counts.values().sum();
    let mut idx = 0;
    for (action, count) in action_counts.iter() {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(idx, 0), (idx + 1, *count)],
            *&Palette99::pick(*action as usize).filled(),
        )))?;
        idx += 1;
    }

    Ok(())
}
pub fn plot_loss_values(loss_values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("res/loss/new.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_loss_value = loss_values.iter().fold(f32::MIN, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption("Loss Values", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0f32..(loss_values.len() * 2) as f32, 0f32..max_loss_value)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        loss_values.iter().enumerate().map(|(x, y)| (x as f32, *y)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}

// pub fn plot_loss_values(loss_values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
//     let root = BitMapBackend::new("res/loss/new2.png", (640, 480)).into_drawing_area();
//     root.fill(&WHITE)?;
//
//     let max_loss_value = loss_values.iter().fold(f32::MIN, |a, &b| a.max(b));
//
//     let mut chart = ChartBuilder::on(&root)
//         .caption("Loss Values", ("sans-serif", 50).into_font())
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_ranged(0f32..loss_values.len() as f32, 0f32..max_loss_value)?;
//
//     chart.configure_mesh().draw()?;
//
//     chart.draw_series(LineSeries::new(
//         loss_values.iter().enumerate().map(|(x, y)| (x as f32, *y)),
//         &RED,
//     ))?;
//
//     root.present()?;
//     Ok(())
// }
impl<E, F, EM, M, Z> PowerMutationalStageWithId<E, F, EM, E::Input, M, Z>
    where
        E: Executor<EM, Z> + HasObservers,
        EM: UsesState<State = E::State>,
        F: TestcaseScoreWithId<E::State>,
        M: Mutator<E::Input, E::State>,
        E::State: HasClientPerfMonitor + HasCorpus + HasMetadata + HasRand,
        Z: Evaluator<E, EM, State = E::State>,
{
    /// Creates a new [`PowerMutationalStageWithId`]
    pub fn new(mutator: M) -> Self {
        Self::transforming(mutator)
    }
}

impl<E, F, EM, I, M, Z> PowerMutationalStageWithId<E, F, EM, I, M, Z>
    where
        E: Executor<EM, Z> + HasObservers,
        EM: UsesState<State = E::State>,
        F: TestcaseScoreWithId<E::State>,
        M: Mutator<I, E::State>,
        E::State: HasClientPerfMonitor + HasCorpus + HasMetadata + HasRand,
        Z: Evaluator<E, EM, State = E::State>,
{
    /// Creates a new transforming [`PowerMutationalStageWithId`]
    pub fn transforming(mutator: M) -> Self {
        Self {
            mutator,
            phantom: PhantomData,
        }
    }
}
