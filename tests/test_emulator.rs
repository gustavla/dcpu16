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
