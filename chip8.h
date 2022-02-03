
#include <vector> 

typedef unsigned char BYTE; 
typedef unsigned short WORD; 

#define REG_NUM 16 
#define SCREEN_W 64
#define SCREEN_H 32
#define MEM_SIZE 4096

class Chip8 { 

    public: 
        Chip8(); 

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

        void key_pressed(WORD key); 
        void key_released(WORD key); 

    private: 
        void reset(); 
        void clear_screen();
        void clear_memory(); 
        void clear_registers();

        void decode_next_op(); 

        //opcode functions 
        void opcode_0NNN(BYTE opcode); 
        void opcode_00E0(BYTE opcode); 
        void opcode_00EE(BYTE opcode); 
        void opcode_1NNN(BYTE opcode); 
        void opcode_2NNN(BYTE opcode); 
        void opcode_3XNN(BYTE opcode); 
        void opcode_4XNN(BYTE opcode); 
        void opcode_5XY0(BYTE opcode); 
        void opcode_6XNN(BYTE opcode); 
        void opcode_7XNN(BYTE opcode); 
        void opcode_8XY0(BYTE opcode); 
        void opcode_8XY1(BYTE opcode); 
        void opcode_8XY2(BYTE opcode); 
        void opcode_8XY3(BYTE opcode); 
        void opcode_8XY4(BYTE opcode); 
        void opcode_8XY5(BYTE opcode); 
        void opcode_8XY6(BYTE opcode); 
        void opcode_8XY7(BYTE opcode); 
        void opcode_8XYE(BYTE opcode); 
        void opcode_9XY0(BYTE opcode); 
        void opcode_ANNN(BYTE opcode); 
        void opcode_BNNN(BYTE opcode); 
        void opcode_CXNN(BYTE opcode); 
        void opcode_DXYN(BYTE opcode); 
        void opcode_EX9E(BYTE opcode); 
        void opcode_EXA1(BYTE opcode); 
        void opcode_FX07(BYTE opcode); 
        void opcode_FX0A(BYTE opcode); 
        void opcode_FX15(BYTE opcode); 
        void opcode_FX18(BYTE opcode); 
        void opcode_FX1E(BYTE opcode); 
        void opcode_FX29(BYTE opcode); 
        void opcode_FX33(BYTE opcode); 
        void opcode_FX55(BYTE opcode); 
        void opcode_FX65(BYTE opcode); 

        //decode helper functions 
        void decode_0(BYTE opcode); 
        void decode_8(BYTE opcode); 
        void decode_E(BYTE opcode); 
        void decode_F(BYTE opcode); 

};