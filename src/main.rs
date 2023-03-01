use core::fmt;
use rand::prelude::*;
use std::collections::BinaryHeap;
use std::time;

#[derive(Clone, Copy, Eq, PartialEq)]
struct Coord {
    x: usize,
    y: usize,
}

const H: usize = 30; //迷路の高さ
const W: usize = 40; //迷路の幅
const END_TURN: u32 = 100; //ゲーム終了ターン

#[derive(Eq, PartialEq, Clone)]
struct MazeState {
    points: Vec<Vec<ScoreType>>,
    turn: u32,
    character: Coord,
    game_score: i32,
    first_action: Action,
}

impl MazeState {
    fn new(seed: u8) -> Self {
        let seed_for_rng: [u8; 32] = [seed; 32];
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed(seed_for_rng);

        let character = Coord {
            x: rng.gen::<usize>() % W,
            y: rng.gen::<usize>() % H,
        };
        let mut points = vec![vec![0 as ScoreType; W]; H];

        for h in 0..H {
            for w in 0..W {
                if h == character.y && w == character.x {
                    continue;
                }
                points[h][w] = rng.gen_range(0..10);
            }
        }
        MazeState {
            points,
            turn: 0,
            character,
            game_score: 0,
            first_action: None,
        }
    }

    fn is_done(&self) -> bool {
        self.turn == END_TURN
    }

    // overwrite current state
    fn update(&mut self, next_coord: Coord) {
        let point = &mut self.points[next_coord.y][next_coord.x];
        self.game_score += *point;
        *point = 0;

        self.character = next_coord;
        self.turn += 1;
    }

    // do the action once, and return score
    fn advance(&self, next_coord: Coord) -> ScoreType {
        let mut score = self.game_score;
        score += self.points[next_coord.y][next_coord.x];
        score
    }

    fn legal_coords(&self) -> Vec<Coord> {
        let mut next_coords: Vec<Coord> = vec![];

        if self.character.x > 0 {
            next_coords.push(Coord {
                x: self.character.x - 1,
                y: self.character.y,
            });
        }

        if self.character.x < W - 1 {
            next_coords.push(Coord {
                x: self.character.x + 1,
                y: self.character.y,
            });
        }

        if self.character.y > 0 {
            next_coords.push(Coord {
                x: self.character.x,
                y: self.character.y - 1,
            });
        }

        if self.character.y < H - 1 {
            next_coords.push(Coord {
                x: self.character.x,
                y: self.character.y + 1,
            });
        }

        next_coords
    }
}

impl Ord for MazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.game_score.cmp(&other.game_score)
    }
}

impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.game_score.cmp(&other.game_score))
    }
}

// visualize state
impl fmt::Display for MazeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut maze_string = String::from(format!(
            "turn:\t{}\nscore:\t{}\n",
            self.turn, self.game_score
        ));

        for h in 0..H {
            let mut line_string = String::from("");
            for w in 0..W {
                if self.character.x == w && self.character.y == h {
                    line_string.push('@');
                } else {
                    line_string.push_str(&self.points[h][w].to_string());
                }
            }
            line_string.push('\n');
            maze_string.push_str(&line_string);
        }
        write!(f, "{}", maze_string)
    }
}

type State = MazeState;
type Action = Option<Coord>;
type ScoreType = i32;

fn random_action(state: &State, rng: &mut rand::rngs::StdRng) -> Action {
    let next_coords = state.legal_coords();
    if next_coords.len() == 0 {
        None
    } else {
        Some(next_coords[rng.gen::<usize>() % next_coords.len()])
    }
}

fn greedy_action(state: &State) -> Action {
    let legal_actions = state.legal_coords();

    let mut best_score = ScoreType::MIN;
    let mut best_action = None;
    for action in legal_actions {
        let next_score = state.advance(action);
        if best_score < next_score {
            best_score = next_score;
            best_action = Some(action);
        }
    }
    best_action
}

fn beamsearch_action(initial_state: &State, beam_width: u32, beam_depth: u32) -> Action {
    let mut current_beam = BinaryHeap::new();
    let mut wrapped_best_state = None;
    current_beam.push(initial_state.clone());
    for d in 0..beam_depth {
        let mut next_beam = BinaryHeap::new();
        for _w in 0..beam_width {
            let wrapped_state = current_beam.pop();
            if wrapped_state == None {
                break;
            }
            let current_state = wrapped_state.unwrap();
            let legal_actions = current_state.legal_coords();
            for action in legal_actions {
                let mut next_state = current_state.clone();
                next_state.update(action);
                if d == 0 {
                    next_state.first_action = Some(action);
                }
                next_beam.push(next_state);
            }
        }
        current_beam = next_beam;
        wrapped_best_state = current_beam.peek();
    }
    wrapped_best_state?.first_action
}

fn beamsearch_action_with_timelimit(
    initial_state: &State,
    beam_width: u32,
    time_threshold_in_millis: u64,
) -> Action {
    let time_keeper = TimeKeeper::new(time_threshold_in_millis);

    let mut current_beam = BinaryHeap::new();
    let mut wrapped_best_state: Option<MazeState> = None;
    current_beam.push(initial_state.clone());

    for d in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _w in 0..beam_width {
            if time_keeper.is_timeover() {
                return wrapped_best_state?.first_action;
            }

            let wrapped_state = current_beam.pop();
            if wrapped_state == None {
                break;
            }
            let current_state = wrapped_state.unwrap();
            let legal_actions = current_state.legal_coords();
            for action in legal_actions {
                let mut next_state = current_state.clone();
                next_state.update(action);
                if d == 0 {
                    next_state.first_action = Some(action);
                }
                next_beam.push(next_state);
            }
        }
        current_beam = next_beam;
        if current_beam.peek() != None {
            wrapped_best_state = Some(current_beam.peek().unwrap().clone());
        }
    }

    wrapped_best_state?.first_action
}

fn play_game(seed: u8, rng: &mut rand::rngs::StdRng) {
    let mut state = State::new(seed);
    println!("{}", state);
    while !state.is_done() {
        match greedy_action(&state) {
            Some(action) => state.update(action),
            None => panic!("No action found!"),
        }
        println!("{}", state);
    }
}

fn test_random_score(game_number: u32, rng_for_action: &mut rand::rngs::StdRng) {
    let mut rng_for_construct: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let mut score_sum = 0.;
    for _ in 0..game_number {
        let mut state = State::new(rng_for_construct.gen::<u8>());
        while !state.is_done() {
            match random_action(&state, rng_for_action) {
                Some(action) => state.update(action),
                None => panic!("No action found!"),
            }
        }
        score_sum += state.game_score as f64;
    }
    let score_mean = score_sum / game_number as f64;
    println!("Random Score:\t{}", score_mean);
}

fn test_greedy_score(game_number: u32) {
    let mut rng_for_construct: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let mut score_sum = 0.;
    for _ in 0..game_number {
        let mut state = State::new(rng_for_construct.gen::<u8>());
        while !state.is_done() {
            match greedy_action(&state) {
                Some(action) => state.update(action),
                None => panic!("No action found!"),
            }
        }
        score_sum += state.game_score as f64;
    }
    let score_mean = score_sum / game_number as f64;
    println!("Greedy Score:\t{}", score_mean);
}

fn test_beamsearch_score(game_number: u32, beam_width: u32, beam_depth: u32) {
    let mut rng_for_construct: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let mut score_sum = 0.;
    for _ in 0..game_number {
        let mut state = State::new(rng_for_construct.gen::<u8>());
        while !state.is_done() {
            match beamsearch_action(&state, beam_width, beam_depth) {
                Some(action) => state.update(action),
                None => panic!("No action found!"),
            }
        }
        score_sum += state.game_score as f64;
    }
    let score_mean = score_sum / game_number as f64;
    println!("Beam Search Score:\t{}", score_mean);
}

fn test_beamsearch_score_with_timelimit(
    game_number: u32,
    beam_width: u32,
    time_threshold_in_millis: u64,
) {
    let mut rng_for_construct: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let mut score_sum = 0.;
    for _ in 0..game_number {
        let mut state = State::new(rng_for_construct.gen::<u8>());
        while !state.is_done() {
            match beamsearch_action_with_timelimit(&state, beam_width, time_threshold_in_millis) {
                Some(action) => state.update(action),
                None => panic!("No action found!"),
            }
        }
        score_sum += state.game_score as f64;
    }
    let score_mean = score_sum / game_number as f64;
    println!("Beam Search Score:\t{}", score_mean);
}
// 時間を管理する構造体
struct TimeKeeper {
    start_time: time::Instant,
    time_threshold: time::Duration,
}

impl TimeKeeper {
    fn new(time_threshold_in_millis: u64) -> Self {
        TimeKeeper {
            start_time: time::Instant::now(),
            time_threshold: time::Duration::from_millis(time_threshold_in_millis),
        }
    }

    fn is_timeover(&self) -> bool {
        let current_time = time::Instant::now();
        self.start_time + self.time_threshold <= current_time
    }
}

fn main() {
    let seed = 1;
    // let mut rng_for_action: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    // play_game(seed, &mut rng_for_action);
    // test_random_score(100, &mut rng_for_action);
    // test_greedy_score(100);
    // test_beamsearch_score(100, 2, END_TURN);
    test_beamsearch_score_with_timelimit(100, 5, 10);
}
