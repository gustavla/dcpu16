use dcpu16::dcpu::DCPU;

#[test]
fn test_emulerator_basic() {
    {
        let mut cpu = DCPU::new();
        cpu.mem[0] = 0x8801; // SET A, 1
        cpu.tick();
        assert_eq!(cpu.reg[0], 1);
    }

    {
        let mut cpu = DCPU::new();
        cpu.mem[0] = 0x7c01; cpu.mem[1] = 0x0064; // SET A, 100
        cpu.mem[2] = 0x00e1; // SET J, A
        cpu.tick();
        assert_eq!(cpu.reg[0], 100);
        assert_eq!(cpu.reg[7], 0);
        cpu.tick();
        assert_eq!(cpu.reg[0], 100);
        assert_eq!(cpu.reg[7], 100);
    }
}

#[test]
fn add() {
    let mut cpu = DCPU::new();
    cpu.mem[0] = 0xac02; // ADD A, 1
    cpu.mem[1] = 0xac02; // ADD A, 1
    cpu.mem[2] = 0xac02; // ADD A, 1
    cpu.mem[3] = 0x7c02; // ADD A, -
    cpu.mem[4] = 1000;
    cpu.tick();
    assert_eq!(cpu.reg[0], 10);
    cpu.tick();
    assert_eq!(cpu.reg[0], 20);
    cpu.tick();
    assert_eq!(cpu.reg[0], 30);
    cpu.tick();
    assert_eq!(cpu.reg[0], 1030);
}

#[test]
fn sub() {
    let mut cpu = DCPU::new();
    cpu.mem[0] = 0x7c01; // SET A, -
    cpu.mem[1] = 10000;

    cpu.mem[2] = 0xac03; // SUB A, 10
    cpu.mem[3] = 0xac03; // SUB A, 10
    cpu.mem[4] = 0x7c03; // SUB A, -
    cpu.mem[5] = 1000;
    cpu.tick();
    assert_eq!(cpu.reg[0], 10000);
    cpu.tick();
    assert_eq!(cpu.reg[0],  9990);
    cpu.tick();
    assert_eq!(cpu.reg[0],  9980);
    cpu.tick();
    assert_eq!(cpu.reg[0],  8980);
}

#[test]
fn add_overflow() {
    let mut cpu = DCPU::new();
    cpu.mem[0] = 0x7c01; // SET A, -
    cpu.mem[1] = 0xfffe;

    cpu.mem[2] = 0x8802; // ADD A, 1
    cpu.mem[3] = 0x8802; // ADD A, 1
    cpu.mem[4] = 0x8802; // ADD A, 1
    cpu.mem[5] = 0x8802; // ADD A, 1
    cpu.tick();
    assert_eq!(cpu.reg[0], 0xfffe);
    assert_eq!(cpu.ex, 0);
    cpu.tick();
    assert_eq!(cpu.reg[0], 0xffff);
    assert_eq!(cpu.ex, 0);
    cpu.tick();
    assert_eq!(cpu.reg[0], 0);
    assert_eq!(cpu.ex, 1);
    cpu.tick();
    assert_eq!(cpu.reg[0], 1);
    assert_eq!(cpu.ex, 0);
}

#[test]
fn sub_underflow() {
    let mut cpu = DCPU::new();
    cpu.mem[0] = 0x8801; // SET A, 1
    cpu.mem[1] = 0x8803; // SUB A, 1
    cpu.mem[2] = 0x8803; // SUB A, 1
    cpu.mem[3] = 0x8003; // SUB A, -1
    cpu.tick();
    assert_eq!(cpu.reg[0], 1);
    assert_eq!(cpu.ex, 0);
    cpu.tick();
    assert_eq!(cpu.reg[0], 0);
    assert_eq!(cpu.ex, 0);
    cpu.tick();
    assert_eq!(cpu.reg[0], 0xffff);
    assert_eq!(cpu.ex, 0xffff);
    cpu.tick();
    assert_eq!(cpu.reg[0], 0);
    assert_eq!(cpu.ex, 0);
}
