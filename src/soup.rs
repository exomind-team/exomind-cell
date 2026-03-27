//! Soup view helpers for GUI-only layout/render state.
//! 汤视图辅助层：只服务 GUI，可视化坐标不参与 VM 语义。

use std::collections::{HashMap, HashSet};

use crate::cell_vm::CellWorld;

#[derive(Debug, Clone)]
pub struct SoupParticle {
    pub organism_id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone)]
pub struct SoupRenderItem {
    pub organism_id: u64,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub energy_ratio: f32,
    pub freshness_ratio: f32,
    pub cell_count: usize,
    pub generation: u32,
}

#[derive(Debug, Default)]
pub struct SoupLayout {
    particles: HashMap<u64, SoupParticle>,
}

fn stable_unit(id: u64, salt: u64) -> f32 {
    // SplitMix-style hash for deterministic pseudo-random seeds.
    // SplitMix 风格哈希：给 organism 一个稳定但不影响 VM 的显示坐标。
    let mut z = id.wrapping_add(salt).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let normalized = (z as f64) / (u64::MAX as f64);
    normalized as f32
}

fn initial_particle(organism_id: u64) -> SoupParticle {
    let x = 0.1 + stable_unit(organism_id, 0xA11C_E001) * 0.8;
    let y = 0.1 + stable_unit(organism_id, 0x5EED_F00D) * 0.8;
    SoupParticle {
        organism_id,
        x,
        y,
        vx: 0.0,
        vy: 0.0,
    }
}

impl SoupLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sync_from_world(&mut self, world: &CellWorld) {
        let alive_ids: HashSet<u64> = world
            .organisms
            .iter()
            .filter(|org| org.alive)
            .map(|org| org.id)
            .collect();

        self.particles
            .retain(|organism_id, _| alive_ids.contains(organism_id));

        for organism_id in alive_ids {
            self.particles
                .entry(organism_id)
                .or_insert_with(|| initial_particle(organism_id));
        }
    }

    pub fn step(&mut self, world: &CellWorld, dt: f32) {
        self.sync_from_world(world);

        let ids: Vec<u64> = world
            .organisms
            .iter()
            .filter(|org| org.alive)
            .map(|org| org.id)
            .collect();

        if ids.is_empty() {
            return;
        }

        let mut acceleration: HashMap<u64, (f32, f32)> =
            ids.iter().copied().map(|id| (id, (0.0, 0.0))).collect();
        let repel_strength = 0.0018_f32;
        let center_strength = 0.42_f32;
        let damping = 0.92_f32;

        for (index, left_id) in ids.iter().enumerate() {
            let Some(left) = self.particles.get(left_id) else {
                continue;
            };
            for right_id in ids.iter().skip(index + 1) {
                let Some(right) = self.particles.get(right_id) else {
                    continue;
                };

                let dx = right.x - left.x;
                let dy = right.y - left.y;
                let dist_sq = (dx * dx + dy * dy).max(0.0001);
                let dist = dist_sq.sqrt();
                let force = repel_strength / dist_sq;
                let fx = force * dx / dist;
                let fy = force * dy / dist;

                if let Some((ax, ay)) = acceleration.get_mut(left_id) {
                    *ax -= fx;
                    *ay -= fy;
                }
                if let Some((ax, ay)) = acceleration.get_mut(right_id) {
                    *ax += fx;
                    *ay += fy;
                }
            }
        }

        for organism_id in ids {
            let Some(particle) = self.particles.get_mut(&organism_id) else {
                continue;
            };
            let (mut ax, mut ay) = acceleration.get(&organism_id).copied().unwrap_or((0.0, 0.0));
            ax += (0.5 - particle.x) * center_strength;
            ay += (0.5 - particle.y) * center_strength;

            particle.vx = (particle.vx + ax * dt) * damping;
            particle.vy = (particle.vy + ay * dt) * damping;
            particle.x += particle.vx * dt;
            particle.y += particle.vy * dt;

            if particle.x < 0.02 {
                particle.x = 0.02;
                particle.vx *= -0.3;
            } else if particle.x > 0.98 {
                particle.x = 0.98;
                particle.vx *= -0.3;
            }

            if particle.y < 0.02 {
                particle.y = 0.02;
                particle.vy *= -0.3;
            } else if particle.y > 0.98 {
                particle.y = 0.98;
                particle.vy *= -0.3;
            }
        }
    }

    pub fn snapshot(&self, world: &CellWorld) -> Vec<SoupRenderItem> {
        world.organisms
            .iter()
            .filter(|org| org.alive)
            .filter_map(|org| {
                let particle = self.particles.get(&org.id)?;
                let energy_capacity = org
                    .cells
                    .iter()
                    .filter(|cell| cell.is_energy())
                    .count()
                    .max(1) as f32
                    * world.config.cell_energy_max as f32;
                let energy_ratio = (org.total_energy().max(0) as f32 / energy_capacity).clamp(0.0, 1.0);
                let freshness_ratio =
                    (org.min_freshness() as f32 / world.config.freshness_max.max(1) as f32)
                        .clamp(0.0, 1.0);

                Some(SoupRenderItem {
                    organism_id: particle.organism_id,
                    x: particle.x.clamp(0.0, 1.0),
                    y: particle.y.clamp(0.0, 1.0),
                    radius: 6.0 + (org.cells.len() as f32).sqrt() * 2.2,
                    energy_ratio,
                    freshness_ratio,
                    cell_count: org.cells.len(),
                    generation: org.generation,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_vm::{cell_seed_a, cell_seed_b, CellConfig};

    fn seeded_world() -> CellWorld {
        let mut config = CellConfig::experimental();
        config.cell_energy_max = 50;
        config.food_per_tick = 500;
        let mut world = CellWorld::new(config.clone(), 42);
        world.add_organism(cell_seed_a(&config));
        world.add_organism(cell_seed_b(&config));
        world
    }

    #[test]
    fn test_soup_layout_initializes_one_particle_per_alive_organism() {
        let world = seeded_world();
        let alive = world.organisms.iter().filter(|org| org.alive).count();

        let mut layout = SoupLayout::new();
        layout.sync_from_world(&world);

        let snapshot = layout.snapshot(&world);
        assert_eq!(
            snapshot.len(),
            alive,
            "Soup snapshot should track one particle per alive organism",
        );
    }

    #[test]
    fn test_soup_layout_step_keeps_particles_inside_view_bounds() {
        let mut world = seeded_world();
        for _ in 0..30 {
            world.tick();
        }

        let mut layout = SoupLayout::new();
        layout.sync_from_world(&world);
        for _ in 0..32 {
            layout.step(&world, 0.016);
        }

        let snapshot = layout.snapshot(&world);
        assert!(
            !snapshot.is_empty(),
            "Soup snapshot should stay non-empty after layout stepping",
        );
        assert!(snapshot.iter().all(|item| (0.0..=1.0).contains(&item.x)));
        assert!(snapshot.iter().all(|item| (0.0..=1.0).contains(&item.y)));
    }
}
