# Knockout Analysis: Minimum Essential Instruction Set

Each Code cell replaced with NOP one at a time. Sandbox: 10k ticks, abundant food.

## Seed_A (minimal survival)

Code cells: 4, Total cells: 7

| Pos | Instruction | Result | Survival | Energy | Pop |
|-----|------------|--------|----------|--------|-----|
| 0 | EAT | lethal | 100 | 0 | 0 |
| 1 | LOAD r0 d0 | lethal | 100 | 0 | 0 |
| 2 | REFRESH | lethal | 254 | 0 | 0 |
| 3 | JMP -3 | neutral | 10000 | 98 | 1 |

Knockout Summary (4 code cells):
- Lethal: 3 (75%)
- Severe defect: 0 (0%)
- Mild defect: 0 (0%)
- Neutral: 1 (25%)
- Beneficial: 0 (0%)
- Essential (lethal+severe): 3 (75%)
- Minimum instruction set: 3 of 4 code cells are essential


## Seed_B (conditional divide)

Code cells: 9, Total cells: 14

| Pos | Instruction | Result | Survival | Energy | Pop |
|-----|------------|--------|----------|--------|-----|
| 0 | EAT | lethal | 62 | 0 | 0 |
| 1 | LOAD r0 d0 | lethal | 62 | 0 | 0 |
| 2 | REFRESH | neutral | 10000 | 1151 | 8 |
| 3 | SENSE_SELF r1 | lethal | 254 | 0 | 0 |
| 4 | CMP r1 r5 | lethal | 254 | 0 | 0 |
| 5 | JNZ 2 | lethal | 254 | 0 | 0 |
| 6 | JMP -6 | neutral | 10000 | 877 | 6 |
| 7 | DIVIDE | lethal | 254 | 0 | 0 |
| 8 | JMP -8 | neutral | 10000 | 1172 | 8 |

Knockout Summary (9 code cells):
- Lethal: 6 (67%)
- Severe defect: 0 (0%)
- Mild defect: 0 (0%)
- Neutral: 3 (33%)
- Beneficial: 0 (0%)
- Essential (lethal+severe): 6 (67%)
- Minimum instruction set: 6 of 9 code cells are essential


## Seed_G (GATE evaluation)

Code cells: 10, Total cells: 15

| Pos | Instruction | Result | Survival | Energy | Pop |
|-----|------------|--------|----------|--------|-----|
| 0 | SENSE_SELF r1 | neutral | 10000 | 950 | 10 |
| 1 | EAT | lethal | 100 | 0 | 0 |
| 2 | LOAD r0 d0 | lethal | 100 | 0 | 0 |
| 3 | SENSE_SELF r2 | lethal | 346 | 0 | 0 |
| 4 | CMP r2 r1 | lethal | 346 | 0 | 0 |
| 5 | STORE r0 d0 | lethal | 346 | 0 | 0 |
| 6 | GATE | neutral | 10000 | 950 | 10 |
| 7 | DIVIDE | lethal | 352 | 0 | 0 |
| 8 | REFRESH | beneficial | 10000 | 1075 | 26 |
| 9 | JMP -9 | neutral | 10000 | 968 | 15 |

Knockout Summary (10 code cells):
- Lethal: 6 (60%)
- Severe defect: 0 (0%)
- Mild defect: 0 (0%)
- Neutral: 3 (30%)
- Beneficial: 1 (10%)
- Essential (lethal+severe): 6 (60%)
- Minimum instruction set: 6 of 10 code cells are essential


