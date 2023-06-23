use std::env;

#[derive(Debug)]
enum MainError {
	MissingInput,
	ParseGrid(sudoku::ParseGridError),
	Solver(sudoku::SolverError),
}

fn main() -> Result<(), MainError> {
	let mut args = env::args();
	let _progname = args.next();

	let grid = args
		.next()
		.ok_or(MainError::MissingInput)?
		.parse::<sudoku::Grid>()
		.map_err(MainError::ParseGrid)?;
	println!("Input:{grid}");

	println!("Solving");
	let solver = grid.solve().map_err(MainError::Solver)?;
	for (n, solution) in solver.enumerate() {
		println!("solution[{n}]:{solution}");
	}
	println!("Solved");

	Ok(())
}
