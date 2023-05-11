#![allow(dead_code)]
#![allow(unused_variables)]
use std::str;
use std::fs::File;
use std::io::{ BufReader, BufRead };
use std::num::IntErrorKind;
use std::collections::HashMap;
use bytemuck::{ Pod, Zeroable };

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Element
{
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub color: [f32; 3],
}

struct Face
{
	v: usize,
	vt: usize,
	vn: usize,
	m: String
}

// https://en.wikipedia.org/wiki/Wavefront_.obj_file
pub fn load_wfo(fname: &str) -> Result<Vec<Element>, Box<dyn std::error::Error>>
{
	let mut colors: HashMap<String, [f32; 3]> = HashMap::new();
	let mut material: String = "".to_string();

	let file = File::open(format!("{}.mtl", fname))?;
	let reader = BufReader::new(file);

	for line in reader.lines()
	{
		let line = line?;

		if line.len() > 6 && line[0..6] == *"newmtl"
		{
			material = line[7..].to_string();
			colors.insert(material.clone(), [0.0, 0.0, 0.0]);
		}
		else if line.len() > 2 && line[0..2] == *"Kd"
		{
			let line = line.as_bytes();
			let index0 = find_next_space(3, line);
			let index1 = find_next_space(index0.1, line);
			let index2 = find_next_space(index1.1, line);

			unsafe
			{
				*colors.get_mut(&material).unwrap() =
				[
					str::from_utf8_unchecked(&line[index0.0..index0.1]).parse()?,
					str::from_utf8_unchecked(&line[index1.0..index1.1]).parse()?,
					str::from_utf8_unchecked(&line[index2.0..index2.1]).parse()?
				];
			}
		}
	}

	let mut elements: Vec<Element> = Vec::new();
	let mut vertices: Vec<[f32; 3]> = Vec::new();
	let mut normals: Vec<[f32; 3]> = Vec::new();
	let mut material: String = "".to_string();

	let file = File::open(format!("{}.obj", fname))?;
	let reader = BufReader::new(file);

	for line in reader.lines()
	{
		let line = line?;

		if line.len() < 2
		{
			continue;
		}

		if line[0..2] == *"v "
		{
			let line = line.as_bytes();
			let index0 = find_next_space(2, line);
			let index1 = find_next_space(index0.1, line);
			let index2 = find_next_space(index1.1, line);

			unsafe
			{
				vertices.push(
				[
					-str::from_utf8_unchecked(&line[index0.0..index0.1]).parse()?,
					str::from_utf8_unchecked(&line[index1.0..index1.1]).parse()?,
					str::from_utf8_unchecked(&line[index2.0..index2.1]).parse()?
				]);
			}
		}
		else if line[0..2] == *"vn"
		{
			let line = line.as_bytes();
			let index0 = find_next_space(3, line);
			let index1 = find_next_space(index0.1, line);
			let index2 = find_next_space(index1.1, line);

			unsafe
			{
				normals.push(
				[
					-str::from_utf8_unchecked(&line[index0.0..index0.1]).parse()?,
					str::from_utf8_unchecked(&line[index1.0..index1.1]).parse()?,
					str::from_utf8_unchecked(&line[index2.0..index2.1]).parse()?
				]);
			}
		}
		else if line.len() > 6 && line[0..6] == *"usemtl"
		{
			material = line[7..].to_string();
		}
		else if line[0..2] == *"f "
		{
			let line = line.as_bytes();
			let mut faces: Vec<Face> = Vec::new();
			let mut start = 2;

			loop
			{
				while start < line.len() && line[start] == b' '
				{
					start += 1;
				}

				if start == line.len()
				{
					break;
				}

				let index0 = find_next_slash(start, line);
				let index1 = find_next_slash(index0.1, line);
				let index2 = find_next_slash(index1.1, line);
				start = index2.1;

				unsafe
				{
					faces.push(Face
					{
						v: match str::from_utf8_unchecked(&line[index0.0..index0.1]).parse()
						{
							Ok(n) => n - 1,
							Err::<usize, _>(error) => match error.kind()
							{
								IntErrorKind::Empty => 0,
								_ => panic!("{}", error)
							}
						},
						vt: match str::from_utf8_unchecked(&line[index1.0..index1.1]).parse()
						{
							Ok(n) => n - 1,
							Err::<usize, _>(error) => match error.kind()
							{
								IntErrorKind::Empty => 0,
								_ => panic!("{}", error)
							}
						},
						vn: match str::from_utf8_unchecked(&line[index2.0..index2.1]).parse()
						{
							Ok(n) => n - 1,
							Err::<usize, _>(error) => match error.kind()
							{
								IntErrorKind::Empty => 0,
								_ => panic!("{}", error)
							}
						},
						m: material.clone()
					});
				}
			}

			// triangulation (0, 2, 1) (0, 3, 2) (0, 4, 3) ...
			if faces.len() >= 3
			{
				let mut start: usize = 1;
				while start + 1 < faces.len()
				{
					elements.push(Element{ position: vertices[faces[0].v], normal: normals[faces[0].vn], color: colors[&faces[0].m] });
					elements.push(Element{ position: vertices[faces[start + 1].v], normal: normals[faces[start + 1].vn], color: colors[&faces[start + 1].m] });
					elements.push(Element{ position: vertices[faces[start].v], normal: normals[faces[start].vn], color: colors[&faces[start].m] });

					start += 1;
				}
			}
		}
	}

	Ok(elements)
}

fn find_next_space(mut i: usize, line: &[u8]) -> (usize, usize)
{
	while line[i] == b' '
	{
		i += 1;
	}

	let mut j = i;
	while j < line.len() && line[j] != b' '
	{
		j += 1;
	}

	(i, j)
}

fn find_next_slash(mut i: usize, line: &[u8]) -> (usize, usize)
{
	if line[i] == b'/'
	{
		i += 1;
	}

	let mut j = i;
	while j < line.len() && line[j] != b' ' && line[j] != b'/'
	{
		j += 1;
	}

	(i, j)
}
