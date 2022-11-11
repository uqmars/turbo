use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};


/// Advent of Code Leaderboard
///
/// Activates the Advent of Code Python Script

pub async fn aoc() {
    Python::with_gil(|py| {
        let aoc_help = PyModule::from_code(py, #r"run", "/python-scripts/aoc/main.py", "help")?;
    })
}
