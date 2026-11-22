#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "world.h"
#define RW 320
#define RH 180
#define PI 3.14159265358979323846
static uint32_t pixels[RW*RH];
static HWND window;
static int running=1, locked=0, selected=1, dirty=0, grounded=0;
static Vec3 player={32.5,0,32.5}; /* y is the player's feet */
static double yaw=.45, pitch=-.10, vertical_velocity=0;
static char save_path[MAX_PATH];
static const char *names[BLOCK_COUNT]={"Air","Grass","Dirt","Stone","Wood","Leaves","Planks","Brick","Sand","Glass"};
static const unsigned colors[BLOCK_COUNT]={0,0x6cb64d,0x87563a,0x858b8f,0x8e6135,0x397e3c,0xc49a62,0xad5e50,0xd9c481,0x9bdbe5};
static int key(int k){return (GetAsyncKeyState(k)&0x8000)!=0;}
static double clampd(double x,double a,double b){return x<a?a:x>b?b:x;}
static void save_world(void){
    if(!save_path[0])return;
    char tmp[MAX_PATH]; if(snprintf(tmp,sizeof(tmp),"%s.tmp",save_path)>=(int)sizeof(tmp))return;
    FILE *f=fopen(tmp,"wb"); if(!f)return;
    const char magic[8]={'S','I','G','M','A','0','0','1'};
    int ok=fwrite(magic,1,8,f)==8 && fwrite(world,1,WORLD_SIZE,f)==WORLD_SIZE;
    if(fclose(f)!=0)ok=0;
    if(ok && MoveFileExA(tmp,save_path,MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH)) dirty=0;
    else DeleteFileA(tmp);
}
static void load_world(void){
    char folder[MAX_PATH]; DWORD n=GetEnvironmentVariableA("APPDATA",folder,MAX_PATH);
    if(!n || n>=MAX_PATH-32){generate_world();return;}
    strcat(folder,"\\SigmaCraft"); CreateDirectoryA(folder,NULL);
    if(snprintf(save_path,sizeof(save_path),"%s\\world.dat",folder)>=(int)sizeof(save_path)){save_path[0]=0;generate_world();return;}
    FILE *f=fopen(save_path,"rb");char magic[8];int valid=0;
    if(f){valid=fread(magic,1,8,f)==8 && memcmp(magic,"SIGMA001",8)==0 && fread(world,1,WORLD_SIZE,f)==WORLD_SIZE && fgetc(f)==EOF; fclose(f);}
    if(valid){for(int i=0;i<WORLD_SIZE;i++)if(world[i]>=BLOCK_COUNT){valid=0;break;}}
    if(!valid){
        if(f){
            MessageBoxA(NULL,"Your saved world could not be read. Saving is disabled to protect it.","Sigma Craft",MB_OK|MB_ICONWARNING);
            save_path[0]=0;
        }
        generate_world();dirty=1;
    }
}
static void lock_mouse(int value){
    if(value==locked)return;locked=value;
    if(locked){SetCapture(window);ShowCursor(FALSE);}
    else {ReleaseCapture();ShowCursor(TRUE);}
}
static Vec3 camera_dir(void){double c=cos(pitch);return (Vec3){cos(yaw)*c,sin(pitch),sin(yaw)*c};}
static void interact(int place){
    Vec3 origin={player.x,player.y+1.62,player.z}; Hit h=cast_ray(origin,camera_dir(),7.0);
    if(!h.hit)return;
    if(!place){set_block(h.x,h.y,h.z,AIR);dirty=1;return;}
    if(h.px==h.x&&h.py==h.y&&h.pz==h.z)return;
    if(h.px<0||h.px>=WORLD_X||h.py<0||h.py>=WORLD_Y||h.pz<0||h.pz>=WORLD_Z)return;
    if(block_at(h.px,h.py,h.pz))return;
    set_block(h.px,h.py,h.pz,selected);
    if(player_collides(player.x,player.y,player.z))set_block(h.px,h.py,h.pz,AIR);
    else dirty=1;
}
static LRESULT CALLBACK procedure(HWND h,UINT msg,WPARAM wp,LPARAM lp){
    switch(msg){
    case WM_CLOSE: save_world(); DestroyWindow(h);return 0;
    case WM_DESTROY: running=0;PostQuitMessage(0);return 0;
    case WM_KILLFOCUS:lock_mouse(0);return 0;
    case WM_KEYDOWN:
        if(wp==VK_ESCAPE)lock_mouse(0);
        else if(wp>='1'&&wp<='9')selected=(int)(wp-'0');
        else if(wp==VK_F5)save_world();
        return 0;
    case WM_LBUTTONDOWN:
        if(!locked)lock_mouse(1);else interact(0);return 0;
    case WM_RBUTTONDOWN:
        if(locked)interact(1);return 0;
    case WM_MOUSEWHEEL:
        if(locked){int delta=GET_WHEEL_DELTA_WPARAM(wp);selected+=delta>0?-1:1;if(selected<1)selected=9;if(selected>9)selected=1;}return 0;
    }
    return DefWindowProcA(h,msg,wp,lp);
}
static void update_mouse(void){
    if(!locked)return;
    RECT r; POINT center, cursor; GetClientRect(window,&r);
    center.x=(r.right-r.left)/2;center.y=(r.bottom-r.top)/2;ClientToScreen(window,&center);
    GetCursorPos(&cursor);
    yaw+=(cursor.x-center.x)*.0025;pitch-=(cursor.y-center.y)*.0025;
    pitch=clampd(pitch,-1.5,1.5);SetCursorPos(center.x,center.y);
}
static void move_axis(double dx,double dz){
    if(!player_collides(player.x+dx,player.y,player.z))player.x+=dx;
    if(!player_collides(player.x,player.y,player.z+dz))player.z+=dz;
}
static void tick(double dt){
    if(!locked)return;
    update_mouse();
    double forward=(key('W')?1:0)-(key('S')?1:0), side=(key('D')?1:0)-(key('A')?1:0);
    double length=sqrt(forward*forward+side*side);if(length>0){forward/=length;side/=length;}
    double speed=key(VK_CONTROL)?7.0:4.3;
    double dx=(cos(yaw)*forward-sin(yaw)*side)*speed*dt, dz=(sin(yaw)*forward+cos(yaw)*side)*speed*dt;
    move_axis(dx,dz);
    grounded=player_collides(player.x,player.y-.035,player.z);
    if(grounded && key(VK_SPACE) && vertical_velocity<=0) vertical_velocity=7.2;
    vertical_velocity-=20.0*dt;
    double step=vertical_velocity*dt;
    if(!player_collides(player.x,player.y+step,player.z))player.y+=step;
    else {
        /* Sweep to the obstacle so a large frame cannot leave the player hovering. */
        double lo=0,hi=1;
        for(int i=0;i<16;i++){
            double mid=(lo+hi)*.5;
            if(player_collides(player.x,player.y+step*mid,player.z))hi=mid;
            else lo=mid;
        }
        player.y+=step*lo;
        if(vertical_velocity<0)grounded=1;
        vertical_velocity=0;
    }
    if(player.y>WORLD_Y+10){player.x=32.5;player.z=32.5;player.y=terrain_height(32,32)+1.01;vertical_velocity=0;}
}
static unsigned hash3(int a,int b,int c){unsigned h=(unsigned)a*73856093u^(unsigned)b*19349663u^(unsigned)c*83492791u;h^=h>>13;h*=1274126177u;return h;}
static unsigned shade(unsigned col,double f){int r=(int)(((col>>16)&255)*f),g=(int)(((col>>8)&255)*f),b=(int)((col&255)*f);if(r>255)r=255;if(g>255)g=255;if(b>255)b=255;return (unsigned)((r<<16)|(g<<8)|b);}
static unsigned mix(unsigned a,unsigned b,double t){int ar=(a>>16)&255,ag=(a>>8)&255,ab=a&255,br=(b>>16)&255,bg=(b>>8)&255,bb=b&255;return (unsigned)(((int)(ar+(br-ar)*t)<<16)|((int)(ag+(bg-ag)*t)<<8)|(int)(ab+(bb-ab)*t));}
static void render(void){
    Vec3 eye={player.x,player.y+1.62,player.z}, f=camera_dir();
    Vec3 right={-sin(yaw),0,cos(yaw)};
    Vec3 up={-cos(yaw)*sin(pitch),cos(pitch),-sin(yaw)*sin(pitch)};
    for(int y=0;y<RH;y++){
        double v=(1.0-2.0*(y+.5)/RH)*.57735;
        for(int x=0;x<RW;x++){
            double u=(2.0*(x+.5)/RW-1.0)*1.0264;
            Vec3 d={f.x+right.x*u+up.x*v,f.y+up.y*v,f.z+right.z*u+up.z*v};
            Hit h=cast_ray(eye,d,48);
            double sky_t=clampd(.5+d.y*.45,0,1);unsigned sky=mix(0xb7daf5,0x5795d4,sky_t);
            if(!h.hit){pixels[y*RW+x]=sky;continue;}
            unsigned col=colors[h.block];
            if(h.block==GRASS && h.face!=1)col=0x826043;
            double inv=1.0/sqrt(d.x*d.x+d.y*d.y+d.z*d.z);
            int tx=(int)floor((eye.x+d.x*h.distance*inv)*12.0);
            int ty=(int)floor((eye.y+d.y*h.distance*inv)*12.0);
            int tz=(int)floor((eye.z+d.z*h.distance*inv)*12.0);
            unsigned noise=hash3(tx,ty,tz);double grain=.90+(noise%18)*.01;
            double light=h.face==1?(d.y<0?1.08:.58):(h.face==0?.80:.91);
            unsigned shaded=shade(col,grain*light);
            double fog=clampd((h.distance-25.0)/25.0,0,.72);
            pixels[y*RW+x]=mix(shaded,sky,fog);
        }
    }
}
static void text(HDC dc,int x,int y,const char *s,COLORREF color){SetTextColor(dc,RGB(0,0,0));TextOutA(dc,x+2,y+2,s,(int)strlen(s));SetTextColor(dc,color);TextOutA(dc,x,y,s,(int)strlen(s));}
static void present(void){
    RECT rc;GetClientRect(window,&rc);int w=rc.right,h=rc.bottom;if(w<1||h<1)return;
    HDC dc=GetDC(window);BITMAPINFO bmi;ZeroMemory(&bmi,sizeof(bmi));
    bmi.bmiHeader.biSize=sizeof(BITMAPINFOHEADER);bmi.bmiHeader.biWidth=RW;bmi.bmiHeader.biHeight=-RH;bmi.bmiHeader.biPlanes=1;bmi.bmiHeader.biBitCount=32;bmi.bmiHeader.biCompression=BI_RGB;
    SetStretchBltMode(dc,COLORONCOLOR);StretchDIBits(dc,0,0,w,h,0,0,RW,RH,pixels,&bmi,DIB_RGB_COLORS,SRCCOPY);
    SetBkMode(dc,TRANSPARENT);
    HFONT font=CreateFontA(18,0,0,0,FW_BOLD,FALSE,FALSE,FALSE,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,ANTIALIASED_QUALITY,DEFAULT_PITCH,"Segoe UI");HFONT old=(HFONT)SelectObject(dc,font);
    text(dc,16,12,"SIGMA CRAFT  |  by SIGMA BANKU",RGB(255,255,255));
    int cx=w/2,cy=h/2; HPEN pen=CreatePen(PS_SOLID,2,RGB(255,255,255));HPEN oldpen=(HPEN)SelectObject(dc,pen);
    MoveToEx(dc,cx-8,cy,NULL);LineTo(dc,cx+9,cy);MoveToEx(dc,cx,cy-8,NULL);LineTo(dc,cx,cy+9);SelectObject(dc,oldpen);DeleteObject(pen);
    int slot=w/30;if(slot>48)slot=48;if(slot<22)slot=22;int total=slot*9,start=(w-total)/2,top=h-slot-20;
    for(int i=1;i<=9;i++){
        RECT box={start+(i-1)*slot+2,top+2,start+i*slot-2,top+slot-2};
        unsigned c=colors[i];HBRUSH brush=CreateSolidBrush(RGB((c>>16)&255,(c>>8)&255,c&255));FillRect(dc,&box,brush);DeleteObject(brush);
        HBRUSH border=CreateSolidBrush(i==selected?RGB(255,255,255):RGB(55,55,55));FrameRect(dc,&box,border);DeleteObject(border);
        char num[2]={(char)('0'+i),0};text(dc,box.left+3,box.top+2,num,RGB(255,255,255));
    }
    char label[80];snprintf(label,sizeof(label),"%s  |  1-9 / mouse wheel",names[selected]);text(dc,16,h-36,label,RGB(255,255,255));
    if(!locked){
        RECT panel={w/2-260,h/2-105,w/2+260,h/2+92};HBRUSH bg=CreateSolidBrush(RGB(25,20,35));FillRect(dc,&panel,bg);DeleteObject(bg);
        text(dc,panel.left+18,panel.top+12,"SIGMA CRAFT - click to play",RGB(230,205,255));
        text(dc,panel.left+18,panel.top+42,"W A S D: move     Mouse: look     Space: jump",RGB(255,255,255));
        text(dc,panel.left+18,panel.top+68,"Left click: break     Right click: place",RGB(255,255,255));
        text(dc,panel.left+18,panel.top+94,"1-9 / wheel: choose block     Ctrl: run",RGB(255,255,255));
        text(dc,panel.left+18,panel.top+120,"Esc: pause / release mouse     F5: save",RGB(255,255,255));
        text(dc,panel.left+18,panel.top+150,"Free creative sandbox - no Minecraft install needed",RGB(199,190,215));
    }
    SelectObject(dc,old);DeleteObject(font);ReleaseDC(window,dc);
}
int WINAPI WinMain(HINSTANCE instance,HINSTANCE previous,LPSTR command,int show){
    (void)previous;(void)command;(void)show;
    WNDCLASSA wc;ZeroMemory(&wc,sizeof(wc));wc.lpfnWndProc=procedure;wc.hInstance=instance;wc.lpszClassName="SigmaCraftWindow";wc.hCursor=LoadCursor(NULL,IDC_ARROW);
    if(!RegisterClassA(&wc))return 1;
    window=CreateWindowExA(0,wc.lpszClassName,"Sigma Craft - by SIGMA BANKU",WS_OVERLAPPEDWINDOW,CW_USEDEFAULT,CW_USEDEFAULT,1280,720,NULL,NULL,instance,NULL);
    if(!window)return 1;load_world();player.y=terrain_height(32,32)+1.01;
    while(player_collides(player.x,player.y,player.z) && player.y<WORLD_Y+2)player.y+=1;
    ShowWindow(window,SW_SHOW);UpdateWindow(window);
    LARGE_INTEGER freq,last,now;QueryPerformanceFrequency(&freq);QueryPerformanceCounter(&last);double save_timer=0;
    while(running){
        MSG msg;while(PeekMessageA(&msg,NULL,0,0,PM_REMOVE)){if(msg.message==WM_QUIT){running=0;break;}TranslateMessage(&msg);DispatchMessageA(&msg);}if(!running)break;
        QueryPerformanceCounter(&now);double dt=(double)(now.QuadPart-last.QuadPart)/(double)freq.QuadPart;last=now;dt=clampd(dt,0,.05);
        tick(dt);render();present();save_timer+=dt;if(save_timer>5){if(dirty)save_world();save_timer=0;}Sleep(1);
    }
    lock_mouse(0);return 0;
}
