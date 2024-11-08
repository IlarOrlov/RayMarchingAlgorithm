use once_cell::sync::Lazy;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

// Define the volume size and the tables here
const SIZE: usize = 64;
//const SIZE_CUBED: usize = SIZE * SIZE * SIZE;
const SIZE_CUBED: usize = 1000; // Example size for the volume data

// Edge and triangle tables (define these tables based on Marching Cubes algorithm)
static EDGE_TABLE: &[u32; 256] = &[
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
    0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
    0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c,
    0x99c, 0x895, 0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90,
    0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435, 0x53c,
    0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30,
    0x3a0, 0x2a9, 0x1a3, 0x0aa, 0x7a6, 0x6af, 0x5a5, 0x4ac,
    0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0,
    0x460, 0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c,
    0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963, 0xa69, 0xb60,
    0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc,
    0xdfc, 0xcf5, 0xfff, 0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0,
    0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
    0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950,
    0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6, 0x2cf, 0x1c5, 0x0cc,
    0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0,
    0x8c0, 0x9c9, 0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc,
    0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9, 0x7c0,
    0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c,
    0x15c, 0x055, 0x35f, 0x256, 0x55a, 0x453, 0x759, 0x650,
    0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc,
    0x2fc, 0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0,
    0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f, 0xd65, 0xc6c,
    0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460,
    0xca0, 0xda9, 0xea3, 0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac,
    0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
    0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c,
    0x53c, 0x435, 0x73f, 0x636, 0x13a, 0x033, 0x339, 0x230,
    0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c,
    0x69c, 0x795, 0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190,
    0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905, 0x80c,
    0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000,
];

static TRIANGLE_TABLE: Lazy<Vec<Vec<i32>>> = Lazy::new(|| vec![
    vec![ -1 ],
	vec![ 0, 3, 8, -1 ],
	vec![ 0, 9, 1, -1 ],
	vec![ 3, 8, 1, 1, 8, 9, -1 ],
	vec![ 2, 11, 3, -1 ],
	vec![ 8, 0, 11, 11, 0, 2, -1 ],
	vec![ 3, 2, 11, 1, 0, 9, -1 ],
	vec![ 11, 1, 2, 11, 9, 1, 11, 8, 9, -1 ],
	vec![ 1, 10, 2, -1 ],
	vec![ 0, 3, 8, 2, 1, 10, -1 ],
	vec![ 10, 2, 9, 9, 2, 0, -1 ],
	vec![ 8, 2, 3, 8, 10, 2, 8, 9, 10, -1 ],
	vec![ 11, 3, 10, 10, 3, 1, -1 ],
	vec![ 10, 0, 1, 10, 8, 0, 10, 11, 8, -1 ],
	vec![ 9, 3, 0, 9, 11, 3, 9, 10, 11, -1 ],
	vec![ 8, 9, 11, 11, 9, 10, -1 ],
	vec![ 4, 8, 7, -1 ],
	vec![ 7, 4, 3, 3, 4, 0, -1 ],
	vec![ 4, 8, 7, 0, 9, 1, -1 ],
	vec![ 1, 4, 9, 1, 7, 4, 1, 3, 7, -1 ],
	vec![ 8, 7, 4, 11, 3, 2, -1 ],
	vec![ 4, 11, 7, 4, 2, 11, 4, 0, 2, -1 ],
	vec![ 0, 9, 1, 8, 7, 4, 11, 3, 2, -1 ],
	vec![ 7, 4, 11, 11, 4, 2, 2, 4, 9, 2, 9, 1, -1 ],
	vec![ 4, 8, 7, 2, 1, 10, -1 ],
	vec![ 7, 4, 3, 3, 4, 0, 10, 2, 1, -1 ],
	vec![ 10, 2, 9, 9, 2, 0, 7, 4, 8, -1 ],
	vec![ 10, 2, 3, 10, 3, 4, 3, 7, 4, 9, 10, 4, -1 ],
	vec![ 1, 10, 3, 3, 10, 11, 4, 8, 7, -1 ],
	vec![ 10, 11, 1, 11, 7, 4, 1, 11, 4, 1, 4, 0, -1 ],
	vec![ 7, 4, 8, 9, 3, 0, 9, 11, 3, 9, 10, 11, -1 ],
	vec![ 7, 4, 11, 4, 9, 11, 9, 10, 11, -1 ],
	vec![ 9, 4, 5, -1 ],
	vec![ 9, 4, 5, 8, 0, 3, -1 ],
	vec![ 4, 5, 0, 0, 5, 1, -1 ],
	vec![ 5, 8, 4, 5, 3, 8, 5, 1, 3, -1 ],
	vec![ 9, 4, 5, 11, 3, 2, -1 ],
	vec![ 2, 11, 0, 0, 11, 8, 5, 9, 4, -1 ],
	vec![ 4, 5, 0, 0, 5, 1, 11, 3, 2, -1 ],
	vec![ 5, 1, 4, 1, 2, 11, 4, 1, 11, 4, 11, 8, -1 ],
	vec![ 1, 10, 2, 5, 9, 4, -1 ],
	vec![ 9, 4, 5, 0, 3, 8, 2, 1, 10, -1 ],
	vec![ 2, 5, 10, 2, 4, 5, 2, 0, 4, -1 ],
	vec![ 10, 2, 5, 5, 2, 4, 4, 2, 3, 4, 3, 8, -1 ],
	vec![ 11, 3, 10, 10, 3, 1, 4, 5, 9, -1 ],
	vec![ 4, 5, 9, 10, 0, 1, 10, 8, 0, 10, 11, 8, -1 ],
	vec![ 11, 3, 0, 11, 0, 5, 0, 4, 5, 10, 11, 5, -1 ],
	vec![ 4, 5, 8, 5, 10, 8, 10, 11, 8, -1 ],
	vec![ 8, 7, 9, 9, 7, 5, -1 ],
	vec![ 3, 9, 0, 3, 5, 9, 3, 7, 5, -1 ],
	vec![ 7, 0, 8, 7, 1, 0, 7, 5, 1, -1 ],
	vec![ 7, 5, 3, 3, 5, 1, -1 ],
	vec![ 5, 9, 7, 7, 9, 8, 2, 11, 3, -1 ],
	vec![ 2, 11, 7, 2, 7, 9, 7, 5, 9, 0, 2, 9, -1 ],
	vec![ 2, 11, 3, 7, 0, 8, 7, 1, 0, 7, 5, 1, -1 ],
	vec![ 2, 11, 1, 11, 7, 1, 7, 5, 1, -1 ],
	vec![ 8, 7, 9, 9, 7, 5, 2, 1, 10, -1 ],
	vec![ 10, 2, 1, 3, 9, 0, 3, 5, 9, 3, 7, 5, -1 ],
	vec![ 7, 5, 8, 5, 10, 2, 8, 5, 2, 8, 2, 0, -1 ],
	vec![ 10, 2, 5, 2, 3, 5, 3, 7, 5, -1 ],
	vec![ 8, 7, 5, 8, 5, 9, 11, 3, 10, 3, 1, 10, -1 ],
	vec![ 5, 11, 7, 10, 11, 5, 1, 9, 0, -1 ],
	vec![ 11, 5, 10, 7, 5, 11, 8, 3, 0, -1 ],
	vec![ 5, 11, 7, 10, 11, 5, -1 ],
	vec![ 6, 7, 11, -1 ],
	vec![ 7, 11, 6, 3, 8, 0, -1 ],
	vec![ 6, 7, 11, 0, 9, 1, -1 ],
	vec![ 9, 1, 8, 8, 1, 3, 6, 7, 11, -1 ],
	vec![ 3, 2, 7, 7, 2, 6, -1 ],
	vec![ 0, 7, 8, 0, 6, 7, 0, 2, 6, -1 ],
	vec![ 6, 7, 2, 2, 7, 3, 9, 1, 0, -1 ],
	vec![ 6, 7, 8, 6, 8, 1, 8, 9, 1, 2, 6, 1, -1 ],
	vec![ 11, 6, 7, 10, 2, 1, -1 ],
	vec![ 3, 8, 0, 11, 6, 7, 10, 2, 1, -1 ],
	vec![ 0, 9, 2, 2, 9, 10, 7, 11, 6, -1 ],
	vec![ 6, 7, 11, 8, 2, 3, 8, 10, 2, 8, 9, 10, -1 ],
	vec![ 7, 10, 6, 7, 1, 10, 7, 3, 1, -1 ],
	vec![ 8, 0, 7, 7, 0, 6, 6, 0, 1, 6, 1, 10, -1 ],
	vec![ 7, 3, 6, 3, 0, 9, 6, 3, 9, 6, 9, 10, -1 ],
	vec![ 6, 7, 10, 7, 8, 10, 8, 9, 10, -1 ],
	vec![ 11, 6, 8, 8, 6, 4, -1 ],
	vec![ 6, 3, 11, 6, 0, 3, 6, 4, 0, -1 ],
	vec![ 11, 6, 8, 8, 6, 4, 1, 0, 9, -1 ],
	vec![ 1, 3, 9, 3, 11, 6, 9, 3, 6, 9, 6, 4, -1 ],
	vec![ 2, 8, 3, 2, 4, 8, 2, 6, 4, -1 ],
	vec![ 4, 0, 6, 6, 0, 2, -1 ],
	vec![ 9, 1, 0, 2, 8, 3, 2, 4, 8, 2, 6, 4, -1 ],
	vec![ 9, 1, 4, 1, 2, 4, 2, 6, 4, -1 ],
	vec![ 4, 8, 6, 6, 8, 11, 1, 10, 2, -1 ],
	vec![ 1, 10, 2, 6, 3, 11, 6, 0, 3, 6, 4, 0, -1 ],
	vec![ 11, 6, 4, 11, 4, 8, 10, 2, 9, 2, 0, 9, -1 ],
	vec![ 10, 4, 9, 6, 4, 10, 11, 2, 3, -1 ],
	vec![ 4, 8, 3, 4, 3, 10, 3, 1, 10, 6, 4, 10, -1 ],
	vec![ 1, 10, 0, 10, 6, 0, 6, 4, 0, -1 ],
	vec![ 4, 10, 6, 9, 10, 4, 0, 8, 3, -1 ],
	vec![ 4, 10, 6, 9, 10, 4, -1 ],
	vec![ 6, 7, 11, 4, 5, 9, -1 ],
	vec![ 4, 5, 9, 7, 11, 6, 3, 8, 0, -1 ],
	vec![ 1, 0, 5, 5, 0, 4, 11, 6, 7, -1 ],
	vec![ 11, 6, 7, 5, 8, 4, 5, 3, 8, 5, 1, 3, -1 ],
	vec![ 3, 2, 7, 7, 2, 6, 9, 4, 5, -1 ],
	vec![ 5, 9, 4, 0, 7, 8, 0, 6, 7, 0, 2, 6, -1 ],
	vec![ 3, 2, 6, 3, 6, 7, 1, 0, 5, 0, 4, 5, -1 ],
	vec![ 6, 1, 2, 5, 1, 6, 4, 7, 8, -1 ],
	vec![ 10, 2, 1, 6, 7, 11, 4, 5, 9, -1 ],
	vec![ 0, 3, 8, 4, 5, 9, 11, 6, 7, 10, 2, 1, -1 ],
	vec![ 7, 11, 6, 2, 5, 10, 2, 4, 5, 2, 0, 4, -1 ],
	vec![ 8, 4, 7, 5, 10, 6, 3, 11, 2, -1 ],
	vec![ 9, 4, 5, 7, 10, 6, 7, 1, 10, 7, 3, 1, -1 ],
	vec![ 10, 6, 5, 7, 8, 4, 1, 9, 0, -1 ],
	vec![ 4, 3, 0, 7, 3, 4, 6, 5, 10, -1 ],
	vec![ 10, 6, 5, 8, 4, 7, -1 ],
	vec![ 9, 6, 5, 9, 11, 6, 9, 8, 11, -1 ],
	vec![ 11, 6, 3, 3, 6, 0, 0, 6, 5, 0, 5, 9, -1 ],
	vec![ 11, 6, 5, 11, 5, 0, 5, 1, 0, 8, 11, 0, -1 ],
	vec![ 11, 6, 3, 6, 5, 3, 5, 1, 3, -1 ],
	vec![ 9, 8, 5, 8, 3, 2, 5, 8, 2, 5, 2, 6, -1 ],
	vec![ 5, 9, 6, 9, 0, 6, 0, 2, 6, -1 ],
	vec![ 1, 6, 5, 2, 6, 1, 3, 0, 8, -1 ],
	vec![ 1, 6, 5, 2, 6, 1, -1 ],
	vec![ 2, 1, 10, 9, 6, 5, 9, 11, 6, 9, 8, 11, -1 ],
	vec![ 9, 0, 1, 3, 11, 2, 5, 10, 6, -1 ],
	vec![ 11, 0, 8, 2, 0, 11, 10, 6, 5, -1 ],
	vec![ 3, 11, 2, 5, 10, 6, -1 ],
	vec![ 1, 8, 3, 9, 8, 1, 5, 10, 6, -1 ],
	vec![ 6, 5, 10, 0, 1, 9, -1 ],
	vec![ 8, 3, 0, 5, 10, 6, -1 ],
	vec![ 6, 5, 10, -1 ],
	vec![ 10, 5, 6, -1 ],
	vec![ 0, 3, 8, 6, 10, 5, -1 ],
	vec![ 10, 5, 6, 9, 1, 0, -1 ],
	vec![ 3, 8, 1, 1, 8, 9, 6, 10, 5, -1 ],
	vec![ 2, 11, 3, 6, 10, 5, -1 ],
	vec![ 8, 0, 11, 11, 0, 2, 5, 6, 10, -1 ],
	vec![ 1, 0, 9, 2, 11, 3, 6, 10, 5, -1 ],
	vec![ 5, 6, 10, 11, 1, 2, 11, 9, 1, 11, 8, 9, -1 ],
	vec![ 5, 6, 1, 1, 6, 2, -1 ],
	vec![ 5, 6, 1, 1, 6, 2, 8, 0, 3, -1 ],
	vec![ 6, 9, 5, 6, 0, 9, 6, 2, 0, -1 ],
	vec![ 6, 2, 5, 2, 3, 8, 5, 2, 8, 5, 8, 9, -1 ],
	vec![ 3, 6, 11, 3, 5, 6, 3, 1, 5, -1 ],
	vec![ 8, 0, 1, 8, 1, 6, 1, 5, 6, 11, 8, 6, -1 ],
	vec![ 11, 3, 6, 6, 3, 5, 5, 3, 0, 5, 0, 9, -1 ],
	vec![ 5, 6, 9, 6, 11, 9, 11, 8, 9, -1 ],
	vec![ 5, 6, 10, 7, 4, 8, -1 ],
	vec![ 0, 3, 4, 4, 3, 7, 10, 5, 6, -1 ],
	vec![ 5, 6, 10, 4, 8, 7, 0, 9, 1, -1 ],
	vec![ 6, 10, 5, 1, 4, 9, 1, 7, 4, 1, 3, 7, -1 ],
	vec![ 7, 4, 8, 6, 10, 5, 2, 11, 3, -1 ],
	vec![ 10, 5, 6, 4, 11, 7, 4, 2, 11, 4, 0, 2, -1 ],
	vec![ 4, 8, 7, 6, 10, 5, 3, 2, 11, 1, 0, 9, -1 ],
	vec![ 1, 2, 10, 11, 7, 6, 9, 5, 4, -1 ],
	vec![ 2, 1, 6, 6, 1, 5, 8, 7, 4, -1 ],
	vec![ 0, 3, 7, 0, 7, 4, 2, 1, 6, 1, 5, 6, -1 ],
	vec![ 8, 7, 4, 6, 9, 5, 6, 0, 9, 6, 2, 0, -1 ],
	vec![ 7, 2, 3, 6, 2, 7, 5, 4, 9, -1 ],
	vec![ 4, 8, 7, 3, 6, 11, 3, 5, 6, 3, 1, 5, -1 ],
	vec![ 5, 0, 1, 4, 0, 5, 7, 6, 11, -1 ],
	vec![ 9, 5, 4, 6, 11, 7, 0, 8, 3, -1 ],
	vec![ 11, 7, 6, 9, 5, 4, -1 ],
	vec![ 6, 10, 4, 4, 10, 9, -1 ],
	vec![ 6, 10, 4, 4, 10, 9, 3, 8, 0, -1 ],
	vec![ 0, 10, 1, 0, 6, 10, 0, 4, 6, -1 ],
	vec![ 6, 10, 1, 6, 1, 8, 1, 3, 8, 4, 6, 8, -1 ],
	vec![ 9, 4, 10, 10, 4, 6, 3, 2, 11, -1 ],
	vec![ 2, 11, 8, 2, 8, 0, 6, 10, 4, 10, 9, 4, -1 ],
	vec![ 11, 3, 2, 0, 10, 1, 0, 6, 10, 0, 4, 6, -1 ],
	vec![ 6, 8, 4, 11, 8, 6, 2, 10, 1, -1 ],
	vec![ 4, 1, 9, 4, 2, 1, 4, 6, 2, -1 ],
	vec![ 3, 8, 0, 4, 1, 9, 4, 2, 1, 4, 6, 2, -1 ],
	vec![ 6, 2, 4, 4, 2, 0, -1 ],
	vec![ 3, 8, 2, 8, 4, 2, 4, 6, 2, -1 ],
	vec![ 4, 6, 9, 6, 11, 3, 9, 6, 3, 9, 3, 1, -1 ],
	vec![ 8, 6, 11, 4, 6, 8, 9, 0, 1, -1 ],
	vec![ 11, 3, 6, 3, 0, 6, 0, 4, 6, -1 ],
	vec![ 8, 6, 11, 4, 6, 8, -1 ],
	vec![ 10, 7, 6, 10, 8, 7, 10, 9, 8, -1 ],
	vec![ 3, 7, 0, 7, 6, 10, 0, 7, 10, 0, 10, 9, -1 ],
	vec![ 6, 10, 7, 7, 10, 8, 8, 10, 1, 8, 1, 0, -1 ],
	vec![ 6, 10, 7, 10, 1, 7, 1, 3, 7, -1 ],
	vec![ 3, 2, 11, 10, 7, 6, 10, 8, 7, 10, 9, 8, -1 ],
	vec![ 2, 9, 0, 10, 9, 2, 6, 11, 7, -1 ],
	vec![ 0, 8, 3, 7, 6, 11, 1, 2, 10, -1 ],
	vec![ 7, 6, 11, 1, 2, 10, -1 ],
	vec![ 2, 1, 9, 2, 9, 7, 9, 8, 7, 6, 2, 7, -1 ],
	vec![ 2, 7, 6, 3, 7, 2, 0, 1, 9, -1 ],
	vec![ 8, 7, 0, 7, 6, 0, 6, 2, 0, -1 ],
	vec![ 7, 2, 3, 6, 2, 7, -1 ],
	vec![ 8, 1, 9, 3, 1, 8, 11, 7, 6, -1 ],
	vec![ 11, 7, 6, 1, 9, 0, -1 ],
	vec![ 6, 11, 7, 0, 8, 3, -1 ],
	vec![ 11, 7, 6, -1 ],
	vec![ 7, 11, 5, 5, 11, 10, -1 ],
	vec![ 10, 5, 11, 11, 5, 7, 0, 3, 8, -1 ],
	vec![ 7, 11, 5, 5, 11, 10, 0, 9, 1, -1 ],
	vec![ 7, 11, 10, 7, 10, 5, 3, 8, 1, 8, 9, 1, -1 ],
	vec![ 5, 2, 10, 5, 3, 2, 5, 7, 3, -1 ],
	vec![ 5, 7, 10, 7, 8, 0, 10, 7, 0, 10, 0, 2, -1 ],
	vec![ 0, 9, 1, 5, 2, 10, 5, 3, 2, 5, 7, 3, -1 ],
	vec![ 9, 7, 8, 5, 7, 9, 10, 1, 2, -1 ],
	vec![ 1, 11, 2, 1, 7, 11, 1, 5, 7, -1 ],
	vec![ 8, 0, 3, 1, 11, 2, 1, 7, 11, 1, 5, 7, -1 ],
	vec![ 7, 11, 2, 7, 2, 9, 2, 0, 9, 5, 7, 9, -1 ],
	vec![ 7, 9, 5, 8, 9, 7, 3, 11, 2, -1 ],
	vec![ 3, 1, 7, 7, 1, 5, -1 ],
	vec![ 8, 0, 7, 0, 1, 7, 1, 5, 7, -1 ],
	vec![ 0, 9, 3, 9, 5, 3, 5, 7, 3, -1 ],
	vec![ 9, 7, 8, 5, 7, 9, -1 ],
	vec![ 8, 5, 4, 8, 10, 5, 8, 11, 10, -1 ],
	vec![ 0, 3, 11, 0, 11, 5, 11, 10, 5, 4, 0, 5, -1 ],
	vec![ 1, 0, 9, 8, 5, 4, 8, 10, 5, 8, 11, 10, -1 ],
	vec![ 10, 3, 11, 1, 3, 10, 9, 5, 4, -1 ],
	vec![ 3, 2, 8, 8, 2, 4, 4, 2, 10, 4, 10, 5, -1 ],
	vec![ 10, 5, 2, 5, 4, 2, 4, 0, 2, -1 ],
	vec![ 5, 4, 9, 8, 3, 0, 10, 1, 2, -1 ],
	vec![ 2, 10, 1, 4, 9, 5, -1 ],
	vec![ 8, 11, 4, 11, 2, 1, 4, 11, 1, 4, 1, 5, -1 ],
	vec![ 0, 5, 4, 1, 5, 0, 2, 3, 11, -1 ],
	vec![ 0, 11, 2, 8, 11, 0, 4, 9, 5, -1 ],
	vec![ 5, 4, 9, 2, 3, 11, -1 ],
	vec![ 4, 8, 5, 8, 3, 5, 3, 1, 5, -1 ],
	vec![ 0, 5, 4, 1, 5, 0, -1 ],
	vec![ 5, 4, 9, 3, 0, 8, -1 ],
	vec![ 5, 4, 9, -1 ],
	vec![ 11, 4, 7, 11, 9, 4, 11, 10, 9, -1 ],
	vec![ 0, 3, 8, 11, 4, 7, 11, 9, 4, 11, 10, 9, -1 ],
	vec![ 11, 10, 7, 10, 1, 0, 7, 10, 0, 7, 0, 4, -1 ],
	vec![ 3, 10, 1, 11, 10, 3, 7, 8, 4, -1 ],
	vec![ 3, 2, 10, 3, 10, 4, 10, 9, 4, 7, 3, 4, -1 ],
	vec![ 9, 2, 10, 0, 2, 9, 8, 4, 7, -1 ],
	vec![ 3, 4, 7, 0, 4, 3, 1, 2, 10, -1 ],
	vec![ 7, 8, 4, 10, 1, 2, -1 ],
	vec![ 7, 11, 4, 4, 11, 9, 9, 11, 2, 9, 2, 1, -1 ],
	vec![ 1, 9, 0, 4, 7, 8, 2, 3, 11, -1 ],
	vec![ 7, 11, 4, 11, 2, 4, 2, 0, 4, -1 ],
	vec![ 4, 7, 8, 2, 3, 11, -1 ],
	vec![ 9, 4, 1, 4, 7, 1, 7, 3, 1, -1 ],
	vec![ 7, 8, 4, 1, 9, 0, -1 ],
	vec![ 3, 4, 7, 0, 4, 3, -1 ],
	vec![ 7, 8, 4, -1 ],
	vec![ 11, 10, 8, 8, 10, 9, -1 ],
	vec![ 0, 3, 9, 3, 11, 9, 11, 10, 9, -1 ],
	vec![ 1, 0, 10, 0, 8, 10, 8, 11, 10, -1 ],
	vec![ 10, 3, 11, 1, 3, 10, -1 ],
	vec![ 3, 2, 8, 2, 10, 8, 10, 9, 8, -1 ],
	vec![ 9, 2, 10, 0, 2, 9, -1 ],
	vec![ 8, 3, 0, 10, 1, 2, -1 ],
	vec![ 2, 10, 1, -1 ],
	vec![ 2, 1, 11, 1, 9, 11, 9, 8, 11, -1 ],
	vec![ 11, 2, 3, 9, 0, 1, -1 ],
	vec![ 11, 0, 8, 2, 0, 11, -1 ],
	vec![ 3, 11, 2, -1 ],
	vec![ 1, 8, 3, 9, 8, 1, -1 ],
	vec![ 1, 9, 0, -1 ],
	vec![ 8, 3, 0, -1 ],
	vec![ -1 ],
]);

// Struct for a 3D point to avoid interpolation
#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
}

// Function to generate triangles for each cube configuration
fn generate_triangles(volume: &[u8]) -> Vec<[Point; 3]> {
    let mut triangles = Vec::new();
    
    // Iterate over each voxel in the 64x64x64 volume
    for x in 0..SIZE - 1 {
        for y in 0..SIZE - 1 {
            for z in 0..SIZE - 1 {
                let cube_index = get_cube_index(volume, x, y, z);
                
                // Check if any triangles are to be created for this cube configuration
                if EDGE_TABLE[cube_index] == 0 {
                    continue;
                }
                
                // Generate the triangles for the current cube based on its index in the triangle table
                for tri in TRIANGLE_TABLE[cube_index].chunks(3) {
                    if tri[0] == -1 {
                        break;
                    }
                    
                    // Add each triangle as a 3-Point tuple to the triangle vector
                    triangles.push([
                        get_vertex_position(tri[0] as usize, x, y, z),
                        get_vertex_position(tri[1] as usize, x, y, z),
                        get_vertex_position(tri[2] as usize, x, y, z),
                    ]);
                }
            }
        }
    }
    
    triangles
}

// Compute the cube index based on the volume data
fn get_cube_index(volume: &[u8], x: usize, y: usize, z: usize) -> usize {
    let mut cube_index = 0;
    // Loop through each of the 8 corner points of the current voxel
    for (i, &offset) in [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0),
                         (0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]
                         .iter()
                         .enumerate()
    {
        if volume[(x + offset.0) * SIZE * SIZE + (y + offset.1) * SIZE + (z + offset.2)] > 0 {
            cube_index |= 1 << i;
        }
    }
    cube_index
}

// Map edge index to 3D point position, simplified to eliminate interpolation
fn get_vertex_position(edge_index: usize, x: usize, y: usize, z: usize) -> Point {
    // Define edge positions based on a standard voxel layout
    let edges = [
        (0.0, 0.5, 0.0), // Edge 0: Between vertices (0,0,0) and (1,0,0)
        (1.0, 0.5, 0.0), // Edge 1: Between vertices (1,0,0) and (1,1,0)
        (1.0, 0.5, 1.0), // Edge 2: Between vertices (1,1,0) and (0,1,0)
        (0.0, 0.5, 1.0), // Edge 3: Between vertices (0,1,0) and (0,0,0)
        (0.5, 0.0, 0.0), // Edge 4: Between vertices (0,0,0) and (0,0,1)
        (0.5, 1.0, 0.0), // Edge 5: Between vertices (0,1,0) and (1,1,0)
        (0.5, 1.0, 1.0), // Edge 6: Between vertices (1,1,1) and (1,0,1)
        (0.5, 0.0, 1.0), // Edge 7: Between vertices (0,0,1) and (0,1,1)
        (0.0, 0.0, 0.5), // Edge 8: Between vertices (0,0,0) and (0,0,1)
        (1.0, 0.0, 0.5), // Edge 9: Between vertices (1,0,0) and (1,0,1)
        (1.0, 1.0, 0.5), // Edge 10: Between vertices (1,1,0) and (1,1,1)
        (0.0, 1.0, 0.5), // Edge 11: Between vertices (0,1,0) and (0,1,1)
    ];
    
    // Map the edge to its 3D point, offset by the voxel's position
    let (ex, ey, ez) = edges[edge_index];
    Point { 
        x: x as f32 + ex, 
        y: y as f32 + ey, 
        z: z as f32 + ez 
    }
}

fn main() {
    let volume = Arc::new(vec![0u8; SIZE_CUBED]); // Sample volume data wrapped in Arc
    let now = Instant::now();

    // Parallelize the generation of triangles across 8 threads
    let triangles = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..8).map(|_| {
        let volume = Arc::clone(&volume);
        let triangles = Arc::clone(&triangles);
        
        thread::spawn(move || {
            let local_triangles = generate_triangles(&volume);
            triangles.lock().unwrap().extend(local_triangles);
        })
    }).collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = now.elapsed();
    println!(
        "Generated {} triangles in {:?}ms",
        triangles.lock().unwrap().len(),
        elapsed.as_micros() as f64 / 1000.0
    );
}