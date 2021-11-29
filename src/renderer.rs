use std::thread;
use std::time::Duration;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;

use swarm::Swarm;
use swarm::control::SwarmControl;


pub trait Drawable {
    fn draw(&mut self, gfx: &mut Gfx);
}

pub struct Renderer<P> {
    gfx: Gfx,
    props: P,
}

pub struct Gfx {
    pub canvas: Canvas<Window>,
    pub events: EventPump,
}

pub fn new<T: Drawable + Default + Clone, P>
    (width: u32, height: u32, properties: P) -> Result<Swarm<T, Renderer<P>>, String> 
{
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window(
            "rust-sdl2_gfx: draw line & FPSManager",
            width,
            height,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
        
    let renderer = Renderer {
        gfx: Gfx {
            canvas: window.into_canvas().build().map_err(|e| e.to_string())?,
            events: sdl_context.event_pump()?
        },
        props: properties,
    };

    Ok (Swarm::<T, Renderer<P>>::new(1000, renderer))
}

pub fn run<T: Drawable + Default + Clone, P>(
    swarm: &mut Swarm<T, Renderer<P>>, 
    update: fn(&mut SwarmControl<T, Renderer<P>>)
) {
    'main: loop {
        for event in swarm.properties.gfx.events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if keycode == Keycode::Escape {
                        break 'main;
                    }
                },
                _ => {},
            }
        }

        swarm.update(update);

        swarm.properties.gfx.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        swarm.properties.gfx.canvas.clear();

        swarm.for_all(|t, pool, props| 
            pool[*t].draw(&mut props.gfx)
        );

        swarm.properties.gfx.canvas.present();
        thread::sleep(Duration::from_millis(10));
    }
}