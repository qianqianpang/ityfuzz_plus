//! The power schedules. This stage should be invoked after the calibration
//! stage.
use core::{fmt::Debug, marker::PhantomData};
use std::sync::atomic::Ordering;

use libafl::{
    corpus::{Corpus, CorpusId},
    Error,
    executors::{Executor, HasObservers},
    fuzzer::Evaluator,
    mutators::Mutator,
    prelude::Testcase,
    stages::{mutational::MutatedTransform, MutationalStage, Stage},
    state::{HasClientPerfMonitor, HasCorpus, HasMetadata, HasRand, UsesState},
};
use plotters::prelude::*;
use libafl_bolts::ErrorBacktrace;

use crate::evm::LOSS_VALUES;
// use crate::evm::{AGENT, ENV, EPISODES, BATCH_SIZE};
use crate::global_info::MUTATE_SUCCESS_COUNT;

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

        MUTATE_SUCCESS_COUNT.fetch_add(1, Ordering::SeqCst);
        println!("===============================================================执行mutate stage perform======================================================================");

        //dqn_1
        let mut env = crate::evm::ENV.lock().unwrap();
        let episodes = *crate::evm::EPISODES.lock().unwrap();
        let batch_size = *crate::evm::BATCH_SIZE.lock().unwrap();
        let mut agent = crate::evm::AGENT.lock().unwrap();
        let mut state_tensor = env.reset();
        let epsilon = 0.8;
        let (action,action_index) = agent.get_action(&state_tensor, epsilon);
        env.step_1(action);

        //执行变异
        let ret = self.perform_mutational(fuzzer, executor, state, manager, corpus_idx);

        //dqn_评估
        let (next_state, reward) = env.step_2();
        agent.replay_buffer.push(state_tensor, action_index, reward, next_state.clone(&next_state));
        state_tensor=next_state;
        agent.update_model(batch_size as usize);
        println!("update model===========");
        if MUTATE_SUCCESS_COUNT.load(Ordering::SeqCst) > episodes as usize {
            // Save the model
            agent.model.save("./test_model").unwrap();
            let loss_values = LOSS_VALUES.lock().unwrap();

            match plot_loss_values(&loss_values) {
                Ok(_) => (),
                Err(e) => return Err(libafl::Error::Unknown(format!("{}", e), ErrorBacktrace::new())),
            }
            std::process::exit(0);
        }
        // let avg_reward = agent.evaluate(&mut *env, episodes.try_into().unwrap());
        // println!("Average reward: {}", avg_reward);

        // calculate_value();
        // adjust_p_table();
        ret
    }
}


fn plot_loss_values(loss_values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("loss_values.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_loss_value = loss_values.iter().fold(f32::MIN, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption("Loss Values", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0f32..loss_values.len() as f32, 0f32..max_loss_value)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        loss_values.iter().enumerate().map(|(x, y)| (x as f32, *y)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}
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
