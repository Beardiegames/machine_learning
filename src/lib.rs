
use rand::Rng;
use rand::prelude::{ThreadRng};


// Modifiers Concept:
// ---
//  diff_a = comp_a - input[0];
//  diff_b = comp_b - input[1];
//  diff_c = comp_c - input[2];
//  diff = diff_c - (diff_b - diff_a);
//  vector = multiplier_value * diff;
//  p3 = (cmp[2] - input[2]) - cmp[1] - input[1] - cmp[0] - input[0]) * mul;

#[derive(Clone)]
struct Modifiers {
    comperators: Vec<f32>,
    multiplier: f32,
    age: u64,
    rewards: u64,
}

impl Modifiers {
    fn new(input_count: usize) -> Self {
        Modifiers {
            comperators: vec![0.0; input_count],
            multiplier: 1.0,
            age: 0, rewards: 0,
        }
    }
    fn evolve(&mut self, rng: &mut ThreadRng) {
        self.age = 0;
        self.rewards = 0;

        let flip_bounds = |x: f32, c: f32|
            if x + c < 1.0 && x + c > -1.0 { x } else { -x };

        // tweak multiplier
        self.multiplier += flip_bounds(
            rng.gen_range(-0.1..0.1), 
            self.multiplier
        );

        // tweak a single comperator
        let index = rng.gen_range(0..self.comperators.len());
        self.comperators[index] += flip_bounds(
            rng.gen_range(-0.1..0.1), 
            self.comperators[index]
        );
    }
}

// Evolution concept:
// ---
//  - evolve after every 3 mistakes
//  - if the active setup had a longer successrun (larger age) than the previous setup
//      the active setup becomes the previous setup, and is evolved into a new active one,
//      otherwise the active setup is archived and the previous setup evolves again.
//  - if the active setup has more rewards than the previous setup or the other way around,
//      evolution is based whom has the most rewards
//  - rewards are compared first, if equal compare ages.

#[derive(Clone)]
pub struct Pilot {
    input_count: usize,
    active_setup: Modifiers,
    prev_setup: Modifiers,
    strikes: u8,
    rng: ThreadRng,
}

impl Pilot {
    pub fn new(mut input_count: usize) -> Self {
        Pilot {
            input_count: input_count.clamp(1, 12),
            active_setup: Modifiers::new(input_count),
            prev_setup: Modifiers::new(input_count),
            strikes: 0,
            rng: rand::thread_rng(),
        }
    }

    pub fn steer(&mut self, input: &[f32]) -> Option<f32> {
        if input.len() == self.input_count {

            let mut diff = 0.0;
            for i in 0..self.input_count { 
                diff -= self.active_setup.comperators[i] - input[i]; 
            }
            self.active_setup.age += 1;
            Some((diff / self.input_count as f32) * self.active_setup.multiplier)
        }
        else { None }
    }

    pub fn punish(&mut self) {
        // if self.strikes < 2 { self.strikes += 1; }
        // else {
        //     self.strikes = 0;
        //     self.evolve();
        // }
        self.evolve();
    }

    fn evolve(&mut self) {
        if self.prev_setup.rewards < self.active_setup.rewards {
            self.prev_setup = self.active_setup.clone();
        }
        else if self.prev_setup.rewards > self.active_setup.rewards {
            self.active_setup = self.prev_setup.clone();
        }
        else {
            match self.prev_setup.age < self.active_setup.age {
                true => self.prev_setup = self.active_setup.clone(),
                false => self.active_setup = self.prev_setup.clone(),
            }
        }
        self.active_setup.evolve(&mut self.rng);
    }

    fn input_count(&self) -> usize {
        self.input_count
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn steer_returns_an_option_none_if_input_count_is_unexpected() {
        let mut pilot = Pilot::new(2);
        assert_eq!(pilot.steer(&[0.5]), None);
        assert_eq!(pilot.steer(&[0.5, 0.5]), Some(0.5));
        assert_eq!(pilot.steer(&[0.5, 0.5, 0.5]), None);
    }

    #[test]
    fn steer_calculates_a_vector_based_on_all_user_input() {
        let mut pilot = Pilot::new(1);
        assert_eq!(pilot.steer(&[0.5]), Some(0.5));
    }

    #[test]
    fn a_pilots_inputs_are_user_defined() {
        let pilot = Pilot::new(3);
        assert_eq!(pilot.input_count(), 3);
    }

    #[test]
    fn has_a_minimum_of_one_input() {
        let pilot = Pilot::new(0);
        assert_eq!(pilot.input_count(), 1);
    }

    #[test]
    fn has_a_maximum_of_twelve_inputs() {
        let pilot = Pilot::new(5000);
        assert_eq!(pilot.input_count(), 12);
    }
}
