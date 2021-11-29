extern crate sdl2;
extern crate swarm_pool as swarm;

mod renderer;
use rand::prelude::ThreadRng;
use renderer::{Drawable, Gfx, Renderer};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;
use rand::Rng;
use swarm::control::SwarmControl;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const PLANE_SIZE: f32 = 8.0;
const SPAWN_DURATION: u16 = 3000;
const SPAWN_GLOW: f64 = 200.0 / SPAWN_DURATION as f64;
const COLLISION_RAD: f32 = PLANE_SIZE * 2.0;
const PLANE_COUNT: usize = 100;
const PLANE_MOVE_SPEED: f32 = 2.0;
const PLANE_ROTATE_SPEED: f32 = 0.1;
const BULLET_SPEED: f32 = 5.0;
const NUM_COMMAND_LINES: usize = 250;

type Bullet = (f32, f32, f32, usize);


pub struct Properties {
    rng: ThreadRng,
}

fn main() -> Result<(), String> {

    let props = Properties { rng: rand::thread_rng() };

    let mut swarm = renderer::
        new::<RenderObject, Properties>(SCREEN_WIDTH, SCREEN_HEIGHT, props)?;
    
    renderer::run(&mut swarm, |c| {
        match c.target() {
            RenderObject::Plane(p) => update_plane(p),
            RenderObject::Bullet(b) => update_bullet(b),
            RenderObject::None => {},
        }
    });
    Ok(())
}

fn update_plane(p: &mut Plane) {
    p.steer();
    p.fly();
    p.bounds();
}

fn update_bullet(b: &mut Bullet) {
    
}

static RAD120: f32 = (3.14159 * 2.0) / 3.0;
static RAD240: f32 = -RAD120;

#[derive(Clone)]
enum RenderObject {
    None,
    Plane (Plane),
    Bullet (Bullet),
}

impl Drawable for RenderObject {
    fn draw(&mut self, gfx: &mut Gfx) {
        match self {

            RenderObject::Plane(p) => {
                let rotation2 = p.rotation + RAD120;
                let rotation3 = rotation2 + RAD120;

                let alpha = match p.spawn_time > 0 { true => 50, false => 250 };

                let _result = gfx.canvas.filled_trigon(
                    (p.position.0 + p.rotation.sin() * p.size) as i16,
                    (p.position.1 - p.rotation.cos() * p.size) as i16,
                    (p.position.0 + rotation2.sin() * p.size) as i16,
                    (p.position.1 - rotation2.cos() * p.size) as i16,
                    (p.position.0 + rotation3.sin() * p.size) as i16,
                    (p.position.1 - rotation3.cos() * p.size) as i16,
                    (50, alpha, alpha, 255),
                );
            },

            RenderObject::Bullet(b) => {
                let _result = gfx.canvas.pixel(
                    b.0 as i16,
                    b.1 as i16,
                    (250, 250, 50, 255),
                );
            },

            RenderObject::None => {},
        }
    }
}

impl Default for RenderObject {
    fn default() -> Self { RenderObject::None }
}

#[derive(Default, Clone)]
struct Plane {
    id: usize,
    position: (f32, f32),
    velocity: f32,
    rotation: f32,
    size: f32,
    pilot: Pilot,
    age: u128,
    fire: (bool, u8),
    spawn_time: u16,
}

impl Plane {
    fn new(id: usize, rng: &mut ThreadRng) -> Self {
        let mut plane = Plane {
            id,
            position: (0.0, 0.0), 
            velocity: PLANE_MOVE_SPEED,
            rotation: 0.0, 
            size: PLANE_SIZE,
            pilot: Pilot::new(rng),
            age: 0,
            fire: (false, 0),
            spawn_time: 0,
        };

        plane.reset(rng);
        plane.spawn_time = rng.gen_range(0..SPAWN_DURATION);
        plane
    }

    fn reset(&mut self, rng: &mut ThreadRng) {
        self.position = (   
            rng.gen_range(50..SCREEN_WIDTH - 50) as f32, 
            rng.gen_range(50..SCREEN_HEIGHT - 50) as f32
        );
        self.rotation = rng.gen_range(0..628) as f32 / 100.0;
        self.age = 0;
        self.fire = (false, 0);
        self.spawn_time = SPAWN_DURATION;
        self.pilot.command_line = 0;
    }

    fn evolve(&mut self) {
        let mut rng = rand::thread_rng();
        self.reset(&mut rng);
        for cmd in &mut self.pilot.commands {
            cmd.evolve(&mut rng);
        }
    }

    fn fire(&mut self, bullets: &mut Vec<Bullet>) {
        if self.spawn_time > 0 { return; }

        if bullets.len() < 1000 && self.fire.0  && self.fire.1 == 0 {
            self.fire = (false, 50);
            bullets.push((
                self.position.0 + self.rotation.sin() * (self.size + 1.0),
                self.position.1 - self.rotation.cos() * (self.size + 1.0),
                self.rotation,
                self.id,
            ));
        }
        else if self.fire.1 > 0 { self.fire.1 -= 1; }
    }

    fn bounds(&mut self) -> bool {
        // detect out of bounds
        if self.position.0 < COLLISION_RAD 
        || self.position.0 > SCREEN_WIDTH as f32 - COLLISION_RAD
        || self.position.1 < COLLISION_RAD 
        || self.position.1 > SCREEN_HEIGHT as f32 - COLLISION_RAD {
            true
        } else {
            false
        }
    }

    fn collisions (planes: &mut Vec<Plane>) {
        // detect collision
        'outer: for i in 0..planes.len() {
            if planes[i].spawn_time > 0 { continue 'outer; }
            
            'inner: for j in 0..planes.len() {
                if planes[j].spawn_time > 0 { continue 'inner; } 

                if planes[i].id != planes[j].id
                    && planes[i].position.0 > planes[j].position.0 - COLLISION_RAD
                    && planes[i].position.0 < planes[j].position.0 + COLLISION_RAD
                    && planes[i].position.1 > planes[j].position.1 - COLLISION_RAD
                    && planes[i].position.1 < planes[j].position.1 + COLLISION_RAD 
                {                    
                    planes[i].evolve();
                    planes[j].evolve();
                }
            }
        }
    }

    fn steer(&mut self) {
        
        let result = self.pilot.execute([
            self.position.0, self.position.1, self.velocity, self.rotation
        ]);
        
        match result.0 {
            // Skip Command
            0 => self.pilot.command_line += 1,
            // Goto Commandline,
            1 => self.pilot.command_line = result.1 as usize,
            // Rotate left,
            2 => self.rotation -= PLANE_ROTATE_SPEED / result.1 as f32,
            // Rotate right,
            3 => self.rotation += PLANE_ROTATE_SPEED / result.1 as f32,
            // Fire
            4 => self.fire.0 = true,
            // Reload
            _ => {} //if self.fire.1 > 0 { self.fire.1 -= 1; },
        }
    }

    fn fly(&mut self) {
        if self.position.0 < 0.0 { self.position.0 = 0.0; }
        else if self.position.0 > SCREEN_WIDTH as f32 { self.position.0 = SCREEN_WIDTH as f32 ; }
        if self.position.1 < 0.0 { self.position.1 = 0.0; }
        else if self.position.1 > SCREEN_HEIGHT as f32  { self.position.1 = SCREEN_HEIGHT as f32; }

        if self.spawn_time > 0 { return; }

        self.position.0 += self.rotation.sin() * self.velocity;
        self.position.1 -= self.rotation.cos() * self.velocity;
        self.age += 1;
    }
}

#[derive(Default, Clone)]
struct Pilot {
    commands: Vec<Command>,
    command_line: usize,
}

impl Pilot {
    fn new(rng: &mut ThreadRng) -> Self {
        Pilot {
            commands: vec![Command::new(rng); NUM_COMMAND_LINES],
            command_line: 0, 
        }
    }
    fn inherit(&mut self, other: &Self) {
        for i in 0..NUM_COMMAND_LINES {
            self.commands[i] = other.commands[i].clone();
        }
    }

    fn execute(&mut self, input: [f32; 4]) -> (u8, usize) {
        if self.command_line >= NUM_COMMAND_LINES { self.command_line = 0; }
        let doing = &self.commands[self.command_line];
        self.command_line += 1;
        
        match match &doing.func {
            0 => input[doing.in1 as usize] > input[doing.in2 as usize], 
            1 => input[doing.in1 as usize] < input[doing.in2 as usize], 
            _ => input[doing.in1 as usize] == input[doing.in2 as usize], 
        } {
            true => (doing.out1, doing.val),
            false => (doing.out2, doing.val),
        }
    }
}

const IN_COUNT: u8 = 4;
const FUNC_COUNT: u8 = 3;
const OUT_COUNT: u8 = 6;
const VAL_COUNT: usize = NUM_COMMAND_LINES;

#[derive(Clone)]
struct Command {
    in1: u8, in2: u8, 
    func: u8, 
    out1: u8, out2: u8,
    val: usize,
}

impl Command {
    fn new(rng: &mut ThreadRng) -> Self {
        Command {
            in1: rng.gen_range(0..IN_COUNT), 
            in2: rng.gen_range(0..IN_COUNT), 
            func: rng.gen_range(0..FUNC_COUNT), 
            out1: rng.gen_range(0..OUT_COUNT), 
            out2: rng.gen_range(0..OUT_COUNT),
            val: rng.gen_range(0..VAL_COUNT),
        }
    }
    fn evolve(&mut self, rng: &mut ThreadRng) {
        match rng.gen_range(0..2) {
            0 => {  self.in1 += 1;
                    if self.in1 > IN_COUNT-1 { self.in1 = 0; } 
            },
            _ => {  if self.in1 == 0 { self.in1 = IN_COUNT-1; }
                    else { self.in1 -= 1; }
            },
        };
        match rng.gen_range(0..2) {
            0 => {  self.in2 += 1;
                    if self.in2 > IN_COUNT-1 { self.in2 = 0; } 
            },
            _ => {  if self.in2 == 0 { self.in2 = IN_COUNT-1; }
                    else { self.in2 -= 1; }
            },
        };
        match rng.gen_range(0..2) {
            0 => {  self.out1 += 1;
                    if self.out1 > OUT_COUNT-1 { self.out1 = 0; } 
            },
            _ => {  if self.out1 == 0 { self.out1 = OUT_COUNT-1; }
                    else { self.out1 -= 1; }
            },
        };
        match rng.gen_range(0..2) {
            0 => {  self.out2 += 1;
                    if self.out2 > OUT_COUNT-1 { self.out2 = 0; } 
            },
            _ => {  if self.out2 == 0 { self.out2 = OUT_COUNT-1; }
                    else { self.out2 -= 1; }
            },
        };
        match rng.gen_range(0..4) {
            0 => {  self.func += 1;
                    if self.func > FUNC_COUNT-1 { self.func = 0; } 
            },
            1 => {  if self.func == 0 { self.func = FUNC_COUNT-1; }
                    else { self.func -= 1; }
            },
            _ => {},
        };
        match rng.gen_range(0..2) {
            0 => {  self.val += rng.gen_range(0..10);
                    if self.val > VAL_COUNT-1 { self.val = 0; } 
            },
            _ => {
                let range = rng.gen_range(0..10);
                if range > self.val { self.val = 0; }
                else { self.val -= range; }
                if self.val > VAL_COUNT-1 { self.val = VAL_COUNT-1; } 
            },
        }
    }
}