mod base64;
mod generic;

use std::path::PathBuf;

use base64::{base64_mutators, Base64Generator};

use generic::{
    cov_feedback::CovFeedback,
    executor::CoverageCommandExecutor,
    shmem::{get_coverage_shmem_size, get_shmem},
};

use libafl::{
    corpus::OnDiskCorpus,
    events::{EventConfig, Launcher, LlmpRestartingEventManager},
    feedback_or_fast,
    feedbacks::{AflMapFeedback, CrashFeedback, DiffExitKindFeedback, TimeoutFeedback},
    mutators::StdMOptMutator,
    observers::{
        HitcountsIterableMapObserver, StdErrObserver, StdMapObserver, StdOutObserver, TimeObserver,
    },
    schedulers::{powersched::PowerSchedule, StdWeightedScheduler},
    stages::StdMutationalStage,
    state::StdState,
    Error, Fuzzer, StdFuzzer,
};

use libafl_bolts::{
    cli::parse_args,
    core_affinity::CoreId,
    current_nanos,
    rands::StdRand,
    shmem::{ShMemProvider, StdShMemProvider},
    tuples::tuple_list,
    AsSliceMut,
};

#[cfg(feature = "differential")]
use {
    generic::stdio::DiffStdIOMetadataPseudoFeedback,
    libafl::{
        executors::DiffExecutor,
        feedback_and_fast, feedback_or,
        feedbacks::{differential::DiffResult, ConstFeedback, DiffFeedback, TimeFeedback},
        observers::MultiMapObserver,
    },
    libafl_bolts::ownedref::OwnedMutSlice,
};

#[cfg(feature = "tui")]
use libafl::monitors::tui::{ui::TuiUI, TuiMonitor};
#[cfg(not(feature = "tui"))]
use libafl::monitors::MultiMonitor;

#[cfg(feature = "uutils")]
pub static UUTILS_PREFIX: &str = "./target/uutils_coreutils/target/release/";
#[cfg(feature = "gnu")]
pub static GNU_PREFIX: &str = "./target/GNU_coreutils/src/";
#[cfg(feature = "gnu")]
pub static GNU_GCOV_PREFIX: &str = "./target/GNU_coreutils_coverage/src/";

pub fn main() {
    let util = "base64";
    match fuzz(util) {
        Ok(_) => (),
        Err(Error::ShuttingDown) => {
            println!("Orderly shutdown");
        }
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }
}

fn fuzz(util: &str) -> Result<(), Error> {
    let options = parse_args();

    #[cfg(not(feature = "tui"))]
    let monitor = MultiMonitor::new(|s| println!("{}", s));
    #[cfg(feature = "tui")]
    let monitor = TuiMonitor::new(TuiUI::new("coreutils fuzzer".to_string(), true));

    #[cfg(feature = "uutils")]
    let (uutils_coverage_shmem_size, uutils_path) =
        get_coverage_shmem_size(format!("{UUTILS_PREFIX}{util}"))?;
    #[cfg(feature = "gnu")]
    let (gnu_coverage_shmem_size, gnu_path) =
        get_coverage_shmem_size(format!("{GNU_PREFIX}{util}"))?;

    let run_client = |state: Option<_>,
                      mut mgr: LlmpRestartingEventManager<_, _, _>,
                      core_id: CoreId|
     -> Result<(), Error> {
        #[cfg(feature = "uutils")]
        let (mut uutils_coverage_shmem, uutils_coverage_shmem_description) =
            get_shmem(uutils_coverage_shmem_size)?;

        #[cfg(feature = "gnu")]
        let (mut gnu_coverage_shmem, gnu_coverage_shmem_description) =
            get_shmem(gnu_coverage_shmem_size)?;

        #[cfg(feature = "differential")]
        let combined_coverage_observer = HitcountsIterableMapObserver::new(
            MultiMapObserver::differential("combined-coverage", unsafe {
                vec![
                    OwnedMutSlice::from_raw_parts_mut(
                        uutils_coverage_shmem.as_mut_ptr(),
                        uutils_coverage_shmem.len(),
                    ),
                    OwnedMutSlice::from_raw_parts_mut(
                        gnu_coverage_shmem.as_mut_ptr(),
                        gnu_coverage_shmem.len(),
                    ),
                ]
            }),
        );

        #[cfg(feature = "uutils")]
        let uutils_stdout_observer = StdOutObserver::new("uutils-stdout-observer");
        #[cfg(feature = "uutils")]
        let uutils_stderr_observer = StdErrObserver::new("uutils-stderr-observer");
        #[cfg(feature = "uutils")]
        let uutils_time_observer = TimeObserver::new("uutils-time-observer");
        #[cfg(feature = "uutils")]
        let uutils_coverage_observer = unsafe {
            StdMapObserver::new(
                "uutils-coverage-observer",
                uutils_coverage_shmem.as_slice_mut(),
            )
        };

        #[cfg(feature = "gnu")]
        let gnu_stdout_observer = StdOutObserver::new("gnu-stdout-observer");
        #[cfg(feature = "gnu")]
        let gnu_stderr_observer = StdErrObserver::new("gnu-stderr-observer");
        #[cfg(feature = "gnu")]
        let gnu_time_observer = TimeObserver::new("gnu-time-observer");
        #[cfg(feature = "gnu")]
        let gnu_coverage_observer = unsafe {
            StdMapObserver::new("gnu-coverage-observer", gnu_coverage_shmem.as_slice_mut())
        };

        #[cfg(feature = "differential")]
        let (mut feedback, mut objective) = (|| -> Result<_, Error> {
            let stdout_diff_feedback = DiffFeedback::new(
                "stdout-eq-diff-feedback",
                &uutils_stdout_observer,
                &gnu_stdout_observer,
                |o1, o2| {
                    if o1.stdout != o2.stdout {
                        DiffResult::Diff
                    } else {
                        DiffResult::Equal
                    }
                },
            )?;

            // let stderr_xor_feedback = DiffFeedback::new(
            //     "stderr-eq-diff-feedback",
            //     &uutils_stderr_observer,
            //     &gnu_stderr_observer,
            //     |o1, o2| {
            //         if let Some(r1) = has_stderr(o1) {
            //             if let Some(r2) = has_stderr(o2) {
            //                 if r1 == r2 {
            //                     return DiffResult::Equal;
            //                 }
            //             } else {
            //                 println!("XOR: No stderr for GNU");
            //             }
            //         } else {
            //             println!("XOR: No stderr for uutils");
            //         }
            //         DiffResult::Diff
            //     },
            // )?;

            let stderr_neither_feedback = DiffFeedback::new(
                "stderr-neither-diff-feedback",
                &uutils_stderr_observer,
                &gnu_stderr_observer,
                |o1, o2| {
                    if let Some(r1) = has_stderr(o1) {
                        if let Some(r2) = has_stderr(o2) {
                            if !r1 && !r2 {
                                return DiffResult::Diff; // trigger the feedback
                            }
                        } else {
                            println!("DIFF: No stderr for GNU");
                        }
                    } else {
                        println!("DIFF: No stderr for uutils");
                    }
                    DiffResult::Equal
                },
            )?;

            let gcov_feedback = CovFeedback::new(
                true,
                format!("{GNU_GCOV_PREFIX}{util}"),
                format!("cov-{:?}", core_id.0),
            );

            let metadata_pseudo_feedback = DiffStdIOMetadataPseudoFeedback::new(
                &uutils_path,
                &gnu_path,
                &uutils_stderr_observer,
                &gnu_stderr_observer,
                &uutils_stdout_observer,
                &gnu_stdout_observer,
            );

            let coverage_feedback = AflMapFeedback::new(&combined_coverage_observer);

            let feedback = feedback_or_fast!(
                feedback_and_fast!(coverage_feedback, gcov_feedback),
                metadata_pseudo_feedback.clone()
            );

            // only add logger feedbacks if something was found
            let objective = feedback_and_fast!(
                feedback_or_fast!(
                    // only test stdout equality if neither has a stderr
                    DiffExitKindFeedback::new(),
                    CrashFeedback::new(),
                    TimeoutFeedback::new(),
                    feedback_and_fast!(stderr_neither_feedback, stdout_diff_feedback) // ,stderr_xor_feedback
                ),
                feedback_or!(
                    metadata_pseudo_feedback,
                    TimeFeedback::new(&uutils_time_observer),
                    TimeFeedback::new(&gnu_time_observer),
                    ConstFeedback::new(true) // to ensure the whole block to be interesting
                )
            );

            Ok((feedback, objective))
        })()?;

        #[cfg(all(not(feature = "differential"), feature = "gnu"))]
        let (mut feedback, mut objective) = {
            let gcov_feedback = CovFeedback::new(
                true,
                format!("{GNU_GCOV_PREFIX}{util}"),
                format!("cov-{:?}", core_id.0),
            );
            let feedback =
                feedback_and_fast!(AflMapFeedback::new(&gnu_coverage_observer), gcov_feedback);
            let objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());
            (feedback, objective)
        };
        #[cfg(all(not(feature = "differential"), feature = "uutils"))]
        let (mut feedback, mut objective) = {
            let feedback = AflMapFeedback::new(&uutils_coverage_observer);
            let objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());
            (feedback, objective)
        };
        let mut state = state.unwrap_or_else(|| {
            StdState::new(
                StdRand::with_seed(current_nanos()),
                OnDiskCorpus::new(PathBuf::from("corpus")).unwrap(),
                OnDiskCorpus::new(PathBuf::from(&options.output)).unwrap(),
                &mut feedback,
                &mut objective,
            )
            .unwrap()
        });

        let scheduler = StdWeightedScheduler::with_schedule(
            &mut state,
            #[cfg(feature = "differential")]
            &combined_coverage_observer,
            #[cfg(all(not(feature = "differential"), feature = "uutils"))]
            &uutils_coverage_observer,
            #[cfg(all(not(feature = "differential"), feature = "gnu"))]
            &gnu_coverage_observer,
            Some(PowerSchedule::FAST),
        );

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
        #[cfg(feature = "uutils")]
        let uutils_executor = CoverageCommandExecutor::new(
            &uutils_coverage_shmem_description,
            tuple_list!(
                uutils_coverage_observer,
                uutils_stdout_observer,
                uutils_stderr_observer,
                uutils_time_observer
            ),
            &uutils_path,
            format!("uutils-{:?}", core_id.0),
        );

        #[cfg(feature = "gnu")]
        let gnu_executor = CoverageCommandExecutor::new(
            &gnu_coverage_shmem_description,
            tuple_list!(
                gnu_coverage_observer,
                gnu_stdout_observer,
                gnu_stderr_observer,
                gnu_time_observer
            ),
            &gnu_path,
            format!("gnu-{:?}", core_id.0),
        );

        #[cfg(feature = "differential")]
        let diff_executor = DiffExecutor::new(
            uutils_executor,
            gnu_executor,
            tuple_list!(combined_coverage_observer),
        );
        #[cfg(feature = "differential")]
        let mut executor = diff_executor;
        #[cfg(all(not(feature = "differential"), feature = "uutils"))]
        let mut executor = uutils_executor;
        #[cfg(all(not(feature = "differential"), feature = "gnu"))]
        let mut executor = gnu_executor;

        if state.must_load_initial_inputs() {
            state.generate_initial_inputs(
                &mut fuzzer,
                &mut executor,
                &mut Base64Generator::new(2),
                &mut mgr,
                8,
            )?
        }

        let mut stages = tuple_list!(StdMutationalStage::new(StdMOptMutator::new(
            &mut state,
            base64_mutators(),
            7,
            5
        )?));

        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
    };

    let launcher_shmem_provider = StdShMemProvider::new()?;

    Launcher::builder()
        .configuration(EventConfig::AlwaysUnique)
        .shmem_provider(launcher_shmem_provider)
        .monitor(monitor)
        .run_client(run_client)
        .cores(&options.cores)
        .broker_port(options.broker_port)
        .stdout_file(Some(&options.stdout))
        .remote_broker_addr(options.remote_broker_addr)
        .build()
        .launch()
}

pub fn has_stderr(o: &StdErrObserver) -> Option<bool> {
    o.stderr.as_ref().map(|e| !e.is_empty())
}
