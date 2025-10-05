// node-subtract/src/domain.rs
//! Regra de negócio do node “subtract”: subtrai inteiros.

pub fn subtract_all(nums: &[i64]) -> i64 {
    if nums.is_empty() {
        return 0;
    }

    let first = nums[0];
    nums[1..].iter().fold(first, |acc, &num| acc - num)
}

// Best Practice: Módulo de testes no mesmo arquivo da lógica que ele testa.
#[cfg(test)]
mod tests {
    use super::*; // Importa a função `subtract_all` do módulo pai.

    #[test]
    fn test_subtract_basic() {
        assert_eq!(subtract_all(&[100, 10, 5]), 85);
    }

    #[test]
    fn test_subtract_with_negatives() {
        // 10 - (-5) = 15
        assert_eq!(subtract_all(&[10, -5]), 15);
    }

    #[test]
    fn test_subtract_single_number() {
        // Apenas um número, não subtrai nada.
        assert_eq!(subtract_all(&[42]), 42);
    }

    #[test]
    fn test_subtract_empty_list() {
        // Lista vazia deve retornar 0.
        assert_eq!(subtract_all(&[]), 0);
    }
}