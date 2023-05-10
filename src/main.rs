// https://medium.com/codex/rust-modules-and-project-structure-832404a33e2e
use frost::run;
mod wfo;

fn main()
{
	let ferris = wfo::load_wfo("rustacean-3d").unwrap();

	for e in ferris
	{
		println!("{:?}", e);
	}

	run();
}
