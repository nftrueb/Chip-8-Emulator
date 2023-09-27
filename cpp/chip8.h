
#include <vector>

typedef unsigned char BYTE; 
typedef unsigned short WORD; 

#define REG_NUM 16 
#define SCREEN_W 64
#define SCREEN_H 32
#define MEM_SIZE 4096
#define FONT_START 0
#define PROG_START 0x200 

class Chip8 { 

    public: 
        Chip8(char *rom); 

        //emulated components
        BYTE memory[MEM_SIZE]; 
        BYTE screen[SCREEN_H*SCREEN_W]; 
        BYTE registers[REG_NUM]; 
        std::vector<WORD> stack; 
        BYTE delay_timer; 
        BYTE sound_timer; 
        WORD keys; 

        //address variable
        WORD I; 
        //program counter capable of having range [0,4096)
        WORD PC;

        char *rom_; 

        void key_pressed(WORD key); 
        void key_released(WORD key); 

        int load_rom(); 
        void reset(); 
        void init_font(); 
        void clear_screen();
        void clear_memory(); 
        void clear_registers();

        void decode_next_op(); 

        //opcode functions 
        void opcode_0NNN(WORD opcode); 
        void opcode_00E0(WORD opcode); 
        void opcode_00EE(WORD opcode); 
        void opcode_1NNN(WORD opcode); 
        void opcode_2NNN(WORD opcode); 
        void opcode_3XNN(WORD opcode); 
        void opcode_4XNN(WORD opcode); 
        void opcode_5XY0(WORD opcode); 
        void opcode_6XNN(WORD opcode); 
        void opcode_7XNN(WORD opcode); 
        void opcode_8XY0(WORD opcode); 
        void opcode_8XY1(WORD opcode); 
        void opcode_8XY2(WORD opcode); 
        void opcode_8XY3(WORD opcode); 
        void opcode_8XY4(WORD opcode); 
        void opcode_8XY5(WORD opcode); 
        void opcode_8XY6(WORD opcode); 
        void opcode_8XY7(WORD opcode); 
        void opcode_8XYE(WORD opcode); 
        void opcode_9XY0(WORD opcode); 
        void opcode_ANNN(WORD opcode); 
        void opcode_BNNN(WORD opcode); 
        void opcode_CXNN(WORD opcode); 
        void opcode_DXYN(WORD opcode); 
        void opcode_EX9E(WORD opcode); 
        void opcode_EXA1(WORD opcode); 
        void opcode_FX07(WORD opcode); 
        void opcode_FX0A(WORD opcode); 
        void opcode_FX15(WORD opcode); 
        void opcode_FX18(WORD opcode); 
        void opcode_FX1E(WORD opcode); 
        void opcode_FX29(WORD opcode); 
        void opcode_FX33(WORD opcode); 
        void opcode_FX55(WORD opcode); 
        void opcode_FX65(WORD opcode); 

        //decode helper functions 
        void decode_0(WORD opcode); 
        void decode_8(WORD opcode); 
        void decode_E(WORD opcode); 
        void decode_F(WORD opcode); 

};