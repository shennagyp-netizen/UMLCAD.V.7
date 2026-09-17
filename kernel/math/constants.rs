//! Mathematical constants used by the UMLCAD authority.
//!
//! Keep named constants here instead of scattering decimal approximations
//! through numerical algorithms. Geometry remains `f64` unless an operation
//! explicitly states otherwise.

pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = std::f64::consts::TAU;
pub const E: f64 = std::f64::consts::E;
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
pub const SQRT_3: f64 = 1.732_050_807_568_877_293_527_446_341_505_872_366_942_805_253_810;
pub const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;
pub const QUARTER_PI: f64 = std::f64::consts::FRAC_PI_4;
pub const MACHINE_EPSILON: f64 = f64::EPSILON;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_standard_f64_values() {
        assert_eq!(PI, std::f64::consts::PI);
        assert_eq!(TAU, 2.0 * PI);
        assert_eq!(E, std::f64::consts::E);
        assert!((SQRT_2 * SQRT_2 - 2.0).abs() < 4.0 * MACHINE_EPSILON);
        assert!((SQRT_3 * SQRT_3 - 3.0).abs() < 8.0 * MACHINE_EPSILON);
        assert_eq!(HALF_PI, PI * 0.5);
        assert_eq!(QUARTER_PI, PI * 0.25);
        assert_eq!(MACHINE_EPSILON, f64::EPSILON);
    }
}
