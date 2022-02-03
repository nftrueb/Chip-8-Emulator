#include <iostream> 
#include <SDL2/SDL.h>
#include "chip8.h"

#define SCREEN_WIDTH 640
#define SCREEN_HEIGHT 480

void handle_input(Chip8 *cpu, SDL_Event event) { 
    WORD key = 0x0000; 
    // _ _ _ _   _ _ _ _   _ _ _ _   _ _ _ _
    switch(event.key.keysym.sym) { 
        case SDLK_0: key = 0x0001; break;
        case SDLK_1: key = 0x0002; break;
        case SDLK_2: key = 0x0004; break;
        case SDLK_3: key = 0x0008; break;
        case SDLK_4: key = 0x0010; break;
        case SDLK_5: key = 0x0020; break;
        case SDLK_6: key = 0x0040; break;
        case SDLK_7: key = 0x0080; break;
        case SDLK_8: key = 0x0100; break;
        case SDLK_9: key = 0x0200; break;
        case SDLK_a: key = 0x0400; break;
        case SDLK_b: key = 0x0800; break;
        case SDLK_c: key = 0x1000; break;
        case SDLK_d: key = 0x2000; break;
        case SDLK_e: key = 0x4000; break;
        case SDLK_f: key = 0x8000; break;
    }

    if( event.type == SDL_KEYDOWN ) { 
        cpu->key_pressed(key);  
    } else if ( event.type == SDL_KEYUP ) { 
        cpu->key_released(key); 
    }

    printf("Keys Value: %04X\n", cpu->keys); 
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
            SDL_SetRenderDrawColor(renderer, 100,100,100, 255);
        }
         
        SDL_RenderFillRect(renderer, &cells[i]);
        SDL_SetRenderDrawColor(renderer, 0,0,0, 255); 
        SDL_RenderDrawRect(renderer, &cells[i]);  
    }
}

int main(int argc, char* args[]) {
    SDL_Window* window = NULL;
    SDL_Surface* screenSurface = NULL;
    Chip8 cpu = Chip8(); 
    int scale = 10; 

    //init SDL2
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        fprintf(stderr, "could not initialize sdl2: %s\n", SDL_GetError());
        return 1;
    }
    //std::cout << cpu.scale << std::endl;
    //create window
    window = SDL_CreateWindow(
                    "Chip-8 Emulator",
                    SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                    SCREEN_W*scale, SCREEN_H*scale,
                    SDL_WINDOW_SHOWN
                    );

    //check if window was successfully created
    if (window == NULL) {
        fprintf(stderr, "could not create window: %s\n", SDL_GetError());
        return 1;
    }

    //update screen and window
    SDL_Renderer* renderer = SDL_CreateRenderer(window, -1, 0); 
    //screen = SDL_GetWindowSurface(window);
    // SDL_FillRect(screenSurface, NULL, SDL_MapRGB(screenSurface->format, 0xFF, 0x00, 0x00));
    // SDL_UpdateWindowSurface(window);

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
        SDL_Delay(16);
        SDL_RenderClear(renderer); 

        SDL_SetRenderDrawColor(renderer, 255, 255, 255, SDL_ALPHA_OPAQUE);
        SDL_RenderFillRect(renderer, NULL);  

        // SDL_SetRenderDrawColor(renderer, 0,0,0, SDL_ALPHA_OPAQUE);
        // SDL_RenderDrawPoint(renderer,(int) SCREEN_W*scale / 2, (int) SCREEN_H*scale / 2); 

        render(&cpu, renderer, scale); 
        SDL_RenderPresent(renderer); 
    }

    //destroy window and exit
    SDL_DestroyWindow(window);
    SDL_DestroyRenderer(renderer); 
    SDL_Quit();
    return 0;
}