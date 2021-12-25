extern crate swarm_pool as swarm;
mod renderer;
mod cookies;
mod timer;

use rand::{Rng, prelude::SliceRandom};
use rand::prelude::ThreadRng;
use renderer::{Drawable, Renderer};
use sdl2::gfx::primitives::DrawRenderer;
use swarm::{Swarm, Spawn, control::SwarmControl};
use cookies::Cookies;

// CONST VALUES:

const SCREEN_SIZE: (u32, u32) = (600, 500);
const PLANE_VSPEED: f32 = 8.0;
const PLANE_RSPEED: f32 = 4.0;
const PLANE_RAD: f32 = 10.0;

mod precal {
    pub const FR: f32 = 3.14159 / 180.0;
    pub const R120: f32 = FR * 120.0;
    pub const R240: f32 = FR * 240.0;
}

// STATE MACHINE CONCEPT:

// A network has N systems
// each system has a predefines set of blocks
// every block either returns an output or executes the next block
// an output can either be 0, 1 or 2
// there are 6 input bool values per system
// for every possible combination there exists only one block (64 combinations)
// if a system perform bad either swap 2 blocks, swap a block output/next or change the output value

#[derive(Clone)]
struct System {
    pointer: u8, // current block
    blocks: [Block; 16],
    input: u8,
    rng: ThreadRng,
    outblock: (u8, bool),
}

impl System {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut blocks = [Block::default(); 16];

        for i in 0..blocks.len() { 
            let out_on = rng.gen_range(0..2) == 0;
            let out_val = rng.gen_range(0..3);

            blocks[i] = Block::new(i as u8, out_on, out_val);
        }
        blocks.shuffle(&mut rng);
        System { pointer: 0, blocks, input: 0, rng, outblock: (0, false) }
    }

    fn update(&mut self, input: [bool; 4]) -> u8 {
        self.input = 0;
        self.pointer = 0;
        let mut out: Option<u8> = None;

        for i in 0..input.len() { 
            self.input += match input[i] { true => 1, false => 0 } << i
        }

        while let None = out {
            let block = self.blocks[self.pointer as usize];
            if block.out_on == (block.cmp == self.input) {
                out = Some(block.out_val)
            } else {
                self.pointer += 1;
            }
        }

        self.outblock.0 = self.pointer;
        if let Some(r) = out { r } else { 0 }
    }

    fn punish(&mut self) {
        if self.outblock.1 == false {
            let new_val = match self.blocks[self.outblock.0 as usize].out_val {
                0 => 1,
                1 => 2,
                _ => 1,
            };
            self.blocks[self.outblock.0 as usize].out_val = new_val;
            self.outblock.1 = true;
        } else {
            let mut swap_with = self.rng.gen_range(0..16);
            while swap_with == self.outblock.0 {
                swap_with = self.rng.gen_range(0..16);
            }

            let outblock = self.blocks[self.outblock.0 as usize].clone();
            self.blocks[self.outblock.0 as usize] = self.blocks[swap_with as usize];
            self.blocks[swap_with as usize] = outblock;
        }
    }
}

#[derive(Default, Copy, Clone)]
struct Block {
    cmp: u8,
    out_on: bool,
    out_val: u8,
}

impl Block {
    fn new(num: u8, out_on: bool, mut out_val: u8) -> Self {
        if out_val > 2 { out_val = 0; } 
        let cmp = num & 15;

        Block { cmp, out_on, out_val, }
    }
}

fn main() -> Result<(), String> {
    let mut sys = System::new();
    let mut swarm: Swarm<GameObject, Renderer<ThreadRng>> = renderer::new(SCREEN_SIZE.0, SCREEN_SIZE.1, rand::thread_rng())?;
   
    swarm.add_factory(0, |o: &mut GameObject, r: &mut Renderer<ThreadRng>| {
        
        o.position = (
            r.props.gen_range(10..SCREEN_SIZE.0-10) as f32, 
            r.props.gen_range(10..SCREEN_SIZE.0-10) as f32
        );
        o.rotation = r.props.gen_range(10..SCREEN_SIZE.0-360) as f32;
    });

    for i in 0.. 100 {
        swarm.spawn_type(0);
    }

    renderer::run(
        &mut swarm,
        |control: &mut SwarmControl<GameObject, Renderer<ThreadRng>>| {
            let t = control.target();

            t.detect_collision();
            t.think();
            t.turn();
            t.bounds();
            t.fly();
            t.learn();

        },
    );
    Ok(())
}


#[derive(Clone)]
struct GameObject {
    position: (f32, f32),
    rotation: f32,
    speed: f32,
    age: u8,
    steer_sys: System,
    steer_fail: bool,
    input: [bool; 4],
    output: u8,
}

impl Default for GameObject {
    fn default() -> Self {
        GameObject {
            position: (0.0, 0.0),
            rotation: 0.0,
            speed: PLANE_VSPEED,
            age: 0,
            steer_sys: System::new(),
            steer_fail: false,
            input: [false; 4],
            output: 0,
        }
    }
}

impl GameObject {

    fn fly(&mut self) {
        let fdr1 = self.rotation * precal::FR;

        self.position.0 += fdr1.cos() * self.speed;
        self.position.1 -= fdr1.sin() * self.speed;
    }

    fn turn(&mut self) {
        let left = self.output == 1;
        let right = self.output == 2;

        if !left && right { self.rotation += PLANE_RSPEED; }
        else if left && !right { self.rotation -= PLANE_RSPEED; }

        if self.rotation > 359.0 { self.rotation -= 359.0; }
        else if self.rotation < 0.0 { self.rotation += 359.0; }
    }

    fn throttle(&mut self) {
        // let accel = self.output[2];
        // let decel = self.output[3];

        // if !decel && accel { self.speed = PLANE_SPEED * 1.1; }
        // else if decel && !accel { self.speed = PLANE_SPEED * 0.9; }
        // else  { self.speed = PLANE_SPEED; }
    }

    fn detect_collision(&mut self) {
        let fdr1 = self.rotation * precal::FR;
        let cos = fdr1.cos();
        let sin = fdr1.sin();
        
        for at_time in 0..10 {
            let future_pos = (
                self.position.0 + (cos * (self.speed * at_time as f32)),
                self.position.1 - (sin * (self.speed * at_time as f32))
            );

            // detect boundaries
            if future_pos.0 <= 10.0 { self.input[0] = true; }
            if future_pos.0 > SCREEN_SIZE.0 as f32 - 10.0 { self.input[1] = true; }
            if future_pos.1 <= 10.0 { self.input[2] = true; }
            if future_pos.1 > SCREEN_SIZE.1 as f32 - 10.0 { self.input[3] = true; }
        }
    }

    fn think(&mut self) {
        self.output = self.steer_sys.update(self.input);
    }

    fn bounds(&mut self) {
        if self.position.0 <= 10.0 { 
            self.rotation = 0.0;
            self.steer_fail = true; 
        }
        else if self.position.0 >= SCREEN_SIZE.0 as f32-10.0 { 
            self.rotation = 180.0; 
            self.steer_fail = true;        
        }
        if self.position.1 <= 10.0 { 
            self.rotation = 270.0;
            self.steer_fail = true;        
        }
        else if self.position.1 >= SCREEN_SIZE.1 as f32-10.0 { 
            self.rotation = 90.0;
            self.steer_fail = true;         
        }
    }

    fn learn(&mut self) {
        if self.steer_fail {
            self.steer_sys.punish();
            self.steer_fail = false;
        }
    }
}

impl Drawable for GameObject {
    fn draw(&mut self, gfx: &mut renderer::Gfx) {
        let fdr1 = self.rotation * precal::FR;
        let fdr2 = fdr1 + precal::R120;
        let fdr3 = fdr1 + precal::R240;

        let _d = gfx.canvas.trigon(
            (self.position.0 + (fdr1.cos() * PLANE_RAD)).round() as i16,
            (self.position.1 - (fdr1.sin() * PLANE_RAD)).round() as i16,
            (self.position.0 + (fdr2.cos() * PLANE_RAD)).round() as i16,
            (self.position.1 - (fdr2.sin() * PLANE_RAD)).round() as i16,
            (self.position.0 + (fdr3.cos() * PLANE_RAD)).round() as i16,
            (self.position.1 - (fdr3.sin() * PLANE_RAD)).round() as i16,
            (250,250,250,255),
        );
    }
}
