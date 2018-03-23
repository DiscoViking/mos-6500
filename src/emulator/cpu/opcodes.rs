macro_rules! opcode {
    ($name:ident, $value:expr) => (pub const $name: u8 = $value;)
}

opcode!(ADC_IMM, 0x69);
opcode!(ADC_ZPG, 0x65);
opcode!(ADC_ZPG_X, 0x75);
opcode!(ADC_ABS, 0x6D);
opcode!(ADC_ABS_X, 0x7D);
opcode!(ADC_ABS_Y, 0x79);
opcode!(ADC_IX_IND, 0x61);
opcode!(ADC_IND_IX, 0x71);

opcode!(AND_IMM, 0x29);
opcode!(AND_ZPG, 0x25);
opcode!(AND_ZPG_X, 0x35);
opcode!(AND_ABS, 0x2D);
opcode!(AND_ABS_X, 0x3D);
opcode!(AND_ABS_Y, 0x39);
opcode!(AND_IX_IND, 0x21);
opcode!(AND_IND_IX, 0x31);

opcode!(LDA_IMM, 0xA9);
opcode!(LDA_ZPG, 0xA5);
opcode!(LDA_ZPG_X, 0xB5);
opcode!(LDA_ABS, 0xAD);
opcode!(LDA_ABS_X, 0xBD);
opcode!(LDA_ABS_Y, 0xB9);
opcode!(LDA_IX_IND, 0xA1);
opcode!(LDA_IND_IX, 0xB1);

opcode!(SBC_IMM, 0xE9);
opcode!(SBC_ZPG, 0xE5);
opcode!(SBC_ZPG_X, 0xF5);
opcode!(SBC_ABS, 0xED);
opcode!(SBC_ABS_X, 0xFD);
opcode!(SBC_ABS_Y, 0xF9);
opcode!(SBC_IX_IND, 0xE1);
opcode!(SBC_IND_IX, 0xF1);

opcode!(SEC, 0x38);

opcode!(STA_ZPG, 0x85);
opcode!(STA_ZPG_X, 0x95);
opcode!(STA_ABS, 0x8D);
opcode!(STA_ABS_X, 0x9D);
opcode!(STA_ABS_Y, 0x99);
opcode!(STA_IX_IND, 0x81);
opcode!(STA_IND_IX, 0x91);
