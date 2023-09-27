#include "chip8.h" 
#include <iostream>
#include <fstream> 

using std::vector; 

Chip8::Chip8(char *rom) {
    rom_ = rom; 
    reset(); 
}

int Chip8::load_rom() { 

    FILE *input; 
    input = fopen(rom_, "rb"); 

    if(input) { 
        fseek(input, 0, SEEK_END); 
        int file_len = ftell(input); 
        rewind(input); 

        int result = fread(memory+PROG_START, 1, file_len, input); 
        if(result != file_len) { 
            std:: cout << "Failed to read full ROM" << std::endl; 
            fclose(input); 
            return 1; 
        }
    } else { 
        std::cout << "Failed opening ROM" << std::endl; 
        return 1; 
    }   

    fclose(input); 
    return 0; 
}

void Chip8::reset() { 
    PC = PROG_START; 
    I = 0X0000; 
    delay_timer = 0; 
    sound_timer = 0; 
    keys = 0x00; 
    stack.empty(); 
    clear_registers(); 
    clear_screen(); 
    clear_memory(); 
    init_font(); 
    load_rom(); 

    // int x = (int)4;
    // int y = (int)4; 
    // screen[y * SCREEN_W + x] = 0x01; 
}

void Chip8::init_font() { 
    int idx = -1;
    // 0 
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90;
    memory[++idx] = 0x90; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0x60; 

    // 1 
    memory[++idx] = 0x20; 
    memory[++idx] = 0x60;
    memory[++idx] = 0x20; 
    memory[++idx] = 0x20; 
    memory[++idx] = 0x70; 

    // 2
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90;
    memory[++idx] = 0x20; 
    memory[++idx] = 0x40; 
    memory[++idx] = 0xF0; 

    // 3
    memory[++idx] = 0x60; 
    memory[++idx] = 0x10;
    memory[++idx] = 0x70; 
    memory[++idx] = 0x10; 
    memory[++idx] = 0x60; 

    // 4
    memory[++idx] = 0x90; 
    memory[++idx] = 0x90;
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x10; 
    memory[++idx] = 0x10; 

    // 5
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x80;
    memory[++idx] = 0xE0; 
    memory[++idx] = 0x10; 
    memory[++idx] = 0xE0;

    // 6
    memory[++idx] = 0x60; 
    memory[++idx] = 0x80;
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0x60;

    // 7
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x10;
    memory[++idx] = 0x10; 
    memory[++idx] = 0x10; 
    memory[++idx] = 0x10;


    // 8
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90;
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0x60;

    // 9
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90;
    memory[++idx] = 0x70; 
    memory[++idx] = 0x10; 
    memory[++idx] = 0x10;

    // A
    memory[++idx] = 0x60; 
    memory[++idx] = 0x90;
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0x90;

    // B
    memory[++idx] = 0x80; 
    memory[++idx] = 0x80;
    memory[++idx] = 0xE0; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0xE0;

    // C
    memory[++idx] = 0x70; 
    memory[++idx] = 0x80;
    memory[++idx] = 0x80; 
    memory[++idx] = 0x80; 
    memory[++idx] = 0x70;

    // D
    memory[++idx] = 0xE0; 
    memory[++idx] = 0x90;
    memory[++idx] = 0x90; 
    memory[++idx] = 0x90; 
    memory[++idx] = 0xE0;

    // E
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x80;
    memory[++idx] = 0xE0; 
    memory[++idx] = 0x80; 
    memory[++idx] = 0xF0;

    // F
    memory[++idx] = 0xF0; 
    memory[++idx] = 0x80;
    memory[++idx] = 0xE0; 
    memory[++idx] = 0x80; 
    memory[++idx] = 0x80;
}

void Chip8::clear_screen() { 
    for( int i = 0; i < SCREEN_H*SCREEN_W; i++ ) {
        screen[i] = 0x00; 
    }
}

void Chip8::clear_memory() { 
    for( WORD i = 0; i < MEM_SIZE; i++) {
        memory[i] = 0x00; 
    }
}

void Chip8::clear_registers() { 
    for( BYTE i = 0; i < REG_NUM; i++ ) { 
        registers[i] = 0x00; 
    }   
}

void Chip8::key_pressed(WORD key) { 
    keys |= key;  
}

void Chip8::key_released(WORD key) { 
    keys -= key; 
}

void Chip8::decode_next_op() { 
    WORD opcode = (memory[PC] << 8); 
    opcode |= memory[PC+1];
    BYTE op = (opcode >> 12) & 0xF; 

    if(false) {
        printf("Opcode: 0x%04x\n", opcode); 
        printf("PC: %04x\n", PC); 
        printf("I: %04x\n", I); 

        for(int i = 0; i < 10; i++) { 
            printf("V%d: %04x\n", i, registers[i]); 
        }


        if (stack.size() > 0) { 
            printf("Return Stack Size: %lu", stack.size()); 
            for(int i = 0; i < stack.size(); i++) {
                printf("    %04x", stack[i]);
            }
        }
    }

    switch(op) { 
        case 0x1: opcode_1NNN(opcode); break; 
        case 0x2: opcode_2NNN(opcode); break; 
        case 0x3: opcode_3XNN(opcode); break; 
        case 0x4: opcode_4XNN(opcode); break; 
        case 0x5: opcode_5XY0(opcode); break; 
        case 0x6: opcode_6XNN(opcode); break; 
        case 0x7: opcode_7XNN(opcode); break; 
        case 0x9: opcode_9XY0(opcode); break; 
        case 0xA: opcode_ANNN(opcode); break; 
        case 0xB: opcode_BNNN(opcode); break; 
        case 0xC: opcode_CXNN(opcode); break; 
        case 0xD: opcode_DXYN(opcode); break; 

        case 0x0: decode_0(opcode); break; 
        case 0x8: decode_8(opcode); break; 
        case 0xE: decode_E(opcode); break; 
        case 0xF: decode_F(opcode); break; 
        default: break;
    }

    //advance PC and keep at 12-bits for valid memory access 
    PC += 2; 
    PC &= 0xFFF; 
}

void Chip8::decode_0(WORD opcode) { 
    switch(opcode) { 
        case 0x00E0: opcode_00E0(opcode); break; 
        case 0X00EE: opcode_00EE(opcode); break; 
        default: opcode_0NNN(opcode); break; 
    }
}

void Chip8::decode_8(WORD opcode) { 
    switch(opcode & 0xF) {
        case 0x0: opcode_8XY0(opcode); break; 
        case 0x1: opcode_8XY1(opcode); break; 
        case 0x2: opcode_8XY2(opcode); break; 
        case 0x3: opcode_8XY3(opcode); break;  
        case 0x4: opcode_8XY4(opcode); break;  
        case 0x5: opcode_8XY5(opcode); break;  
        case 0x6: opcode_8XY6(opcode); break;  
        case 0x7: opcode_8XY7(opcode); break;  
        case 0xE: opcode_8XYE(opcode); break;  
        default: break; 
    }
}

void Chip8::decode_E(WORD opcode) { 
    switch(opcode & 0xFF) { 
        case 0x9E: opcode_EX9E(opcode); break;
        case 0xA1: opcode_EXA1(opcode); break;
        default: break; 
    }
}

void Chip8::decode_F(WORD opcode) { 
    switch(opcode & 0xFF) { 
        case 0x07: opcode_FX07(opcode); break;
        case 0x0A: opcode_FX0A(opcode); break;
        case 0x15: opcode_FX15(opcode); break;
        case 0x18: opcode_FX18(opcode); break;
        case 0x1E: opcode_FX1E(opcode); break;
        case 0x29: opcode_FX29(opcode); break;
        case 0x33: opcode_FX33(opcode); break;
        case 0x55: opcode_FX55(opcode); break;
        case 0x65: opcode_FX65(opcode); break;
        default: break; 
    }
}

//call machine code routine at address NNN
void Chip8::opcode_0NNN(WORD opcode) {}

//clear screen 
void Chip8::opcode_00E0(WORD opcode) {
    clear_screen(); 
}

//return from subroutine
void Chip8::opcode_00EE(WORD opcode) {
    PC = stack.back(); 
    stack.pop_back(); 
}

//jump to address NNN
void Chip8::opcode_1NNN(WORD opcode) {
    PC = opcode & 0x0FFF;
    PC -= 2;  
}

//call subroutine at NNN
void Chip8::opcode_2NNN(WORD opcode) {
    stack.push_back(PC); 
    PC = opcode & 0x0FFF; 
    PC -= 2; 
}

//skip next instruction if VX == NN
void Chip8::opcode_3XNN(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if(registers[vx] == (opcode & 0xFF)) { 
        PC += 2;
    }
}

//skip next instruction if VX != NN
void Chip8::opcode_4XNN(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if(registers[vx] != (opcode & 0xFF)) { 
        PC += 2;
    }
}

//skip next instruction if VX == VY
void Chip8::opcode_5XY0(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    BYTE vy = (opcode >> 4) & 0xF; 
    if(registers[vx] == registers[vy]) { 
        PC += 2;
    }
}

//set VX to NN
void Chip8::opcode_6XNN(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[vx] = opcode & 0xFF; 
}

//Add NN to VX (no carry flag)
void Chip8::opcode_7XNN(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    WORD sum = (registers[vx] + (opcode & 0xFF)) & 0xFF; 
    registers[vx] = sum;
}

//set VX to value of VY
void Chip8::opcode_8XY0(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] = registers[vy];
}

//set Vx = VX |= VY
void Chip8::opcode_8XY1(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] |= registers[vy];
}

//set Vx = VX &= VY
void Chip8::opcode_8XY2(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] &= registers[vy];
}

//set Vx = VX ^= VY
void Chip8::opcode_8XY3(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] ^= registers[vy];
}

//set Vx = VX + VY; update VF carry
void Chip8::opcode_8XY4(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    WORD sum = registers[vx] + registers[vy]; 
    registers[0xF] = (sum & 0x100) >> 8; //carry bit 
    registers[vx] = sum & 0xFF; 
}

//set Vx = VX - VY; update VF carry
void Chip8::opcode_8XY5(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[0xF] = (int) (registers[vy] > registers[vx]); 
    registers[vx] -= registers[vy]; 
}

//Vx >>= 1
void Chip8::opcode_8XY6(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[0xF] = registers[vx] & 0x1; 
    registers[vx] >>= 1; 
}

//Vx = Vy - Vx
void Chip8::opcode_8XY7(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[0xF] = (int) (registers[vy] < registers[vx]); 
    registers[vx] = registers[vy] - registers[vx]; 
}

//Vx <<= 1
void Chip8::opcode_8XYE(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[0xF] = (registers[vx] >> 15) & 0x1; 
    registers[vx] <<= 1; 
}

//if (Vx != Vy)
void Chip8::opcode_9XY0(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    BYTE vy = (opcode >> 4) & 0xF; 
    if(registers[vx] != registers[vy]) { 
        PC += 2;
    }
}

//I = NNN
void Chip8::opcode_ANNN(WORD opcode) {
    I = opcode & 0xFFF; 
}

//PC = V0 + NNN
void Chip8::opcode_BNNN(WORD opcode) {
    PC = registers[0] + (opcode & 0xFFF); 
    PC -= 2; 
}

//Vx = rand() & NN
void Chip8::opcode_CXNN(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    int rand_num = (int) (rand() % 256); 
    registers[vx] = rand_num & (opcode & 0xFF); 
}

//draw(Vx, Vy, N)
void Chip8::opcode_DXYN(WORD opcode) {
    BYTE vx = (opcode & 0x0F00) >> 8; 
    BYTE vy = (opcode >> 4) & 0xF; 
    BYTE n = opcode & 0xF;
    
    registers[0xF] = 0; 
    // printf("0x%04x 0x%04x %d\n", vx, vy, n); //(opcode >> 8) & 0xF); 
    // printf("%d %d\n", registers[vx], registers[vy]); 
    for(int i = 0; i < n; i++) { 
        BYTE row = memory[I+i]; 
        for(int j = 0; j < 8; j++) {

            int val = (row & 0x80) >> 7; 
            int x = (j + registers[vx]); 
            int y = (i + registers[vy]); 
            int idx = y * SCREEN_W + x; 

            //check if pixel is onscreen 
            if(x >= SCREEN_W || y >= SCREEN_H ) {
                break;
            }

            //check for pixel collisions
            if (val == 1) {
                if (screen[idx] == 1) { 
                    screen[idx] = 0; 
                    registers[0xF] = 1; 
                } else { 
                    screen[idx] = 1; 
                }
            } 
            row <<= 1; 
        }
    }
}

//skip next instruction if key stored in Vx is pressed
void Chip8::opcode_EX9E(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if((keys >> registers[vx]) & 0x1) { 
        PC += 2; 
    }
}

//skip next instruction if key stored in Vx is not pressed 
void Chip8::opcode_EXA1(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if(!((keys >> registers[vx]) & 0x1)) { 
        PC += 2; 
    }
}

//set Vx to the value of the delay timer
void Chip8::opcode_FX07(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[vx] = delay_timer; 
}

//wait at instruction until key is pressed and store in Vx
void Chip8::opcode_FX0A(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    WORD temp = keys; 
    if(keys == 0) { 
        PC -= 2; 
    } else { 
        BYTE counter = 0x0; 
        while ((temp & 0x1) == 0x0) { 
            temp >>= 1;
            counter++; 
        }
        registers[vx] = counter; 
    }
}

//set delay timer to Vx
void Chip8::opcode_FX15(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    delay_timer = registers[vx]; 
}

//set sound timer to Vx
void Chip8::opcode_FX18(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    sound_timer = registers[vx]; 
}

//add Vx to I; do not update VF carry
void Chip8::opcode_FX1E(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    I += registers[vx];
    I &= 0x0FFF;     
}

//set I to the location of the sprite for a character in Vx
void Chip8::opcode_FX29(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    I = FONT_START + 5 * (registers[vx] & 0xF); 
}

//store hundreds, tens, ones digits in consecutive memory locations
void Chip8::opcode_FX33(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    BYTE temp = registers[vx]; 
    memory[I+2] = temp%10; 
    temp = (int) (temp/10); 

    memory[I+1] = temp%10; 
    temp = (int) (temp/10); 

    memory[I] = temp%10; 
}

//store V0-Vx in memory starting at address I; do not modify I
void Chip8::opcode_FX55(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    for(BYTE i = 0; i <= vx; i++) {
        memory[I+i] = registers[i];
    }
}

//Fill V0-Vx from memory starting at address I; do not modify I
void Chip8::opcode_FX65(WORD opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    for(BYTE i = 0; i <= vx; i++) {
        registers[i] = memory[I+i]; 
    }    
}