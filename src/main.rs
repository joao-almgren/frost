use frost::run;
mod wfo;
// https://medium.com/codex/rust-modules-and-project-structure-832404a33e2e

use bytemuck::{ Pod, Zeroable };

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex
{
	_pos: [f32; 4],
	_col: [f32; 4],
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
