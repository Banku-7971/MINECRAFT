#include "world.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
int main(void) {
 generate_world();
 int h=terrain_height(32,32);
 assert(block_at(32,h,32)==GRASS); assert(!player_collides(32.5,h+1.01,32.5));
 assert(player_collides(32.5,h-.5,32.5));
 memset(world,0,sizeof(world)); set_block(3,4,5,STONE);
 Hit hit=cast_ray((Vec3){.5,4.5,5.5},(Vec3){1,0,0},10);
 assert(hit.hit && hit.x==3 && hit.px==2 && hit.face==0 && fabs(hit.distance-2.5)<.00001);
 assert(!cast_ray((Vec3){.5,4.5,5.5},(Vec3){1,0,0},2).hit);
 hit=cast_ray((Vec3){3.5,7.5,5.5},(Vec3){0,-1,0},10);
 assert(hit.hit && hit.y==4 && hit.py==5 && hit.face==1);
 assert(!cast_ray((Vec3){0,0,0},(Vec3){0,0,0},10).hit);
 puts("World, collisions and raycasting passed.");return 0;
}
