#[cfg(test)]
mod tests {
    fn calculate_position(base: f64, max: f64, confidence_high: f64, confidence: f64) -> f64 {
        if confidence >= confidence_high {
            max
        } else if confidence >= 0.6 {
            max * 0.3
        } else {
            base
        }
    }

    #[test]
    fn test_position_sizing_high_confidence_uses_max() {
        let position = calculate_position(100.0, 1000.0, 0.8, 0.92);
        assert_eq!(position, 1000.0);
    }

    #[test]
    fn test_position_sizing_mid_confidence_uses_scaled_max() {
        let position = calculate_position(100.0, 1000.0, 0.8, 0.7);
        assert_eq!(position, 300.0);
    }

    #[test]
    fn test_position_sizing_low_confidence_uses_base() {
        let position = calculate_position(100.0, 1000.0, 0.8, 0.55);
        assert_eq!(position, 100.0);
    }
}
