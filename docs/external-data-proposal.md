# External Data Ingestion Design Proposal

## Overview

Enable D0 VM organisms to interact with real-world data by converting external data into "food" that requires processing to digest. This creates a bridge between artificial life and practical computation.

## Architecture

```
External Data Source (file / API / sensor)
  |
  v
DataSource trait (impl for each source type)
  |
  v
Data Injector (runs every N ticks)
  |
  v
Encapsulated Food = data value wrapped in a "shell"
  |
  v
Food Pool (organisms EAT encapsulated food)
  |
  v
Organism must PROCESS (new instruction) to crack the shell
  |
  v
Correct processing = energy reward; incorrect = wasted effort
```

## DataSource Trait

```rust
trait DataSource {
    /// Get the next data item. Returns None if source exhausted.
    fn next_item(&mut self) -> Option<DataItem>;

    /// How many ticks between injections.
    fn injection_interval(&self) -> u64;
}

struct DataItem {
    input: Vec<u8>,      // The data to process
    expected: Vec<u8>,   // Expected output (for reward calculation)
    energy_reward: u8,   // Energy gained for correct processing
}
```

## MVP Data Source: CSV Number Sequence

Simplest possible source: a CSV file with rows of numbers.

```rust
struct CsvDataSource {
    rows: Vec<Vec<u8>>,
    cursor: usize,
}

impl DataSource for CsvDataSource {
    fn next_item(&mut self) -> Option<DataItem> {
        let row = self.rows.get(self.cursor)?;
        self.cursor += 1;
        Some(DataItem {
            input: row[..row.len()-1].to_vec(),  // all but last = input
            expected: vec![*row.last()?],         // last column = expected output
            energy_reward: 20,
        })
    }
    fn injection_interval(&self) -> u64 { 100 } // every 100 ticks
}
```

**Example CSV** (predict next number):
```
1,2,3
2,4,6
3,6,9
```
Input: [1,2], expected output: [3]. Organism must "compute" 1+2=3 (or learn the pattern).

## Encapsulated Food

```rust
struct EncapsulatedFood {
    shell: Vec<u8>,       // Input data (readable via SAMPLE-like instruction)
    kernel_energy: u8,    // Energy inside (only accessible after correct processing)
    expected: Vec<u8>,    // What the organism must output to crack the shell
}
```

**Placed in food pool** as a special food type. Regular food (unencapsulated) still exists for basic survival. Encapsulated food offers bonus energy but requires correct processing.

## Organism Interaction

### Reading input
- New instruction `READ_FOOD(r)`: reads byte from current encapsulated food's shell into register r
- Or: food input is placed into the organism's Stomach cells (different encoding than regular food)

### Writing output (answering)
- Organism uses `STORE` to write answer into Data cell(s)
- New instruction `SUBMIT`: compares Data cell contents with expected output
  - Match: kernel_energy deposited into Energy cells (big reward)
  - Mismatch: small energy penalty (wasted effort)

### Evolution pressure
- Organisms that evolve to correctly process data get more energy
- More energy = more DIVIDE = more offspring
- Natural selection for "intelligence" (correct data processing)

## Implementation Phases

### Phase 1: Static injection (MVP)
- Load CSV at startup
- Inject one encapsulated food every N ticks
- SUBMIT instruction checks answer
- No runtime data source changes

### Phase 2: Streaming
- DataSource trait with multiple implementations
- File watcher (re-read CSV when modified)
- API endpoint (HTTP GET for next data item)

### Phase 3: Bidirectional
- Organisms can EMIT results that are collected externally
- External system receives organism "answers"
- Feedback loop: external system adjusts difficulty based on accuracy

## Considerations

### Encoding
- How to represent multi-byte data in single-byte cells?
  - Option A: one cell per byte (simple, wastes space)
  - Option B: pack into registers (compact, needs more instructions)

### Difficulty scaling
- Start with trivial tasks (copy input to output)
- Gradually increase (add 1, XOR, pattern recognition)
- Match difficulty to organism capability (no point in unsolvable tasks)

### Reward structure
- Base survival food should always be available (organisms must survive)
- Encapsulated food is a bonus, not a requirement
- This ensures the experiment tests whether organisms CHOOSE to process data, not whether they're forced to

### Relation to D2 (Value Evaluation)
- Data processing creates a scenario where organisms must CHOOSE between:
  - Easy food (regular, always available)
  - Hard food (encapsulated, higher reward but requires processing)
- This is exactly the "value evaluation" that D1 (the next developmental stage) requires
- If organisms evolve to prefer encapsulated food when hungry and regular food when not, that's emergent value evaluation

## Not Addressed

- Multi-organism cooperation on data tasks
- Real-time data sources (sensors, stock prices)
- Security (preventing organisms from gaming the reward system)
- Scaling beyond simple byte-level computation
