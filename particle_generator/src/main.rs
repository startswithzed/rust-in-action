use graphics::math::{Vec2d, add, mul_scalar};
use piston_window::*;
use rand::prelude::*;
use std::alloc::{GlobalAlloc, System, Layout};
use std::time::Instant;


#[global_allocator] // marks ALLOCATOR as satisfying the GlobalAlloc trait.
static ALLOCATOR: ReportingAllocator = ReportingAllocator;

struct ReportingAllocator;

unsafe impl GlobalAlloc for ReportingAllocator {
    /// prints the time taken for each memory allocation to stdout
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();
        let ptr = System.alloc(layout); // use system's default memory allocator
        let end = Instant::now();
        let time_taken = end - start;
        let bytes_requested = layout.size();

        eprintln!("bytes requested: {}\t time taken(ns): {}", bytes_requested, time_taken.as_nanos());

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

struct World {
    current_turn: u64,
    particles: Vec<Box<Particle>>,
    height: f64,
    width: f64,
    rng: ThreadRng,
}

struct Particle {
    height: f64,
    width: f64,
    position: Vec2d<f64>,
    velocity: Vec2d<f64>,
    acceleration: Vec2d<f64>,
    color: [f32; 4],
}

impl Particle {
    fn new(world: &World) -> Particle {
        let mut rng = thread_rng();

        // start at a random location along the bottom of the window
        let x = rng.gen_range(0.0..=world.width);
        let y = world.height;

        // moves with a vertical velocity
        let x_velocity = 0.0;
        let y_velocity = rng.gen_range(-2.0..0.0);

        // speed of rise increases over time
        let x_acceleration = 0.0;
        let y_acceleration = rng.gen_range(0.0..0.15);

        Particle {
            height: 4.0,
            width: 4.0,
            position: [x, y].into(), // into() converts the arrays of type [f64; 2] into Vec2d
            velocity: [x_velocity, y_velocity].into(),
            acceleration: [x_acceleration, y_acceleration].into(),
            color: [1.0, 1.0, 1.0, 0.99] // white with a bit of transparency
        }
    }

    fn update(&mut self) {
        // move the particle
        self.velocity = add(self.velocity, self.acceleration);
        self.position = add(self.position, self.velocity);
        self.acceleration = mul_scalar(self.acceleration, 0.7); // decrease acceleration

        // make the particle more transparent over time
        self.color[3] *= 0.995;
    }
}

impl World {
    fn new(width: f64, height: f64) -> World {
        World {
            current_turn: 0,
            // Uses Box<Particle> rather than Particle to incur an extra memory allocation when every particle is created
            particles: Vec::<Box<Particle>>::new(),
            height,
            width,
            rng: thread_rng(),
        }
    }

    fn add_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let particle = Particle::new(&self); // particle variable is stored as local variable on stack
            // takes ownership of particle
            // moves its data to heap
            // creates reference to that data on the stack
            let boxed_particle = Box::new(particle);
            self.particles.push(boxed_particle);
        }
    }

    fn remove_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let mut to_delete = None;
            // get iterator that returns
            let particle_iter = self.particles
                .iter()
                .enumerate();

            // removes the first particle that's invisible
            for (i, particle) in particle_iter {
                if particle.color[3] < 0.02 {
                    to_delete = Some(i);
                }
                break;
            }

            // if there are no invisible particles then it removes the oldest particle
            if let Some(i) = to_delete {
                self.particles.remove(i);
            } else {
                self.particles.remove(0);
            };
        }
    }

    fn update(&mut self) {
        let  n = self.rng.gen_range(-3..=3);

        if n > 0 {
            self.add_shapes(n);
        } else {
            self.remove_shapes(n);
        }

        self.particles.shrink_to_fit();

        for shape in &mut self.particles {
            shape.update();
        }

        self.current_turn += 1;
    }
}

fn main() {
    let (width, height) = (1280.0, 960.0);
    let mut window: PistonWindow = WindowSettings::new(
        "particles", [width, height]
    )
        .exit_on_esc(true)
        .build()
        .expect("could not create a window.");

    let mut world = World::new(width, height);
    world.add_shapes(1000);

    while let Some(event) = window.next() {
        world.update();

        window.draw_2d(&event, |ctx, renderer, _device| {
            clear([0.15, 0.17, 0.17, 0.9], renderer);

            for s in &mut world.particles {
                let size = [s.position[0], s.position[1], s.width, s.height];
                rectangle(s.color, size, ctx.transform, renderer);
            }
        });
    }
}
