//! D0 instruction set — minimal but sufficient for operational closure.

use rand::prelude::*;
use std::fmt;

/// The D0 instruction set — minimal but sufficient for operational closure.
///
/// Design rationale: EAT + REFRESH are *necessary* for survival (operational closure).
/// DIVIDE is *optional* — reproduction is not the core of life, persistence is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Nop,            // 0x00: No operation
    Inc(u8),        // 0x01: Register += 1
    Dec(u8),        // 0x02: Register -= 1
    Cmp(u8, u8),    // 0x03: Compare two registers, set flag in R3
    Jmp(i16),       // 0x04: Unconditional jump (relative offset)
    Jnz(i16),       // 0x05: Jump if R0 != 0
    Load(u8, u8),   // 0x06: Load from data[reg] to register
    Store(u8, u8),  // 0x07: Store register to data[reg]
    SenseSelf(u8),  // 0x08: Read own energy into register
    Eat,            // 0x09: Consume food from pool
    Refresh,        // 0x0A: Reset freshness (whole organism, simplified)
    Divide,         // 0x0B: Self-replicate with mutation
}

impl Instruction {
    /// Total number of instruction variants (for random generation/mutation).
    pub const VARIANT_COUNT: usize = 12;

    /// Generate a random instruction.
    pub fn random(rng: &mut impl Rng) -> Self {
        let variant = rng.gen_range(0..Self::VARIANT_COUNT);
        Self::from_variant(variant, rng)
    }

    /// Create instruction from variant index with random operands.
    pub fn from_variant(variant: usize, rng: &mut impl Rng) -> Self {
        match variant {
            0 => Instruction::Nop,
            1 => Instruction::Inc(rng.gen_range(0..8)),
            2 => Instruction::Dec(rng.gen_range(0..8)),
            3 => Instruction::Cmp(rng.gen_range(0..8), rng.gen_range(0..8)),
            4 => Instruction::Jmp(rng.gen_range(-16..16)),
            5 => Instruction::Jnz(rng.gen_range(-16..16)),
            6 => Instruction::Load(rng.gen_range(0..8), rng.gen_range(0..8)),
            7 => Instruction::Store(rng.gen_range(0..8), rng.gen_range(0..8)),
            8 => Instruction::SenseSelf(rng.gen_range(0..8)),
            9 => Instruction::Eat,
            10 => Instruction::Refresh,
            11 => Instruction::Divide,
            _ => Instruction::Nop,
        }
    }

    /// Mutate this instruction into a random different one.
    pub fn mutate(&self, rng: &mut impl Rng) -> Self {
        loop {
            let new = Self::random(rng);
            if std::mem::discriminant(&new) != std::mem::discriminant(self) {
                return new;
            }
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Inc(r) => write!(f, "INC r{}", r),
            Instruction::Dec(r) => write!(f, "DEC r{}", r),
            Instruction::Cmp(a, b) => write!(f, "CMP r{} r{}", a, b),
            Instruction::Jmp(off) => write!(f, "JMP {}", off),
            Instruction::Jnz(off) => write!(f, "JNZ {}", off),
            Instruction::Load(r, idx) => write!(f, "LOAD r{} d{}", r, idx),
            Instruction::Store(r, idx) => write!(f, "STORE r{} d{}", r, idx),
            Instruction::SenseSelf(r) => write!(f, "SENSE_SELF r{}", r),
            Instruction::Eat => write!(f, "EAT"),
            Instruction::Refresh => write!(f, "REFRESH"),
            Instruction::Divide => write!(f, "DIVIDE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_instruction_coverage() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            let instr = Instruction::random(&mut rng);
            seen.insert(std::mem::discriminant(&instr));
        }
        assert_eq!(seen.len(), Instruction::VARIANT_COUNT,
            "All instruction variants should be generated");
    }

    #[test]
    fn test_mutate_produces_different_variant() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = Instruction::Eat;
        for _ in 0..100 {
            let mutated = original.mutate(&mut rng);
            assert_ne!(
                std::mem::discriminant(&original),
                std::mem::discriminant(&mutated),
                "Mutated instruction should be a different variant"
            );
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Instruction::Nop), "NOP");
        assert_eq!(format!("{}", Instruction::Eat), "EAT");
        assert_eq!(format!("{}", Instruction::Inc(3)), "INC r3");
        assert_eq!(format!("{}", Instruction::Jmp(-5)), "JMP -5");
    }
}
