// Interactive tutorial system for teaching KenKen solving strategies

use std::fmt;

/// A single tutorial lesson teaching a KenKen concept.
#[derive(Debug, Clone)]
pub struct Lesson {
    /// Lesson number (1-5)
    pub id: u8,
    /// Title of lesson
    pub title: String,
    /// Learning objective
    pub objective: String,
    /// Grid size for lesson puzzle (2-6)
    pub grid_size: u8,
    /// Puzzle description in SGT format
    pub puzzle_desc: String,
    /// Expected solution as flat grid
    pub solution: Vec<u8>,
    /// Step-by-step hints/deductions
    pub steps: Vec<LessonStep>,
}

/// Single step within a lesson.
#[derive(Debug, Clone)]
pub struct LessonStep {
    /// Step number within lesson
    pub number: u16,
    /// What to try or observe
    pub instruction: String,
    /// Why this is correct (reasoning)
    pub explanation: String,
    /// Hint if student is stuck
    pub hint: Option<String>,
}

impl Lesson {
    /// Create Lesson 1: Latin Square Basics (2x2).
    pub fn lesson_1() -> Self {
        Lesson {
            id: 1,
            title: "Latin Square Basics".to_string(),
            objective: "Learn that each row and column must contain each digit exactly once"
                .to_string(),
            grid_size: 2,
            puzzle_desc: "a__,a__".to_string(),
            solution: vec![1, 2, 2, 1],
            steps: vec![
                LessonStep {
                    number: 1,
                    instruction: "Look at row 0 (top row). It has cells (0,0) and (0,1)."
                        .to_string(),
                    explanation: "In a 2x2 grid, each row must have digits 1 and 2.".to_string(),
                    hint: Some("Count how many unique values can go in each cell".to_string()),
                },
                LessonStep {
                    number: 2,
                    instruction: "Column 0 (left column) has cells (0,0) and (1,0).".to_string(),
                    explanation: "Just like rows, columns must also have each digit 1-2."
                        .to_string(),
                    hint: Some("Try placing 1 at (0,0). What must go in (1,0)?".to_string()),
                },
                LessonStep {
                    number: 3,
                    instruction: "If (0,0) = 1, then (1,0) must be 2 (column constraint)."
                        .to_string(),
                    explanation: "Since column 0 needs both 1 and 2, and (0,0)=1, then (1,0)=2."
                        .to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 4,
                    instruction: "Then (0,1) must be 2 and (1,1) must be 1 (row constraints)."
                        .to_string(),
                    explanation:
                        "Row 0 needs both 1 and 2. Since (0,0)=1, (0,1)=2. Same logic for row 1."
                            .to_string(),
                    hint: None,
                },
            ],
        }
    }

    /// Create Lesson 2: Cage Operations (3x3).
    pub fn lesson_2() -> Self {
        Lesson {
            id: 2,
            title: "Cage Operations".to_string(),
            objective: "Learn how cage constraints (Add, Mul, Sub, Div) further restrict values"
                .to_string(),
            grid_size: 3,
            puzzle_desc: "a__,a__,a__".to_string(),
            solution: vec![1, 2, 3, 2, 3, 1, 3, 1, 2],
            steps: vec![
                LessonStep {
                    number: 1,
                    instruction: "Look for 2-cell cages with operation targets."
                        .to_string(),
                    explanation: "An Add=3 cage with 2 cells can only be 1+2 (in a 3x3 grid)."
                        .to_string(),
                    hint: Some("What are all pairs of digits 1-3 that sum to 3?".to_string()),
                },
                LessonStep {
                    number: 2,
                    instruction: "A Mul=2 cage must be 1*2, since 1*1=1 and 2*2=4 (too large)."
                        .to_string(),
                    explanation: "Multiplication constrains which pairs are possible."
                        .to_string(),
                    hint: Some("What products can you make with digits 1-3?".to_string()),
                },
                LessonStep {
                    number: 3,
                    instruction: "Once a cage is determined, cells get assigned values."
                        .to_string(),
                    explanation: "Knowing a 2-cell cage is 1+2 means those cells are 1 and 2 (in some order)."
                        .to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 4,
                    instruction: "Use row/column constraints to determine order within cages."
                        .to_string(),
                    explanation: "If row 0 already has 1, then 1+2 cage in row 0 must have 2 first."
                        .to_string(),
                    hint: None,
                },
            ],
        }
    }

    /// Create Lesson 3: Deduction Strategies (4x4).
    pub fn lesson_3() -> Self {
        Lesson {
            id: 3,
            title: "Deduction Strategies".to_string(),
            objective: "Learn techniques to deduce values without guessing".to_string(),
            grid_size: 4,
            puzzle_desc: "a___,a___,a___,a___".to_string(),
            solution: vec![1, 2, 3, 4, 2, 1, 4, 3, 3, 4, 1, 2, 4, 3, 2, 1],
            steps: vec![
                LessonStep {
                    number: 1,
                    instruction: "Naked single: if cell can only be one value, place it."
                        .to_string(),
                    explanation:
                        "Apply row, column, and cage constraints to eliminate possibilities."
                            .to_string(),
                    hint: Some("Count which digits are already in the row and column".to_string()),
                },
                LessonStep {
                    number: 2,
                    instruction:
                        "Hidden single: if only one cell in a unit can have a value, place it."
                            .to_string(),
                    explanation: "Scan the row/column and find where each missing digit can go."
                        .to_string(),
                    hint: Some(
                        "For each missing digit in row 0, count valid placements".to_string(),
                    ),
                },
                LessonStep {
                    number: 3,
                    instruction: "Cage constraints reduce possibilities quickly in 4x4."
                        .to_string(),
                    explanation: "Large cages (3+ cells) have few valid combinations.".to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 4,
                    instruction: "Try solving with only naked/hidden singles - no guessing needed!"
                        .to_string(),
                    explanation: "Many 4x4 puzzles solve deterministically with good deduction."
                        .to_string(),
                    hint: None,
                },
            ],
        }
    }

    /// Create Lesson 4: Backtracking Logic (5x5).
    pub fn lesson_4() -> Self {
        Lesson {
            id: 4,
            title: "Backtracking Logic".to_string(),
            objective: "Understand why solvers sometimes guess and backtrack when deduction stalls"
                .to_string(),
            grid_size: 5,
            puzzle_desc: "a____,a____,a____,a____,a____".to_string(),
            solution: vec![
                1, 2, 3, 4, 5,
                2, 3, 4, 5, 1,
                3, 4, 5, 1, 2,
                4, 5, 1, 2, 3,
                5, 1, 2, 3, 4,
            ],
            steps: vec![
                LessonStep {
                    number: 1,
                    instruction: "After applying all deductions, some cells may have multiple possibilities."
                        .to_string(),
                    explanation: "Deduction finds forced moves. When it stalls, guess at cell with fewest options."
                        .to_string(),
                    hint: Some("Look for cells with exactly 2 possible values (lowest branching factor)"
                        .to_string()),
                },
                LessonStep {
                    number: 2,
                    instruction: "Make a guess: pick a cell and try one candidate value."
                        .to_string(),
                    explanation: "Apply deduction with this assumption. Either it works or leads to contradiction."
                        .to_string(),
                    hint: Some("Remember which cell you guessed so you can backtrack if needed".to_string()),
                },
                LessonStep {
                    number: 3,
                    instruction: "Continue deducing with your guess in place."
                        .to_string(),
                    explanation: "The guess may cascade: new constraints from new assignments enable more deductions."
                        .to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 4,
                    instruction: "If you reach a contradiction, backtrack and try next candidate."
                        .to_string(),
                    explanation: "Contradiction means your guess was wrong. Undo it and try alternative."
                        .to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 5,
                    instruction: "Backtracking with smart guessing (minimum remaining values) is very efficient."
                        .to_string(),
                    explanation: "By choosing cells with fewer options, you minimize search tree depth."
                        .to_string(),
                    hint: None,
                },
            ],
        }
    }

    /// Create Lesson 5: Advanced Constraints (6x6).
    pub fn lesson_5() -> Self {
        Lesson {
            id: 5,
            title: "Advanced Constraints".to_string(),
            objective: "Master complex cage operations and multi-cell interactions"
                .to_string(),
            grid_size: 6,
            puzzle_desc: "a_____,a_____,a_____,a_____,a_____,a_____".to_string(),
            solution: vec![
                1, 2, 3, 4, 5, 6,
                2, 3, 4, 5, 6, 1,
                3, 4, 5, 6, 1, 2,
                4, 5, 6, 1, 2, 3,
                5, 6, 1, 2, 3, 4,
                6, 1, 2, 3, 4, 5,
            ],
            steps: vec![
                LessonStep {
                    number: 1,
                    instruction: "In 6x6, large 3+ cell cages have many combinations to consider."
                        .to_string(),
                    explanation: "Enumerate valid tuples respecting target and no repeats in cage."
                        .to_string(),
                    hint: Some("For Add=15 with 3 cells: what digit triplets sum to 15?".to_string()),
                },
                LessonStep {
                    number: 2,
                    instruction: "Sub and Div cages constrain ordering: a-b must equal target, a/b too."
                        .to_string(),
                    explanation: "Unlike Add/Mul (commutative), Sub/Div care about cell order."
                        .to_string(),
                    hint: Some("For Sub=1 cage: pairs (a,b) where a-b=1: (2,1), (3,2), ..., (6,5)"
                        .to_string()),
                },
                LessonStep {
                    number: 3,
                    instruction: "Interaction: cage tuple constraints + row/column constraints."
                        .to_string(),
                    explanation: "If row has 1,2,3 and cage needs tuple (4,5,6), problem solved."
                        .to_string(),
                    hint: None,
                },
                LessonStep {
                    number: 4,
                    instruction: "Some 6x6 puzzles require backtracking despite advanced deduction."
                        .to_string(),
                    explanation: "Search space grows: intelligent guess + constraint propagation finds solution."
                        .to_string(),
                    hint: None,
                },
            ],
        }
    }

    /// Get all available lessons.
    pub fn all() -> Vec<Self> {
        vec![
            Self::lesson_1(),
            Self::lesson_2(),
            Self::lesson_3(),
            Self::lesson_4(),
            Self::lesson_5(),
        ]
    }

    /// Get lesson by ID.
    pub fn get(id: u8) -> Option<Self> {
        Self::all().into_iter().find(|l| l.id == id)
    }

    /// Get lesson step by number (1-indexed).
    pub fn step(&self, num: u16) -> Option<&LessonStep> {
        self.steps.iter().find(|s| s.number == num)
    }

    /// Check if user solution matches expected solution.
    pub fn verify_solution(&self, user_solution: &[u8]) -> bool {
        user_solution == self.solution
    }
}

impl fmt::Display for Lesson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lesson {}: {} ({}x{} grid)\n{}",
            self.id, self.title, self.grid_size, self.grid_size, self.objective
        )
    }
}

/// Tutorial progress tracking.
#[derive(Debug, Clone)]
pub struct TutorialProgress {
    /// Completed lesson IDs
    pub completed: Vec<u8>,
    /// Current lesson ID (if in progress)
    pub current: Option<u8>,
    /// Current step within lesson
    pub current_step: u16,
}

impl Default for TutorialProgress {
    fn default() -> Self {
        TutorialProgress {
            completed: Vec::new(),
            current: None,
            current_step: 1,
        }
    }
}

impl TutorialProgress {
    /// Start lesson.
    pub fn start_lesson(&mut self, lesson_id: u8) {
        self.current = Some(lesson_id);
        self.current_step = 1;
    }

    /// Complete current lesson.
    pub fn complete_lesson(&mut self) {
        if let Some(id) = self.current
            && !self.completed.contains(&id)
        {
            self.completed.push(id);
        }
        self.current = None;
        self.current_step = 1;
    }

    /// Move to next step.
    pub fn next_step(&mut self) -> bool {
        let changed = self.current_step < 999;
        if changed {
            self.current_step += 1;
        }
        changed
    }

    /// Get progress percentage.
    pub fn completion_percent(&self) -> f32 {
        let total = Lesson::all().len() as f32;
        if total == 0.0 {
            0.0
        } else {
            (self.completed.len() as f32 / total) * 100.0
        }
    }

    /// Check if all lessons completed.
    pub fn all_lessons_complete(&self) -> bool {
        self.completed.len() == Lesson::all().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_1_basics() {
        let lesson = Lesson::lesson_1();
        assert_eq!(lesson.id, 1);
        assert_eq!(lesson.grid_size, 2);
        assert_eq!(lesson.solution.len(), 4);
        assert!(!lesson.steps.is_empty());
    }

    #[test]
    fn lesson_all_retrieval() {
        let all = Lesson::all();
        assert!(all.len() >= 3);
        assert_eq!(all[0].id, 1);
    }

    #[test]
    fn lesson_get_by_id() {
        let lesson = Lesson::get(1);
        assert!(lesson.is_some());
        assert_eq!(lesson.unwrap().id, 1);
    }

    #[test]
    fn lesson_step_retrieval() {
        let lesson = Lesson::lesson_1();
        let step = lesson.step(1);
        assert!(step.is_some());
        assert_eq!(step.unwrap().number, 1);
    }

    #[test]
    fn lesson_verify_solution() {
        let lesson = Lesson::lesson_1();
        let correct = vec![1, 2, 2, 1];
        let wrong = vec![1, 1, 2, 2];

        assert!(lesson.verify_solution(&correct));
        assert!(!lesson.verify_solution(&wrong));
    }

    #[test]
    fn tutorial_progress_tracking() {
        let mut progress = TutorialProgress::default();
        assert_eq!(progress.completed.len(), 0);

        progress.start_lesson(1);
        assert_eq!(progress.current, Some(1));

        progress.complete_lesson();
        assert!(progress.completed.contains(&1));
    }

    #[test]
    fn tutorial_progress_steps() {
        let mut progress = TutorialProgress::default();
        progress.start_lesson(1);
        assert_eq!(progress.current_step, 1);

        progress.next_step();
        assert_eq!(progress.current_step, 2);
    }

    #[test]
    fn tutorial_completion_percent() {
        let mut progress = TutorialProgress::default();
        assert_eq!(progress.completion_percent(), 0.0);

        for lesson in Lesson::all() {
            progress.completed.push(lesson.id);
        }

        assert_eq!(progress.completion_percent(), 100.0);
    }
}
