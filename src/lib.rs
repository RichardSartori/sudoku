use std::{cmp, default, fmt, iter, ops, slice, str};

/** SETUP **/

const ROW_PER_BLOCK: usize = 3;
const COL_PER_BLOCK: usize = 3;
const SYMBOLS: [ &str; SIZE ] = [ "1", "2", "3", "4", "5", "6", "7", "8", "9" ];
const PLACEHOLDER: &str = "_";
const SEPARATOR: char = ',';

// conversions between symbols and indices
fn from_symbol(symbol: &str) -> Option<usize> {
	SYMBOLS.iter().position(|&s| s == symbol)
}
fn from_index(index: usize) -> &'static str {
	SYMBOLS[index]
}

/** CONSTANTS **/

const SIZE: usize = ROW_PER_BLOCK * COL_PER_BLOCK;
const INDICES: ops::Range<usize> = 0..SIZE;

/** DIGIT **/

#[derive(Clone, Copy, Default, Eq)]
enum Digit {
	#[default]
	None,
	Some { given: bool, index: usize },
}

#[derive(Debug)]
pub enum ParseDigitError {
	Unknown,
}

impl str::FromStr for Digit {
	type Err = ParseDigitError;
	fn from_str(slice: &str) -> Result<Self, Self::Err> {
		if slice == PLACEHOLDER {
			return Ok(Digit::None);
		}
		let index = from_symbol(slice)
			.ok_or(ParseDigitError::Unknown)?;
		let given = true;
		Ok(Digit::Some { given, index })
	}
}

impl fmt::Display for Digit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Digit::None => write!(f, "{}", PLACEHOLDER),
			Digit::Some { index, .. } => {
				let symbol = from_index(index);
				write!(f, "{}", symbol)
			},
		}
	}
}

impl cmp::PartialEq for Digit {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Digit::None, Digit::None) => true,
			(Digit::Some { index: a, .. }, Digit::Some { index: b, .. }) => {
				a == b
			},
			_ => false,
		}
	}
}

/** BITSET **/

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct BitSet([ bool; SIZE ]);

impl BitSet {

	fn set(&mut self, index: usize) {
		(self.0)[index] = true;
	}

	fn reset(&mut self, index: usize) {
		(self.0)[index] = false;
	}

	fn contains(&self, index: usize) -> bool {
		(self.0)[index]
	}
}

impl ops::BitOr for BitSet {
	type Output = Self;
	fn bitor(self, rhs: Self) -> Self::Output {
		let mut result: Self = Default::default();
		for index in INDICES {
			if self.contains(index) | rhs.contains(index) {
				result.set(index)
			}
		}
		result
	}
}

/** GRID **/

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Grid([[ Digit; SIZE ]; SIZE ]);

impl Grid {

	fn iter_mut<'a>(&'a mut self)
		-> iter::Flatten<slice::IterMut<'a, [ Digit; SIZE ]>>
	{
		return self.0.iter_mut().flatten()
	}

	pub fn solve(self) -> Result<Solver, SolverError> {
		Solver::try_from(self)
	}
}

#[derive(Debug)]
pub enum ParseGridError {
	ParseDigit(ParseDigitError),
	InvalidDigitCount,
}

impl str::FromStr for Grid {
	type Err = ParseGridError;
	fn from_str(value: &str) -> Result<Self, Self::Err> {
		let mut result: Self = Default::default();
		let mut grid_iter = result.iter_mut();
		let mut value_iter = value.split(SEPARATOR);
		loop {
			match (value_iter.next(), grid_iter.next()) {
				(None, None) => {
					break;
				},
				(Some(_), None) | (None, Some(_)) => {
					return Err(ParseGridError::InvalidDigitCount);
				},
				(Some(slice), Some(cell)) => {
					*cell = slice
						.parse::<Digit>()
						.map_err(|err| ParseGridError::ParseDigit(err))?;
				},
			}
		}
		Ok(result)
	}
}

impl fmt::Display for Grid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for row in self.0 {
			write!(f, "\n")?;
			for digit in row {
				write!(f, "{digit},")?;
			}
		}
		Ok(())
	}
}

impl fmt::Debug for Grid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		<Self as fmt::Display>::fmt(self, f)
	}
}

/** CURSOR **/

#[derive(Clone, Copy)]
struct Cursor {
	index: usize,
	forward: bool,
}

type CursorItem = (usize, usize, usize);

enum CursorError {
	BeforeBegin,
	AfterEnd,
}

impl Cursor {

	fn item(&self) -> CursorItem {
		let row = self.index / SIZE;
		let col = self.index % SIZE;
		let blk = (row / ROW_PER_BLOCK) * ROW_PER_BLOCK + (col / COL_PER_BLOCK);
		(row, col, blk)
	}

	fn next(&mut self) -> Result<(), CursorError> {
		if self.index == SIZE * SIZE - 1 {
			self.forward = false;
			return Err(CursorError::AfterEnd);
		}
		self.index += 1;
		self.forward = true;
		Ok(())
	}

	fn prev(&mut self) -> Result<(), CursorError> {
		if self.index == 0 {
			self.forward = true;
			return Err(CursorError::BeforeBegin);
		}
		self.index -= 1;
		self.forward = false;
		Ok(())
	}

	fn skip(&mut self) -> Result<(), CursorError> {
		match self.forward {
			true  => self.next(),
			false => self.prev(),
		}
	}
}

impl default::Default for Cursor {
	fn default() -> Self {
		Cursor { index: 0, forward: true }
	}
}

/** SOLVER **/

pub struct Solver {
	grid: Grid,
	cursor: Cursor,
	row_states: [ BitSet; SIZE ],
	col_states: [ BitSet; SIZE ],
	blk_states: [ BitSet; SIZE ],
}

impl Solver {
	fn new(grid: Grid) -> Self {
		let cursor = Default::default();
		let row_states = Default::default();
		let col_states = Default::default();
		let blk_states = Default::default();
		Solver { grid, cursor, row_states, col_states, blk_states }
	}
}

#[derive(Debug)]
pub enum SolverError {
	InvalidGrid,
}

impl TryFrom<Grid> for Solver {
	type Error = SolverError;
	fn try_from(value: Grid) -> Result<Self, Self::Error> {
		let mut result = Solver::new(value);
		let mut cursor: Cursor = Default::default();
		loop {
			let (row, col, blk) = cursor.item();
			if let Digit::Some { index, .. } = (result.grid.0)[row][col] {
				let taken = result.row_states[row]
					| result.col_states[col]
					| result.blk_states[blk];
				if taken.contains(index) {
					return Err(SolverError::InvalidGrid);
				}
				result.row_states[row].set(index);
				result.col_states[col].set(index);
				result.blk_states[blk].set(index);
			}
			if cursor.next().is_err() {
				return Ok(result);
			}
		}
	}
}

fn next_index(digit: Digit, taken: &BitSet) -> Option<usize> {
	for available in INDICES {
		if taken.contains(available) {
			continue;
		}
		if let Digit::Some { index, .. } = digit {
			if available <= index {
				continue;
			}
		}
		return Some(available);
	}
	None
}

impl Iterator for Solver {
	type Item = Grid;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (row, col, blk) = self.cursor.item();
			let digit = (self.grid.0)[row][col];
			if let Digit::Some { given, .. } = digit {
				if given {
					// cursor points to given digit, skip it
					if let Err(err) = Cursor::skip(&mut self.cursor) {
						match err {
							CursorError::BeforeBegin => {
								return None;
							},
							CursorError::AfterEnd => {
								return Some(self.grid.clone());
							},
						}
					}
					continue;
				}
			}
			let taken = self.row_states[row]
				| self.col_states[col]
				| self.blk_states[blk];
			if let Digit::Some { index, .. } = digit {
				self.row_states[row].reset(index);
				self.col_states[col].reset(index);
				self.blk_states[blk].reset(index);
				(self.grid.0)[row][col] = Digit::None;
			}
			match next_index(digit, &taken) {
				None => {
					// no index is possible, decrease cursor
					if Cursor::prev(&mut self.cursor).is_err() {
						return None;
					}
				},
				Some(index) => {
					// an index is available, set digit and increase cursor
					self.row_states[row].set(index);
					self.col_states[col].set(index);
					self.blk_states[blk].set(index);
					let given = false;
					(self.grid.0)[row][col] = Digit::Some { given, index };
					if Cursor::next(&mut self.cursor).is_err() {
						return Some(self.grid.clone());
					}
				},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn invalid_inputs() {
		let not_digit = "\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,x,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_";
		assert!(not_digit.parse::<Grid>().is_err());
		let not_in_range = "\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,0,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_";
		assert!(not_in_range.parse::<Grid>().is_err());
		let invalid_count = "\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_";
		assert!(invalid_count.parse::<Grid>().is_err());
		let repetition = "\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,1,_,\
            _,_,_,_,_,_,_,_,1";
		let repetition = repetition.parse::<Grid>().unwrap();
		assert!(repetition.solve().is_err());
	}

	#[test]
	#[ignore] // take longer than other tests (~7 sec)
	fn no_solution() {
		let input = "\
            _,_,_,1,2,_,_,_,_,\
            _,_,_,_,_,_,1,2,_,\
            _,_,_,_,_,_,_,_,_,\
            1,_,_,_,_,_,_,_,_,\
            2,_,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_,\
            _,1,_,_,_,_,_,_,_,\
            _,2,_,_,_,_,_,_,_,\
            _,_,_,_,_,_,_,_,_";
		let input = input.parse::<Grid>().unwrap();
		let mut solutions = input.solve().unwrap();
		assert!(solutions.next().is_none());
	}

	#[test]
	fn one_solution() {
		let input = "\
            5,3,_,_,7,_,_,_,_,\
            6,_,_,1,9,5,_,_,_,\
            _,9,8,_,_,_,_,6,_,\
            8,_,_,_,6,_,_,_,3,\
            4,_,_,8,_,3,_,_,1,\
            7,_,_,_,2,_,_,_,6,\
            _,6,_,_,_,_,2,8,_,\
            _,_,_,4,1,9,_,_,5,\
            _,_,_,_,8,_,_,7,9";
		let expected = "\
            5,3,4,6,7,8,9,1,2,\
            6,7,2,1,9,5,3,4,8,\
            1,9,8,3,4,2,5,6,7,\
            8,5,9,7,6,1,4,2,3,\
            4,2,6,8,5,3,7,9,1,\
            7,1,3,9,2,4,8,5,6,\
            9,6,1,5,3,7,2,8,4,\
            2,8,7,4,1,9,6,3,5,\
            3,4,5,2,8,6,1,7,9";
		let input = input.parse::<Grid>().unwrap();
		let expected = expected.parse::<Grid>().unwrap();
		assert_ne!(input, expected);
		let mut solver = input.solve().unwrap();
		assert_eq!(solver.next().unwrap(), expected);
		assert!(solver.next().is_none());
	}

	#[test]
	fn two_solutions() {
		// only the last 2 digits differ from one_solution
		let input = "\
            5,3,_,_,7,_,_,_,_,\
            6,_,_,1,9,5,_,_,_,\
            _,9,8,_,_,_,_,6,_,\
            8,_,_,_,6,_,_,_,3,\
            4,_,_,8,_,3,_,_,1,\
            7,_,_,_,2,_,_,_,6,\
            _,6,_,_,_,_,2,8,_,\
            _,_,_,4,1,9,_,_,5,\
            _,_,_,_,8,_,_,_,_";
		let input = input.parse::<Grid>().unwrap();
		let mut solver = input.solve().unwrap();
		assert!(solver.next().is_some());
		assert!(solver.next().is_some());
		assert!(solver.next().is_none());
	}
}
