//! Regra de negócio do node “subtract”: subtrai inteiros.

pub fn subtract_all(nums: &[i64]) -> i64 {
    // Usa o primeiro número como o valor inicial e subtrai os demais.
    // Se a lista estiver vazia, o resultado é 0.
    if nums.is_empty() {
        return 0;
    }

    let first = nums[0];
    // O `fold` começa com `first` e aplica a operação de subtração para cada `&num` no resto do slice `&nums[1..]`.
    nums[1..].iter().fold(first, |acc, &num| acc - num)
}