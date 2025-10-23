//! Vector distance operators for SQL
//!
//! Implements PostgreSQL-compatible vector distance operators:
//! - `<->` : L2 distance (Euclidean)
//! - `<#>` : Negative inner product (for max similarity)
//! - `<=>` : Cosine distance

use crate::value::Value;
use crate::vector::VectorValue;
use anyhow::{anyhow, Result};

/// Vector distance operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorOperator {
    /// L2 distance: `<->`
    /// Formula: sqrt(sum((a[i] - b[i])^2))
    L2Distance,

    /// Negative inner product: `<#>`
    /// Formula: -sum(a[i] * b[i])
    /// Note: Negative to support ASC order (smaller = better)
    NegativeInnerProduct,

    /// Cosine distance: `<=>`
    /// Formula: 1 - (dot(a, b) / (||a|| * ||b||))
    CosineDistance,
}

impl VectorOperator {
    /// Parse operator from SQL symbol
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "<->" => Some(VectorOperator::L2Distance),
            "<#>" => Some(VectorOperator::NegativeInnerProduct),
            "<=>" => Some(VectorOperator::CosineDistance),
            _ => None,
        }
    }

    /// Get SQL symbol for operator
    pub fn to_symbol(&self) -> &'static str {
        match self {
            VectorOperator::L2Distance => "<->",
            VectorOperator::NegativeInnerProduct => "<#>",
            VectorOperator::CosineDistance => "<=>",
        }
    }

    /// Evaluate distance operator on two values
    ///
    /// # Arguments
    /// * `left` - Left operand (must be Vector)
    /// * `right` - Right operand (must be Vector)
    ///
    /// # Returns
    /// Float64 distance value
    pub fn evaluate(&self, left: &Value, right: &Value) -> Result<Value> {
        // Extract vectors from values
        let left_vec = match left {
            Value::Vector(v) => v,
            _ => return Err(anyhow!("Left operand of {} must be a vector", self.to_symbol())),
        };

        let right_vec = match right {
            Value::Vector(v) => v,
            _ => return Err(anyhow!("Right operand of {} must be a vector", self.to_symbol())),
        };

        // Compute distance based on operator
        let distance = match self {
            VectorOperator::L2Distance => left_vec.l2_distance(right_vec)?,
            VectorOperator::NegativeInnerProduct => {
                let inner_product = left_vec.inner_product(right_vec)?;
                -inner_product // Negative for ASC ordering
            }
            VectorOperator::CosineDistance => left_vec.cosine_distance(right_vec)?,
        };

        Ok(Value::Float64(distance as f64))
    }
}

/// Evaluate vector distance operator directly on VectorValue types
///
/// This is a convenience function for use within the vector module.
pub fn eval_vector_distance(
    op: VectorOperator,
    left: &VectorValue,
    right: &VectorValue,
) -> Result<f32> {
    match op {
        VectorOperator::L2Distance => left.l2_distance(right),
        VectorOperator::NegativeInnerProduct => {
            let inner_product = left.inner_product(right)?;
            Ok(-inner_product)
        }
        VectorOperator::CosineDistance => left.cosine_distance(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_from_symbol() {
        assert_eq!(
            VectorOperator::from_symbol("<->"),
            Some(VectorOperator::L2Distance)
        );
        assert_eq!(
            VectorOperator::from_symbol("<#>"),
            Some(VectorOperator::NegativeInnerProduct)
        );
        assert_eq!(
            VectorOperator::from_symbol("<=>"),
            Some(VectorOperator::CosineDistance)
        );
        assert_eq!(VectorOperator::from_symbol("+"), None);
    }

    #[test]
    fn test_operator_to_symbol() {
        assert_eq!(VectorOperator::L2Distance.to_symbol(), "<->");
        assert_eq!(VectorOperator::NegativeInnerProduct.to_symbol(), "<#>");
        assert_eq!(VectorOperator::CosineDistance.to_symbol(), "<=>");
    }

    #[test]
    fn test_evaluate_l2_distance() {
        let v1 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();
        let v2 = VectorValue::new(vec![0.0, 1.0, 0.0]).unwrap();

        let result = VectorOperator::L2Distance
            .evaluate(&Value::Vector(v1), &Value::Vector(v2))
            .unwrap();

        if let Value::Float64(dist) = result {
            assert!((dist - 1.414).abs() < 0.001); // sqrt(2)
        } else {
            panic!("Expected Float64 result");
        }
    }

    #[test]
    fn test_evaluate_negative_inner_product() {
        let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
        let v2 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

        let result = VectorOperator::NegativeInnerProduct
            .evaluate(&Value::Vector(v1), &Value::Vector(v2))
            .unwrap();

        if let Value::Float64(dist) = result {
            assert_eq!(dist, -32.0); // -(1*4 + 2*5 + 3*6) = -32
        } else {
            panic!("Expected Float64 result");
        }
    }

    #[test]
    fn test_evaluate_cosine_distance() {
        let v1 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();
        let v2 = VectorValue::new(vec![1.0, 0.0, 0.0]).unwrap();

        let result = VectorOperator::CosineDistance
            .evaluate(&Value::Vector(v1), &Value::Vector(v2))
            .unwrap();

        if let Value::Float64(dist) = result {
            assert!(dist < 0.0001); // Same direction = 0 distance
        } else {
            panic!("Expected Float64 result");
        }
    }

    #[test]
    fn test_evaluate_non_vector_error() {
        let v1 = VectorValue::new(vec![1.0, 2.0]).unwrap();
        let int_val = Value::Int64(42);

        let result = VectorOperator::L2Distance
            .evaluate(&Value::Vector(v1), &int_val);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be a vector"));
    }

    #[test]
    fn test_eval_vector_distance() {
        let v1 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();
        let v2 = VectorValue::new(vec![4.0, 5.0, 6.0]).unwrap();

        // L2 distance
        let dist = eval_vector_distance(VectorOperator::L2Distance, &v1, &v2).unwrap();
        assert!((dist - 5.196).abs() < 0.001); // sqrt(27)

        // Negative inner product
        let inner = eval_vector_distance(VectorOperator::NegativeInnerProduct, &v1, &v2).unwrap();
        assert_eq!(inner, -32.0);

        // Cosine distance
        let cos_dist = eval_vector_distance(VectorOperator::CosineDistance, &v1, &v2).unwrap();
        assert!(cos_dist < 0.05); // Vectors pointing in similar direction
    }

    #[test]
    fn test_dimension_mismatch() {
        let v1 = VectorValue::new(vec![1.0, 2.0]).unwrap();
        let v2 = VectorValue::new(vec![1.0, 2.0, 3.0]).unwrap();

        let result = VectorOperator::L2Distance
            .evaluate(&Value::Vector(v1), &Value::Vector(v2));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Dimension mismatch"));
    }
}
