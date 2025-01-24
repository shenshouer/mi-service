use rand::{distributions::Alphanumeric, thread_rng, Rng};

// 生成随机字符串的辅助函数
pub fn get_random(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
