use rand::Rng;
use rand::prelude::{ThreadRng};
use crate::config as cfg;


pub(crate)type Chain = [Command; cfg::CHAIN];
pub type Output = [bool; cfg::BLOCK];


#[derive(Default, Copy, Clone)]
pub(crate) struct Command { 
    pub addr_in: u8, 
    pub addr_out: u8,
    pub opp: Opp, 
}


#[derive(Copy, Clone)]
pub(crate) enum Opp { And, Or, Xor, Not }

impl Opp {
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen_range(0..4) { 0 => Opp::And, 1 => Opp::Or, 2 => Opp::Xor, _ => Opp::Not, }
    }
}

impl Default for Opp {
    fn default() -> Self { Opp::And }
}
