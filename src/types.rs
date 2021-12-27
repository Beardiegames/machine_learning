use rand::Rng;
use rand::prelude::{ThreadRng};
use crate::config as cfg;


pub type Output = [bool; cfg::OUT_BLOCK];
pub type DataBuffer = [bool; 24]; // input: 0-7, cache: 8-15, output: 16-23
pub type ReadIndex = usize;
pub type WriteIndex = usize;


#[derive(Clone)]
pub(crate) struct Chain {
    pub opp_list: [Opperation; cfg::CHAIN],
    pub age: u64,
}

impl Chain {
    pub fn new(mut rng: &mut ThreadRng) -> Chain {
        let mut opp_list = [Opperation::default(); cfg::CHAIN as usize];
        for i in 0..cfg::CHAIN { opp_list[i] = Opperation::from_rng(i, &mut rng); }

        Chain { opp_list, age: 0 }
    }

    pub fn execute(&mut self, data_buffer: &mut DataBuffer) {
        for opp in &mut self.opp_list {
            match opp {
                Opperation::And(r, w) => data_buffer[*w] &= data_buffer[*r],
                Opperation::Or(r, w)  => data_buffer[*w] |= data_buffer[*r],
                Opperation::Xor(r, w) => data_buffer[*w] ^= data_buffer[*r],
                Opperation::Not(r, w) => data_buffer[*w] = !data_buffer[*r],
            }
        }
        self.age += 1;
    }

    pub fn evolve(&mut self, rng: &mut ThreadRng) {
        // Do one of 4 things:
        match rng.gen_range(0..4) { 
            // 1. change a random input opperations write address 
            0 => self.opp_list[rng.gen_range(0..8)].set_write(rng.gen_range(8..16)),

            // 2. change the order of two random opperations
            1 => {
                let (a, b) = match rng.gen_bool(0.5) {
                    true => (rng.gen_range(0..4), rng.gen_range(4..8)),
                    false => (rng.gen_range(8..10), rng.gen_range(10..12)),    
                };
                let opp_a = self.opp_list[a];
                self.opp_list[a] = self.opp_list[b];
                self.opp_list[b] = opp_a;
            },

            // 3. change an opperations opperant
            _ => self.opp_list[rng.gen_range(0..cfg::CHAIN)].cycle(),
        };
    }
}

// Opperations of index 0-7 refer read from addresses 0-7 and write to addresses 8-15
// Opperations of index 8-15 refer read from adresses 8-15 and write to adresses 16-23
#[derive(Copy, Clone)]
pub(crate) enum Opperation { 
    And(ReadIndex, WriteIndex), 
    Or(ReadIndex, WriteIndex), 
    Xor(ReadIndex, WriteIndex), 
    Not(ReadIndex, WriteIndex) 
}

impl Opperation {
    pub(crate) fn from_rng(index: ReadIndex, rng: &mut ThreadRng) -> Self {
        match rng.gen_range(0..4) { 
            0 => Opperation::And(index, index + 8), 
            1 => Opperation::Or(index, index + 8), 
            2 => Opperation::Xor(index, index + 8), 
            _ => Opperation::Not(index, index + 8), 
        }
    }

    pub fn cycle(mut self) {
        self = match &self {
            Opperation::And(r, w) => Opperation::Or(*r,*w),
            Opperation::Or(r, w)  => Opperation::Xor(*r,*w),
            Opperation::Xor(r, w) => Opperation::Not(*r,*w),
            Opperation::Not(r, w) => Opperation::And(*r,*w),
        }
    }

    pub fn set_write(&mut self, to_index: ReadIndex) {
        match self {
            Opperation::And(r, ..) => *r = to_index,
            Opperation::Or(r, ..)  => *r = to_index,
            Opperation::Xor(r, ..) => *r = to_index,
            Opperation::Not(r, ..) => *r = to_index,
        }
    }
}

impl Default for Opperation {
    fn default() -> Self { Opperation::And(0, 7) }
}
