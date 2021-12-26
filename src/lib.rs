mod types;
mod config;

use rand::Rng;
use rand::prelude::{ThreadRng};
use types::{Chain, Output, Command, Opp};
use config as cfg;


fn gen_chain(mut rng: &mut ThreadRng) -> Chain {
    let mut chain = [Command::default(); cfg::CHAIN as usize];

    for i in 0..cfg::CHAIN {
        let mut cmd = chain[i];
        cmd.addr_in = i as u8; // a command can only take input and cache addresses
        cmd.addr_out = cfg::BLOCK as u8 + i as u8; // and can only write to cache or output addresses
        cmd.opp = Opp::random(&mut rng);
    }
    chain
}


struct System {
    mem: [bool; 24], // input: 0-7, cache: 8-15, output: 16-23
    chain: Chain,
    rng: ThreadRng,
}

impl System {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        System { mem: [false; 24], chain: gen_chain(&mut rng), rng, }
    }

    pub fn compute(&mut self) -> Output {
        for cmd in &mut self.chain {
            let addr_in = cmd.addr_in as usize;
            let addr_out = cmd.addr_out as usize;

            match cmd.opp {
                Opp::And => self.mem[addr_out] &= self.mem[addr_in],
                Opp::Or => self.mem[addr_out] |= self.mem[addr_in],
                Opp::Xor => self.mem[addr_out] ^= self.mem[addr_in],
                Opp::Not => self.mem[addr_out] = !self.mem[addr_in],
            }
        }

        let mut output = [false; cfg::BLOCK];
        for i in 0..cfg::BLOCK {
            output[i] = self.mem[i + cfg::ADDR_OUT];
        }
        output
    }

    pub fn apply_feedback(&mut self, failed_ouput_indexes: &[u8]) {
        for i in failed_ouput_indexes {

            // Get command with same output from chain
            let mut failed_cmd_index = None;
            
            'find_cmd: for c in 0..self.chain.len() {
                if self.chain[c].addr_out == i + cfg::ADDR_OUT as u8 {
                    failed_cmd_index = Some(c);
                    break 'find_cmd;
                }
            } 

            // Change that command if found
            if let Some(target) = failed_cmd_index {

                // Decide to swap order OR change cmd opperant
                if self.rng.gen_bool(0.5) {

                    // Swap chain order
                    // get random other chain cmd as long as it is not the target cmd
                    let mut other =  self.rng.gen_range(0..self.chain.len());
                    while other != target { other = self.rng.gen_range(0..self.chain.len()); }

                    let cmd = self.chain[target].clone();
                    self.chain[target] = self.chain[other].clone();
                    self.chain[other] = cmd;
                } 
                else {
                    // Change opperant
                    self.chain[target].opp = Opp::random(&mut self.rng);
                }
            }
        }
    }
}
