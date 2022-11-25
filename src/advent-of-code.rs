use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};


/// Advent of Code Leaderboard
///
/// Activates the Advent of Code Python Script

const AOC_SCRIPT: &str = "/python-scripts/aoc/main.py".to_string();

pub async fn aoc(ctx: CTX, msg: Message) -> String {
    let aoc_help = PyModule::from_code(py, #r"
        async def help(ctx, *args):
        ", AOC_SCRIPT, "help")?;

    let aoc-help-message: String = aoc_help.getattr("help")?
        .call(msg.to_string())?
        .extract()?;
    
    return aoc-help-message.to_string();
 }
