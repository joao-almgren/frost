// https://medium.com/codex/rust-modules-and-project-structure-832404a33e2e
use bytemuck::{ Pod, Zeroable };
use frost::run;
mod wfo;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex
{
	pos: [f32; 4],
	col: [f32; 4],
}

fn main()
{
	let ferris = wfo::load_wfo("rustacean-3d").unwrap();

	for e in ferris
	{
		println!("{:?}", e);
	}

	run();
}
