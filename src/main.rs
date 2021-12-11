extern crate swarm_pool as swarm;
mod renderer;
mod cookies;

use rand::Rng;
use rand::prelude::ThreadRng;
use renderer::{Drawable, Renderer};
use sdl2::gfx::primitives::DrawRenderer;
use swarm::{Swarm, Spawn, control::SwarmControl};
use cookies::Cookies;

// network settings
// mod net {
//     pub const DEPTH: u8 = 8;
//     pub const LEVEL_SIZE: u8 = 8;
//     pub const NODE_COUNT: usize = DEPTH as usize * LEVEL_SIZE as usize;
// }

// test settings
const PLANE_RAD: f32 = 12.0;
const PLANE_SPEED: f32 = 5.0;
const SCREEN_SIZE: (u32, u32) = (1024, 768);

mod precal {
    pub const F_DR: f32 = 0.01745329252;
    pub const R120: f32 = 2.0943951024;
    pub const R240: f32 = 4.1887902048;
}

// REWARD SYSTEM CONCEPT:

// Methode:
// Supply and demand: Bargaining cookies for work
// Nodes negotiate a cooperation contracts:
//  - Supply: A Node cannot offer more cookies than it allready has
//  - Demand: A Node will deal with the highest bidder, or the most trusted bidder     
// Top down system: Nodes can only connect to higher level Nodes (preventing infinate loops)

// Promise:
//  - Bad: flying into walls
//  - Good: turn to avoid walls

// Evaluation:
//  - Every Node has a fixed consumption rate to keep a constant demand for cookies
//      this is done by giving all input types a cost of one cookie

// Resolution:
//  - When out of cookies: a change in supplier must be made
//  - Output Nodes that give a Good results are rewarded a number of cookies equal to 
//      half the number of Nodes in a Network
//  - Node always look for better suppliers. If allready connected and income is good, 
//      changes will only be made higher bids are offered.

#[derive(Clone)]
struct AI {
    net: Network,
    age: u8,
    rewards: [u8; 8],
}

impl AI {
    pub fn new() -> AI {
        let mut rng = rand::thread_rng();
        let mut net = [[Node::default(); 8]; 8];

        for x in 0..net.len() {
            for y in 0..net[x].len() {
                net[x][y] = Node::from_rng(&mut rng, x as u8, y as u8);
            }
        }
        AI { net, age: 0, rewards: [128; 8] }
    }

    pub fn update(&mut self, input: [bool; 8]) -> [bool; 8] {
        let pulses = [
            self.age % 2 == 0,
            self.age % 3 == 0,
            self.age % 7 == 0,
            self.age % 11 == 0,
            self.age % 13 == 0,
            self.age % 17 == 0,
            self.age % 19 == 0,
            self.age % 31 == 0,
        ];

        self.age += 1;
        if self.age >= 62 { self.age = 0; }

        let mut results = [false; 8];
        let states = States { input, pulses };

        for i in 0..8 {
            let mut node = self.net[0][i].clone();
            results[i] = node.work(Cookies(self.rewards[i]), &states, &mut self.net);
            self.net[0][i] = node;
        }
        results
    }

    pub fn reward(&mut self, output_results: [u8; 8])  {
        self.rewards = output_results;
    }
}

type Network = [[Node; 8]; 8];

#[derive(Copy, Clone)]
struct NodeID { pub pos: u8, pub depth: u8 }

// struct Nodes([[Node; net::LEVEL_SIZE as usize]; net::DEPTH as usize]);

#[derive(Copy, Clone)]
struct States { input: [bool; 8], pulses: [bool; 8] }


#[derive(Default, Copy, Clone)]
struct Node { 
    //resource1: Link, resourecs are found dynamically per cycle
    //resource2: Link,
    //contract: Cookies, contract is always equal to proposition
    task: Task,
    stock: Cookies,
    depth: u8,
    pos: u8,
}

impl Node {
    fn from_rng(rng: &mut ThreadRng, depth: u8, pos: u8) -> Self {
        Node {
            task: Task::from_rng(rng),
            stock: Cookies(0),
            depth,
            pos,
        }
    }

    fn proposition(&self, others: &Network) -> Cookies {
        let expenses = self.cost(&others);
        expenses.0 + expenses.1 + Cookies(1)
    }

    pub fn work(&mut self, reward: Cookies, states: &States, others: &mut Network) -> bool {
        self.find_resources(others);
        let proposal = self.proposition(others);
        
        if self.stock >= proposal - Cookies(1) {
            self.stock += reward;
            self.task.exec(self.outsource((self.stock - Cookies(1)).half(), states, others))
        } else {
            false
        }
    }

    fn find_resources(&self, others: &Network) -> (Link, Link) {
        let mut cheapest_1: (usize, Cookies) = (0, Cookies(255));
        let mut cheapest_2: (usize, Cookies) = (0, Cookies(255));

        if self.depth < 7 {
            for i in 0..others[self.depth as usize + 1].len() {
                let node = &others[self.depth as usize + 1][i];
                let proposal = node.proposition(others);

                if proposal < cheapest_1.1 { cheapest_1 = (i, proposal); }
                if proposal < cheapest_2.1 { cheapest_2 = (i, proposal); }
            }
        }

        ( 
        match cheapest_1.1 > self.stock.half() {
            true => match self.depth % 2 == 0 { 
                true => Link::Pulse(self.pos),
                false => Link::Input(self.pos),
            },
            false => Link::Node(cheapest_1.0 as u8),
        },
        match cheapest_2.1 > self.stock.half() {
            true => match self.depth % 2 == 1 { 
                true => Link::Pulse(self.pos),
                false => Link::Input(self.pos),
            },
            false => Link::Node(cheapest_2.0 as u8),
        })
    }

    fn cost(&self, others: &Network) -> (Cookies, Cookies) {
        let resources = self.find_resources(others);
        ( 
        match resources.0 {
            Link::Input(i) => Cookies(0),
            Link::Pulse(i) => Cookies(0),
            Link::Node(i) => others[self.depth as usize + 1][i as usize].proposition(others), 
        },
        match resources.1 {
            Link::Input(i) => Cookies(0),
            Link::Pulse(i) => Cookies(0),
            Link::Node(i) => others[self.depth as usize + 1][i as usize].proposition(others), 
        })
    }
    
    fn outsource(&self, reward: Cookies, states: &States, others: &mut Network) -> (bool, bool) {
        let resources = self.find_resources(others).clone();
        (   match resources.0 {
                Link::Input(i) => states.input[i as usize],
                Link::Pulse(i) => states.pulses[i as usize],
                Link::Node(i) => {
                    let mut node = others[self.depth as usize + 1][i as usize];
                    node.work(reward, states, others)
                }, 
            },
            match resources.1 {
                Link::Input(i) => states.input[i as usize],
                Link::Pulse(i) => states.pulses[i as usize],
                Link::Node(i) => {
                    let mut node = others[self.depth as usize + 1][i as usize];
                    node.work(reward, states, others)
                },
            }
        )
    }
}

#[derive(Copy, Clone)]
enum Link { Input(u8), Pulse(u8), Node(u8) }

impl Link {
    fn from_rng(rng: &mut ThreadRng, incl_node: bool) -> Self {
        match rng.gen_range(0..match incl_node { true => 3, false => 2 }) {
            0 => Self::Input(rng.gen_range(0..8)),
            1 => Self::Pulse(rng.gen_range(0..8)),
            _ => Self::Node(rng.gen_range(0..8)),
        }
    }
}

#[derive(Copy, Clone)]
enum Task { And, Nand, Or, Xor }

impl Task {
    fn exec(&self, resources: (bool, bool)) -> bool {
        match self {
            Task::And => resources.0 & resources.1,
            Task::Nand => !(resources.0 & resources.1),
            Task::Or => resources.0 | resources.1,
            Task::Xor => resources.0 ^ resources.1,
        }
    }

    fn from_rng(rng: &mut ThreadRng) -> Self {
        match rng.gen_range(0..4) {
            0 => Self::And,
            1 => Self::Nand,
            2 => Self::Or,
            _ => Self::Xor,
        }
    }
}

impl Default for Task {
    fn default() -> Task { Task::Or }
}


// TESTCASE

struct GameData;

fn main() ->Result<(),String> {
    let data = GameData;
    let mut swarm = renderer::new::<GameObject, GameData>(
        SCREEN_SIZE.0, 
        SCREEN_SIZE.1, 
        data
    )?;
    swarm.add_factory(0, |s|{
        s.position = (100.0, 100.0);
        s.speed = 1.0;
    });
    swarm.spawn_type(0);

    renderer::run(
        &mut swarm,
        |c: &mut SwarmControl<GameObject, Renderer<GameData>>| {
            let plane = c.target();

            // gather and process input
            plane.detect_collision();
            plane.think();

            // perform results
            plane.turn();
            plane.throttle();
            plane.fly();

            // learn
            plane.learn();
        }
    );
    Ok(())
}




#[derive(Clone)]
struct GameObject {
    position: (f32, f32),
    rotation: f32,
    speed: f32,
    age: u8,
    rng: ThreadRng,

    ai: AI,
    input: [bool; 8],
    output: [bool; 8],
    rewards: [u8; 8],
}

impl Default for GameObject {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let position = (
            rng.gen_range(10..SCREEN_SIZE.0-10) as f32, 
            rng.gen_range(10..SCREEN_SIZE.0-10) as f32
        );
        let rotation = rng.gen_range(10..SCREEN_SIZE.0-360) as f32;
        
        GameObject {
            position,
            rotation,
            speed: PLANE_SPEED,
            age: 0,
            rng,   
            ai: AI::new(),
            input: [false; 8],
            output: [false; 8],
            rewards: [128; 8],
        }
    }
}

impl GameObject {

    fn fly(&mut self) {
        let fdr1 = self.rotation * precal::F_DR;

        self.position.0 += fdr1.cos() * self.speed;
        self.position.1 -= fdr1.sin() * self.speed;
    }

    fn turn(&mut self) {
        let left = self.output[0];
        let right = self.output[1];

        if !left && right { self.rotation += 1.0; }
        else if left && !right { self.rotation -= 1.0; }

        if self.rotation > 359.0 { self.rotation -= 359.0; }
        else if self.rotation < 0.0 { self.rotation += 359.0; }
    }

    fn throttle(&mut self) {
        let accel = self.output[2];
        let decel = self.output[3];

        if !decel && accel { self.speed = PLANE_SPEED * 1.1; }
        else if decel && !accel { self.speed = PLANE_SPEED * 0.9; }
        else  { self.speed = PLANE_SPEED; }
    }

    fn detect_collision(&mut self) {
        let fdr1 = self.rotation * precal::F_DR;
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
        self.output = self.ai.update(self.input);

        println!("{}, {}, {}, {}, {}, {}, {}, {}", 
            self.output[0],
            self.output[1],
            self.output[2],
            self.output[3],
            self.output[4],
            self.output[5],
            self.output[6],
            self.output[7],
        );
    }

    fn learn(&mut self) {
        self.rewards[0] = 128; 
        self.rewards[1] = 128;

        if self.position.0 <= 10.0 { 
            self.rotation = 180.0 - self.rotation; 
            self.rewards[0] = 0; 
            self.rewards[1] = 0; 
        }
        else if self.position.0 >= SCREEN_SIZE.0 as f32-10.0 { 
            self.rotation = 180.0 - self.rotation; 
            self.rewards[0] = 0; 
            self.rewards[1] = 0;        
        }
        if self.position.1 <= 10.0 { 
            self.rotation = 90.0 + self.rotation; 
            self.rewards[0] = 0; 
            self.rewards[1] = 0;        
        }
        else if self.position.1 >= SCREEN_SIZE.1 as f32-10.0 { 
            self.rotation = 270.0 + self.rotation; 
            self.rewards[0] = 0; 
            self.rewards[1] = 0;         
        }

        if self.rotation > 359.0 { self.rotation -= 359.0; }
        else if self.rotation < 0.0 { self.rotation += 359.0; }

        self.ai.reward(self.rewards)
    }
}

impl Drawable for GameObject {
    fn draw(&mut self, gfx: &mut renderer::Gfx) {
        let fdr1 = self.rotation * precal::F_DR;
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




// impl AI {
//     pub fn new() -> Self {
//         let mut rng = rand::thread_rng();
//         let mut network = [[Node::Opp(Opperation::new(&mut rng)); NET_SIZE as usize]; NET_COUNT as usize];

//         for net in &mut network {
//             for i in 0..net.len() {
//                 net[i] = match rng.gen_range(0..2) {
//                     1 => Node::Opp(Opperation::new(&mut rng)),
//                     2 => Node::Out(Output::new(&mut rng)),
//                     _ => Node::Stop,
//                 };
//             }
//         }

//         AI {
//             rng,
//             network,
//             scores: [0; NET_COUNT as usize],
//             curr_net: 0,
//             curr_score: 0,
//         }
//     }

//     pub fn rank_up(&mut self) {
//         self.curr_score += 1;
//     }

//     pub fn think(&mut self, input: &[u8; 8]) -> [bool; 8] {
//         let mut iter = 0;
//         let mut target = 0;
//         let max = MAX_ITER as usize;
//         let mut output = [false; 8];

//         'busy: while iter < max {
//             match &mut self.network[self.curr_net as usize][target] {
//                 Node::Opp(o) => {
//                     target = o.apply(input) as usize;
//                 },
//                 Node::Out(o) => {
//                     target = o.goto as usize;
//                     output[o.channel as usize] = true;
//                 },
//                 Node::Stop => break 'busy,
//             };
//             iter += 1;  
//             if iter == max { 
//                 self.try_next(0);
//                 break 'busy;
//             }
//         }
//         output
//     }

//     pub fn try_next(&mut self, run_score: u16) {
//         self.scores[self.curr_net as usize] = run_score;
//         self.curr_net += 1;
//         if self.curr_net > NET_COUNT {
//             self.curr_net = 0;

//             //get top 3 -> equalize other with top 3 behaviours
//             let mut top3: (Score, Score, Score) = 
//                 (Score { t:0, s:0 }, Score { t:0, s:0 }, Score { t:0, s:0 });

//             for i in 0..self.scores.len() {
//                 if self.scores[i] > top3.0.s { 
//                     top3.2.t = i; top3.2.s = self.scores[i]; <- todo
//                     top3.1.t = i; top3.1.s = self.scores[i]; <- todo
//                     top3.0.t = i; top3.0.s = self.scores[i]; <- todo
//                 }
//                 else if self.scores[i] > top3.1.s { top3.1.t = i; top3.1.s = self.scores[i]; }
//                 else if self.scores[i] > top3.2.s { top3.2.t = i; top3.2.s = self.scores[i]; }

//                 self.scores[i] = 0;
//             }
            
            
//         }
//     } 
// }

// struct Score { t: usize, s: u16 }

// #[derive(Copy, Clone)]
// pub enum Node { Opp(Opperation), Out(Output), Stop }


// #[derive(Copy, Clone)]
// pub struct Output { channel: u8, goto: u8, }

// impl Output {
//     pub fn new(rng: &mut ThreadRng) -> Output {
//         Output { 
//             channel: rng.gen_range(0..8), 
//             goto: rng.gen_range(0..NET_SIZE),
//         }
//     }
// }


// #[derive(Copy, Clone)]
// pub struct Opperation { in1: u8, in2: u8, opp: u8, goto1: u8, goto2: u8, }

// impl Opperation {
//     pub fn new(rng: &mut ThreadRng) -> Opperation {
//         Opperation { 
//             in1: rng.gen_range(0..8), 
//             in2: rng.gen_range(0..8),  
//             opp: rng.gen_range(0..5),  
//             goto1: rng.gen_range(0..NET_SIZE),  
//             goto2: rng.gen_range(0..NET_SIZE),  
//         }
//     }

//     pub fn apply(&self, input: &[u8; 8]) -> u8 {
//         if match self.opp {
//             0 => input[self.in1 as usize] > input[self.in2 as usize],
//             1 => input[self.in1 as usize] < input[self.in2 as usize],
//             2 => input[self.in1 as usize] == input[self.in2 as usize],
//             3 => input[self.in1 as usize] != input[self.in2 as usize],
//             _ => true,
//         } {
//             self.goto1
//         } else {
//             self.goto2
//         }
//     }

//     pub fn evolve(&mut self, rng: &mut ThreadRng) {
//         match rng.gen_range(0..10) {
//             0 => if self.in1 < 7 { self.in1 += 1 } else { self.in1 = 0; },
//             1 => if self.in1 > 0 { self.in1 -= 1 } else { self.in1 = 7; },
//             2 => if self.in2 < 7 { self.in2 += 1 } else { self.in2 = 0; },
//             3 => if self.in2 > 0 { self.in2 -= 1 } else { self.in2 = 7; },
//             4 => if self.opp < 255 { self.opp += 1 } else { self.opp = 0; },
//             5 => if self.opp > 0 { self.opp -= 1 } else { self.opp = 255; },
//             6 => if self.goto1 < 255 { self.goto1 += 1 } else { self.goto1 = 0; },
//             7 => if self.goto1 > 0 { self.goto1 -= 1 } else { self.goto1 = 255; },
//             8 => if self.goto2 < 255 { self.goto2 += 1 } else { self.goto2 = 0; },
//             _ => if self.goto2 > 0 { self.goto2 -= 1 } else { self.goto2 = 255; },
//         };
//     }
// }