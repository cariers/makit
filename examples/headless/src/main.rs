//! 通过引擎输入运行准备场景的无界面宿主。

mod cli;

use std::process::ExitCode;

use makit::{Engine, EngineInput, EngineState, Error, IntoMachineState};
use makit_headless::{
    DemoError, DemoEvent, DemoInput, DemoMachine, DemoPhase, DemoRule, DemoVariant,
};

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
#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("input {input} was not handled in state {state}")]
    UnhandledInput {
        input: &'static str,
        state: &'static str,
    },
    #[error("input {input} failed in state {state}: {source}")]
    Business {
        input: &'static str,
        state: &'static str,
        #[source]
        source: DemoError,
    },
    #[error("the preparation script ended without a final output in state {state}")]
    MissingOutput { state: &'static str },
}

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
            Ok(events) => {
                println!("Result: Handled ({} events)", events.len());
                for event in &events {
                    print_event(event);
                }
            }
            Err(Error::Unhandled) => {
                let state = state_name(&machine);
                println!("Result: Unhandled");
                println!("State: {state}");
                return Err(RunError::UnhandledInput { input: name, state });
            }
            Err(Error::Custom(source)) => {
                let state = state_name(&machine);
                println!("Result: Failed");
                println!("State: {state}");
                return Err(RunError::Business {
                    input: name,
                    state,
                    source,
                });
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
