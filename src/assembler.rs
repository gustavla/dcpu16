use std::io::prelude::*;
use std::fmt;
use std::str;
use std::ascii::AsciiExt;
use std::collections::HashMap;

use dcpu::MEMORY_SIZE;
use instructions::*;

struct UnassignedLabel {
    addr: u16,
    label: u16,
    offset: u16,
}

// PCPU contains the parsing state of a DCPU
pub struct PCPU {
    // Memory
    pub mem: [u16; MEMORY_SIZE],

    // Current position
    pub pc: u16,

    // Label ID -> Addr look-up table
    labels: HashMap<u16, u16>,

    // The tokenizer assigns an integer ID to each label
    label_to_id: HashMap<String, u16>,

    // first appearane of labels
    label_first_line_error: HashMap<u16, ParsingError>,

    // Only needed for user feedback
    id_to_label: HashMap<u16, String>,

    // Addresses with unseen label
    unassigned_addresses: Vec<UnassignedLabel>,

    // Next label token
    next_label_id: u16,

    // String literals (referred to by tokens)
    string_literals: Vec<String>,

    // Next string literal id
    next_string_id: u16,
}

impl PCPU {
    pub fn new() -> PCPU {
        PCPU {
            mem: [0; MEMORY_SIZE],
            pc: 0,
            labels: HashMap::new(),
            label_to_id: HashMap::new(),
            id_to_label: HashMap::new(),
            label_first_line_error: HashMap::new(),
            unassigned_addresses: Vec::new(),
            next_label_id: 0,
            string_literals: Vec::new(),
            next_string_id: 0,
        }
    }
}

/*
Grammar (EBNF)

program = line, { '\n', line } ;

line = instr | label ;

instr = basic_op, value, ',', value
      | special_op, value
      | data_op, data_value, { ',', data_value }
      | ':', label
      ;

basic_op = 'SET' | 'ADD' | ... | 'STD' ;
special_op = 'JSR' | ... | 'HWI' ;
data_op = 'DAT' ;

value = numerical_literal
      | register
      | label
      | '[', literal, ']'
      | '[', register, ']'
      | '[', numerical_literal, '+', register, ']'
      | '[', register, '+', numerical_literal, ']'
      | '[', label, ']'
      ;

label = letter, { alphanumeric }

register = 'A' | 'B' | 'C' | 'X' | 'Y' | 'Z' | 'I' | 'J' ;

numerical_literal = decimal | hexadecimal ;   (e.g. -123, 0xff)

string_literal = c_like_string (e.g. "This is \"the\" string\n")
*/

//const BUFSIZE: usize = 256;
//const BUFFLUSH: usize = BUFSIZE - 16;

#[derive(Debug, Copy, Clone)]
pub enum ParsingErrorType {
    InvalidLiteral,
    UnclosedStringLiteral,
    IllegalCharacter,
    IllegalLineStart,
    IllegalLvalue,
    ExpectingComma,
    ExpectingOperand,
    ExpectingLiteral,
    ExpectingRightBracket,
    ExpectingLabel,
    EndOfTokens,
    ExtraTokens,
    IncorrectPushPop,
    UnknownLabel(u16),
}

pub fn format_error(etype: &ParsingErrorType, cpu: &PCPU) -> String {
    match etype {
        &ParsingErrorType::InvalidLiteral =>
            format!("Invalid literal"),
        &ParsingErrorType::UnclosedStringLiteral =>
            format!("Unclosed string literal"),
        &ParsingErrorType::IllegalCharacter =>
            format!("Illegal character"),
        &ParsingErrorType::IllegalLineStart =>
            format!("Line must start with an instruction or a label"),
        &ParsingErrorType::IllegalLvalue=>
            format!("Illegal L-value; unable to assign to numeric literal"),
        &ParsingErrorType::ExpectingComma =>
            format!("Expecting comma"),
        &ParsingErrorType::ExpectingOperand =>
            format!("Expecting operand"),
        &ParsingErrorType::ExpectingLiteral =>
            format!("Expecting literal or label"),
        &ParsingErrorType::ExpectingRightBracket =>
            format!("Expecting closing bracket"),
        &ParsingErrorType::ExpectingLabel =>
            format!("Expecting label name"),
        &ParsingErrorType::EndOfTokens =>
            format!("Expecting more tokens after this one"),
        &ParsingErrorType::ExtraTokens =>
            format!("Extra tokens found after successfully parsed line"),
        &ParsingErrorType::IncorrectPushPop =>
            format!("Push cannot be used as a-operand / Pop cannot be used as b-operand"),
        &ParsingErrorType::UnknownLabel(label) => {
            match cpu.id_to_label.get(&label) {
                Some(s) => format!("Label definition not found: {}", s),
                None => format!("Unknown label definition not found"),
            }
        },
    }
}

#[derive(Copy, Clone)]
pub struct ParsingError {
    pub line: usize,
    col: usize,
    len: usize,
    global: bool,
    etype: ParsingErrorType,
}

struct ParsingInfo {
    operand: u16,
    extra_byte: Option<u16>,
    unassigned: bool,
    offset: u16,
}

impl ParsingInfo {
    fn new() -> ParsingInfo {
        ParsingInfo { operand: 0, extra_byte: None, unassigned: false, offset: 0 }
    }

    fn new_single(operand: u16) -> ParsingInfo {
        ParsingInfo{operand: operand, extra_byte: None, unassigned: false, offset: 0}
    }

    fn new_extra(operand: u16, extra: u16) -> ParsingInfo {
        ParsingInfo{operand: operand, extra_byte: Some(extra), unassigned: false, offset: 0}
    }
}

#[derive(Debug)]
pub enum TokenType {
    NumericLiteral(u16),
    StringLiteral(u16),
    BasicOpcode(usize),
    SpecialOpcode(usize),
    DataOpcode,
    Label(u16),
    Registry(u16),
    Pick,
    Peek,
    Push,
    Pop,
    PC,
    SP,
    EX,
    Addition,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
}

pub struct Token {
    pub ttype: TokenType,
    pub col: usize,
    pub len: usize,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TokenType::NumericLiteral(i) => write!(f, "NumericLiteral({})", i),
            &TokenType::StringLiteral(i) => write!(f, "StringLiteral({})", i),
            &TokenType::BasicOpcode(i) => write!(f, "BasicOpcode({})", i),
            &TokenType::SpecialOpcode(i) => write!(f, "SpecialOpcode({})", i),
            &TokenType::DataOpcode => write!(f, "DataOpcode"),
            &TokenType::Label(i) => write!(f, "Label({})", i),
            &TokenType::Registry(i) => write!(f, "Registry({})", i),
            &TokenType::Pick => write!(f, "Pick"),
            &TokenType::Peek => write!(f, "Peek"),
            &TokenType::Push => write!(f, "Push"),
            &TokenType::Pop => write!(f, "Pop"),
            &TokenType::PC => write!(f, "PC"),
            &TokenType::SP => write!(f, "SP"),
            &TokenType::EX => write!(f, "EX"),
            &TokenType::Addition => write!(f, "Addition"),
            &TokenType::LeftBracket => write!(f, "LeftBracket"),
            &TokenType::RightBracket => write!(f, "RightBracket"),
            &TokenType::Comma => write!(f, "Comma"),
            &TokenType::Colon => write!(f, "Colon"),
        }
    }
}

fn legal_label_char(c: char) -> bool {
    match c {
        'A' ... 'Z' | 'a' ... 'z' | '_' | '0' ... '9' => true,
        _ => false,
    }
}

fn registry_char(c: char) -> isize {
    match c {
        'A' | 'a' => 0,
        'B' | 'b' => 1,
        'C' | 'c' => 2,
        'X' | 'x' => 3,
        'Y' | 'y' => 4,
        'Z' | 'z' => 5,
        'I' | 'i' => 6,
        'J' | 'j' => 7,
        _ => -1,
    }
}

fn keyword_token(s: &str) -> Option<TokenType> {
    match &s.to_ascii_uppercase()[..] {
        "PICK" => Some(TokenType::Pick),
        "PEEK" => Some(TokenType::Peek),
        "PUSH" => Some(TokenType::Push),
        "POP" => Some(TokenType::Pop),
        "PC" => Some(TokenType::PC),
        "SP" => Some(TokenType::SP),
        "EX" => Some(TokenType::EX),
        _ => None
    }
}

fn basic_opcode(s: &str) -> Option<usize> {
    match &s.to_ascii_uppercase()[..] {
        "SET" => Some(SET),
        "ADD" => Some(ADD),
        "SUB" => Some(SUB),
        "MUL" => Some(MUL),
        "MLI" => Some(MLI),
        "DIV" => Some(DIV),
        "DVI" => Some(DVI),
        "MOD" => Some(MOD),
        "MDI" => Some(MDI),
        "AND" => Some(AND),
        "BOR" => Some(BOR),
        "XOR" => Some(XOR),
        "SHR" => Some(SHR),
        "ASR" => Some(ASR),
        "SHL" => Some(SHL),
        "IFB" => Some(IFB),
        "IFC" => Some(IFC),
        "IFE" => Some(IFE),
        "IFN" => Some(IFN),
        "IFG" => Some(IFG),
        "IFA" => Some(IFA),
        "IFL" => Some(IFL),
        "IFU" => Some(IFU),
        "ADX" => Some(ADX),
        "SBX" => Some(SBX),
        "STI" => Some(STI),
        "STD" => Some(STD),
        _ => None,
    }
}

fn special_opcode(s: &str) -> Option<usize> {
    match &s.to_ascii_uppercase()[..] {
        "JSR" => Some(JSR),
        "INT" => Some(INT),
        "IAG" => Some(IAG),
        "IAS" => Some(IAS),
        "RFI" => Some(RFI),
        "IAQ" => Some(IAQ),
        "HWN" => Some(HWN),
        "HWQ" => Some(HWQ),
        "HWI" => Some(HWI),
        // Extra
        "OUT" => Some(OUT),
        _ => None,
    }
}

pub fn tokenize(line_no: usize, line: &str, cpu: &mut PCPU) -> Result<Vec<Token>, ParsingError> {
    let mut tokens: Vec<Token> = vec!();

    let mut i = 0;
    while i < line.len() {
        let token = match line.chars().nth(i).unwrap() {
            '[' => Token { ttype: TokenType::LeftBracket, col: i, len: 1 },
            ']' => Token { ttype: TokenType::RightBracket, col: i, len: 1 },
            '+' => Token { ttype: TokenType::Addition, col: i, len: 1},
            ',' => Token { ttype: TokenType::Comma, col: i, len: 1},
            ':' => Token { ttype: TokenType::Colon, col: i, len: 1},
            ';' => break,
            '"' => {
                let col = i;
                let mut s: String = String::new();
                let mut backslash = false;
                let mut closed = false;
                i += 1;
                while i < line.len() {
                    let c = line.chars().nth(i).unwrap();
                    i += 1;
                    if backslash {
                        if c == 'n' {
                            s.push('\n');
                        } else if c == 'r' {
                            s.push('\r');
                        } else if c == 't' {
                            s.push('\t');
                        } else if c == '"' {
                            s.push('"');
                        }
                        // Else don't add any
                        backslash = false;
                    } else if c == '"' {
                        closed = true;
                        i -= 1;
                        break;
                    } else if c == '\\' {
                        backslash = true;
                    } else {
                        s.push(c);
                    }
                }
                let id = cpu.next_string_id;
                let len = s.len();

                if !closed {
                    let err = ParsingError{ line: line_no,
                                            col: col,
                                            len: len+1,
                                            global: false,
                                            etype: ParsingErrorType::UnclosedStringLiteral };
                    return Err(err);
                }

                cpu.string_literals.push(s);
                cpu.next_string_id += 1;

                Token { ttype: TokenType::StringLiteral(id),
                        col: col,
                        len: len+2 }
            },
            '0' ... '9' | '-' => {
                // Parse literal (dec/hex)
                //let mut s: String = String::new();
                let col = i;
                let mut end_col = i;
                let minus = line.chars().nth(i).unwrap() == '-';
                if minus {
                    i += 1;
                    end_col += 1;
                }

                let res: Option<isize> = if i + 1 < line.len() &&
                                            line.chars().nth(i).unwrap() == '0' &&
                                            (line.chars().nth(i+1).unwrap() == 'X' ||
                                             line.chars().nth(i+1).unwrap() == 'x') {
                    i += 2;
                    // Hexadecimal
                    let mut s: String = String::new();
                    while i < line.len() {
                        let c = line.chars().nth(i).unwrap();
                        if legal_label_char(c) {
                            s.push(c);
                            i += 1;
                            end_col += 1;
                        } else {
                            i -= 1;
                            break;
                        }
                    }

                    match isize::from_str_radix(&s[..], 16) {
                        Ok(s) => Some(s),
                        Err(_) => None,
                    }
                } else {
                    // Decimal
                    let mut s: String = String::new();
                    while i < line.len() {
                        let c = line.chars().nth(i).unwrap();
                        if legal_label_char(c) {
                            s.push(c);
                            i += 1;
                            end_col += 1;
                        } else {
                            i -= 1;
                            break;
                        }
                    }

                    match str::FromStr::from_str(&s[..]) {
                        Ok(s) => Some(s),
                        Err(_) => None,
                    }
                };
                match res {
                    Some(value) => {
                        Token { ttype: TokenType::NumericLiteral((if minus { -value } else { value }) as u16),
                                col: col,
                                len: end_col-col }
                    },
                    None => {
                        let err = ParsingError{ line: line_no,
                                                col: col,
                                                len: i-col,
                                                global: false,
                                                etype: ParsingErrorType::InvalidLiteral };
                        return Err(err);
                    }
                }
            },
            ' ' | '\n' | '\r' | '\t' => { i += 1; continue },
            _ => {
                // Read as label, instruction or registry
                let col = i;

                let mut s: String = String::new();
                while i < line.len() {
                    let c = line.chars().nth(i).unwrap();
                    if legal_label_char(c) {
                        s.push(c);
                        i += 1;
                    } else {
                        i -= 1;
                        break;
                    }
                }
                //println!("i = {}, col = {}", i, col);
                if i + 1 == col {
                    let err = ParsingError{ line: line_no,
                                            col: col,
                                            len: 0,
                                            global: false,
                                            etype: ParsingErrorType::IllegalCharacter };
                    return Err(err);
                } else if s.len() == 1 && registry_char(s.chars().nth(0).unwrap()) >= 0 {
                    Token { ttype: TokenType::Registry(registry_char(s.chars().nth(0).unwrap()) as u16),
                            col: col,
                            len: 1 }
                } else if let Some(ttype) = keyword_token(&s[..]) {
                    Token { ttype: ttype,
                            col: col,
                            len: s.len() }
                } else if s.len() == 3 && basic_opcode(&s[..]).is_some() {
                    Token { ttype: TokenType::BasicOpcode(basic_opcode(&s[..]).unwrap()),
                            col: col,
                            len: 3 }
                } else if s.len() == 3 && special_opcode(&s[..]).is_some() {
                    Token { ttype: TokenType::SpecialOpcode(special_opcode(&s[..]).unwrap()),
                            col: col,
                            len: 3 }
                } else if s.to_ascii_uppercase() == "DAT" {
                    Token { ttype: TokenType::DataOpcode,
                            col: col,
                            len: 3 }
                } else {
                    // Treat it as a label
                    let len = s.len();
                    let label = match cpu.label_to_id.get(&s[..]).cloned() {
                        Some(v) => {
                            v
                        },
                        None => {
                            let l = cpu.next_label_id;
                            cpu.next_label_id += 1;
                            let s_copy = s.clone();
                            cpu.label_to_id.insert(s, l);
                            cpu.id_to_label.insert(l, s_copy);
                            let err = ParsingError {line: line_no,
                                                    col: col,
                                                    len: len,
                                                    global: false,
                                                    etype: ParsingErrorType::UnknownLabel(l)};


                            cpu.label_first_line_error.insert(l, err);
                            l
                        },
                    };

                    Token { ttype: TokenType::Label(label as u16),
                            col: col,
                            len: len }
                }
            }
        };

        tokens.push(token);
        i += 1;
    }

    Ok(tokens)
}

fn process_value(value: u16, allow_inline: bool) -> Result<ParsingInfo, ParsingError> {
    let info = if value == 0xffff && allow_inline {
        ParsingInfo::new_single(0x20)
    } else if value <= 0x1e && allow_inline {
        ParsingInfo::new_single(value + 0x21)
    } else {
        ParsingInfo::new_extra(0x1f, value)
    };
    Ok(info)
}

fn parse_value(line_no: usize, tokens: &Vec<Token>, cur: &mut usize,
               cpu: &mut PCPU, lvalue: bool) -> Result<ParsingInfo, ParsingError> {
    let ttype = try!(get_token_type(line_no, tokens, *cur));
    match *ttype {
        TokenType::NumericLiteral(value) => {
            // Can't have numeric literals as lvalues
            /*
            TODO: Disabled, since sometimes permissble, such as with IF*
            if lvalue {
                return Err(ParsingError{line: line_no,
                                        col: tokens[*cur].col,
                                        len: tokens[*cur].len,
                                        global: false,
                                        etype: ParsingErrorType::IllegalLvalue});
            }
            */
            *cur += 1;
            // L-values are not allowed to have inlined values
            Ok(try!(process_value(value, !lvalue)))
        },
        TokenType::Label(id) => {
            match cpu.labels.get(&id) {
                Some(label) => {
                    *cur += 1;
                    Ok(try!(process_value(*label, !lvalue)))
                },
                None => {
                    *cur += 1;
                    Ok(ParsingInfo{operand: 0x1f, extra_byte: Some(id), unassigned: true, offset: 0})
                },
            }
        }
        TokenType::Registry(reg) => {
            *cur += 1;
            let info = ParsingInfo::new_single(reg as u16);
            Ok(info)
        },
        TokenType::Pick => {
            *cur += 1;
            let ttype0 = try!(get_token_type(line_no, tokens, *cur));
            match *ttype0 {
                TokenType::NumericLiteral(v) => {
                    *cur += 1;
                    let info = ParsingInfo::new_extra(0x1a, v);
                    Ok(info)
                }
                _ => {
                    let err = ParsingError{ line: line_no,
                                            col: tokens[*cur].col,
                                            len: tokens[*cur].len,
                                            global: false,
                                            etype: ParsingErrorType::ExpectingLiteral};
                    Err(err)
                }
            }
        },
        TokenType::Peek => {
            *cur += 1;
            let info = ParsingInfo::new_single(0x19);
            Ok(info)
        },
        TokenType::Push => {
            if !lvalue {
                let err = ParsingError{ line: line_no,
                                        col: tokens[*cur].col,
                                        len: tokens[*cur].len,
                                        global: false,
                                        etype: ParsingErrorType::IncorrectPushPop };
                Err(err)
            } else {
                *cur += 1;
                let info = ParsingInfo::new_single(0x18);
                Ok(info)
            }
        },
        TokenType::Pop => {
            if lvalue {
                let err = ParsingError{ line: line_no,
                                        col: tokens[*cur].col,
                                        len: tokens[*cur].len,
                                        global: false,
                                        etype: ParsingErrorType::IncorrectPushPop};
                Err(err)
            } else {
                *cur += 1;
                let info = ParsingInfo::new_single(0x18);
                Ok(info)
            }
        },
        TokenType::SP => {
            *cur += 1;
            let info = ParsingInfo::new_single(0x1b);
            Ok(info)
        },
        TokenType::PC => {
            *cur += 1;
            let info = ParsingInfo::new_single(0x1c);
            Ok(info)
        },
        TokenType::EX => {
            *cur += 1;
            let info = ParsingInfo::new_single(0x1d);
            Ok(info)
        },
        /**
          * This whole business needs refactoring. Every case is handled by
          * an explicit case, which is extremely verbose.
          */

        TokenType::LeftBracket => {
            *cur += 1;
            let ttype0 = try!(get_token_type(line_no, tokens, *cur));
            match *ttype0 {
                TokenType::Registry(reg) => {
                    *cur += 1;
                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                    match *ttype1 {
                        TokenType::RightBracket => {
                            *cur += 1;
                            let info = ParsingInfo::new_single(0x08+reg);
                            Ok(info)
                        },
                        TokenType::Addition => {
                            *cur += 1;
                            let ttype2 = try!(get_token_type(line_no, tokens, *cur));
                            match *ttype2 {
                                TokenType::NumericLiteral(offset) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;
                                            let info = ParsingInfo::new_extra(0x10 + reg, offset);
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                TokenType::Label(id) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;
                                            let info = match cpu.labels.get(&id) {
                                                Some(label) => {
                                                    let pos = *label;
                                                    ParsingInfo::new_extra(0x10 + reg, pos)
                                                },
                                                None => {
                                                    ParsingInfo{operand: 0x10 + reg, extra_byte: Some(id), unassigned: true, offset: 0}
                                                }
                                            };
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                _ => {
                                    let err = ParsingError{ line: line_no,
                                                            col: tokens[*cur].col,
                                                            len: tokens[*cur].len,
                                                            global: false,
                                                            etype: ParsingErrorType::ExpectingLiteral };
                                    Err(err)
                                }
                            }
                        },
                        _ => {
                            let err = ParsingError{ line: line_no,
                                                    col: tokens[*cur].col,
                                                    len: tokens[*cur].len,
                                                    global: false,
                                                    etype: ParsingErrorType::ExpectingRightBracket };
                            Err(err)
                        }
                    }
                },
                TokenType::NumericLiteral(v0) => {
                    *cur += 1;
                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                    match *ttype1 {
                        TokenType::RightBracket => {
                            *cur += 1;
                            let info = ParsingInfo::new_extra(0x1e, v0);
                            Ok(info)
                        },
                        TokenType::Addition => {
                            *cur += 1;
                            let ttype2 = try!(get_token_type(line_no, tokens, *cur));
                            match *ttype2 {
                                TokenType::Registry(reg) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;
                                            let info = ParsingInfo::new_extra(0x10 + reg, v0);
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                TokenType::Label(id) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;
                                            let info = match cpu.labels.get(&id) {
                                                Some(label) => {
                                                    let pos = (((*label as usize) + (v0 as usize)) % MEMORY_SIZE) as u16;
                                                    ParsingInfo::new_extra(0x1e, pos)
                                                },
                                                None => {
                                                    ParsingInfo{operand: 0x1e, extra_byte: Some(id), unassigned: true, offset: v0}
                                                }
                                            };
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                _ => {
                                    let err = ParsingError{ line: line_no,
                                                            col: tokens[*cur].col,
                                                            len: tokens[*cur].len,
                                                            global: false,
                                                            etype: ParsingErrorType::ExpectingLiteral };
                                    Err(err)
                                }
                            }
                        },
                        _ => {
                            let err = ParsingError{ line: line_no,
                                                    col: tokens[*cur].col,
                                                    len: tokens[*cur].len,
                                                    global: false,
                                                    etype: ParsingErrorType::ExpectingRightBracket };
                            Err(err)
                        }
                    }
                },
                TokenType::Label(id) => {
                    *cur += 1;
                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                    match *ttype1 {
                        TokenType::RightBracket => {
                            *cur += 1;
                            let info = match cpu.labels.get(&id) {
                                Some(label) => {
                                    ParsingInfo::new_extra(0x1e, *label)
                                },
                                None => {
                                    ParsingInfo{operand: 0x1e, extra_byte: Some(id), unassigned: true, offset: 0}
                                },
                            };
                            Ok(info)
                        },
                        TokenType::Addition => {
                            *cur += 1;
                            let ttype2 = try!(get_token_type(line_no, tokens, *cur));
                            match *ttype2 {
                                TokenType::NumericLiteral(offset) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;

                                            let info = match cpu.labels.get(&id) {
                                                Some(label) => {
                                                    let pos = (((*label as usize) + (offset as usize)) % MEMORY_SIZE) as u16;
                                                    ParsingInfo::new_extra(0x1e, pos)
                                                },
                                                None => {
                                                    ParsingInfo{operand: 0x1e, extra_byte: Some(id), unassigned: true, offset: offset}
                                                }
                                            };
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                TokenType::Registry(reg) => {
                                    *cur += 1;
                                    let ttype1 = try!(get_token_type(line_no, tokens, *cur));
                                    match *ttype1 {
                                        TokenType::RightBracket => {
                                            *cur += 1;
                                            let info = match cpu.labels.get(&id) {
                                                Some(label) => {
                                                    ParsingInfo::new_extra(0x10 + reg, *label)
                                                },
                                                None => {
                                                    ParsingInfo{operand: 0x10 + reg, extra_byte: Some(id), unassigned: true, offset: 0}
                                                },
                                            };
                                            Ok(info)
                                        },
                                        _ => {
                                            let err = ParsingError{ line: line_no,
                                                                    col: tokens[*cur].col,
                                                                    len: tokens[*cur].len,
                                                                    global: false,
                                                                    etype: ParsingErrorType::ExpectingRightBracket };
                                            Err(err)
                                        }
                                    }
                                },
                                _ => {
                                    let err = ParsingError{ line: line_no,
                                                            col: tokens[*cur].col,
                                                            len: tokens[*cur].len,
                                                            global: false,
                                                            etype: ParsingErrorType::ExpectingLiteral };
                                    Err(err)
                                }
                            }
                        },
                        _ => {
                            let err = ParsingError{line: line_no, col: tokens[*cur].col,
                                                   len: tokens[*cur].len,
                                                   global: false,
                                                   etype: ParsingErrorType::ExpectingRightBracket };
                            Err(err)
                        }
                    }
                },
                _ => {
                    let err = ParsingError{ line: line_no,
                                            col: tokens[*cur].col,
                                            len: tokens[*cur].len,
                                            global: false,
                                            etype: ParsingErrorType::ExpectingLiteral};
                    Err(err)
                },
            }
        }
        _ => {
            let err = ParsingError { line: line_no,
                                     col: tokens[*cur].col,
                                     len: tokens[*cur].len,
                                     global: false,
                                     etype: ParsingErrorType::ExpectingOperand};
            Err(err)
        },
    }
    //Ok(ParsingInfo::new())
}

fn get_token_type(line_no: usize, tokens: &Vec<Token>, cur: usize) -> Result<&TokenType, ParsingError> {
    if cur >= tokens.len() {
        let err = ParsingError { line: line_no,
                                 col: tokens[tokens.len() - 1].col,
                                 len: tokens[tokens.len() - 1].len,
                                 global: false,
                                 etype: ParsingErrorType::EndOfTokens };
        return Err(err);
    }

    let ttype = &tokens[cur].ttype;
    return Ok(ttype);
}

fn check_comma(line_no: usize, tokens: &Vec<Token>,
               cur: &mut usize) -> Result<(), ParsingError> {
    let ttype = try!(get_token_type(line_no, tokens, *cur));
    match *ttype {
        TokenType::Comma => {
            *cur += 1;
            Ok(())
        },
        _ => {
            let err = ParsingError { line: line_no,
                                     col: tokens[*cur].col,
                                     len: tokens[*cur].len,
                                     global: false,
                                     etype: ParsingErrorType::ExpectingComma };
            Err(err)
        }
    }
}

fn check_end_of_line(line_no: usize, tokens: &Vec<Token>,
                     cur: &mut usize) -> Result<(), ParsingError> {
    if tokens.len() <= *cur {
        Ok(())
    } else {
        println!("{} {} {}", tokens[tokens.len() - 1].col, tokens[*cur].col, tokens[tokens.len()-1].len);
        let err = ParsingError { line: line_no,
                                 col: tokens[*cur].col,
                                 len: tokens[tokens.len()-1].col - tokens[*cur].col + tokens[tokens.len()-1].len,
                                 global: false,
                                 etype: ParsingErrorType::ExtraTokens };
        Err(err)
    }
}

fn parse_basic_opcode(line_no: usize,
                      tokens: &Vec<Token>,
                      cur: &mut usize,
                      cpu: &mut PCPU) -> Result<(), ParsingError> {
    let ttype = try!(get_token_type(line_no, tokens, *cur));
    match ttype {
        &TokenType::BasicOpcode(opcode) => {
            *cur += 1;
            let b = try!(parse_value(line_no, tokens, cur, cpu, true));

            try!(check_comma(line_no, tokens, cur));

            let a = try!(parse_value(line_no, tokens, cur, cpu, false));

            try!(check_end_of_line(line_no, tokens, cur));

            // Pack byte as aaaaaabbbbbooooo
            let byte: u16 = (a.operand << 10) + (b.operand << 5) + opcode as u16;
            cpu.mem[cpu.pc as usize] = byte;
            cpu.pc += 1;

            match a.extra_byte {
                Some(byte) => {
                    cpu.mem[cpu.pc as usize] = byte;
                    if a.unassigned {
                        let ul = UnassignedLabel{addr: cpu.pc, label: byte, offset: a.offset};
                        cpu.unassigned_addresses.push(ul);
                    }

                    cpu.pc += 1;
                },
                None => {},
            }

            match b.extra_byte {
                Some(byte) => {
                    cpu.mem[cpu.pc as usize] = byte;
                    if b.unassigned {
                        let ul = UnassignedLabel{addr: cpu.pc, label: byte, offset: b.offset};
                        cpu.unassigned_addresses.push(ul);
                    }
                    cpu.pc += 1;
                },
                None => {},
            }
            //let a = parse_value(tokens, &mut cur);
        }
        _ => {
            // Should be impossible actually
        }
    }

    Ok(())
}

fn parse_special_opcode(line_no: usize,
                        tokens: &Vec<Token>,
                        cur: &mut usize,
                        cpu: &mut PCPU) -> Result<(), ParsingError> {
    let ttype = try!(get_token_type(line_no, tokens, *cur));
    match ttype {
        &TokenType::SpecialOpcode(opcode) => {
            *cur += 1;

            let a = try!(parse_value(line_no, tokens, cur, cpu, false));

            try!(check_end_of_line(line_no, tokens, cur));

            // Pack byte as aaaaaaooooo00000
            let byte: u16 = (a.operand << 10) + ((opcode as u16) << 5);
            cpu.mem[cpu.pc as usize] = byte;
            cpu.pc += 1;

            match a.extra_byte {
                Some(byte) => {
                    cpu.mem[cpu.pc as usize] = byte;
                    if a.unassigned {
                        let ul = UnassignedLabel{addr: cpu.pc, label: byte, offset: a.offset};
                        cpu.unassigned_addresses.push(ul);
                    }
                    cpu.pc += 1;
                },
                None => {},
            }
        }
        _ => {
            // Should be impossible actually
        }
    }

    Ok(())
}

fn parse_data_opcode(line_no: usize,
                     tokens: &Vec<Token>,
                     cur: &mut usize,
                     cpu: &mut PCPU) -> Result<ParsingInfo, ParsingError> {

    // Eat the DAT opcode
    *cur += 1;

    let mut first = true;
    loop {
        if *cur >= tokens.len() {
            break;
        }

        if !first {
            try!(check_comma(line_no, tokens, cur));
        }
        first = false;

        let ttype = try!(get_token_type(line_no, tokens, *cur));
        match ttype {
            &TokenType::NumericLiteral(value) => {
                *cur += 1;
                cpu.mem[cpu.pc as usize] = value;
                cpu.pc += 1;
            },
            &TokenType::StringLiteral(id) => {
                *cur += 1;
                for c in cpu.string_literals[id as usize].chars() {
                    cpu.mem[cpu.pc as usize] = c as u16;
                    cpu.pc += 1;
                }
            }
            _ => {
                let err = ParsingError { line: line_no,
                                         col: tokens[*cur].col,
                                         len: tokens[*cur].len,
                                         global: false,
                                         etype: ParsingErrorType::ExpectingLiteral };
                return Err(err);
            }
        }
    }
    Ok(ParsingInfo::new())
}

fn parse_line(line_no: usize, tokens: &Vec<Token>, cpu: &mut PCPU, cur: &mut usize) -> Result<(), ParsingError> {
    if *cur >= tokens.len() {
        return Ok(());
    }
    match try!(get_token_type(line_no, tokens, *cur)) {
        &TokenType::BasicOpcode(_) => {
            try!(parse_basic_opcode(line_no, tokens, cur, cpu));
            Ok(())
        },
        &TokenType::SpecialOpcode(_) => {
            try!(parse_special_opcode(line_no, tokens, cur, cpu));
            Ok(())
        },
        &TokenType::DataOpcode => {
            try!(parse_data_opcode(line_no, tokens, cur, cpu));
            Ok(())
        },
        &TokenType::Colon => {
            *cur += 1;
            match try!(get_token_type(line_no, tokens, *cur)) {
                &TokenType::Label(v) => {
                    let label = cpu.pc;
                    if !cpu.labels.contains_key(&v) {
                        cpu.labels.insert(v, label);
                    }

                    // Allow more parsing after a label
                    *cur += 1;
                    Ok(try!(parse_line(line_no, tokens, cpu, cur)))
                },
                _ => {
                    let err = ParsingError { line: line_no,
                                             col: tokens[*cur].col,
                                             len: tokens[*cur].len,
                                             global: false,
                                             etype: ParsingErrorType::ExpectingLabel };
                    Err(err)
                }
            }
        },
        _ => {
            let err = ParsingError { line: line_no,
                                     col: tokens[*cur].col,
                                     len: tokens[*cur].len,
                                     global: false,
                                     etype: ParsingErrorType::IllegalLineStart };
            Err(err)
        },
    }
}

pub fn parse(lines: &Vec<String>, cpu: &mut PCPU) -> Result<(), ParsingError> {
    // We're going to use the PC register to keep track of the position
    cpu.pc = 0;

    let mut line_no = 0usize;
    for line in lines.iter() {
        let l = &line[..];

        if l.len() == 0 {
            line_no += 1;
            continue;
        }

        let tokens = try!(tokenize(line_no, l, cpu));
        let mut cur = 0;
        try!(parse_line(line_no, &tokens, cpu, &mut cur));

        line_no += 1;
    }

    for ul in cpu.unassigned_addresses.iter() {
        // Note, the label ID was stored temporarily in the memory
        // address.
        //let label_id = cpu.mem[ul.addr as usize];
        match cpu.labels.get(&ul.label) {
            Some(v) => {
                println!("here: {} {} {}", ul.addr, ul.label, ul.offset);
                cpu.mem[ul.addr as usize] = ul.offset + *v;
            },
            None => {
                let err = match cpu.label_first_line_error.get(&ul.label) {
                    Some(s) => *s,
                    None => panic!("Unknown error"),
                };

                return Err(err);
            }
        }
    }

    Ok(())
}

pub fn print_parse_error(cpu: &PCPU, line: &str, err: ParsingError) {
    println!("Parse failed");
    if err.global {
        println!("\x1b[1;31merror:\x1b[1;37m {}\x1b[0m", format_error(&err.etype, cpu));
    } else {
        println!(":{} \x1b[1;31merror:\x1b[1;37m {}\x1b[0m", err.line+1, format_error(&err.etype, cpu));
        println!("{}", line);
        for _ in 0..err.col {
            print!(" ");
        }
        print!("\x1b[1;31m^");
        for _ in 1..err.len {
            print!("~");
        }
        println!("\x1b[0m");
    }
}
