import sys 

def add_hex(string, hex): 
    return string + hex + ' '

def write_bytes(fn, hex, data): 
    with open('./out.c8', 'wb') as f: 
        f.write(bytes.fromhex(hex))
        f.write(data)

def error(word): 
    print('Error on token', word)
    sys.exit(1)

def get_hex(word, labels={}): 
    # if word == '': error(word)
    if word[:2] == '0x': 
        word = word[2:]
        try: 
            temp = int(word, 16) 
            return word
        except ValueError: 
            error(word)
    elif word[0] == 'v':
        word = word[1:]
        if len(word) != 1: 
            error(word)
        try: 
            temp = int(word, 16)
            return word
        except ValueError:
            error(word)
    else: 
        if word in labels: 
            return hex(labels[word])[2:]
        else: 
            error(word) 

def lex_line(line, labels): 
    ret_val = '' 
    words = line.split() 

    if len(words) == 0: 
        return ''
    print(words)

    # check for valid opcode 
    if words[0][:2] == '0x': 
        if len(words) == 1: 
            return get_hex(words[0]) + '00'
        elif len(words) == 2: 
            return get_hex(words[0]) + get_hex(words[1])
        else: 
            error(words[0])
    elif words[0] == 'cls': 
        if len(words) != 1: error(words[0])
        return '00E0'
    elif words[0] == 'return': 
        if len(words) != 1: error(words[0])
        return '00EE'
    elif words[0] == 'jump': 
        if len(words) != 2: error(words[0])
        hex_value = get_hex(words[1], labels)
        return '1' + hex_value
    elif words[0] == 'call': 
        if len(words) != 2: error(words[0])
        hex_value = get_hex(words[1], labels)
        return '2' + hex_value
    elif words[0] == 'skip_eq':  
        if len(words) != 3: error(words[0])
        reg_val = get_hex(words[1])
        hex_val = get_hex(words[2])
        return '3' + reg_val + hex_val
    elif words[0] == 'skip_neq': 
        if len(words) != 3: error(words[0])
        reg_val = get_hex(words[1])
        hex_val = get_hex(words[2])
        return '4' + reg_val + hex_val
    elif words[0] == 'skip_reg_eq': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '5' + reg1_val + reg2_val + '0'
    elif words[0] == 'mov': 
        if len(words) != 3: error(words[0])
        reg_val = get_hex(words[1])
        hex_val = get_hex(words[2])
        return '6' + reg_val + hex_val
    elif words[0] == 'add': 
        if len(words) != 3: error(words[0])
        reg_val = get_hex(words[1])
        hex_val = get_hex(words[2])
        return '7' + reg_val + hex_val
    elif words[0] == 'set': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '0'
    elif words[0] == 'or': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '1'
    elif words[0] == 'and': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '2'
    elif words[0] == 'xor': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '3'
    elif words[0] == 'reg_add': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '4' 
    elif words[0] == 'reg_sub': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '5'
    elif words[0] == 'shr': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '6'
    elif words[0] == 'rev_minus': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + '7'
    elif words[0] == 'shl':  
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '8' + reg1_val + reg2_val + 'E'
    elif words[0] == 'skip_reg_neq': 
        if len(words) != 3: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        return '9' + reg1_val + reg2_val + '0'
    elif words[0] == 'I':  
        if len(words) != 2: error(words[0])
        address = get_hex(words[1], labels)
        return 'A' + address
    elif words[0] == 'PC_offset':  
        if len(words) != 2: error(words[0])
        address = get_hex(words[1])
        return 'B' + address
    elif words[0] == 'rand':  
        if len(words) != 3: error(words[0]) 
        reg = get_hex(words[1]) 
        nn = get_hex(words[2])
        return 'C' + reg + nn
    elif words[0] == 'draw':  
        if len(words) != 4: error(words[0])
        reg1_val = get_hex(words[1])
        reg2_val = get_hex(words[2])
        n = get_hex(words[3])
        return 'D' + reg1_val + reg2_val + n
    elif words[0] == 'skip_key_eq':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'E' + reg + '9E'
    elif words[0] == 'skip_key_neq': 
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'E' + reg + 'A1'
    elif words[0] == 'get_delay':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '07'
    elif words[0] == 'get_key':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '0A'
    elif words[0] == 'set_delay':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '15'
    elif words[0] == 'set_sound':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '15'
    elif words[0] == 'add_I':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '1E'
    elif words[0] == 'sprite':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '29'
    elif words[0] == 'bcd':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '33'
    elif words[0] == 'reg_save':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '55'
    elif words[0] == 'reg_load':  
        if len(words) != 2: error(words[0])
        reg = get_hex(words[1])
        return 'F' + reg + '65'
    else: 
        print(words[0], 'not recognized') 
        sys.exit(1) 

def get_label(line): 
    words = line.split() 
    return words[0][:-1] if len(words) == 1 and words[0][-1] == ':' else None

def main(): 
    hex = '' 
    args = sys.argv
    if len(args) != 2: 
        print('Provide filename as argument')
        sys.exit(1) 

    fn = args[1]
    contents = []
    #load contents of input file 
    with open(fn, 'r') as f: 
        for line in f: 
            contents.append(line) 

    labels = {} 
    address = 0x200
    contents_filtered = []

    # check for labels 
    for line in contents: 
        #skip empty lines
        if line.strip() == '': 
            continue 

        label = get_label(line)
        if label: 
            labels[label] = address
        else: 
            address += 2 
            contents_filtered.append(line)

    # encode opcodes 
    for line in contents_filtered: 
        instruction = lex_line(line, labels)
        hex = add_hex(hex, instruction)

    data = []
    write_bytes('out.c8', hex, bytes(data))

if __name__ == '__main__': 
    main()