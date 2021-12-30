mod types;
mod config;

use rand::Rng;
use rand::prelude::{ThreadRng};
use types::*;
use config as cfg;


struct Frame {
    data_buffer: DataBuffer,
    active_chain: Chain,
    previous_chain: Chain,
    strikes: u8,
    archive: Vec<Chain>,
    rng: ThreadRng,
}

impl Frame {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Frame { 
            data_buffer: [false; 24], 
            active_chain: Chain::new(&mut rng), 
            previous_chain: Chain::new(&mut rng), 
            strikes: 0,
            archive: Vec::new(),
            rng, 
        }
    }

    pub fn compute(&mut self) -> Output {
        self.active_chain.execute(&mut self.data_buffer);

        let mut output = [false; cfg::OUT_BLOCK];
        for i in 0..cfg::OUT_BLOCK {
            output[i] = self.data_buffer[i + cfg::ADDR_OUT];
        }
        output
    }

    pub fn reward(&mut self) {
        self.active_chain.rewards += 1;
    }

    pub fn punish(&mut self) {
        if self.strikes < 2 { self.strikes += 1; } 
        else {
            self.strikes = 0;

            if self.previous_chain.rewards < self.active_chain.rewards {
                self.previous_chain = self.active_chain.clone();
            }
            else if self.previous_chain.rewards > self.active_chain.rewards {
                self.active_chain = self.previous_chain.clone();
            }
            else {
                match self.previous_chain.age < self.active_chain.age {
                    true => self.previous_chain = self.active_chain.clone(),
                    false => self.active_chain = self.previous_chain.clone(),
                }
            }
            self.reset_active_chain();
            self.active_chain.evolve(&mut self.rng);
        }
    }

    pub fn reset_active_chain(&mut self) {
        self.active_chain.age = 0;
        self.active_chain.rewards = 0;
    }
}
