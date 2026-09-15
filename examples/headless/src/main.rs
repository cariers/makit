//! 通过引擎输入运行准备场景的无界面宿主。

mod cli;

use std::{fmt, process::ExitCode};

use makit::{DispatchResult, Engine, EngineInput, EngineState, IntoMachineState};
use makit_headless::{DemoEvent, DemoInput, DemoMachine, DemoPhase, DemoRule, DemoVariant};

fn main() -> ExitCode {
    let options = match cli::parse(std::env::args_os().skip(1)) {
        Ok(cli::Command::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(cli::Command::Run(options)) => options,
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Use --help for usage.");
            return ExitCode::from(2);
        }
    };

    match run(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// 预设脚本无法完成时，保留失败输入及实际阶段供诊断。
#[derive(Debug)]
enum RunError {
    UnhandledInput {
        input: &'static str,
        state: &'static str,
    },
    MissingOutput {
        state: &'static str,
    },
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnhandledInput { input, state } => {
                write!(formatter, "input {input} was not handled in state {state}")
            }
            Self::MissingOutput { state } => write!(
                formatter,
                "the preparation script ended without a final output in state {state}"
            ),
        }
    }
}

impl std::error::Error for RunError {}

fn run(options: cli::Options) -> Result<(), RunError> {
    let rule = DemoRule::new(options.seed, options.align_east);
    let mut machine: DemoMachine = Engine::<DemoVariant>::new(rule).into_machine();

    println!("Scenario: Shuffle and determine dealer");
    println!("Align east: {}", options.align_east);
    println!("State: {}", state_name(&machine));

    // 宿主只提交脚本输入；行动启动、事件包装和阶段切换均由演示 Phase 处理。
    let script = [
        ("Start", EngineInput::Start),
        ("RunShuffle", EngineInput::Phase(DemoInput::RunShuffle)),
        (
            "DetermineDealer",
            EngineInput::Phase(DemoInput::DetermineDealer),
        ),
    ];
    for (name, input) in script {
        println!("Input: {name}");
        match machine.dispatch(&input) {
            DispatchResult::Handled { events } => {
                println!("Result: Handled ({} events)", events.len());
                for event in &events {
                    print_event(event);
                }
            }
            DispatchResult::Unhandled => {
                let state = state_name(&machine);
                println!("Result: Unhandled");
                println!("State: {state}");
                return Err(RunError::UnhandledInput { input: name, state });
            }
        }
        println!("State: {}", state_name(&machine));
    }

    let output = machine.state().output().ok_or(RunError::MissingOutput {
        state: state_name(&machine),
    })?;
    let wall: Vec<_> = output.wall.iter().map(|tile| tile.code()).collect();
    println!("Preparation complete");
    println!("Dice: {:?}", output.dice);
    println!("Dealer: {}", output.dealer.code());
    println!("East: {}", output.east.seat().code());
    println!("Remaining tiles: {}", output.wall.len());
    println!("Wall codes: {wall:?}");
    Ok(())
}

fn state_name(machine: &DemoMachine) -> &'static str {
    match machine.state() {
        EngineState::Preparing => "Preparing",
        EngineState::Running(DemoPhase::Shuffling(_)) => "Running(Shuffling)",
        EngineState::Running(DemoPhase::DeterminingDealer(_)) => "Running(DeterminingDealer)",
        EngineState::Finished(_) => "Finished",
    }
}

fn print_event(event: &DemoEvent) {
    match event {
        DemoEvent::Shuffled(event) => {
            println!("Event: Shuffled (tile_count={})", event.tile_count);
        }
        DemoEvent::DealerDetermined(event) => println!(
            "Event: DealerDetermined (dice={:?}, dealer={}, east={})",
            event.dice,
            event.dealer.code(),
            event.east.seat().code()
        ),
    }
}
