use core::fmt;
use std::fmt::{format, write};

use rand::prelude::*;

#[derive(Clone, Copy)]
struct Coord {
    x: usize,
    y: usize,
}

const H: usize = 3; //迷路の高さ
const W: usize = 4; //迷路の幅
const END_TURN: u32 = 4; //ゲーム終了ターン

struct MazeState {
    points: Vec<Vec<ScoreType>>,
    turn: u32,
    pub character: Coord,
    pub game_score: i32,
    first_action: i32,
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
            first_action: -1,
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

    fn legal_actions(&self) -> Vec<Coord> {
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
type ScoreType = i32;

fn random_action(state: &State, rng: &mut rand::rngs::StdRng) -> Coord {
    let next_coords = state.legal_actions();
    next_coords[rng.gen::<usize>() % next_coords.len()]
}

fn greedy_action(state: &State) -> Option<Coord> {
    let legal_actions = state.legal_actions();

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
            state.update(random_action(&state, rng_for_action));
        }
        score_sum += state.game_score as f64;
    }
    let score_mean = score_sum / game_number as f64;
    println!("Score:\t{}", score_mean);
}

fn test_greedy_score(game_number: u32, rng_for_action: &mut rand::rngs::StdRng) {
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
    println!("Score:\t{}", score_mean);
}

fn main() {
    let seed = 1;
    let mut rng_for_action: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    // play_game(seed, &mut rng_for_action);
    // test_random_score(100, &mut rng_for_action);
    test_greedy_score(100, &mut rng_for_action);
}
