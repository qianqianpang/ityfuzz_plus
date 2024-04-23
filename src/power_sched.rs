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

use crate::global_info::{adjust_p_table, USE_MULTI_ARMED_BANDIT,MUTATE_SUCCESS_COUNT};

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
        let ret = self.perform_mutational(fuzzer, executor, state, manager, corpus_idx);
        // print_feedback_info();
        // calculate_value();
        // println!("===============================更新ptable之前=============================");
        // print_value();
        // print_mutation_op();
        // print_p_table();
        let use_multi_armed_bandit = USE_MULTI_ARMED_BANDIT.lock().unwrap();
        if *use_multi_armed_bandit {
            adjust_p_table();
        }
        // println!("===============================更新ptable之后=============================");
        // print_mutation_op();
        // print_p_table();
        ret
    }
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
