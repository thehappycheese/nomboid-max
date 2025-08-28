pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    use std::f32::consts::PI;
   
    let diff = (b - a + PI).rem_euclid(2.0 * PI) - PI;
    a + diff * t
}