use std::cmp::Ordering;
use std::collections::HashMap;

/// 从多个标题中提取共同的作品名（改进版）
///
/// # 参数
/// - `titles`: 包含多个标题的切片
/// - `remove_unclosed_pair`: 是否移除未闭合括号对（默认为 true）
/// - `remove_tailing_sign_list`: 需要移除的尾部符号列表（默认为空）
///
/// # 返回值
/// 提取出的共同作品名（经过后处理）
pub fn extract_work_name(
    titles: &[&str],
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    // 如果标题列表为空，直接返回空字符串
    if titles.is_empty() {
        return "[!!! EMPTY !!!]".to_string();
    }

    // 使用 HashMap 统计所有可能前缀的出现次数
    let mut prefix_counts: HashMap<String, usize> = HashMap::new();

    // 遍历每个标题
    for &title in titles {
        // 生成标题的所有前缀（按字符长度）
        for i in 1..=title.chars().count() {
            // 获取前 i 个字符组成的前缀
            let prefix: String = title.chars().take(i).collect();
            // 更新前缀计数
            *prefix_counts.entry(prefix).or_insert(0) += 1;
        }
    }

    // 找出最大出现次数
    let max_count = *prefix_counts.values().max().unwrap_or(&0);

    // 筛选候选前缀：出现次数超过最大次数的 2/3
    let mut candidates: Vec<(String, usize)> = prefix_counts
        .into_iter()
        .filter(|(_, count)| *count as f32 >= max_count as f32 * 0.67)
        .collect();

    // 对候选前缀排序（优先长度降序，其次次数降序，最后字典序升序）
    candidates.sort_by(|a, b| {
        // 1. 按长度降序排序
        let len_cmp = b.0.len().cmp(&a.0.len());
        if len_cmp != Ordering::Equal {
            return len_cmp;
        }

        // 2. 按出现次数降序排序
        let count_cmp = b.1.cmp(&a.1);
        if count_cmp != Ordering::Equal {
            return count_cmp;
        }

        // 3. 按字典序升序排序
        a.0.cmp(&b.0)
    });

    // 提取最优候选（若存在则取第一个，否则为空字符串）
    let best_candidate = candidates
        .first()
        .map(|(s, _)| s.clone())
        .unwrap_or_default();

    // 对最优候选进行后处理
    extract_work_name_post_process(
        &best_candidate,
        remove_unclosed_pair,
        remove_tailing_sign_list,
    )
}

/// 作品名后处理函数：移除未闭合括号和尾部符号
///
/// # 参数
/// - `s`: 原始字符串
/// - `remove_unclosed_pair`: 是否处理未闭合括号
/// - `remove_tailing_sign_list`: 需要移除的尾部符号列表
///
/// # 返回值
/// 处理后的字符串
fn extract_work_name_post_process(
    s: &str,
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    // 先去除首尾空白字符
    let mut result = s.trim().to_string();

    // 定义支持的括号对（包含全角和半角）
    const PAIRS: [(char, char); 7] = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('（', '）'),
        ('［', '］'),
        ('｛', '｝'),
        ('【', '】'),
    ];

    // 循环处理直到没有变化
    loop {
        let mut changed = false;

        // 处理未闭合括号
        if remove_unclosed_pair {
            // 存储括号状态（括号字符 + 字节位置）
            let mut stack: Vec<(char, usize)> = Vec::new();

            // 遍历字符串的每个字符（带字节索引）
            for (byte_idx, c) in result.char_indices() {
                // 检查是否是开括号
                if PAIRS.iter().any(|&(open, _)| c == open) {
                    stack.push((c, byte_idx));
                }
                // 检查是否是闭括号
                else if let Some(&(last_open, _)) = stack.last() {
                    // 查找匹配的闭括号
                    if let Some(&(_, close)) = PAIRS.iter().find(|&&(open, _)| open == last_open)
                        && c == close
                    {
                        stack.pop();
                    }
                }
            }

            // 如果存在未闭合括号
            if let Some(&(_, unmatched_pos)) = stack.last() {
                // 截断到第一个未匹配括号的位置
                result.truncate(unmatched_pos);
                // 移除截断后的尾部空白
                result = result.trim_end().to_string();
                changed = true;
            }
        }

        // 处理尾部符号
        for &sign in remove_tailing_sign_list {
            if result.ends_with(sign) {
                // 移除匹配的尾部符号
                result.truncate(result.len() - sign.len());
                // 移除尾部空白
                result = result.trim_end().to_string();
                changed = true;
                // 一次只移除一个符号，然后重新检查
                break;
            }
        }

        // 如果没有变化则结束处理
        if !changed {
            break;
        }
    }

    result
}
