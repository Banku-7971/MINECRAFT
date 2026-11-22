#ifndef SIGMA_WORLD_H
#define SIGMA_WORLD_H
#include <stdint.h>
#define WORLD_X 64
#define WORLD_Y 32
#define WORLD_Z 64
#define WORLD_SIZE (WORLD_X * WORLD_Y * WORLD_Z)
enum { AIR, GRASS, DIRT, STONE, WOOD, LEAVES, PLANKS, BRICK, SAND, GLASS, BLOCK_COUNT };
typedef struct { double x, y, z; } Vec3;
typedef struct { int hit, x, y, z, px, py, pz, face; double distance; int block; } Hit;
extern unsigned char world[WORLD_SIZE];
int block_at(int x, int y, int z);
void set_block(int x, int y, int z, int block);
int terrain_height(int x, int z);
void generate_world(void);
Hit cast_ray(Vec3 origin, Vec3 direction, double max_distance);
int player_collides(double x, double y, double z);
#endif
