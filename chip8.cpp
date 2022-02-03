#include "chip8.h" 

Chip8::Chip8() {
    reset(); 
}

void Chip8::reset() { 
    PC = 0x0000; 
    I = 0X0000; 
    delay_timer = 0; 
    sound_timer = 0; 
    keys = 0x00; 
    stack.empty(); 
    clear_registers(); 
    clear_screen(); 
    clear_memory(); 

    int x = (int)4;
    int y = (int)4; 
    screen[y * SCREEN_W + x] = 0x01; 
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
    BYTE opcode = 0x0000; 
    opcode |= (memory[PC] << 8); 
    opcode |= memory[PC+1];
    BYTE op = (opcode >> 12) & 0xF; 

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

void Chip8::decode_0(BYTE opcode) { 
    switch(opcode) { 
        case 0x00E0: opcode_00E0(opcode); break; 
        case 0X00EE: opcode_00E0(opcode); break; 
        default: opcode_0NNN(opcode); break; 
    }
}

void Chip8::decode_8(BYTE opcode) { 
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

void Chip8::decode_E(BYTE opcode) { 
    switch(opcode & 0xFF) { 
        case 0x9E: opcode_EX9E(opcode); break;
        case 0xA1: opcode_EXA1(opcode); break;
        default: break; 
    }
}

void Chip8::decode_F(BYTE opcode) { 
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
void Chip8::opcode_0NNN(BYTE opcode) {}

//clear screen 
void Chip8::opcode_00E0(BYTE opcode) {
    clear_screen(); 
}

//return from subroutine
void Chip8::opcode_00EE(BYTE opcode) {
    PC = stack.back(); 
    stack.pop_back(); 
}

//jump to address NNN
void Chip8::opcode_1NNN(BYTE opcode) {
    PC = opcode & 0x0FFF; 
}

//call subroutine at NNN
void Chip8::opcode_2NNN(BYTE opcode) {
    stack.push_back(PC); 
    PC = opcode & 0x0FFF; 
}

//skip next instruction if VX == NN
void Chip8::opcode_3XNN(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if(registers[vx] == (opcode & 0xFF)) { 
        PC += 2;
    }
}

//skip next instruction if VX != NN
void Chip8::opcode_4XNN(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    if(registers[vx] != (opcode & 0xFF)) { 
        PC += 2;
    }
}

//skip next instruction if VX == VY
void Chip8::opcode_5XY0(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    BYTE vy = (opcode >> 4) & 0xF; 
    if(registers[vx] == registers[vy]) { 
        PC += 2;
    }
}

//set VX to NN
void Chip8::opcode_6XNN(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[vx] = opcode & 0xFF; 
}

//Add NN to VX (no carry flag)
void Chip8::opcode_7XNN(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    WORD sum = (registers[vx] + (opcode & 0xFF)) & 0xFF; 
    registers[vx] = sum;
}

//set VX to value of VY
void Chip8::opcode_8XY0(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] = registers[vy];
}

//set Vx = VX |= VY
void Chip8::opcode_8XY1(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] |= registers[vy];
}

//set Vx = VX &= VY
void Chip8::opcode_8XY2(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] &= registers[vy];
}

//set Vx = VX ^= VY
void Chip8::opcode_8XY3(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    registers[vx] ^= registers[vy];
}

//set Vx = VX + VY; update VF carry
void Chip8::opcode_8XY4(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE vy = (opcode >> 4) & 0xF;
    WORD sum = registers[vx] + registers[vy]; 
    registers[0xF] = sum & 0x100; //carry bit 
    registers[vx] = sum & 0xFF; 
}

//set Vx = VX - VY; update VF carry
void Chip8::opcode_8XY5(BYTE opcode) {

}

//Vx >>= 1
void Chip8::opcode_8XY6(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[0xF] = registers[vx] & 0x1; 
    registers[vx] >>= 1; 
}

//Vx = Vy - Vx
void Chip8::opcode_8XY7(BYTE opcode) {}

//Vx <<= 1
void Chip8::opcode_8XYE(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[0xF] = registers[vx] >> 15; 
    registers[vx] <<= 1; 
}

//if (Vx != Vy)
void Chip8::opcode_9XY0(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF; 
    BYTE vy = (opcode >> 4) & 0xF; 
    if(registers[vx] != registers[vy]) { 
        PC += 2;
    }
}

//I = NNN
void Chip8::opcode_ANNN(BYTE opcode) {
    I = opcode & 0xFFF; 
}

//PC = V0 + NNN
void Chip8::opcode_BNNN(BYTE opcode) {
    PC = registers[0] + (opcode & 0xFFF); 
}

//Vx = rand() & NN
void Chip8::opcode_CXNN(BYTE opcode) {}

//draw(Vx, Vy, N)
void Chip8::opcode_DXYN(BYTE opcode) {}

//skip next instruction if key stored in Vx is pressed
void Chip8::opcode_EX9E(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE stored_key = registers[vx] & 0xF;
    if ((keys >> stored_key) & 0x1) { 
        PC += 2;
    }
}

//skip next instruction if key stored in Vx is not pressed 
void Chip8::opcode_EXA1(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    BYTE stored_key = registers[vx] & 0xF;
    if (!((keys >> stored_key) & 0x1)) { 
        PC += 2;
    }
}

//set Vx to the value of the delay timer
void Chip8::opcode_FX07(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    registers[vx] = delay_timer; 
}

//wait at instruction until key is pressed and store in Vx
void Chip8::opcode_FX0A(BYTE opcode) {
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
void Chip8::opcode_FX15(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    delay_timer = registers[vx]; 
}

//set sound timer to Vx
void Chip8::opcode_FX18(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    sound_timer = registers[vx]; 
}

//add Vx to I; do not update VF carry
void Chip8::opcode_FX1E(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    I += registers[vx];    
}

//set I to the location of the sprite for a character in Vx
void Chip8::opcode_FX29(BYTE opcode) {}

//
void Chip8::opcode_FX33(BYTE opcode) {}

//store V0-Vx in memory starting at address I; do not modify I
void Chip8::opcode_FX55(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    for(BYTE i = 0; i <= vx; i++) {
        memory[I+i] = registers[i];
    }
}

//Fill V0-Vx from memory starting at address I; do not modify I
void Chip8::opcode_FX65(BYTE opcode) {
    BYTE vx = (opcode >> 8) & 0xF;
    for(BYTE i = 0; i <= vx; i++) {
        registers[i] = memory[I+i]; 
    }    
}