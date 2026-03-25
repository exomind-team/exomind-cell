//! Statistical analysis: bootstrap CI, Mann-Whitney U, effect sizes.

use rand::prelude::*;

/// Result of comparing two groups statistically.
pub struct GroupComparison {
    pub metric_name: String,
    pub exp_mean: f64,
    pub exp_sd: f64,
    pub ctrl_mean: f64,
    pub ctrl_sd: f64,
    pub mean_diff: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub u_statistic: f64,
    pub p_value: f64,      // Mann-Whitney p
    pub rank_biserial: f64,
    pub cohens_d: f64,
    pub ks_d: f64,         // KS D statistic
    pub ks_p: f64,         // KS p-value
}

impl GroupComparison {
    pub fn to_markdown_row(&self) -> String {
        format!(
            "| {} | {:.3} +/- {:.3} | {:.3} +/- {:.3} | {:.3} | [{:.3}, {:.3}] | {:.4} | {:.3} | {:.3} | {:.3} | {:.4} |",
            self.metric_name,
            self.exp_mean, self.exp_sd,
            self.ctrl_mean, self.ctrl_sd,
            self.mean_diff,
            self.ci_lower, self.ci_upper,
            self.p_value,
            self.rank_biserial, self.cohens_d,
            self.ks_d, self.ks_p,
        )
    }
}

/// Compute mean of a slice.
fn mean(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    data.iter().sum::<f64>() / data.len() as f64
}

/// Compute standard deviation (sample).
fn std_dev(data: &[f64]) -> f64 {
    if data.len() < 2 { return 0.0; }
    let m = mean(data);
    let var = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    var.sqrt()
}

/// Bootstrap 95% confidence interval for the mean difference (exp - ctrl).
/// Resamples 10000 times with replacement.
pub fn bootstrap_ci(exp: &[f64], ctrl: &[f64], n_resamples: usize, seed: u64) -> (f64, f64) {
    let mut rng = StdRng::seed_from_u64(seed);
    let n_exp = exp.len();
    let n_ctrl = ctrl.len();
    let mut diffs: Vec<f64> = Vec::with_capacity(n_resamples);

    for _ in 0..n_resamples {
        // Resample with replacement
        let exp_sample: f64 = (0..n_exp)
            .map(|_| exp[rng.gen_range(0..n_exp)])
            .sum::<f64>() / n_exp as f64;
        let ctrl_sample: f64 = (0..n_ctrl)
            .map(|_| ctrl[rng.gen_range(0..n_ctrl)])
            .sum::<f64>() / n_ctrl as f64;
        diffs.push(exp_sample - ctrl_sample);
    }

    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lower_idx = (0.025 * n_resamples as f64) as usize;
    let upper_idx = (0.975 * n_resamples as f64) as usize;
    (diffs[lower_idx], diffs[upper_idx.min(diffs.len() - 1)])
}

/// Mann-Whitney U test (two-tailed).
/// Returns (U statistic, approximate p-value).
pub fn mann_whitney_u(exp: &[f64], ctrl: &[f64]) -> (f64, f64) {
    let n1 = exp.len() as f64;
    let n2 = ctrl.len() as f64;

    // Count how many times exp[i] > ctrl[j]
    let mut u: f64 = 0.0;
    for &e in exp {
        for &c in ctrl {
            if e > c {
                u += 1.0;
            } else if (e - c).abs() < f64::EPSILON {
                u += 0.5;
            }
        }
    }

    // Normal approximation for p-value (valid for n >= 20)
    let mu = n1 * n2 / 2.0;
    let sigma = (n1 * n2 * (n1 + n2 + 1.0) / 12.0).sqrt();
    let z = if sigma > 0.0 { (u - mu) / sigma } else { 0.0 };

    // Two-tailed p-value from z using approximation
    let p = 2.0 * normal_cdf_complement(z.abs());

    (u, p)
}

/// Complement of standard normal CDF: P(Z > z).
/// Uses Abramowitz and Stegun approximation.
fn normal_cdf_complement(z: f64) -> f64 {
    if z < 0.0 { return 1.0 - normal_cdf_complement(-z); }
    let t = 1.0 / (1.0 + 0.2316419 * z);
    let d = 0.3989422804014327; // 1/sqrt(2*pi)
    let p = d * (-z * z / 2.0).exp();
    let poly = t * (0.319381530
        + t * (-0.356563782
        + t * (1.781477937
        + t * (-1.821255978
        + t * 1.330274429))));
    p * poly
}

/// Two-sample Kolmogorov-Smirnov test.
/// Returns (D statistic, approximate p-value).
pub fn ks_test(a: &[f64], b: &[f64]) -> (f64, f64) {
    let mut a_sorted: Vec<f64> = a.to_vec();
    let mut b_sorted: Vec<f64> = b.to_vec();
    a_sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b_sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());

    let na = a_sorted.len() as f64;
    let nb = b_sorted.len() as f64;

    // Merge and compute max CDF difference
    let mut i = 0usize;
    let mut j = 0usize;
    let mut d_max: f64 = 0.0;

    while i < a_sorted.len() || j < b_sorted.len() {
        let cdf_a = i as f64 / na;
        let cdf_b = j as f64 / nb;
        d_max = d_max.max((cdf_a - cdf_b).abs());

        let val_a = if i < a_sorted.len() { a_sorted[i] } else { f64::INFINITY };
        let val_b = if j < b_sorted.len() { b_sorted[j] } else { f64::INFINITY };

        if val_a <= val_b { i += 1; } else { j += 1; }
    }
    // Check final point
    d_max = d_max.max((1.0 - j as f64 / nb).abs());

    // Approximate p-value using asymptotic formula
    let n_eff = (na * nb) / (na + nb);
    let lambda = (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt()) * d_max;
    // Kolmogorov distribution approximation
    let p = 2.0 * (-2.0 * lambda * lambda).exp();
    let p = p.max(0.0).min(1.0);

    (d_max, p)
}

/// Cohen's d effect size.
pub fn cohens_d(exp: &[f64], ctrl: &[f64]) -> f64 {
    let m1 = mean(exp);
    let m2 = mean(ctrl);
    let s1 = std_dev(exp);
    let s2 = std_dev(ctrl);
    let n1 = exp.len() as f64;
    let n2 = ctrl.len() as f64;
    // Pooled standard deviation
    let sp = (((n1 - 1.0) * s1 * s1 + (n2 - 1.0) * s2 * s2) / (n1 + n2 - 2.0)).sqrt();
    if sp > 0.0 { (m1 - m2) / sp } else { 0.0 }
}

/// Run full comparison between two groups.
pub fn compare_groups(name: &str, exp: &[f64], ctrl: &[f64]) -> GroupComparison {
    let exp_mean = mean(exp);
    let ctrl_mean = mean(ctrl);
    let (ci_lower, ci_upper) = bootstrap_ci(exp, ctrl, 10_000, 12345);
    let (u_stat, p_val) = mann_whitney_u(exp, ctrl);
    let n1 = exp.len() as f64;
    let n2 = ctrl.len() as f64;
    let rb = 1.0 - 2.0 * u_stat / (n1 * n2); // rank-biserial correlation

    let (ks_d, ks_p) = ks_test(exp, ctrl);

    GroupComparison {
        metric_name: name.to_string(),
        exp_mean,
        exp_sd: std_dev(exp),
        ctrl_mean,
        ctrl_sd: std_dev(ctrl),
        mean_diff: exp_mean - ctrl_mean,
        ci_lower,
        ci_upper,
        u_statistic: u_stat,
        p_value: p_val,
        rank_biserial: rb,
        cohens_d: cohens_d(exp, ctrl),
        ks_d,
        ks_p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_ci_same_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (lo, hi) = bootstrap_ci(&data, &data, 1000, 42);
        // CI of difference should contain 0
        assert!(lo <= 0.0 && hi >= 0.0,
            "CI [{}, {}] should contain 0 for identical groups", lo, hi);
    }

    #[test]
    fn test_mann_whitney_different_groups() {
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let low = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (u, p) = mann_whitney_u(&high, &low);
        assert_eq!(u, 25.0, "U should be n1*n2=25 when all exp > ctrl");
        assert!(p < 0.05, "p-value {} should be significant", p);
    }

    #[test]
    fn test_cohens_d_large_effect() {
        let a = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let d = cohens_d(&a, &b);
        assert!(d > 3.0, "Cohen's d {} should be very large for separated groups", d);
    }
}
