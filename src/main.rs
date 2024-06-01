#![feature(generic_const_exprs)]
#![allow(dead_code)]

mod game;
mod solver;
// mod search;
mod nn;

pub use crate::game::*;
pub use crate::solver::*;
// pub use crate::search::*;
pub use crate::nn::*;

pub(crate) fn print_probs_2d(probs: &[E], width: usize) {
    for (i, prob) in probs.iter().enumerate() {
        print!("{prob:5.3} ");

        if i % width == width - 1 {
            println!()
        }
    }
}

fn generate_game() -> InternalGame<Graph2d> {
    InternalGame::new(10, StartType::Safe, Graph2d::new(9, 9, &MOORE_NEIGHBORHOOD))
}

fn train_main() {
    use dfdx::{
        optim::{Adam, AdamConfig},
        prelude::*,
    };

    let dev = D::default();
    let mut net = dev.build_module::<Net, E>();
    net.load_safetensors("nets/run1_net1500.safetensors").unwrap();
    let mut optimizer: Adam<BuiltNet, E, D> = Adam::new(
        &net,
        AdamConfig {
            lr: 5e-5,
            ..AdamConfig::default()
        },
    );

    for epoch in 1501.. {
        println!("begin epoch {epoch}");
        let games = std::iter::repeat_with(generate_game).take(250);

        train(&dev, &mut net, &mut optimizer, games);

        if epoch % 50 == 0 {
            net.save_safetensors(format!("nets/run1_net{epoch}.safetensors")).expect("Failed to save network!");
        }
    }
}

fn eval_main() {
    use dfdx::prelude::*;

    let dev = D::default();
    let mut net = dev.build_module::<Net, E>();
    net.load_safetensors("nets/run1_net3600.safetensors").unwrap();

    const NUM_GAMES: usize = 500;

    let mut total_won = 0;
    let mut total_games = 0;

    loop {
        let games = std::iter::repeat_with(generate_game).take(NUM_GAMES);
        total_won += evaluate_performance(&dev, &net, games);
        total_games += NUM_GAMES;

        println!("{:.5}, {total_won}/{total_games}", total_won as f64 / total_games as f64);
    }
}

fn main() {
    eval_main()
}
