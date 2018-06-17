use std::iter::Iterator;

const ISOLATE_COMPONENT_MASKS: [u64; 3] = [
	0b100100100100100100100100100100100100100100100100100100100100100,
	0b010010010010010010010010010010010010010010010010010010010010010,
	0b001001001001001001001001001001001001001001001001001001001001001
];

const ISOLATED_COMPONENT_SHIFTS: [u32; 3] = [ 2, 1, 0 ];

const CONTRACTION_MASKS: [u64; 5] = [
	0b000011000011000011000011000011000011000011000011000011000011,
	0b000000001111000000001111000000001111000000001111000000001111,
	0b000000000000000011111111000000000000000011111111,
	0b0000000000000000111111111111111100000000000000001111111111111111,
	0b1111111111111111
];

const DILATION_MASKS: [u64; 5] = [
	0b000011000011000011000011000011000011000011000011000011000011,
	0b000000001111000000001111000000001111000000001111000000001111,
	0b000000000000000000000000000011111111000000000000000011111111,
	0b000000000000111111111111111100000000000000001111111111111111,
	0b1111111111111111
];

pub trait Key: Sized {
    fn root_key() -> Self;
    fn child_key(&self, i: u64) -> Self;
    fn level(&self) -> usize;
    fn neighbor(&self, x: bool, y: bool, z: bool) -> Option<Self>;
}

#[derive(Debug, Copy, Clone, Hash)]
pub struct MortonKey(pub u64);


impl Key for MortonKey {
    fn root_key() -> MortonKey {
        MortonKey(1)
    }
    
    fn child_key (&self, child: u64) -> MortonKey {
        let p = &self.0 << 3;
        MortonKey(p | child)
    }

    fn level(&self) -> usize {
            (((self.0 as f64).log2() / 3.0).floor()) as usize
    }

    fn neighbor(&self, x: bool, y: bool, z: bool) -> Option<MortonKey> {
        None
    }
}

impl MortonKey {
	fn get_component(&self, component: usize) -> u16 {
		let shifted_dilated_component = &self.0 & ISOLATE_COMPONENT_MASKS[component];
		let dilated_component = shifted_dilated_component >> ISOLATED_COMPONENT_SHIFTS[component];
		
		let mut component = dilated_component;
		for i in 0..5 {
			// Since we have three dimensions, the gaps starts as 2, then doubles
			// so 2^1, 2^2, .., 2^4
			let gap_size = 1 << (i + 5); 

			// close the gap
			let shifted_component = component >> gap_size;

			// combine original and gap shifted, then mask
			let combined_segments = shifted_component | component;
			component = combined_segments & CONTRACTION_MASKS[i];
		}
		
		// Since root is one, this ends up looking like a part of the z (2) component
		// So we make an extra mask to remove it. We really only need this for 2 component, but
		// its cheaper than if statement
		let level = self.level();		
		let length_mask: u64 = (1 << level) - 1;
		component &= length_mask;

		// This is a type safety thing, since component can be at most 16 bits 
		component as u16
	}

	fn dilate_component(c: u16) -> u64 {
		let mut component = c as u64;	
		for i in 0..5 {
			let gap_size = 1 << ( 6 - i);
			let shifted_component = component >> gap_size;
			let combined_components = shifted_component | component;
			component = combined_components & DILATION_MASKS[i];
		}

0
	}

	pub fn from_components(x: u16, y:u16, z: u16, level: usize) -> MortonKey {
		MortonKey::root_key()	
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn level() {
		let mut k = MortonKey::root_key();
		assert_eq!(k.level(), 0);
		for i in 0..16 {
			k = k.child_key(4);
			assert_eq!(k.level(), i + 1);
		}

		let mut k = MortonKey::root_key();
		assert_eq!(k.level(), 0);
		for i in 0..16 {
			k = k.child_key(i % 8);
			assert_eq!(k.level(), (i + 1) as usize);
		}
	}
	
	#[test]
	fn get_x() {
		let mut k = MortonKey::root_key();
		for i in 0..5 {
			k = k.child_key(4);
		}
		let x = k.get_component(0);
		assert_eq!( x, (2u32.pow(5) - 1) as u16);

		k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key(4);
		}
		let x = k.get_component(0);
		assert_eq!( x, (2u32.pow(16) - 1) as u16);
	}

	#[test]
	fn get_y() {
		let mut k = MortonKey::root_key();
		for i in 0..5 {
			k = k.child_key(2);
		}
		let y = k.get_component(1);
		assert_eq!( y, (2u32.pow(5) - 1) as u16);

		k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key(2);
		}
		let y = k.get_component(1);
		assert_eq!( y, (2u32.pow(16) - 1) as u16);
	}
	
	#[test]
	fn get_z() {
		let mut k = MortonKey::root_key();
		for i in 0..5 {
			k = k.child_key(1);
		}
		let z = k.get_component(2);
		assert_eq!( z, (2u32.pow(5) - 1) as u16);

		k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key(1);
		}
		let z = k.get_component(2);
		assert_eq!( z, (2u32.pow(16) - 1) as u16);
	}
	
	#[test]
	fn get_all() {
		let mut k = MortonKey::root_key();
		for i in 0..7 {
			k = k.child_key(7);
		}
		let x = k.get_component(0);
		let y = k.get_component(1);
		let z = k.get_component(2);
		assert_eq!( x, (2u32.pow(7) - 1) as u16);
		assert_eq!( y, (2u32.pow(7) - 1) as u16);
		assert_eq!( z, (2u32.pow(7) - 1) as u16);

		k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key(7);
		}
		let x = k.get_component(0);
		let y = k.get_component(1);
		let z = k.get_component(2);
		assert_eq!( x, (2u32.pow(16) - 1) as u16);
		assert_eq!( y, (2u32.pow(16) - 1) as u16);
		assert_eq!( z, (2u32.pow(16) - 1) as u16);
	}

	#[test]
	fn get_mix() {
		let mut k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key(i % 8)
		}
		let x = k.get_component(0);
		let y = k.get_component(1);
		let z = k.get_component(2);
		assert_eq!( x, 0b0000111100001111);
		assert_eq!( y, 0b0011001100110011);
		assert_eq!( z, 0b0101010101010101);

		let mut k = MortonKey::root_key();
		for i in 0..16 {
			k = k.child_key((i % 4) + 3)
		}
		let x = k.get_component(0);
		let y = k.get_component(1);
		let z = k.get_component(2);
		assert_eq!( x, 0b0111011101110111);
		assert_eq!( y, 0b1001100110011001);
		assert_eq!( z, 0b1010101010101010);
	}
}



