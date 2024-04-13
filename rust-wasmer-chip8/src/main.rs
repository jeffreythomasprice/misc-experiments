mod chip8;



use crate::chip8::instructions;

fn chip8_stuff(bytes: &[u8]) -> anyhow::Result<()> {
    println!("total length in bytes: {}", bytes.len());
    for i in (0..(bytes.len())).step_by(2) {
        let high = &bytes[i];
        let low = &bytes[i + 1];
        let byte_str = format!("{high:02X} {low:02X}");
        match instructions::Instruction::from_bytes(&bytes[i..(i + 2)])? {
            Some(i) => println!("{byte_str}  --  {i}"),
            None => println!("{byte_str}"),
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    chip8_stuff(&std::fs::read(
        "/home/jeff/scratch/emulator_resources/chip8/chip8-roms/demos/Zero Demo [zeroZshadow, 2007].ch8",
    )?)?;

    Ok(())
}
