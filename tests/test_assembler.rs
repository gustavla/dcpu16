use dcpu16::assembler::{PCPU,parse};

fn test_case(ll: &[&str], mem: &[u16]) {
    let mut lines: Vec<String> = Vec::new();
    for l in ll {
        lines.push(l.to_string());
    }
    let mut cpu = PCPU::new();
    assert!(parse(&lines, &mut cpu).is_ok());
    let mut i = 0;
    for m in mem {
        //assert_eq!(cpu.mem[i], *m);
        assert!(cpu.mem[i] == *m, "line {}: {:x} != {:x}", i, cpu.mem[i], *m);
        i += 1;
    }
    // TODO: Make sure the rest of the memory is empty
    assert_eq!(cpu.mem[i], 0);
}

#[test]
fn test_assembler_data_literals() {
    test_case(&["DAT 0x1234"], &[0x1234]);
    test_case(&["DAT 1234"], &[1234]);
    test_case(&["DAT 0, 1, 2"], &[0, 1, 2]);
    test_case(&["DAT \"Hello\""], &[0x0048, 0x0065, 0x006c, 0x006c, 0x006f]);
    test_case(&["DAT \"A\nB\", 0x1000"], &[0x0041, 0x000a, 0x0042, 0x1000]);
    test_case(&["DAT \"\t\n\""], &[0x0009, 0x000a]);
    test_case(&["DAT \"A; B\""], &[0x0041, 0x003b, 0x0020, 0x0042]); // not a comment
}

#[test]
fn test_assembler_op_set() {
    test_case(&["SET A, 0x30"], &[0x7c01, 0x0030]);
    test_case(&["SET [0x1000], 0x20"], &[0x7fc1, 0x0020, 0x1000]);
    test_case(&["SET I, 10"], &[0xacc1]);
    test_case(&["SET I, J"], &[0x1cc1]);
    test_case(&["SET [B], C"], &[0x0921]);
}

#[test]
fn test_assembler_unsigned_ops() {
    test_case(&["ADD [X], [Y]"], &[0x3162]);
    test_case(&["SUB [0], Z"], &[0x17c3]);
    test_case(&["MUL [1234], 3"], &[0x93c4, 0x04d2]);
    test_case(&["DIV A, J"], &[0x1c06]);
    test_case(&["MOD [B], 10"], &[0xad28]);
}

#[test]
fn test_assembler_signed_ops() {
    test_case(&["MLI X, 0xffff"], &[0x8065]);
    test_case(&["DVI B, [J]"], &[0x3c27]);
    test_case(&["MDI C, 31"], &[0x7c49, 0x001f]);
}

#[test]
fn test_assembler_binary_ops() {
    test_case(&["AND A, A"], &[0x000a]);
    test_case(&["BOR [5], [5]"], &[0x7bcb, 0x0005, 0x0005]);
    test_case(&["XOR J, [A]"], &[0x20ec]);
}

#[test]
fn test_assembler_shift_ops() {
    test_case(&["SHR A, B"], &[0x040d]);
    test_case(&["ASR [0x00FF], C"], &[0x0bce, 0x00ff]);
    test_case(&["SHL [300], C"], &[0x0bcf, 0x012c]);
}

#[test]
fn test_assembler_control_flow() {
    test_case(&["IFB A, B"], &[0x0410]);
    test_case(&["IFC A, B"], &[0x0411]);
    test_case(&["IFE A, B"], &[0x0412]);
    test_case(&["IFN A, B"], &[0x0413]);
    test_case(&["IFG A, B"], &[0x0414]);
    test_case(&["IFA A, B"], &[0x0415]);
    test_case(&["IFL A, B"], &[0x0416]);
    test_case(&["IFU A, B"], &[0x0417]);
}

#[test]
fn test_assembler_flag_ops() {
    test_case(&["ADX B, A"], &[0x003a]);
    test_case(&["SBX B, A"], &[0x003b]);
    test_case(&["STI B, A"], &[0x003e]);
    test_case(&["STD B, A"], &[0x003f]);
}

#[test]
fn test_assembler_special_ops() {
    test_case(&["JSR A"], &[0x0020]);
    test_case(&["INT A"], &[0x0100]);
    test_case(&["IAG A"], &[0x0120]);
    test_case(&["IAS [0x1000]"], &[0x7940, 0x1000]);
    test_case(&["RFI A"], &[0x0160]);
    test_case(&["IAQ A"], &[0x0180]);
}

#[test]
fn test_assembler_hardware_ops() {
    test_case(&["HWN A"], &[0x0200]);
    test_case(&["HWQ A"], &[0x0220]);
    test_case(&["HWI A"], &[0x0240]);
}

#[test]
fn test_assembler_basic_registers() {
    test_case(&["SET PC, 0"], &[0x8781]);
    test_case(&["ADD SP, 1"], &[0x8b62]);
    test_case(&["ADD I, EX"], &[0x74c2]);
}

#[test]
fn test_assembler_push_pop() {
    test_case(&["SET PUSH, A"], &[0x0301]);
    test_case(&["SET A, POP"], &[0x6001]);
}

#[test]
fn test_assembler_peek() {
    test_case(&["SET A, PEEK"], &[0x6401]);
}

#[test]
fn test_assembler_pick() {
    test_case(&["SET A, PICK 5"], &[0x6801, 0x0005]);
    test_case(&["SET A, PICK 0x1234"], &[0x6801, 0x1234]);
    test_case(&["SET PICK 0xffff, A"], &[0x0341, 0xffff]);
    test_case(&["SET PICK 0xffff, [0x1000]"], &[0x7b41, 0x1000, 0xffff]);
    test_case(&["AIS PICK 1000"], &[0x6940, 0x03e8]);
}

#[test]
fn test_assembler_literal_addition() {
    test_case(&["SET A, [I + 0x1000]"], &[0x5801, 0x1000]);
    test_case(&["SET A, [0x1000 + I]"], &[0x5801, 0x1000]);
    test_case(&["SET A, [J+1]"], &[0x5c01, 0x0001]);
}

#[test]
fn test_assembler_label_addition() {
    test_case(&["DAT 1",
                ":label",
                "DAT 2",
                "SET A, [label + I]"],
              &[1, 2, 0x5801, 0x0001, 0x0004, 0x8421]);
    test_case(&["DAT 1",
                ":label",
                "DAT 2",
                "SET A, [I + label]"],
              &[1, 2, 0x5801, 0x0001, 0x0004, 0x8421]);
}

#[test]
fn test_assembler_whitespace() {
    test_case(&["      SET   A   ,  [  B  ]   "], &[0x2401]);
    test_case(&["SET A,[B]"], &[0x2401]);
}

#[test]
fn test_assembler_case_correctness_instructions() {
    // Instructions and registers are case insensitive
    test_case(&["Set a, [B]"], &[0x2401]);
    test_case(&["set a, [b]"], &[0x2401]);
    test_case(&["set a, pop"], &[0x6001]);
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_assembler_case_correctness_labels() {
    // Labels are case sensitive
    test_case(&[":label SET A, LABEL"], &[0x8401]);
}

#[test]
fn test_assembler_case_correctness_string_literals() {
    // String literals are case sensitive
    test_case(&["DAT \"Aa\""], &[0x0041, 0x0061]);
    test_case(&["Dat \"Aa\""], &[0x0041, 0x0061]);
}

#[test]
fn test_assembler_comments() {
    test_case(&["SET A, [B]  ; This is a comment"], &[0x2401]);
    test_case(&["; Comments", "; More comments", "DAT 1"], &[1]);
    test_case(&[":label;comment", "SET A, label"], &[0x8401]);
}

#[test]
fn test_assembler_simple_labels() {
    test_case(&[":mylabel", "SET A, mylabel"], &[0x8401]);
    // TODO: Both should be fine
    //test_case(&["DAT 1", ":mylabel", "SET A, mylabel"], &[0x0001, 0x7c01, 0x0001]);
    test_case(&["DAT 1", ":mylabel", "SET A, mylabel"], &[0x0001, 0x8801]);
    test_case(&[":begin DAT \"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"", "SET A, begin"],
              &[0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x8401]);
    test_case(&["DAT \"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"", ":self SET A, self"],
              &[0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078, 0x0078,
                0x7c01, 0x0020]);
}

#[test]
fn test_assembler_future_labels() {
    test_case(&["SET A, future",
                  ":future",
                  "SET B, 0"],
                &[0x7c01, 0x0002, 0x8421]);
    test_case(&["SET A, future",
                  "SET B, future",
                  ":future",
                  "SET B, 0"],
                &[0x7c01, 0x0004, 0x7c21, 0x0004, 0x8421]);
}

// TODO: Expect parse error
#[test]
#[should_panic(expected = "assertion failed")]
fn test_assembler_unknown_label() {
    test_case(&["SET A, label"], &[0]);
}
