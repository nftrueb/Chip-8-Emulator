#include <iostream> 
#include <SDL2/SDL.h>
#include "chip8.h"

#define SCREEN_WIDTH 640
#define SCREEN_HEIGHT 480

void draw_test(Chip8* cpu);

void handle_input(Chip8 *cpu, SDL_Event event) { 
    WORD key = 0x0000; 
    // _ _ _ _   _ _ _ _   _ _ _ _   _ _ _ _
    switch(event.key.keysym.sym) { 
        case SDLK_x: key = 0x0001; break;
        case SDLK_1: key = 0x0002; break;
        case SDLK_2: key = 0x0004; break;
        case SDLK_3: key = 0x0008; break;
        case SDLK_q: key = 0x0010; break;
        case SDLK_w: key = 0x0020; break;
        case SDLK_e: key = 0x0040; break;
        case SDLK_a: key = 0x0080; break;
        case SDLK_s: key = 0x0100; break;
        case SDLK_d: key = 0x0200; break;
        case SDLK_z: key = 0x0400; break;
        case SDLK_c: key = 0x0800; break;
        case SDLK_4: key = 0x1000; break;
        case SDLK_r: key = 0x2000; break;
        case SDLK_f: key = 0x4000; break;
        case SDLK_v: key = 0x8000; break;
    }

    if( event.type == SDL_KEYDOWN ) { 
        cpu->key_pressed(key);  
    } else if ( event.type == SDL_KEYUP ) { 
        cpu->key_released(key); 
    }

    printf("Keys Value: %04X\n", cpu->keys); 
}

void draw_test(Chip8* cpu) { 
    cpu->registers[1] = 9;  
    cpu->registers[2] = 5; 
    cpu->registers[3] = 10;   
    //FX29 
    //DXYN
    WORD opcode = 0xF << 12 | 0x1 << 8 | 0x2 << 4 | 0x9;
    cpu->memory[300] = (opcode & 0xFF00) >> 8; 
    cpu->memory[301] = (opcode & 0x00FF); 

    opcode = 0xD << 12 | 0x2 << 8 | 0x3 << 4 | 0x5; 
    cpu->memory[302] = (opcode & 0xFF00) >> 8; 
    cpu->memory[303] = (opcode & 0x00FF); 

    cpu->PC = 300; 
    cpu->decode_next_op(); 
    cpu->decode_next_op(); 
}

void render(Chip8* cpu, SDL_Renderer* renderer, int scale) { 
    int size = SCREEN_W*(SCREEN_H); 
    SDL_Rect cells[size];

    for(int i = 0; i < size; i++) {
        cells[i].x = (i % SCREEN_W)*scale;  
        cells[i].y = (int)(i / SCREEN_W)*scale; 
        cells[i].w = scale; 
        cells[i].h = scale; 

        if (cpu->screen[i] & 0x1) { 
            SDL_SetRenderDrawColor(renderer, 255,255,255, 255);
        } else { 
            SDL_SetRenderDrawColor(renderer, 0,0,0, 255);
        }
         
        SDL_RenderFillRect(renderer, &cells[i]);

        if(false) { 
            SDL_SetRenderDrawColor(renderer, 100,100,100, 255); 
            SDL_RenderDrawRect(renderer, &cells[i]); 
        } 
    }
}

void handle_timing(Chip8 *cpu) {
    if(cpu->delay_timer > 0) { 
        cpu->delay_timer -= 1; 
    } 

    if(cpu->sound_timer > 0) { 
        cpu->delay_timer -= 1; 
    }
}

int main(int argc, char* args[]) {
    SDL_Window* window = NULL;
    SDL_Surface* screenSurface = NULL;

    Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/out.c8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/Maze [David Winter, 199x].ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/Chip8 emulator Logo [Garstyciuks].ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/Keypad Test [Hap, 2006].ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/IBM Logo.ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8-roms/demos/Zero Demo [zeroZshadow, 2007].ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/Stars [Sergey Naydenov, 2010].ch8");
    // Chip8 cpu = Chip8((char *)"/Users/nicktrueb/Programming/c/chip8/roms/Life [GV Samways, 1980].ch8");
    cpu.reset(); 

    int scale = 10; 

    //init SDL2
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        fprintf(stderr, "could not initialize sdl2: %s\n", SDL_GetError());
        return 1;
    }

    //create window
    window = SDL_CreateWindow(
                    "Chip-8 Emulator",
                    SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                    SCREEN_W*scale, SCREEN_H*scale,
                    SDL_WINDOW_SHOWN );

    //check if window was successfully created
    if (window == NULL) {
        fprintf(stderr, "could not create window: %s\n", SDL_GetError());
        return 1;
    }

    //create renderer for hardware rendered graphics
    SDL_Renderer* renderer = SDL_CreateRenderer(window, -1, 0); 

    // A basic main loop to prevent blocking
    bool running = true;
    SDL_Event event;
    while (running) {
        while (SDL_PollEvent(&event)) {
            if (event.type == SDL_QUIT) {
                running = false;
            } else if (event.type == SDL_KEYDOWN || event.type == SDL_KEYUP) { 
                if(event.key.keysym.sym == SDLK_ESCAPE) { 
                    running = false; 
                    continue;
                }
                handle_input(&cpu, event); 
            } 
        }

        //clear rendered texture
        // SDL_Delay(16);
        SDL_RenderClear(renderer); 

        //set background color
        SDL_SetRenderDrawColor(renderer, 255, 255, 255, SDL_ALPHA_OPAQUE);
        SDL_RenderFillRect(renderer, NULL);  

        //render screen with data from cpu screen
        cpu.decode_next_op(); 
        render(&cpu, renderer, scale); 
        handle_timing(&cpu); 

        if(false) { 
            std::cin.get(); 
        }

        SDL_RenderPresent(renderer); 
    }

    //destroy window and exit
    SDL_DestroyWindow(window);
    SDL_DestroyRenderer(renderer); 
    SDL_Quit();
    return 0;
}