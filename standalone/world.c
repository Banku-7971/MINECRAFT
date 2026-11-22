#include "world.h"
#include <math.h>
#include <string.h>
unsigned char world[WORLD_SIZE];
static int in_bounds(int x, int y, int z) { return x>=0 && x<WORLD_X && y>=0 && y<WORLD_Y && z>=0 && z<WORLD_Z; }
static int idx(int x, int y, int z) { return (y * WORLD_Z + z) * WORLD_X + x; }
int block_at(int x, int y, int z) { return in_bounds(x,y,z) ? world[idx(x,y,z)] : AIR; }
void set_block(int x,int y,int z,int b) { if(in_bounds(x,y,z) && b>=0 && b<BLOCK_COUNT) world[idx(x,y,z)]=(unsigned char)b; }
static unsigned hash(int x,int z) { unsigned h=(unsigned)x*0x1f123bb5u ^ (unsigned)z*0x5f356495u ^ 0xa73c91ebu; h^=h>>16;h*=0x7feb352du;h^=h>>15;return h; }
int terrain_height(int x,int z) {
    int h=(int)floor(8.0 + 1.8*sin(x*0.12) + 1.5*cos(z*0.15) + 1.3*sin((x+z)*0.085));
    if(h<4)h=4;
    if(h>14)h=14;
    return h;
}
void generate_world(void) {
    memset(world,0,sizeof(world));
    for(int z=0;z<WORLD_Z;z++) for(int x=0;x<WORLD_X;x++) {
        int h=terrain_height(x,z);
        for(int y=0;y<=h;y++) set_block(x,y,z,y==h?GRASS:(y>=h-2?DIRT:STONE));
    }
    for(int z=3;z<WORLD_Z-3;z++) for(int x=3;x<WORLD_X-3;x++) {
        if((x-32)*(x-32)+(z-32)*(z-32)<49 || hash(x,z)%115 != 0) continue;
        int h=terrain_height(x,z); int trunk=3+(int)(hash(z,x)%2);
        for(int y=1;y<=trunk;y++)set_block(x,h+y,z,WOOD);
        for(int y=h+trunk-1;y<=h+trunk+2;y++) for(int dz=-2;dz<=2;dz++) for(int dx=-2;dx<=2;dx++) {
            int a=dx<0?-dx:dx,b=dz<0?-dz:dz;
            if((y==h+trunk+2 && (a>1||b>1)) || (a==2 && b==2)) continue;
            if(!block_at(x+dx,y,z+dz))set_block(x+dx,y,z+dz,LEAVES);
        }
    }
}
Hit cast_ray(Vec3 o, Vec3 d, double limit) {
    Hit result={0};
    double length=sqrt(d.x*d.x+d.y*d.y+d.z*d.z);
    if(length<1e-9 || limit<=0) return result;
    d.x/=length;d.y/=length;d.z/=length;
    int x=(int)floor(o.x),y=(int)floor(o.y),z=(int)floor(o.z);
    int sx=d.x<0?-1:1,sy=d.y<0?-1:1,sz=d.z<0?-1:1;
    double tx=fabs(d.x)<1e-12?1e30:((d.x<0?o.x-x:x+1-o.x)/fabs(d.x));
    double ty=fabs(d.y)<1e-12?1e30:((d.y<0?o.y-y:y+1-o.y)/fabs(d.y));
    double tz=fabs(d.z)<1e-12?1e30:((d.z<0?o.z-z:z+1-o.z)/fabs(d.z));
    double ax=fabs(d.x)<1e-12?1e30:1/fabs(d.x),ay=fabs(d.y)<1e-12?1e30:1/fabs(d.y),az=fabs(d.z)<1e-12?1e30:1/fabs(d.z);
    int px=x,py=y,pz=z,face=1;double t=0;
    for(int i=0;i<256 && t<=limit;i++) {
        int b=block_at(x,y,z);
        if(b) { result=(Hit){1,x,y,z,px,py,pz,face,t,b};return result; }
        px=x;py=y;pz=z;
        if(tx<ty && tx<tz) {x+=sx;t=tx;tx+=ax;face=0;}
        else if(ty<tz) {y+=sy;t=ty;ty+=ay;face=1;}
        else {z+=sz;t=tz;tz+=az;face=2;}
    }
    return result;
}
int player_collides(double x,double y,double z) {
    const double r=.28, h=1.78;
    int x0=(int)floor(x-r),x1=(int)floor(x+r),y0=(int)floor(y),y1=(int)floor(y+h-.001),z0=(int)floor(z-r),z1=(int)floor(z+r);
    for(int yy=y0;yy<=y1;yy++)for(int zz=z0;zz<=z1;zz++)for(int xx=x0;xx<=x1;xx++) {
        if(xx<0||xx>=WORLD_X||zz<0||zz>=WORLD_Z||yy<0 || block_at(xx,yy,zz))return 1;
    }
    return 0;
}
