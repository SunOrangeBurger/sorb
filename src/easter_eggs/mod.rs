pub mod engine;
pub mod games;

use engine::runner::run_game;
use games::dino::DinoGame;
use games::flappy::FlappyGame;
use games::snake::SnakeGame;
use games::test::TestGame;

/// Check if the input command is a secret easter egg command.
/// If it is, launch the corresponding game and return `true`.
/// Otherwise return `false` so the shell can try external execution.
pub fn try_launch(cmd: &str) -> bool {
    match cmd {
        "sorb-test" => {
            let mut game = TestGame::new();
            run_game(&mut game);
            true
        }
        "sorb-snake" => {
            let mut game = SnakeGame::new();
            run_game(&mut game);
            true
        }
        "sorb-dino" => {
            let mut game = DinoGame::new();
            run_game(&mut game);
            true
        }
        "sorb-flappy" => {
            let mut game = FlappyGame::new();
            run_game(&mut game);
            true
        }
        "sorb-tetris" => {
            let mut game = games::tetris::TetrisGame::new();
            run_game(&mut game);
            true
        }
        "sorb-invaders" => {
            let mut game = games::space_invaders::SpaceInvadersGame::new();
            run_game(&mut game);
            true
        }
        _ => false,
    }
}
