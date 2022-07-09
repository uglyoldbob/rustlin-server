use std::io::Cursor;

const embedded_sprite_table: &[u8] = include_bytes!("sprite_table.txt");

struct SpriteTableEntry {
	
}

impl SpriteTableEntry {
	pub fn load_embedded_table() -> Self {
		let mut cursor = Cursor::new(embedded_sprite_table);
		
		Self {
			
		}
	}
}

struct Sprite {
	
}