// node-sum/src/domain.rs
//! Regra de negócio do node “sum”: soma inteiros.

pub fn sum_all(nums: &[i64]) -> i64 {
    nums.iter().sum()
}

// Best Practice: Módulo de testes no mesmo arquivo da lógica que ele testa.
#[cfg(test)]
mod tests {
    use super::*; // Importa a função `sum_all` do módulo pai.

    #[test]
    fn test_sum_basic() {
        assert_eq!(sum_all(&[10, 20, 5]), 35);
    }

    #[test]
    fn test_sum_with_negatives() {
        assert_eq!(sum_all(&[100, -20, -30]), 50);
    }

    #[test]
    fn test_sum_single_number() {
        assert_eq!(sum_all(&[42]), 42);
    }

    #[test]
    fn test_sum_empty_list() {
        // A soma de uma lista vazia é 0.
        assert_eq!(sum_all(&[]), 0);
    }
}